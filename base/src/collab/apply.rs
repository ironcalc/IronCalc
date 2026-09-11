//! Turning a commit into workbook state.
//!
//! Every patch writes a register guarded by a [`Timestamp`] in [`SheetRegisters`] or
//! [`WorkbookMeta`], and applies iff its stamp is `>=` the stored one — the same `>=` as
//! [`Lww::merge`](crate::collab::log::Lww::merge), so a commit's patches resolve to the last one
//! and redelivery rewrites the same values.
//!
//! Applying is **total**: a patch naming a sheet, row or rule that is not here is a no-op, never an
//! error — a commit is never rejected. Guards outlive their subject, so a late write cannot
//! resurrect a deleted one.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

use crate::cf_types::ConditionalFormatting;
use crate::collab::bind::Host;
use crate::collab::formula::StableFormula;
use crate::collab::fractional_index::{FractionalIndex, FractionalKey};
use crate::collab::hlc::Hlc;
use crate::collab::log::{Commit, Consumer, Lww, SessionId, Snapshot, Timestamp};
use crate::collab::model::{
    default_workbook_views, default_worksheet_views, CollabModel, SheetIndexes, SheetRegisters,
    Stable, StableCellAddress, StableRange,
};
use crate::collab::naming::NameRepair;
use crate::collab::patch::{
    CellInput, CfPropKind, CfProperty, ColState, DefinedNameId, DefinedNameProperty, NamedStyleId,
    NamedStyleProperty, Patch, PropKind, Property, RowState, SheetContent, SheetId, SheetProperty,
    WorkbookProperty,
};
use crate::collab::DynError;
use crate::constants::{
    COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, ROW_HEIGHT_FACTOR,
};
use crate::expressions::parser::stringify::to_english_string;
use crate::expressions::parser::{static_analysis::run_static_analysis_on_node, Node};
use crate::expressions::token;
use crate::types::{
    Alignment, Cell, CellStyles, Col, DefinedName, FormulaValue, Row, SheetState, Style, Styles,
    Workbook, Worksheet,
};

/// Version byte prefixing every [`Snapshot::encode`] payload.
pub const SNAPSHOT_FORMAT_VERSION: u8 = 1;

impl Consumer for CollabModel<'_> {
    type Error = DynError;

    fn apply(&mut self, commit: &Commit) -> Result<(), Self::Error> {
        // Receive rule: this commit's stamp is a watermark for our own clock.
        Hlc::sync(commit.hlc);
        let ts = commit.timestamp();
        for patch in &commit.patches {
            self.apply_patch(patch, &ts);
        }
        self.resync_derived(&commit.patches);
        Ok(())
    }
}

impl Snapshot for CollabModel<'static> {
    /// A version byte, then the workbook. Views are per-user state and are never encoded.
    fn encode(&self) -> Vec<u8> {
        let mut out = vec![SNAPSHOT_FORMAT_VERSION];
        out.extend_from_slice(&bitcode::encode(&self.workbook));
        out
    }

    /// `session` is the **restoring** replica's, not the one that wrote the bytes: new keys must
    /// carry the identity of whoever loaded the snapshot.
    fn decode(bytes: &[u8], session: SessionId) -> Result<Self, Self::Error> {
        let (&version, workbook) = bytes.split_first().ok_or("empty snapshot payload")?;
        if version != SNAPSHOT_FORMAT_VERSION {
            return Err(format!("unsupported snapshot format version: {version}").into());
        }

        let mut model = CollabModel::new(session);
        model.workbook = bitcode::decode(workbook)?;
        // The suffix is never in the payload — see `FractionalIndex::decode`.
        let suffix = model.suffix();
        // Views were not encoded either: the restoring replica opens on the defaults.
        model.workbook.views = default_workbook_views();
        for sheet in &mut model.workbook.worksheets {
            sheet.index.rows.suffix = suffix;
            sheet.index.cols.suffix = suffix;
            sheet.views = default_worksheet_views();
        }
        for (index, text) in model.workbook.shared_strings.iter().enumerate() {
            model.shared_strings.insert(text.clone(), index);
        }
        Ok(model)
    }
}

/// Whether `ts` beats the guard stored for `key`, recording it when it does.
fn wins<K: Clone + Eq + Hash>(
    registers: &mut HashMap<K, Timestamp>,
    key: &K,
    ts: &Timestamp,
) -> bool {
    match registers.get(key) {
        Some(stored) if ts < stored => false,
        _ => {
            registers.insert(key.clone(), *ts);
            true
        }
    }
}

/// Filters out given `props` against the provided timestamp, returning only the more recent ones.
fn winning<'p, K: Clone + Eq + Hash>(
    registers: &mut HashMap<K, Timestamp>,
    key: impl Fn(PropKind) -> K,
    props: &'p [Property],
    ts: &Timestamp,
) -> Vec<&'p Property> {
    props
        .iter()
        .filter(|p| wins(registers, &key(p.kind()), ts))
        .collect()
}

/// An absent alignment and a default one draw the same, and only the absent spelling is what the
/// ordinal model stores — so a style built attribute by attribute has to come back to it.
fn canonical(mut style: Style) -> Style {
    if style.alignment.as_ref() == Some(&Alignment::default()) {
        style.alignment = None;
    }
    style
}

/// `own` with `props` written into it, interned. Index 0 is the default style.
fn overlay(styles: &mut Styles, own: i32, props: &[&Property]) -> i32 {
    let mut style = styles.get_style(own).unwrap_or_default();
    for p in props {
        p.write_into(&mut style);
    }
    styles.get_style_index_or_create(&canonical(style))
}

/// `own` reduced to the attributes `kept` admits, the rest at their defaults, interned.
fn filter_styles(styles: &mut Styles, own: i32, kept: impl Fn(PropKind) -> bool) -> i32 {
    let source = styles.get_style(own).unwrap_or_default();
    let mut style = Style::default();
    for k in PropKind::STYLE.into_iter().filter(|k| kept(*k)) {
        if let Some(p) = Property::read(&source, k) {
            p.write_into(&mut style);
        }
    }
    styles.get_style_index_or_create(&canonical(style))
}

/// The registers a seeded `style` has to guard: only the attributes it sets. A default attribute
/// has no entry, as on a row nobody ever formatted, so a seed of a million plain rows costs nothing.
fn set_kinds(style: &Style) -> impl Iterator<Item = PropKind> {
    Property::diff(&Style::default(), style)
        .0
        .into_iter()
        .map(|p| p.kind())
}

/// Sheet names are case-insensitively unique and capped at Excel's 31, mirrored by
/// [`is_valid_sheet_name`](crate::new_empty::is_valid_sheet_name).
pub(crate) const SHEET_NAMES: NameRepair = NameRepair {
    case_insensitive: true,
    max_len: 31,
};

/// Style names are case-sensitive — `get_style_index_by_name` matches exactly — and capped at
/// OOXML's 255.
pub(crate) const STYLE_NAMES: NameRepair = NameRepair {
    case_insensitive: false,
    max_len: 255,
};

/// Defined names are case-insensitively unique within a scope — every upstream lookup folds case,
/// from `Parser::get_defined_name` to `is_valid_defined_name` — and capped at Excel's 255.
pub(crate) const DEFINED_NAMES: NameRepair = NameRepair {
    case_insensitive: true,
    max_len: 255,
};

/// Whether the style table holds this entry itself rather than deriving it from a register, by
/// `Styles::is_builtin_style`'s rule. No patch authors a built-in — `create_named_style` files
/// `builtin_id: 0`, and every named-style mutator refuses a built-in name — so the rest is derived.
fn is_builtin_entry(entry: &CellStyles) -> bool {
    entry.builtin_id > 0 || entry.name.eq_ignore_ascii_case("normal")
}

/// Whether a commit can have changed which names are live, and so needs the names re-derived.
fn touches_sheet_names(patches: &[Patch]) -> bool {
    patches.iter().any(|patch| {
        matches!(
            patch,
            Patch::AddSheet { .. }
                | Patch::DeleteSheet { .. }
                | Patch::SetSheetProperty {
                    property: SheetProperty::Name(_),
                    ..
                }
        )
    })
}

/// Whether `keys` sit in the last `keys.len()` positions of `index` — an append that displaced no
/// existing element, so no ordinal moved. Read after the insert has been applied, when every key is
/// still active.
///
/// `keys` should always be in ascending order.
fn is_append_only(index: &FractionalIndex, keys: &[FractionalKey]) -> bool {
    debug_assert!(keys.is_sorted());
    let Some(first) = keys.first() else {
        return true;
    };
    keys.len() <= index.len() && index.position_of(first) == Some(index.len() - keys.len())
}

/// Index of `formula` in a sheet's shared formula table, appending it if new. Indices are assigned
/// per replica, which is why patches carry the stream itself.
fn intern_formula(formulas: &mut Vec<StableFormula>, formula: &StableFormula) -> i32 {
    match formulas.iter().position(|f| f == formula) {
        Some(index) => index as i32,
        None => {
            formulas.push(formula.clone());
            formulas.len() as i32 - 1
        }
    }
}

fn cell_style(sheet: &Worksheet<Stable>, at: &StableCellAddress) -> i32 {
    sheet
        .sheet_data
        .get(&at.0)
        .and_then(|row| row.get(&at.1))
        .map(|cell| cell.get_style())
        .unwrap_or(0)
}

fn put_cell(sheet: &mut Worksheet<Stable>, at: &StableCellAddress, cell: Cell) {
    sheet
        .sheet_data
        .entry(at.0.clone())
        .or_default()
        .insert(at.1.clone(), cell);
}

fn remove_cell(sheet: &mut Worksheet<Stable>, at: &StableCellAddress) {
    if let Some(row) = sheet.sheet_data.get_mut(&at.0) {
        row.remove(&at.1);
        // An emptied map must go: a replica whose write lost never filed the row at all.
        if row.is_empty() {
            sheet.sheet_data.remove(&at.0);
        }
    }
}

/// The `(width, height)` an array formula currently spills over, derived from the sheet's ordering.
/// A range that no longer resolves is one cell.
fn array_extent(sheet: &Worksheet<Stable>, range: &StableRange) -> (i32, i32) {
    match range.resolve(&sheet.index) {
        Some((row1, column1, row2, column2)) => (column2 - column1 + 1, row2 - row1 + 1),
        None => (1, 1),
    }
}

/// The cell a [`CellInput`] materializes into, keeping the style already on the cell. Formula text
/// is interned into the sheet's table but never parsed — evaluation is derived state.
fn build_cell(
    sheet: &mut Worksheet<Stable>,
    input: &CellInput,
    style: i32,
    shared_string: i32,
) -> Cell {
    match input {
        CellInput::Number(v) => Cell::NumberCell { v: *v, s: style },
        CellInput::Boolean(v) => Cell::BooleanCell { v: *v, s: style },
        CellInput::Text(_) => Cell::SharedString {
            si: shared_string,
            s: style,
        },
        CellInput::Error(ei) => Cell::ErrorCell {
            ei: ei.clone(),
            s: style,
        },
        CellInput::Formula(formula) => Cell::CellFormula {
            f: intern_formula(&mut sheet.shared_formulas, formula),
            s: style,
            v: FormulaValue::Unevaluated,
        },
        CellInput::Array {
            formula,
            range,
            kind,
        } => {
            let r = array_extent(sheet, range);
            Cell::ArrayFormula {
                f: intern_formula(&mut sheet.shared_formulas, formula),
                s: style,
                r,
                kind: kind.clone(),
                v: FormulaValue::Unevaluated,
            }
        }
    }
}

/// Whether a cell outlives a delete stamped `at`: each half survives only under a guard strictly
/// newer than it, and a surviving half resets the dominated one to its default.
fn keep_cell(
    registers: &SheetRegisters,
    styles: &mut Styles,
    address: &StableCellAddress,
    at: Hlc,
    cell: &mut Cell,
) -> bool {
    let newer_value = if let Some(v) = registers.cell_values.get(address) {
        v.hlc > at
    } else {
        false
    };
    // Per attribute: only the ones stamped after the delete survive it.
    let kept = |k| {
        registers
            .cell_styles
            .get(&(address.clone(), k))
            .is_some_and(|s| s.hlc > at)
    };
    let any_style = PropKind::STYLE.into_iter().any(kept);
    if newer_value && !any_style {
        cell.set_style(0);
    }
    if any_style {
        cell.set_style(filter_styles(styles, cell.get_style(), kept));
        if !newer_value {
            *cell = Cell::EmptyCell {
                s: cell.get_style(),
            };
        }
    }
    newer_value || any_style
}

fn cell_deleted(index: &SheetIndexes, at: &StableCellAddress, ts: Hlc) -> bool {
    match index.rows.removed_at(&at.0) {
        Some(tombstone) if tombstone >= ts => true,
        _ => match index.cols.removed_at(&at.1) {
            Some(tombstone) => tombstone >= ts,
            None => false,
        },
    }
}

/// The row and column record lists are kept sorted by the key they are filed under, so that two
/// replicas that saw the same writes in different orders still hold byte-identical worksheets.
fn row_record<'r>(sheet: &'r mut Worksheet<Stable>, key: &FractionalKey) -> &'r mut Row<Stable> {
    let at = match sheet.rows.binary_search_by(|r| r.r.cmp(key)) {
        Ok(at) => return &mut sheet.rows[at],
        Err(at) => at,
    };
    sheet.rows.insert(
        at,
        Row {
            r: key.clone(),
            height: DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR,
            custom_format: false,
            custom_height: false,
            s: 0,
            hidden: false,
        },
    );
    &mut sheet.rows[at]
}

/// See [`row_record`]; a column record is filed under its span's two corners.
fn col_record<'r>(
    sheet: &'r mut Worksheet<Stable>,
    span: &(FractionalKey, FractionalKey),
) -> &'r mut Col<Stable> {
    let at = match sheet
        .cols
        .binary_search_by(|c| (&c.min, &c.max).cmp(&(&span.0, &span.1)))
    {
        Ok(at) => return &mut sheet.cols[at],
        Err(at) => at,
    };
    sheet.cols.insert(
        at,
        Col {
            min: span.0.clone(),
            max: span.1.clone(),
            width: DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR,
            custom_width: false,
            hidden: false,
            style: None,
        },
    );
    &mut sheet.cols[at]
}

/// Puts the rules back in priority order and renumbers them, which is what priority *is* under
/// stable addressing: a rule sorts by the position key written for it, by its own identity when
/// none was, and identity breaks any tie.
fn sort_cf(sheet: &mut Worksheet<Stable>) {
    let mut order = std::mem::take(&mut sheet.index.registers.cf_order);
    let mut rules = std::mem::take(&mut sheet.conditional_formatting);
    let positions = &sheet.index.registers.cf_positions;
    let mut pairs: Vec<_> = order.drain(..).zip(rules.drain(..)).collect();
    //TODO: optimize?
    pairs.sort_by_cached_key(|(key, _)| {
        let position = positions.get(key).map(|p| &p.value).unwrap_or(key).clone();
        (position, key.clone())
    });
    for (i, (key, mut cf)) in pairs.into_iter().enumerate() {
        cf.priority = i as u32 + 1;
        order.push(key);
        rules.push(cf);
    }
    sheet.index.registers.cf_order = order;
    sheet.conditional_formatting = rules;
}

/// Where rule `key` currently sits in storage.
fn cf_slot(sheet: &Worksheet<Stable>, key: &FractionalKey) -> Option<usize> {
    sheet.index.registers.cf_order.iter().position(|k| k == key)
}

impl CollabModel<'_> {
    fn sheet_index(&self, sheet: SheetId) -> Option<usize> {
        self.workbook
            .worksheets
            .iter()
            .position(|ws| ws.sheet_id == sheet)
    }

    /// Index of `text` in the workbook's shared strings, appending it if it is new.
    fn intern_string(&mut self, text: &str) -> i32 {
        if let Some(&index) = self.shared_strings.get(text) {
            return index as i32;
        }
        let index = self.workbook.shared_strings.len();
        self.workbook.shared_strings.push(text.to_string());
        self.shared_strings.insert(text.to_string(), index);
        index as i32
    }

    /// Index of `style` in the workbook's style table, creating the entry if it is new.
    fn intern_style(&mut self, style: &Style) -> i32 {
        self.workbook.styles.get_style_index_or_create(style)
    }

    /// Puts the worksheets back in tab order. A sheet with no position sorts last by id, which
    /// keeps the ordering total.
    fn sort_sheets(&mut self) {
        let positions = std::mem::take(&mut self.workbook.meta.sheet_positions);
        self.workbook.worksheets.sort_by(|a, b| {
            match (
                positions.get(&a.sheet_id).map(|p| &p.value),
                positions.get(&b.sheet_id).map(|p| &p.value),
            ) {
                // A revived sheet's re-filed key can equal one minted into the gap it left.
                (Some(x), Some(y)) => x.cmp(y).then_with(|| a.sheet_id.cmp(&b.sheet_id)),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => a.sheet_id.cmp(&b.sheet_id),
            }
        });
        self.workbook.meta.sheet_positions = positions;
    }

    /// Files a sheet's authored name, under the same last-write-wins arbitration as every other
    /// register. Both `AddSheet` and a rename write here; nobody writes the display name.
    fn file_sheet_name(&mut self, sheet: SheetId, name: &str, ts: &Timestamp) {
        self.workbook
            .meta
            .sheet_names
            .entry(sheet)
            .or_default()
            .merge(name.to_string(), ts);
    }

    /// Derives every `Worksheet::name` from the authored names, repairing the collisions concurrent
    /// adds and renames can produce — Excel needs names case-insensitively unique, and formulas
    /// resolve sheets by display name.
    ///
    /// Deterministic and commutative: the inputs are the replicated name registers and their total
    /// stamp order, so every replica derives the same names whatever order the commits arrived in.
    ///
    /// Known limit: formulas name sheets by display-name text, so when a repair renames the losing
    /// sheet, a formula written against that name silently retargets to the winner — convergent
    /// everywhere, but not intent-preserving. It goes away once formulas are stored as typed
    /// `Node<A: Position>` holding stable sheet references rather than strings.
    fn normalize_sheet_names(&mut self) {
        let mut authored: Vec<(Timestamp, (SheetId, usize), String)> = Vec::new();
        for (i, sheet) in self.workbook.worksheets.iter().enumerate() {
            match self.workbook.meta.sheet_names.get(&sheet.sheet_id) {
                Some(lww) => authored.push((lww.timestamp, (sheet.sheet_id, i), lww.value.clone())),
                // Every live sheet arrived as an `AddSheet`, which files the register.
                None => debug_assert!(false, "sheet {} has no authored name", sheet.sheet_id),
            }
        }
        for ((_, i), name) in SHEET_NAMES.assign(authored) {
            self.workbook.worksheets[i].name = name;
        }
    }

    /// Every live named style as `(id, display name)`, in the order the style table shows them:
    /// first write wins, around the built-in entries no register owns.
    pub(crate) fn named_style_display(&self) -> Vec<(NamedStyleId, String)> {
        let authored: Vec<(Timestamp, NamedStyleId, String)> = self
            .workbook
            .meta
            .named_styles
            .iter()
            .filter(|(_, state)| state.definition.value.is_some())
            .map(|(id, state)| (state.name.timestamp, *id, state.name.value.clone()))
            .collect();
        let mut taken = STYLE_NAMES.taken(
            self.workbook
                .styles
                .cell_styles
                .iter()
                .filter(|entry| is_builtin_entry(entry))
                .map(|entry| entry.name.as_str()),
        );
        STYLE_NAMES.assign_within(authored, &mut taken)
    }

    /// Derives the style table from the named-style registers, the way sheet names are derived: the
    /// live styles, named first-write-wins, after the built-ins no register owns.
    fn normalize_named_styles(&mut self) {
        let assignments = self.named_style_display();
        self.workbook.styles.cell_styles.retain(is_builtin_entry);
        for (id, name) in assignments {
            let Some(definition) = self
                .workbook
                .meta
                .named_styles
                .get(&id)
                .and_then(|state| state.definition.value.clone())
            else {
                continue;
            };
            // A replicated style includes every formatting category (a quote prefix is a cell's
            // own state, never a style's). Its record is content-addressed, so the table is a
            // function of the registers rather than of the order their writes arrived in.
            let xf_id = self
                .workbook
                .styles
                .get_base_style_index_or_create(&definition.style);
            self.workbook.styles.cell_styles.push(CellStyles {
                name,
                xf_id,
                builtin_id: definition.builtin_id,
            });
        }
    }

    /// Every live defined name as `(id, scope, display name)`, ordered by id so two replicas that
    /// saw the same writes hold byte-identical workbooks.
    ///
    /// Repair runs per scope bucket: uniqueness is per scope, so a global `total` and a
    /// sheet-scoped `total` coexist and never rename each other.
    pub(crate) fn defined_name_display(&self) -> Vec<(DefinedNameId, Option<SheetId>, String)> {
        let mut buckets: HashMap<Option<SheetId>, Vec<(Timestamp, DefinedNameId, String)>> =
            HashMap::new();
        for (id, state) in &self.workbook.meta.defined_names {
            if state.formula.value.is_some() {
                let (scope, name) = &state.name.value;
                buckets
                    .entry(*scope)
                    .or_default()
                    .push((state.name.timestamp, *id, name.clone()));
            }
        }
        let mut display: Vec<(DefinedNameId, Option<SheetId>, String)> = buckets
            .into_iter()
            .flat_map(|(scope, authored)| {
                DEFINED_NAMES
                    .assign(authored)
                    .into_iter()
                    .map(move |(id, name)| (id, scope, name))
            })
            .collect();
        display.sort_by_key(|(id, ..)| *id);
        display
    }

    /// Derives `Workbook::defined_names` from the registers, the way the style table is derived:
    /// the live names, repaired first-write-wins within each scope.
    fn normalize_defined_names(&mut self) {
        let context = self.defined_name_context();
        let display = self.defined_name_display();
        let mut derived = Vec::with_capacity(display.len());
        for (id, sheet_id, name) in display {
            let Some(body) = self.formula_of(id) else {
                continue;
            };
            let Ok(node) = self.lower(&body.formula, &Host::relative(0, 1, 1)) else {
                continue;
            };
            let text = to_english_string(&node, &context);
            derived.push(DefinedName {
                name,
                formula: match body.equals {
                    true => format!("={text}"),
                    false => text,
                },
                sheet_id,
            });
        }
        self.workbook.defined_names = derived;
    }

    /// Re-derives what a commit invalidated. Display names come first: the parse tables resolve
    /// sheets and defined names by them.
    pub(crate) fn resync_derived(&mut self, patches: &[Patch]) {
        if touches_sheet_names(patches) {
            self.normalize_sheet_names();
        }
        if patches
            .iter()
            .any(|patch| matches!(patch, Patch::SetNamedStyle { .. }))
        {
            self.normalize_named_styles();
        }

        // A revival (an insert re-applied over its own delete, e.g. redo) looks like a tail append,
        // but formulas lowered while the key was dead hold stale `#REF!` nodes.
        let revived = std::mem::take(&mut self.local.revived);
        if !revived
            && self.is_content_only(patches)
            && self.parsed_formulas.len() == self.workbook.worksheets.len()
        {
            self.lower_formulas_tail();
        } else {
            #[cfg(test)]
            {
                self.local.full_resyncs += 1;
            }
            self.normalize_defined_names();
            self.resync_parsed();
        }
    }

    /// Return `true` if none of the `patches` introduce changes that may trigger shift
    /// in referenced cell positions.
    pub(crate) fn is_content_only(&self, patches: &[Patch]) -> bool {
        patches.iter().all(|patch| match patch {
            Patch::SetCellValue { .. }
            | Patch::SetArrayValue { .. }
            | Patch::SetCellStyle { .. }
            | Patch::SetRowProperty { .. }
            | Patch::SetColumnSpan { .. }
            | Patch::SetNamedStyle { .. }
            | Patch::AddConditionalFormat { .. }
            | Patch::DeleteConditionalFormat { .. }
            | Patch::MoveConditionalFormats { .. }
            | Patch::SetConditionalFormat { .. }
            | Patch::SetMergedRange { .. }
            | Patch::SetComment { .. } => true,
            Patch::SetSheetProperty { property, .. } => matches!(
                property,
                SheetProperty::Color(_)
                    | SheetProperty::State(_)
                    | SheetProperty::ShowGridLines(_)
                    | SheetProperty::FrozenRows(_)
                    | SheetProperty::FrozenColumns(_)
            ),
            // The locale drives defined-name parsing; the theme and the name drive nothing lowered.
            Patch::SetWorkbookProperty { property, .. } => {
                matches!(
                    property,
                    WorkbookProperty::Theme(_) | WorkbookProperty::Name(_)
                )
            }
            // A tail append displaces no ordinal — which is what materialize-on-bind mints.
            Patch::InsertRows { sheet, keys } => self
                .sheet_index(*sheet)
                .is_some_and(|i| is_append_only(&self.workbook.worksheets[i].index.rows, keys)),
            Patch::InsertColumns { sheet, keys } => self
                .sheet_index(*sheet)
                .is_some_and(|i| is_append_only(&self.workbook.worksheets[i].index.cols, keys)),
            _ => false,
        })
    }

    /// Rebuilds the parse tables a commit invalidated. They are derived from `shared_formulas` and
    /// the defined names, both of which patches append to, and re-deriving is cheaper than tracking
    /// which sheet index moved where. Evaluation stays the caller's business.
    pub(crate) fn resync_parsed(&mut self) {
        let defined_names = self.workbook.get_defined_names_with_scope();
        self.parser
            .set_worksheets_and_names(self.workbook.get_worksheet_names(), defined_names);
        self.lower_formulas();
        self.parse_defined_names();
    }

    /// The stable twin of [`Model::parse_formulas`]: every shared entry lowered once per sheet.
    ///
    /// The lowering is all-absolute, because one entry serves every cell that interned it — `D1`
    /// and `E1` both holding `=A1` share a binding — so the cached node cannot depend on a host.
    /// An entry whose sheet is gone has no node at all and lowers to `#REF!`.
    fn lower_formulas(&mut self) {
        self.parsed_formulas = self
            .workbook
            .worksheets
            .iter()
            .map(|_| Vec::new())
            .collect();
        self.lower_formulas_tail();
    }

    fn lower_formulas_tail(&mut self) {
        for i in 0..self.workbook.worksheets.len() {
            let done = self.parsed_formulas[i].len();
            if self.workbook.worksheets[i].shared_formulas.len() <= done {
                continue;
            }
            // Taken out and put back so the lowering, which reads the whole workbook, can borrow.
            let formulas = std::mem::take(&mut self.workbook.worksheets[i].shared_formulas);
            for formula in &formulas[done..] {
                let node = self
                    .lower(formula, &Host::absolute(i as u32))
                    .unwrap_or(Node::ErrorKind(token::Error::REF));
                let static_result = run_static_analysis_on_node(&node);
                self.parsed_formulas[i].push((node, static_result));
            }
            self.workbook.worksheets[i].shared_formulas = formulas;
        }
    }

    pub(crate) fn apply_patch(&mut self, patch: &Patch, ts: &Timestamp) {
        if let Some(sheet) = patch.target_sheet() {
            if let Some(timestamp) = self.workbook.meta.sheet_existence.get(&sheet) {
                if ts < timestamp && self.sheet_index(sheet).is_some() {
                    return;
                }
            }
        }
        match patch {
            Patch::SetCellValue {
                sheet,
                at,
                value,
                ts: at_ts,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let ts = at_ts.as_ref().unwrap_or(ts);
                let index = &mut self.workbook.worksheets[i].index;
                // The guard is written either way; only the content is dead under a tombstone.
                if !wins(&mut index.registers.cell_values, at, ts)
                    || cell_deleted(index, at, ts.hlc)
                {
                    return;
                }
                self.write_cell(i, at, value.as_ref());
            }
            Patch::SetArrayValue {
                sheet,
                anchor,
                value,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let registers = &mut self.workbook.worksheets[i].index.registers;
                if !wins(&mut registers.arrays, anchor, ts) {
                    return;
                }
                // Only the anchor is stored; the cells it spills into are derived.
                self.write_cell(i, anchor, value.as_ref());
            }
            Patch::SetCellStyle {
                sheet,
                at,
                props,
                ts: at_ts,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let ts = at_ts.as_ref().unwrap_or(ts);
                let index = &mut self.workbook.worksheets[i].index;
                let won = winning(
                    &mut index.registers.cell_styles,
                    |k| (at.clone(), k),
                    props,
                    ts,
                );
                if won.is_empty() || cell_deleted(index, at, ts.hlc) {
                    return;
                }
                let cell = self.workbook.worksheets[i]
                    .sheet_data
                    .get(&at.0)
                    .and_then(|r| r.get(&at.1));
                let empty = matches!(cell, None | Some(Cell::EmptyCell { .. }));
                let own = cell.map_or(0, Cell::get_style);
                let s = overlay(&mut self.workbook.styles, own, &won);
                let sheet = &mut self.workbook.worksheets[i];
                // Nothing left for the cell to hold, so it goes too.
                if s == 0 && empty {
                    remove_cell(sheet, at);
                } else {
                    match sheet
                        .sheet_data
                        .get_mut(&at.0)
                        .and_then(|r| r.get_mut(&at.1))
                    {
                        Some(cell) => cell.set_style(s),
                        None => put_cell(sheet, at, Cell::EmptyCell { s }),
                    }
                }
            }
            Patch::InsertRows { sheet, keys } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let index = &mut self.workbook.worksheets[i].index;
                for key in keys {
                    let seen = index.rows.seen(key);
                    if index.rows.insert_key_at(key.clone(), ts.hlc).is_some() && seen {
                        self.local.revived = true;
                    }
                }
            }
            Patch::InsertColumns { sheet, keys } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let index = &mut self.workbook.worksheets[i].index;
                for key in keys {
                    let seen = index.cols.seen(key);
                    if index.cols.insert_key_at(key.clone(), ts.hlc).is_some() && seen {
                        self.local.revived = true;
                    }
                }
            }
            Patch::DeleteRows { sheet, keys, .. } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let Workbook {
                    worksheets, styles, ..
                } = &mut self.workbook;
                let sheet = &mut worksheets[i];
                for key in keys {
                    sheet.index.rows.remove_key_at(key, ts.hlc);
                    // The delete is one more write per register: it takes what it outranks and
                    // leaves what a newer write already claimed.
                    let registers = &sheet.index.registers;
                    if let Some(row) = sheet.sheet_data.get_mut(key) {
                        row.retain(|col, cell| {
                            keep_cell(registers, styles, &(key.clone(), col.clone()), ts.hlc, cell)
                        });
                        if row.is_empty() {
                            sheet.sheet_data.remove(key);
                        }
                    }
                    let kept = |kind| {
                        registers
                            .rows
                            .get(&(key.clone(), kind))
                            .is_some_and(|guard| guard.hlc > ts.hlc)
                    };
                    if let Ok(at) = sheet.rows.binary_search_by(|row| row.r.cmp(key)) {
                        let row = &mut sheet.rows[at];
                        row.s = filter_styles(styles, row.s, kept);
                        row.custom_format = row.s != 0;
                        if !kept(PropKind::Height) {
                            row.height = DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR;
                            row.custom_height = false;
                        }
                        if !kept(PropKind::Hidden) {
                            row.hidden = false;
                        }
                        if row.is_empty() {
                            sheet.rows.remove(at);
                        }
                    }
                }
            }
            Patch::DeleteColumns { sheet, keys, .. } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let Workbook {
                    worksheets, styles, ..
                } = &mut self.workbook;
                let sheet = &mut worksheets[i];
                for key in keys {
                    sheet.index.cols.remove_key_at(key, ts.hlc);
                    let registers = &sheet.index.registers;
                    // Retain, not `iter_mut`: a row the delete empties has to go with its last cell.
                    sheet.sheet_data.retain(|row_key, row| {
                        let address = (row_key.clone(), key.clone());
                        if let Some(cell) = row.get_mut(key) {
                            if !keep_cell(registers, styles, &address, ts.hlc, cell) {
                                row.remove(key);
                            }
                        }
                        !row.is_empty()
                    });
                    // Spans are regions, not sets of columns: what they cover resolves against the
                    // index, which just shrank.
                }
            }
            Patch::MoveRows { sheet, moves, .. } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let index = &mut self.workbook.worksheets[i].index;
                for (source, dest) in moves {
                    index.rows.apply_move(source, dest.clone(), ts.hlc);
                }
            }
            Patch::MoveColumns { sheet, moves, .. } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let index = &mut self.workbook.worksheets[i].index;
                for (source, dest) in moves {
                    index.cols.apply_move(source, dest.clone(), ts.hlc);
                }
            }
            Patch::SetRowProperty {
                sheet,
                row,
                props,
                ts: at_ts,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let ts = at_ts.as_ref().unwrap_or(ts);
                let index = &mut self.workbook.worksheets[i].index;
                // is row tombstone higher than patch timestamp?
                let row_deleted = if let Some(t) = index.rows.removed_at(row) {
                    t >= ts.hlc
                } else {
                    false
                };
                let won = winning(&mut index.registers.rows, |k| (row.clone(), k), props, ts);
                if row_deleted || won.is_empty() {
                    return;
                }
                let own = self.workbook.worksheets[i]
                    .rows
                    .iter()
                    .find(|r| &r.r == row)
                    .filter(|r| r.custom_format)
                    .map_or(0, |r| r.s);
                let s = overlay(&mut self.workbook.styles, own, &won);
                let record = row_record(&mut self.workbook.worksheets[i], row);
                record.s = s;
                record.custom_format = s != 0;
                // A row has no width; the kinds it does not own are simply skipped.
                for property in won {
                    match property {
                        Property::Height(height) => {
                            record.height = *height;
                            record.custom_height = true;
                        }
                        Property::Hidden(hidden) => record.hidden = *hidden,
                        _ => {}
                    }
                }
            }
            Patch::SetColumnSpan {
                sheet,
                span,
                props,
                ts: at_ts,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let ts = at_ts.as_ref().unwrap_or(ts);
                let registers = &mut self.workbook.worksheets[i].index.registers;
                let won = winning(&mut registers.col_spans, |k| (span.clone(), k), props, ts);
                if won.is_empty() {
                    return;
                }
                let own = self.workbook.worksheets[i]
                    .cols
                    .iter()
                    .find(|c| (&c.min, &c.max) == (&span.0, &span.1))
                    .and_then(|c| c.style)
                    .unwrap_or(0);
                let s = overlay(&mut self.workbook.styles, own, &won);
                let record = col_record(&mut self.workbook.worksheets[i], span);
                record.style = (s != 0).then_some(s);
                // A column has no height; the kinds it does not own are simply skipped.
                for property in won {
                    match property {
                        Property::Width(width) => {
                            record.width = *width;
                            record.custom_width = true;
                        }
                        Property::Hidden(hidden) => record.hidden = *hidden,
                        _ => {}
                    }
                }
            }
            Patch::SetMergedRange {
                sheet,
                range,
                merged,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let sheet = &mut self.workbook.worksheets[i];
                if !wins(&mut sheet.index.registers.merges, range, ts) {
                    return;
                }
                let at = sheet.merged_cells.iter().position(|r| r == range);
                match (merged, at) {
                    (true, None) => sheet.merged_cells.push(range.clone()),
                    (false, Some(at)) => {
                        sheet.merged_cells.remove(at);
                    }
                    _ => {}
                }
            }
            Patch::SetComment {
                sheet, at, comment, ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let sheet = &mut self.workbook.worksheets[i];
                if !wins(&mut sheet.index.registers.comments, at, ts) {
                    return;
                }
                let found = sheet.comments.iter().position(|c| &c.cell_ref == at);
                match (comment, found) {
                    (Some(comment), Some(at)) => sheet.comments[at] = comment.clone(),
                    (Some(comment), None) => sheet.comments.push(comment.clone()),
                    (None, Some(at)) => {
                        sheet.comments.remove(at);
                    }
                    (None, None) => {}
                }
            }
            Patch::AddSheet {
                id,
                name,
                position,
                content,
            } => {
                if !wins(&mut self.workbook.meta.sheet_existence, id, ts) {
                    return;
                }
                self.workbook
                    .meta
                    .sheet_positions
                    .insert(*id, Lww::new(position.clone(), *ts));
                // Same register a rename writes: a redelivered add must not undo a later rename.
                self.file_sheet_name(*id, name, ts);
                if self.sheet_index(*id).is_none() {
                    let sheet = Worksheet {
                        dimension: "A1".to_string(),
                        cols: vec![],
                        rows: vec![],
                        name: name.clone(),
                        sheet_data: Default::default(),
                        shared_formulas: vec![],
                        sheet_id: *id,
                        state: SheetState::Visible,
                        color: Default::default(),
                        merged_cells: vec![],
                        comments: vec![],
                        frozen_rows: 0,
                        frozen_columns: 0,
                        views: default_worksheet_views(),
                        show_grid_lines: true,
                        conditional_formatting: vec![],
                        links: HashMap::new(),
                        index: self.new_indexes(),
                    };
                    self.workbook.worksheets.push(sheet);
                    if let Some(content) = content {
                        let i = self.workbook.worksheets.len() - 1;
                        self.seed_sheet(i, content, ts);
                    }
                }
                self.sort_sheets();
            }
            Patch::DeleteSheet { sheet, .. } => {
                if !wins(&mut self.workbook.meta.sheet_existence, sheet, ts) {
                    return;
                }
                if let Some(i) = self.sheet_index(*sheet) {
                    self.workbook.worksheets.remove(i);
                }
            }
            Patch::SetSheetProperty {
                sheet, property, ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                if let SheetProperty::Position(position) = property {
                    // Tab order outlives the sheet, so its guard rides in `sheet_positions` rather
                    // than the sheet's own registers.
                    if !self
                        .workbook
                        .meta
                        .sheet_positions
                        .entry(*sheet)
                        .or_default()
                        .merge(position.clone(), ts)
                    {
                        return;
                    }
                    self.sort_sheets();
                    return;
                }
                if let SheetProperty::Name(name) = property {
                    // The display name is derived, so a rename only files what the user authored.
                    self.file_sheet_name(*sheet, name, ts);
                    return;
                }
                let registers = &mut self.workbook.worksheets[i].index.registers;
                if !wins(&mut registers.props, &property.kind(), ts) {
                    return;
                }
                let sheet = &mut self.workbook.worksheets[i];
                match property {
                    SheetProperty::Color(color) => sheet.color = color.clone(),
                    SheetProperty::State(state) => sheet.state = state.clone(),
                    SheetProperty::ShowGridLines(show) => sheet.show_grid_lines = *show,
                    SheetProperty::FrozenRows(rows) => sheet.frozen_rows = *rows,
                    SheetProperty::FrozenColumns(columns) => sheet.frozen_columns = *columns,
                    SheetProperty::Name(_) | SheetProperty::Position(_) => {
                        unreachable!("handled above")
                    }
                }
            }
            Patch::SetWorkbookProperty { property, .. } => {
                if !wins(&mut self.workbook.meta.props, &property.kind(), ts) {
                    return;
                }
                match property {
                    WorkbookProperty::Theme(theme) => self.workbook.theme = (**theme).clone(),
                    WorkbookProperty::Locale(locale) => {
                        self.workbook.settings.locale = locale.clone()
                    }
                    WorkbookProperty::Timezone(tz) => self.workbook.settings.tz = tz.clone(),
                    WorkbookProperty::Name(name) => self.workbook.name = name.clone(),
                }
            }
            // `Workbook::defined_names` itself is derived from these registers by
            // `normalize_defined_names`.
            Patch::SetDefinedName { id, property, .. } => {
                let state = self.workbook.meta.defined_names.entry(*id).or_default();
                // Each value guards itself, and the one this patch does not carry keeps the guard
                // it has — zero for an id first seen here, which any real write then beats.
                match property {
                    DefinedNameProperty::Name(address) => {
                        state.name.merge(address.clone(), ts);
                    }
                    DefinedNameProperty::Definition(formula) => {
                        state.formula.merge(formula.clone(), ts);
                    }
                }
            }
            // The style table itself is derived from these registers by `normalize_named_styles`.
            Patch::SetNamedStyle { id, property, .. } => {
                let state = self.workbook.meta.named_styles.entry(*id).or_default();
                // Each value guards itself, and the one this patch does not carry keeps the guard
                // it has — zero for an id first seen here, which any real write then beats.
                match property {
                    NamedStyleProperty::Name(name) => {
                        state.name.merge(name.clone(), ts);
                    }
                    NamedStyleProperty::Definition(definition) => {
                        state.definition.merge(definition.clone(), ts);
                    }
                }
            }
            Patch::AddConditionalFormat {
                sheet,
                key,
                rule,
                ranges,
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let registers = &mut self.workbook.worksheets[i].index.registers;
                let rule_wins = wins(&mut registers.cf, &(key.clone(), CfPropKind::Rule), ts);
                let ranges_win = wins(&mut registers.cf, &(key.clone(), CfPropKind::Ranges), ts);
                if !rule_wins || !ranges_win {
                    return;
                }
                let sheet = &mut self.workbook.worksheets[i];
                // originally cf_order was sorted by key, however now we support move operations
                // which may change keys order
                if cf_slot(sheet, key).is_none() {
                    sheet.index.registers.cf_order.push(key.clone());
                    sheet.conditional_formatting.push(ConditionalFormatting {
                        ranges: ranges.clone(),
                        cf_rule: (**rule).clone(),
                        priority: 0,
                    });
                    sort_cf(sheet);
                }
            }
            Patch::DeleteConditionalFormat { sheet, key, .. } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let sheet = &mut self.workbook.worksheets[i];
                // The guards stay: they keep a concurrent edit from resurrecting the rule.
                if let Some(at) = cf_slot(sheet, key) {
                    sheet.index.registers.cf_order.remove(at);
                    sheet.conditional_formatting.remove(at);
                    sort_cf(sheet);
                }
            }
            Patch::SetConditionalFormat {
                sheet,
                key,
                property,
                ..
            } => {
                let Some(i) = self.sheet_index(*sheet) else {
                    return;
                };
                let sheet = &mut self.workbook.worksheets[i];
                if let CfProperty::Priority(position) = property {
                    // Order outlives the rule's other registers, so its guard rides with its value.
                    if !sheet
                        .index
                        .registers
                        .cf_positions
                        .entry(key.clone())
                        .or_default()
                        .merge(position.clone(), ts)
                    {
                        return; // outdated patch
                    }
                    sort_cf(sheet);
                    return;
                }
                if !wins(
                    &mut sheet.index.registers.cf,
                    &(key.clone(), property.kind()),
                    ts,
                ) {
                    return;
                }
                let Some(at) = cf_slot(sheet, key) else {
                    return;
                };
                match property {
                    CfProperty::Rule(rule) => {
                        sheet.conditional_formatting[at].cf_rule = (**rule).clone()
                    }
                    CfProperty::Ranges(ranges) => {
                        sheet.conditional_formatting[at].ranges = ranges.clone()
                    }
                    CfProperty::Priority(_) => unreachable!("handled above"),
                }
            }
            // A move over `cf_order`, needing the author-minted destinations `MoveRows` carries.
            // Phase 5b.
            Patch::MoveConditionalFormats { .. } => {}
        }
    }

    /// Writes `value` into cell `at` of the `i`-th worksheet, `None` clearing it. The style already
    /// on the cell survives — it is a register of its own.
    fn write_cell(&mut self, i: usize, at: &StableCellAddress, value: Option<&CellInput>) {
        let Some(value) = value else {
            remove_cell(&mut self.workbook.worksheets[i], at);
            return;
        };
        // Interning first: it needs the workbook, the cell needs the sheet.
        let shared_string = match value {
            CellInput::Text(text) => self.intern_string(text),
            _ => 0,
        };
        let sheet = &mut self.workbook.worksheets[i];
        let style = cell_style(sheet, at);
        let cell = build_cell(sheet, value, style, shared_string);
        put_cell(sheet, at, cell);
    }

    /// Fills a freshly created sheet from the payload an `AddSheet` carried.
    ///
    /// Only the keys the content names are seeded into the indexes; a range with a corner outside
    /// them still resolves by clamping, exactly as one whose corner was deleted does.
    fn seed_sheet(&mut self, i: usize, content: &SheetContent, ts: &Timestamp) {
        let sheet = &mut self.workbook.worksheets[i];
        sheet.state = content.state.clone();
        sheet.color = content.color.clone();
        sheet.show_grid_lines = content.show_grid_lines;
        sheet.frozen_rows = content.frozen_rows;
        sheet.frozen_columns = content.frozen_columns;

        let mut rows: Vec<&FractionalKey> = content.rows.iter().map(|(key, _)| key).collect();
        let mut cols: Vec<&FractionalKey> = content
            .columns
            .iter()
            .flat_map(|((min, max), _)| [min, max])
            .filter(|key| !key.is_empty())
            .collect();
        let cells = content
            .cell_values
            .iter()
            .map(|(at, _)| at)
            .chain(content.cell_styles.iter().map(|(at, _)| at))
            .chain(content.comments.iter().map(|c| &c.cell_ref));
        for (row, col) in cells {
            rows.push(row);
            cols.push(col);
        }
        rows.sort();
        rows.dedup();
        cols.sort();
        cols.dedup();
        for key in rows {
            sheet.index.rows.insert_key_at(key.clone(), ts.hlc);
        }
        for key in cols {
            sheet.index.cols.insert_key_at(key.clone(), ts.hlc);
        }

        for (key, state) in &content.rows {
            self.seed_row(i, key, state, ts);
        }
        for (span, state) in &content.columns {
            self.seed_column(i, span, state, ts);
        }
        for (at, value) in &content.cell_values {
            self.workbook.worksheets[i]
                .index
                .registers
                .cell_values
                .insert(at.clone(), *ts);
            self.write_cell(i, at, Some(value));
        }
        for (at, style) in &content.cell_styles {
            let s = self.intern_style(style);
            let sheet = &mut self.workbook.worksheets[i];
            for kind in set_kinds(style) {
                sheet
                    .index
                    .registers
                    .cell_styles
                    .insert((at.clone(), kind), *ts);
            }
            match sheet
                .sheet_data
                .get_mut(&at.0)
                .and_then(|r| r.get_mut(&at.1))
            {
                Some(cell) => cell.set_style(s),
                None => put_cell(sheet, at, Cell::EmptyCell { s }),
            }
        }

        let sheet = &mut self.workbook.worksheets[i];
        for range in &content.merge_cells {
            sheet.index.registers.merges.insert(range.clone(), *ts);
            sheet.merged_cells.push(range.clone());
        }
        for comment in &content.comments {
            sheet
                .index
                .registers
                .comments
                .insert(comment.cell_ref.clone(), *ts);
            sheet.comments.push(comment.clone());
        }
        for (key, state) in &content.conditional_formatting {
            let registers = &mut sheet.index.registers;
            registers.cf.insert((key.clone(), CfPropKind::Rule), *ts);
            registers.cf.insert((key.clone(), CfPropKind::Ranges), *ts);
            registers.cf_order.push(key.clone());
            sheet.conditional_formatting.push(ConditionalFormatting {
                ranges: state.ranges.clone(),
                cf_rule: state.rule.clone(),
                priority: 0,
            });
        }
        // The payload is ordered by key, but nothing stops a peer from sending it otherwise.
        sheet.rows.sort_by(|a, b| a.r.cmp(&b.r));
        sheet
            .cols
            .sort_by(|a, b| (&a.min, &a.max).cmp(&(&b.min, &b.max)));
        sort_cf(sheet);
    }

    fn seed_row(&mut self, i: usize, key: &FractionalKey, state: &RowState, ts: &Timestamp) {
        let s = match &state.style {
            Some(style) => self.intern_style(style),
            None => 0,
        };
        let sheet = &mut self.workbook.worksheets[i];
        // Only the registers the seed actually sets: a default height is not a write, so a snapshot
        // never restores one as custom.
        if state.custom_height {
            sheet
                .index
                .registers
                .rows
                .insert((key.clone(), PropKind::Height), *ts);
        }
        if state.hidden {
            sheet
                .index
                .registers
                .rows
                .insert((key.clone(), PropKind::Hidden), *ts);
        }
        if let Some(style) = &state.style {
            for kind in set_kinds(style) {
                sheet.index.registers.rows.insert((key.clone(), kind), *ts);
            }
        }
        sheet.rows.push(Row {
            r: key.clone(),
            height: state.height,
            custom_format: state.custom_format,
            custom_height: state.custom_height,
            s,
            hidden: state.hidden,
        });
    }

    fn seed_column(
        &mut self,
        i: usize,
        span: &(FractionalKey, FractionalKey),
        state: &ColState,
        ts: &Timestamp,
    ) {
        let style = state.style.as_ref().map(|style| self.intern_style(style));
        let sheet = &mut self.workbook.worksheets[i];
        if state.custom_width {
            sheet
                .index
                .registers
                .col_spans
                .insert((span.clone(), PropKind::Width), *ts);
        }
        if state.hidden {
            sheet
                .index
                .registers
                .col_spans
                .insert((span.clone(), PropKind::Hidden), *ts);
        }
        if let Some(style) = &state.style {
            for kind in set_kinds(style) {
                sheet
                    .index
                    .registers
                    .col_spans
                    .insert((span.clone(), kind), *ts);
            }
        }
        sheet.cols.push(Col {
            min: span.0.clone(),
            max: span.1.clone(),
            width: state.width,
            custom_width: state.custom_width,
            hidden: state.hidden,
            style,
        });
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::cf_types::CfRule;
    use crate::collab::fractional_index::virtual_key;
    use crate::collab::patch::{DefinedNameBody, NamedStyle};
    use crate::types::{Color, Comment, Position, Theme};

    /// Method-syntax delivery, chainable straight off [`rec`].
    trait Deliver {
        fn deliver(&self, model: &mut CollabModel<'_>);
    }

    impl Deliver for Commit {
        fn deliver(&self, model: &mut CollabModel<'_>) {
            model.apply(self).unwrap();
        }
    }

    /// A key some session minted: a position, then the session suffix.
    fn minted(position: &[u8], session: u8) -> FractionalKey {
        let mut bytes = position.to_vec();
        bytes.extend_from_slice(&[0, 0, 0, session]);
        FractionalKey::try_from_bytes(&bytes).unwrap()
    }

    /// A wall-clock millisecond in the past — Sept 2020, comfortably below the Nov 2022 constant
    /// [`crate::mock_time`] serves under `cfg(test)`. Fixed rather than read off the clock:
    /// `Consumer::apply` opens with `Hlc::sync`, so a stamp in the future would become the
    /// process-global high watermark and drag every concurrent test's `Hlc::now()` with it. A stamp
    /// from the past is no watermark at all.
    const PAST: Hlc = Hlc::new(1_600_000_000_000 << 16);

    /// A stamp `counter` steps into the millisecond after [`PAST`]: above anything stamped `PAST`,
    /// and sharing a millisecond with its siblings, so only the counter and session separate them.
    fn same_ms(counter: u64) -> Hlc {
        Hlc::new(PAST.get() + (1 << 16) + counter)
    }

    fn styled() -> Style {
        Style {
            quote_prefix: true,
            ..Default::default()
        }
    }

    /// A named style is defined by its formatting categories, never by a quote prefix.
    fn named_styled() -> Style {
        Style {
            font: crate::types::Font {
                b: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn cf_rule(formula: &str) -> CfRule {
        CfRule::Formula {
            formula: formula.to_string(),
            dxf_id: 0,
            stop_if_true: false,
        }
    }

    const SHEET: SheetId = 42;

    fn cell(model: &CollabModel<'_>, row: &FractionalKey, col: &FractionalKey) -> Option<Cell> {
        let sheet = model.workbook.worksheets.first()?;
        sheet.sheet_data.get(row)?.get(col).cloned()
    }

    /// A sheet with three rows and three columns, as commit 1.
    fn genesis(rows: &[FractionalKey], cols: &[FractionalKey]) -> Commit {
        Commit::new(
            1,
            Hlc::now(),
            vec![
                Patch::AddSheet {
                    id: SHEET,
                    name: "Sheet1".to_string(),
                    position: minted(&[0x10], 1),
                    content: None,
                },
                Patch::InsertRows {
                    sheet: SHEET,
                    keys: rows.to_vec(),
                },
                Patch::InsertColumns {
                    sheet: SHEET,
                    keys: cols.to_vec(),
                },
            ],
        )
    }

    #[test]
    fn apply_registers() {
        let rows: Vec<FractionalKey> = (1..=3).map(virtual_key).collect();
        let cols: Vec<FractionalKey> = (1..=3).map(virtual_key).collect();
        let mut model = CollabModel::new(1);

        let genesis = genesis(&rows, &cols);
        genesis.deliver(&mut model);
        assert_eq!(model.workbook.worksheets.len(), 1);
        assert_eq!(model.workbook.worksheets[0].name, "Sheet1");
        assert_eq!(model.workbook.worksheets[0].sheet_id, SHEET);
        assert_eq!(model.workbook.worksheets[0].index.rows.len(), 3);
        assert_eq!(model.workbook.worksheets[0].index.cols.len(), 3);

        // ---- cells: a literal, a string, a formula, and a clear ----
        let formula = model.bind_text(0, 2, 1, "A1*2");
        let values = Commit::new(
            1,
            Hlc::now(),
            vec![
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    value: Some(CellInput::Number(41.0)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[1].clone()),
                    value: Some(CellInput::Text("hello".to_string())),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[1].clone(), cols[0].clone()),
                    value: Some(CellInput::Formula(formula)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[2].clone(), cols[0].clone()),
                    value: Some(CellInput::Boolean(true)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[2].clone(), cols[0].clone()),
                    value: None,
                    ts: None,
                    prev: Box::default(),
                },
            ],
        );
        values.deliver(&mut model);
        assert_eq!(
            cell(&model, &rows[0], &cols[0]),
            Some(Cell::NumberCell { v: 41.0, s: 0 })
        );
        assert_eq!(
            cell(&model, &rows[0], &cols[1]),
            Some(Cell::SharedString { si: 0, s: 0 })
        );
        assert_eq!(model.workbook.shared_strings, ["hello"]);
        assert_eq!(model.shared_strings.get("hello"), Some(&0));
        assert_eq!(
            cell(&model, &rows[1], &cols[0]),
            Some(Cell::CellFormula {
                f: 0,
                s: 0,
                v: FormulaValue::Unevaluated
            })
        );
        // Interned once, and shown back as what was authored.
        assert_eq!(model.workbook.worksheets[0].shared_formulas.len(), 1);
        assert_eq!(
            model.get_cell_formula(0, 2, 1),
            Ok(Some("=A1*2".to_string()))
        );
        // The last write of a commit is the one that stands, so the cell is gone.
        assert_eq!(cell(&model, &rows[2], &cols[0]), None);

        // ---- every other register kind ----
        let merged = StableRange {
            rows: Some((rows[0].clone(), rows[1].clone())),
            cols: Some((cols[0].clone(), cols[1].clone())),
        };
        let comment = Comment::<Stable> {
            text: "look".to_string(),
            author_name: "me".to_string(),
            author_id: None,
            cell_ref: (rows[1].clone(), cols[1].clone()),
        };
        let cf_key = minted(&[0x20], 1);
        let name_body = DefinedNameBody {
            formula: model.bind_text(0, 1, 1, "Sheet1!$A$1"),
            equals: false,
        };
        let rest = Commit::new(
            1,
            Hlc::now(),
            vec![
                Patch::SetCellStyle {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    props: vec![Property::QuotePrefix(true)],
                    ts: None,
                    prev: Vec::new(),
                },
                Patch::SetRowProperty {
                    sheet: SHEET,
                    row: rows[0].clone(),
                    props: vec![Property::Height(33.0)],
                    ts: None,
                    prev: Vec::new(),
                },
                Patch::SetColumnSpan {
                    sheet: SHEET,
                    span: (cols[0].clone(), cols[1].clone()),
                    props: vec![Property::Width(120.0)],
                    ts: None,
                    prev: Vec::new(),
                },
                // The whole-sheet span: the register `(NULL, NULL)` names and ordinal code cannot.
                Patch::SetColumnSpan {
                    sheet: SHEET,
                    span: (FractionalKey::NULL, FractionalKey::NULL),
                    props: vec![Property::Hidden(true)],
                    ts: None,
                    prev: Vec::new(),
                },
                Patch::SetSheetProperty {
                    sheet: SHEET,
                    property: SheetProperty::Name("Renamed".to_string()),
                    prev: None,
                },
                Patch::SetSheetProperty {
                    sheet: SHEET,
                    property: SheetProperty::Color(Color::Rgb("#ff0000".to_string())),
                    prev: None,
                },
                Patch::SetMergedRange {
                    sheet: SHEET,
                    range: merged.clone(),
                    merged: true,
                    prev: false,
                },
                Patch::SetComment {
                    sheet: SHEET,
                    at: comment.cell_ref.clone(),
                    comment: Some(comment.clone()),
                    prev: None,
                },
                Patch::SetDefinedName {
                    id: 3,
                    property: DefinedNameProperty::Name((None, "total".to_string())),
                    prev: None,
                },
                Patch::SetDefinedName {
                    id: 3,
                    property: DefinedNameProperty::Definition(Some(name_body)),
                    prev: None,
                },
                Patch::SetNamedStyle {
                    id: 7,
                    property: NamedStyleProperty::Definition(Some(Box::new(NamedStyle {
                        style: named_styled(),
                        builtin_id: 0,
                    }))),
                    prev: None,
                },
                Patch::SetNamedStyle {
                    id: 7,
                    property: NamedStyleProperty::Name("Good".to_string()),
                    prev: None,
                },
                Patch::SetWorkbookProperty {
                    property: WorkbookProperty::Locale("es".to_string()),
                    prev: None,
                },
                Patch::SetWorkbookProperty {
                    property: WorkbookProperty::Theme(Box::new(Theme {
                        name: "Dark".to_string(),
                        ..Default::default()
                    })),
                    prev: None,
                },
                Patch::AddConditionalFormat {
                    sheet: SHEET,
                    key: cf_key.clone(),
                    rule: Box::new(cf_rule("A1>0")),
                    ranges: vec![merged.clone()],
                },
                Patch::SetConditionalFormat {
                    sheet: SHEET,
                    key: cf_key.clone(),
                    property: CfProperty::Rule(Box::new(cf_rule("A1>5"))),
                    prev: None,
                },
            ],
        );
        rest.deliver(&mut model);

        let sheet = &model.workbook.worksheets[0];
        let style_index = cell(&model, &rows[0], &cols[0]).unwrap().get_style();
        assert_ne!(style_index, 0);
        assert_eq!(
            model.workbook.styles.get_style(style_index).unwrap(),
            styled()
        );
        assert_eq!(sheet.rows.len(), 1);
        assert_eq!(sheet.rows[0].r, rows[0]);
        assert_eq!(sheet.rows[0].height, 33.0);
        assert!(sheet.rows[0].custom_height);
        // Records are kept sorted by span, so the open-ended one comes first.
        assert_eq!(sheet.cols.len(), 2);
        assert_eq!(sheet.cols[1].width, 120.0);
        assert_eq!(sheet.cols[0].min, FractionalKey::NULL);
        assert!(sheet.cols[0].hidden);
        assert_eq!(sheet.cols[0].resolve(&sheet.index), Some((1, 3)));
        assert_eq!(sheet.name, "Renamed");
        assert_eq!(sheet.color, Color::Rgb("#ff0000".to_string()));
        assert_eq!(sheet.merged_cells, vec![merged.clone()]);
        assert_eq!(sheet.comments, vec![comment.clone()]);
        assert_eq!(sheet.conditional_formatting.len(), 1);
        assert_eq!(sheet.conditional_formatting[0].cf_rule, cf_rule("A1>5"));
        assert_eq!(sheet.conditional_formatting[0].ranges, vec![merged.clone()]);
        assert_eq!(sheet.conditional_formatting[0].priority, 1);
        assert_eq!(sheet.index.registers.cf_order, vec![cf_key.clone()]);
        assert_eq!(model.workbook.defined_names.len(), 1);
        // The body is stored bound, so it shows the sheet's *current* name with nothing rewritten.
        assert_eq!(model.workbook.defined_names[0].formula, "Renamed!$A$1");
        assert_eq!(model.workbook.settings.locale, "es");
        assert_eq!(model.workbook.theme.name, "Dark");
        let named = model
            .workbook
            .styles
            .get_or_create_style_index_by_name("Good")
            .unwrap();
        assert_eq!(
            model.workbook.styles.get_style(named).unwrap(),
            named_styled()
        );

        // ---- redelivery changes nothing ----
        let before = model.workbook.clone();
        rest.deliver(&mut model);
        assert_eq!(model.workbook, before);

        // ---- a write stamped below the guard is ignored ----
        Commit::new(
            1,
            Hlc::new(1),
            vec![
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    value: Some(CellInput::Number(0.0)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetSheetProperty {
                    sheet: SHEET,
                    property: SheetProperty::Name("Stale".to_string()),
                    prev: None,
                },
            ],
        )
        .deliver(&mut model);
        assert_eq!(
            cell(&model, &rows[0], &cols[0]),
            Some(Cell::NumberCell {
                v: 41.0,
                s: style_index
            })
        );
        assert_eq!(model.workbook.worksheets[0].name, "Renamed");

        // ---- a patch for a sheet nobody has is a no-op, not an error ----
        let before = model.workbook.clone();
        Commit::new(
            1,
            Hlc::now(),
            vec![Patch::SetCellValue {
                sheet: 999,
                at: (rows[0].clone(), cols[0].clone()),
                value: Some(CellInput::Number(1.0)),
                ts: None,
                prev: Box::default(),
            }],
        )
        .deliver(&mut model);
        assert_eq!(model.workbook, before);
    }

    #[test]
    fn apply_structural() {
        // Minted keys and virtual ones side by side: both are just keys to the index.
        let rows = vec![virtual_key(1), virtual_key(2), minted(&[0x00, 0x06], 1)];
        let cols = vec![virtual_key(1), virtual_key(2)];
        let mut log = vec![genesis(&rows, &cols)];

        let mut patches = Vec::new();
        for row in &rows {
            for col in &cols {
                patches.push(Patch::SetCellValue {
                    sheet: SHEET,
                    at: (row.clone(), col.clone()),
                    value: Some(CellInput::Text(format!("{row:?}/{col:?}"))),
                    ts: None,
                    prev: Box::default(),
                });
            }
        }
        log.push(Commit::new(1, Hlc::now(), patches));

        // A move is a pair: the identity that moves, and the destination its author minted.
        let dest = minted(&[0x00, 0x09], 1);
        log.push(Commit::new(
            1,
            Hlc::now(),
            vec![Patch::MoveRows {
                sheet: SHEET,
                moves: vec![(rows[0].clone(), dest.clone())],
                prev: vec![],
            }],
        ));
        log.push(Commit::new(
            1,
            Hlc::now(),
            vec![
                Patch::DeleteRows {
                    sheet: SHEET,
                    keys: vec![rows[1].clone()],
                    prev: Vec::new(),
                },
                Patch::DeleteColumns {
                    sheet: SHEET,
                    keys: vec![cols[1].clone()],
                    prev: Vec::new(),
                },
            ],
        ));

        let mut model = CollabModel::new(1);
        for rec in &log {
            rec.deliver(&mut model);
        }

        let sheet = &model.workbook.worksheets[0];
        // The moved row answers to the key it was minted as, at its new position.
        assert_eq!(Stable::row_ordinal(&sheet.index, &rows[0]), Some(2));
        assert_eq!(Stable::row_ordinal(&sheet.index, &rows[2]), Some(1));
        // ...and its cells never moved: they are filed under identity, not position.
        assert!(sheet.sheet_data.contains_key(&rows[0]));
        assert_eq!(sheet.sheet_data[&rows[0]].len(), 1);
        // The deleted row and column took their cells with them.
        assert_eq!(Stable::row_ordinal(&sheet.index, &rows[1]), None);
        assert!(!sheet.sheet_data.contains_key(&rows[1]));
        assert!(sheet
            .sheet_data
            .values()
            .all(|row| !row.contains_key(&cols[1])));
        assert_eq!(sheet.index.rows.len(), 2);
        assert_eq!(sheet.index.cols.len(), 1);

        // The identical sequence lands on the identical workbook — indexes and registers included.
        let mut peer = CollabModel::new(2);
        for rec in &log {
            rec.deliver(&mut peer);
        }
        assert_eq!(peer.workbook, model.workbook);
    }

    #[test]
    fn convergence_smoke() {
        let rows: Vec<FractionalKey> = (1..=3).map(virtual_key).collect();
        let cols: Vec<FractionalKey> = (1..=2).map(virtual_key).collect();
        let doomed: SheetId = 7;

        // The whole scenario is stamped from the fixed past base, so the commits below keep their
        // relative order while none of them touches the process-global clock.
        let root = Commit::new(
            1,
            PAST,
            vec![
                Patch::AddSheet {
                    id: SHEET,
                    name: "Sheet1".to_string(),
                    position: minted(&[0x10], 1),
                    content: None,
                },
                Patch::InsertRows {
                    sheet: SHEET,
                    keys: rows.clone(),
                },
                Patch::InsertColumns {
                    sheet: SHEET,
                    keys: cols.clone(),
                },
                Patch::AddSheet {
                    id: doomed,
                    name: "Doomed".to_string(),
                    position: minted(&[0x20], 1),
                    content: None,
                },
                Patch::InsertRows {
                    sheet: doomed,
                    keys: rows.clone(),
                },
                Patch::InsertColumns {
                    sheet: doomed,
                    keys: cols.clone(),
                },
            ],
        );

        // Everything below is concurrent: same parent, stamps inside one wall millisecond, so only
        // the HLC counter and the session tiebreak separate them.
        let from_a = Commit::new(
            1,
            same_ms(1),
            vec![
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    value: Some(CellInput::Number(1.0)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::MoveRows {
                    sheet: SHEET,
                    moves: vec![(rows[0].clone(), minted(&[0x00, 0x09], 1))],
                    prev: vec![],
                },
                Patch::SetSheetProperty {
                    sheet: SHEET,
                    property: SheetProperty::Name("From A".to_string()),
                    prev: None,
                },
                // Racing the delete below. A literal, not a string: the intern table is local, so a
                // peer that never saw the write would legitimately lack the entry.
                Patch::SetCellValue {
                    sheet: doomed,
                    at: (rows[1].clone(), cols[0].clone()),
                    value: Some(CellInput::Number(99.0)),
                    ts: None,
                    prev: Box::default(),
                },
            ],
        );
        let from_b = Commit::new(
            2,
            same_ms(1),
            vec![
                // Same register, same stamp: the session breaks the tie, identically everywhere.
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    value: Some(CellInput::Number(2.0)),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::MoveRows {
                    sheet: SHEET,
                    moves: vec![(rows[0].clone(), minted(&[0x00, 0x03], 2))],
                    prev: vec![],
                },
                Patch::SetSheetProperty {
                    sheet: SHEET,
                    property: SheetProperty::Name("From B".to_string()),
                    prev: None,
                },
                Patch::DeleteSheet {
                    sheet: doomed,
                    prev: None,
                },
            ],
        );

        // A third concurrent write, one HLC step further: the counter alone puts it above both,
        // low session id notwithstanding.
        let later = Commit::new(
            1,
            same_ms(2),
            vec![Patch::SetSheetProperty {
                sheet: SHEET,
                property: SheetProperty::Name("Later".to_string()),
                prev: None,
            }],
        );

        let mut a = CollabModel::new(1);
        let mut b = CollabModel::new(2);
        root.deliver(&mut a);
        root.deliver(&mut b);
        // Causally respected, but the concurrent commits arrive in different orders.
        from_a.deliver(&mut a);
        later.deliver(&mut a);
        from_b.deliver(&mut a);
        from_b.deliver(&mut b);
        from_a.deliver(&mut b);
        later.deliver(&mut b);

        assert_eq!(a.workbook, b.workbook);
        // The cell tie went to the higher session...
        assert_eq!(
            cell(&a, &rows[0], &cols[0]),
            Some(Cell::NumberCell { v: 2.0, s: 0 })
        );
        // ...but the sheet name went to the highest counter in that millisecond.
        assert_eq!(a.workbook.worksheets[0].name, "Later");
        // One row moved twice, still exactly one row.
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 3);
        assert_eq!(
            a.workbook.worksheets[0]
                .index
                .rows
                .view()
                .filter(|k| **k == rows[0])
                .count(),
            1
        );
        // The delete wins over the write to the sheet it removed, whichever arrived first.
        assert_eq!(a.workbook.worksheets.len(), 1);
        assert!(a.workbook.meta.sheet_existence.contains_key(&doomed));
    }

    /// Overlapping spans and reordered rules: two things only concurrency can produce, and both
    /// have to read the same on every replica whatever order the commits arrived in.
    #[test]
    fn overlap_and_priority() {
        let rows: Vec<FractionalKey> = (1..=2).map(virtual_key).collect();
        let cols: Vec<FractionalKey> = (1..=5).map(virtual_key).collect();
        let (first, second) = (minted(&[0x20], 1), minted(&[0x30], 1));

        let root = Commit::new(1, PAST, genesis(&rows, &cols).patches);
        // A wide span from one replica, a narrower one from another, one HLC step later.
        let wide = Commit::new(
            1,
            same_ms(1),
            vec![
                Patch::SetColumnSpan {
                    sheet: SHEET,
                    span: (cols[0].clone(), cols[3].clone()),
                    props: vec![Property::Width(10.0)],
                    ts: None,
                    prev: Vec::new(),
                },
                Patch::SetColumnSpan {
                    sheet: SHEET,
                    span: (cols[0].clone(), cols[3].clone()),
                    props: vec![Property::Hidden(true)],
                    ts: None,
                    prev: Vec::new(),
                },
                Patch::AddConditionalFormat {
                    sheet: SHEET,
                    key: first.clone(),
                    rule: Box::new(cf_rule("A1>0")),
                    ranges: vec![],
                },
                Patch::AddConditionalFormat {
                    sheet: SHEET,
                    key: second.clone(),
                    rule: Box::new(cf_rule("A1>1")),
                    ranges: vec![],
                },
            ],
        );
        let narrow = Commit::new(
            2,
            same_ms(2),
            vec![
                Patch::SetColumnSpan {
                    sheet: SHEET,
                    span: (cols[1].clone(), cols[1].clone()),
                    props: vec![Property::Width(20.0)],
                    ts: None,
                    prev: Vec::new(),
                },
                // The second rule takes a position below the first one's identity key.
                Patch::SetConditionalFormat {
                    sheet: SHEET,
                    key: second.clone(),
                    property: CfProperty::Priority(minted(&[0x10], 2)),
                    prev: None,
                },
            ],
        );

        let mut a = CollabModel::new(1);
        let mut b = CollabModel::new(2);
        for rec in [&root, &wide, &narrow] {
            rec.deliver(&mut a);
        }
        for rec in [&root, &narrow, &wide] {
            rec.deliver(&mut b);
        }
        assert_eq!(a.workbook, b.workbook);

        // The newest span covering a position wins it, per position and per property kind. Columns
        // 1..4 are hidden — which a width read reports as 0 — so the register is what has to be
        // compared to see which span won the width.
        let width = |m: &CollabModel, column: i32| {
            m.workbook.worksheets[0]
                .covering_col(column, PropKind::Width)
                .map(|col| col.width)
        };
        assert_eq!(width(&a, 1), width(&a, 3));
        assert_ne!(width(&a, 1), width(&a, 2));
        assert_eq!(a.get_column_width(0, 5).unwrap(), DEFAULT_COLUMN_WIDTH);
        for column in 1..=5 {
            assert_eq!(width(&a, column), width(&b, column));
            assert_eq!(
                a.is_column_hidden(0, column).unwrap(),
                b.is_column_hidden(0, column).unwrap()
            );
        }
        // The narrower span wrote no `Hidden` register, so the wide one still owns that property.
        assert!(a.is_column_hidden(0, 2).unwrap());
        assert!(!a.is_column_hidden(0, 5).unwrap());

        // The moved rule sorts first, and priorities are renumbered to storage order.
        let sheet = &a.workbook.worksheets[0];
        assert_eq!(sheet.index.registers.cf_order, vec![second, first]);
        assert_eq!(sheet.conditional_formatting[0].cf_rule, cf_rule("A1>1"));
        let priorities: Vec<u32> = sheet
            .conditional_formatting
            .iter()
            .map(|cf| cf.priority)
            .collect();
        assert_eq!(priorities, vec![1, 2]);
        assert_eq!(
            b.workbook.worksheets[0].index.registers.cf_order,
            sheet.index.registers.cf_order
        );
    }

    #[test]
    fn snapshot_round_trip() {
        let rows: Vec<FractionalKey> = (1..=3).map(virtual_key).collect();
        let cols: Vec<FractionalKey> = (1..=2).map(virtual_key).collect();
        let mut model = CollabModel::new(1);
        genesis(&rows, &cols).deliver(&mut model);
        Commit::new(
            1,
            Hlc::now(),
            vec![
                Patch::SetCellValue {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    value: Some(CellInput::Text("kept".to_string())),
                    ts: None,
                    prev: Box::default(),
                },
                Patch::SetCellStyle {
                    sheet: SHEET,
                    at: (rows[0].clone(), cols[0].clone()),
                    props: vec![Property::QuotePrefix(true)],
                    ts: None,
                    prev: Vec::new(),
                },
            ],
        )
        .deliver(&mut model);

        // A third replica picks the snapshot up, so the keys it mints are its own.
        let restored = CollabModel::decode(&model.encode(), 3).unwrap();
        assert_eq!(restored.workbook, model.workbook);
        assert_eq!(restored.shared_strings, model.shared_strings);
        assert_eq!(
            restored.workbook.worksheets[0].index.rows.suffix,
            3u32.to_be_bytes()
        );

        // ...and it carries on from there.
        let next = Commit::new(
            3,
            Hlc::now(),
            vec![Patch::SetCellValue {
                sheet: SHEET,
                at: (rows[1].clone(), cols[1].clone()),
                value: Some(CellInput::Number(7.0)),
                ts: None,
                prev: Box::default(),
            }],
        );
        let mut restored = restored;
        next.deliver(&mut restored);
        next.deliver(&mut model);
        assert_eq!(restored.workbook, model.workbook);
    }
}
