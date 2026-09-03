//! Local edits: a mutator call turned into patches.
//!
//! Every mutator validates first — ordinal semantics, so a bad row or a bad name errors before a
//! single patch exists — then resolves ordinals to keys, materializing missing rows and columns as
//! `InsertRows`/`InsertColumns` in the same commit as the write, and hands the lot to
//! [`CollabModel::commit_local`]. There is no second path into the document: for [`Stable`] every
//! mutation is a patch.

use crate::cf_types::{CfRuleInput, ConditionalFormattingView};
use crate::collab::apply::SHEET_NAMES;
use crate::collab::bind::MintPlan;
#[cfg(test)]
use crate::collab::formula::StableFormula;
use crate::collab::fractional_index::{CreateKeys, FractionalIndex, FractionalKey, KeyBuf};
use crate::collab::hlc::Hlc;
use crate::collab::log::Timestamp;
use crate::collab::model::{CollabModel, LocalCommit, Stable, StableCellAddress, StableRange};
use crate::collab::naming::{defined_name_id, stable_id};
use crate::collab::patch::{
    CellInput, CfProperty, ColPropKind, ColProperty, ColState, ColumnSnapshot,
    ConditionalFormatState, DefinedNameBody, DefinedNameId, DefinedNameProperty, NamedStyle,
    NamedStyleId, NamedStyleProperty, Patch, RowPropKind, RowProperty, RowSnapshot, RowState,
    SheetContent, SheetId, SheetPropKind, SheetProperty, SheetRestore, WorkbookPropKind,
    WorkbookProperty,
};
use crate::constants::{
    COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, LAST_COLUMN, LAST_ROW,
    ROW_HEIGHT_FACTOR,
};
use crate::expressions::parser::Node;
use crate::expressions::token::get_error_by_name;
use crate::expressions::types::Area;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::expressions::utils::{is_valid_column_number, is_valid_identifier, is_valid_row};
use crate::formatter::format::parse_formatted_number;
use crate::formatter::lexer::is_likely_date_number_format;
use crate::language::get_default_language;
use crate::locale::{get_default_locale, get_locale};
use crate::new_empty::is_valid_sheet_name;
use crate::types::{Cell, Col, Color, Comment, Dxf, Position, RangeRef, SheetState, Style, Theme};
use crate::tz::Tz;
use crate::utils as common;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl CollabModel<'_> {
    /// Applies `patches` as one commit and queues it for the log.
    ///
    /// One mutator call is one commit, minting one [`Hlc`]: every register it writes ends up with
    /// the same [`Timestamp`], which is what keeps the author's guard table equal to its peers'.
    pub(crate) fn commit_local(&mut self, patches: Vec<Patch>) {
        let hlc = Hlc::now();
        let ts = Timestamp::new(hlc, self.local.session);
        for patch in &patches {
            self.apply_patch(patch, &ts);
        }
        self.resync_derived(&patches);
        self.local.pending.push(LocalCommit { hlc, patches });
    }

    /// Takes the commits produced since the last call, for the framework to transport.
    pub fn flush(&mut self) -> Vec<LocalCommit> {
        std::mem::take(&mut self.local.pending)
    }

    /// The id of the sheet at index `sheet`.
    fn sheet_of(&self, sheet: u32) -> Result<SheetId, String> {
        Ok(self.workbook.worksheet(sheet)?.sheet_id)
    }

    /// A fresh sheet id, hashed from this replica's session and a slot past every id it has seen:
    /// session-dependent, so two peers creating a sheet concurrently do not mint the same id.
    /// Existence guards outlive their sheet, so a deleted id is never handed out twice.
    fn new_sheet_id(&self) -> SheetId {
        let seen = |id: &SheetId| {
            self.workbook.worksheets.iter().any(|ws| &ws.sheet_id == id)
                || self.workbook.meta.sheet_existence.contains_key(id)
        };
        let mut slot = u64::from(
            self.workbook
                .worksheets
                .iter()
                .map(|ws| ws.sheet_id)
                .chain(self.workbook.meta.sheet_existence.keys().copied())
                .max()
                .unwrap_or(0),
        ) + 1;
        loop {
            let mut hasher = DefaultHasher::new();
            (self.local.session, slot).hash(&mut hasher);
            let id = hasher.finish() as SheetId;
            // Never 0: the ordinal model starts its ids at 1 and readers may treat 0 as "no sheet".
            if id != 0 && !seen(&id) {
                return id;
            }
            slot += 1;
        }
    }

    /// A tab-order key past every sheet this replica has seen. The session suffix separates two
    /// replicas appending concurrently, so the order stays total.
    fn sheet_position(&self) -> FractionalKey {
        let slot = self.workbook.meta.sheet_existence.len() as u32 + 1;
        let mut buf = KeyBuf::from(&(2 * slot).to_be_bytes()[1..]);
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).expect("position and suffix are 7 bytes")
    }

    fn sheet_ordering_key(&self, at: usize) -> Option<&FractionalKey> {
        let sheet_id = &self.workbook.worksheets.get(at)?.sheet_id;
        Some(&self.workbook.meta.sheet_positions.get(sheet_id)?.value)
    }

    /// A tab-order key landing a sheet at index `at`, between the sheets it comes to sit among.
    fn insert_position(&self, at: usize) -> Result<FractionalKey, String> {
        if at >= self.workbook.worksheets.len() {
            return Ok(self.sheet_position());
        }
        let nil = FractionalKey::NULL;
        let hi = self.sheet_ordering_key(at).unwrap_or(&nil);
        // Inserting at the front has no sheet below it, and NULL is that open lower end.
        let lo = at
            .checked_sub(1)
            .and_then(|below| self.sheet_ordering_key(below))
            .unwrap_or(&nil);
        let no_room = || format!("No room for a sheet at index {at}");
        let (mut buf, _) = CreateKeys::plan(lo.position(), hi.position(), 1).ok_or_else(no_room)?;
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).map_err(|_| no_room())
    }

    /// The key row `row` of worksheet `i` answers to, appending to `patches` whatever has to be
    /// materialized first for it to exist.
    fn row_key(&self, i: usize, id: SheetId, row: i32, patches: &mut Vec<Patch>) -> FractionalKey {
        let index = &self.workbook.worksheets[i].index;
        match Stable::row_at(index, row) {
            Some(key) => key,
            None => {
                let keys = index.rows.plan_virtual(row as usize);
                let key = keys.last().cloned().expect("row is past the index");
                patches.push(Patch::InsertRows { sheet: id, keys });
                key
            }
        }
    }

    /// [`Self::row_key`] for a column.
    fn col_key(
        &self,
        i: usize,
        id: SheetId,
        column: i32,
        patches: &mut Vec<Patch>,
    ) -> FractionalKey {
        let index = &self.workbook.worksheets[i].index;
        match Stable::col_at(index, column) {
            Some(key) => key,
            None => {
                let keys = index.cols.plan_virtual(column as usize);
                let key = keys.last().cloned().expect("column is past the index");
                patches.push(Patch::InsertColumns { sheet: id, keys });
                key
            }
        }
    }

    /// Keys for rows `start..=end`, appending the one `InsertRows` materializing whatever the
    /// index does not hold yet. Planned in one go, so a span reaching past the sheet mints each
    /// missing row exactly once.
    fn row_keys(
        &self,
        i: usize,
        id: SheetId,
        start: i32,
        end: i32,
        patches: &mut Vec<Patch>,
    ) -> Vec<FractionalKey> {
        let index = &self.workbook.worksheets[i].index;
        let len = index.rows.len() as i32;
        let minted = index.rows.plan_virtual(end.max(0) as usize);
        if !minted.is_empty() {
            patches.push(Patch::InsertRows {
                sheet: id,
                keys: minted.clone(),
            });
        }
        (start..=end)
            .map(|row| match Stable::row_at(index, row) {
                Some(key) => key,
                None => minted[(row - len - 1) as usize].clone(),
            })
            .collect()
    }

    /// [`Self::row_keys`] for columns.
    fn col_keys(
        &self,
        i: usize,
        id: SheetId,
        start: i32,
        end: i32,
        patches: &mut Vec<Patch>,
    ) -> Vec<FractionalKey> {
        let index = &self.workbook.worksheets[i].index;
        let len = index.cols.len() as i32;
        let minted = index.cols.plan_virtual(end.max(0) as usize);
        if !minted.is_empty() {
            patches.push(Patch::InsertColumns {
                sheet: id,
                keys: minted.clone(),
            });
        }
        (start..=end)
            .map(|column| match Stable::col_at(index, column) {
                Some(key) => key,
                None => minted[(column - len - 1) as usize].clone(),
            })
            .collect()
    }

    /// Keys for `(row, column)` on worksheet `i`. If row/column IDs have to be created (via virtual
    /// fractional key creation), they will be returned as vec of patches with the result.
    fn resolve_cell(
        &self,
        i: usize,
        id: SheetId,
        row: i32,
        column: i32,
    ) -> (StableCellAddress, Vec<Patch>) {
        let mut patches = Vec::new();
        let row_key = self.row_key(i, id, row, &mut patches);
        let col_key = self.col_key(i, id, column, &mut patches);
        ((row_key, col_key), patches)
    }

    /// The stable twin of an ordinal rectangle, materializing whatever it names. An unbounded axis
    /// stays unbounded: it tracks the sheet rather than a pair of corners.
    fn stable_range(
        &self,
        i: usize,
        id: SheetId,
        range: &RangeRef,
        patches: &mut Vec<Patch>,
    ) -> StableRange {
        let rows = range.rows.map(|(a, b)| {
            (
                self.row_key(i, id, a, patches),
                self.row_key(i, id, b, patches),
            )
        });
        let cols = range.cols.map(|(a, b)| {
            (
                self.col_key(i, id, a, patches),
                self.col_key(i, id, b, patches),
            )
        });
        StableRange { rows, cols }
    }

    /// [`Self::stable_range`] without minting: `None` when a corner is not materialized, and so
    /// names nothing the document could already hold.
    fn resolved_range(&self, i: usize, range: &RangeRef) -> Option<StableRange> {
        let index = &self.workbook.worksheets[i].index;
        let rows = match range.rows {
            Some((a, b)) => Some((Stable::row_at(index, a)?, Stable::row_at(index, b)?)),
            None => None,
        };
        let cols = match range.cols {
            Some((a, b)) => Some((Stable::col_at(index, a)?, Stable::col_at(index, b)?)),
            None => None,
        };
        Some(StableRange { rows, cols })
    }

    /// The row register's current value for `kind`, as the property a write would replace. A row
    /// with no record — or no key at all, being unmaterialized — holds the defaults, which is what
    /// undoing a first write has to put back.
    fn row_prev(
        &self,
        i: usize,
        key: Option<&FractionalKey>,
        kind: RowPropKind,
    ) -> Option<RowProperty> {
        let record = match key {
            None => None,
            Some(key) => self.workbook.worksheets[i]
                .rows
                .iter()
                .find(|r| &r.r == key),
        };
        Some(match kind {
            RowPropKind::Style => RowProperty::Style(
                record
                    .filter(|r| r.custom_format)
                    .and_then(|r| self.workbook.styles.get_style(r.s).ok())
                    .map(Box::new),
            ),
            RowPropKind::Height => RowProperty::Height(
                record.map_or(DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR, |r| r.height),
            ),
            RowPropKind::Hidden => RowProperty::Hidden(record.is_some_and(|r| r.hidden)),
        })
    }

    /// [`Self::row_prev`] for the column span `(key, key)`. Wider spans covering the same column
    /// are not consulted: a point write replaces only the point register.
    fn col_prev(&self, i: usize, key: &FractionalKey, kind: ColPropKind) -> Option<ColProperty> {
        let record = self.workbook.worksheets[i]
            .cols
            .iter()
            .find(|c| &c.min == key && &c.max == key);
        Some(self.col_property(record, kind))
    }

    /// [`Self::col_prev`] as the column reads on screen.
    fn col_effective(&self, i: usize, column: i32, kind: ColPropKind) -> ColProperty {
        let record = self.workbook.worksheets[i].covering_col(column, kind);
        self.col_property(record, kind)
    }

    /// The `kind` property a column record holds; no record means the defaults.
    fn col_property(&self, record: Option<&Col<Stable>>, kind: ColPropKind) -> ColProperty {
        match kind {
            ColPropKind::Style => ColProperty::Style(
                record
                    .and_then(|c| c.style)
                    .and_then(|s| self.workbook.styles.get_style(s).ok())
                    .map(Box::new),
            ),
            ColPropKind::Width => ColProperty::Width(
                record.map_or(DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR, |c| c.width),
            ),
            ColPropKind::Hidden => ColProperty::Hidden(record.is_some_and(|c| c.hidden)),
        }
    }

    /// The name written to a sheet's name register, which its display name is derived from.
    fn authored_name(&self, sheet: SheetId) -> Option<&String> {
        self.workbook
            .meta
            .sheet_names
            .get(&sheet)
            .map(|lww| &lww.value)
    }

    /// The sheet property `kind` currently holds, as the property a write would replace.
    fn sheet_prev(&self, i: usize, kind: SheetPropKind) -> Option<SheetProperty> {
        let sheet = &self.workbook.worksheets[i];
        Some(match kind {
            // The authored name, not the displayed one: undo has to restore what was written.
            SheetPropKind::Name => SheetProperty::Name(self.authored_name(sheet.sheet_id)?.clone()),
            SheetPropKind::Color => SheetProperty::Color(sheet.color.clone()),
            SheetPropKind::State => SheetProperty::State(sheet.state.clone()),
            SheetPropKind::ShowGridLines => SheetProperty::ShowGridLines(sheet.show_grid_lines),
            SheetPropKind::FrozenRows => SheetProperty::FrozenRows(sheet.frozen_rows),
            SheetPropKind::FrozenColumns => SheetProperty::FrozenColumns(sheet.frozen_columns),
            SheetPropKind::Position => SheetProperty::Position(
                self.workbook
                    .meta
                    .sheet_positions
                    .get(&sheet.sheet_id)?
                    .value
                    .clone(),
            ),
        })
    }

    /// What the cell at `at` would be written back as: its authored contents, never its evaluated
    /// value. An array anchor and a spill cell have no input we can reproduce, so they read as
    /// nothing — see [`invert_patches`](crate::collab::patch::Patch).
    fn cell_input(&self, i: usize, at: &StableCellAddress) -> Option<CellInput> {
        let sheet = &self.workbook.worksheets[i];
        let cell = sheet.sheet_data.get(&at.0)?.get(&at.1)?;
        match cell {
            Cell::BooleanCell { v, .. } => Some(CellInput::Boolean(*v)),
            Cell::NumberCell { v, .. } => Some(CellInput::Number(*v)),
            Cell::ErrorCell { ei, .. } => Some(CellInput::Error(ei.clone())),
            Cell::SharedString { si, .. } => self
                .workbook
                .shared_strings
                .get(*si as usize)
                .cloned() //TODO: we can do better than String::clone
                .map(CellInput::Text),
            Cell::CellFormula { f, .. } => sheet
                .shared_formulas
                .get(*f as usize)
                .cloned()
                .map(CellInput::Formula),
            Cell::EmptyCell { .. } | Cell::ArrayFormula { .. } | Cell::SpillCell { .. } => None,
        }
    }

    /// Parses `formula` as the user typed it, anchored at the cell it goes into, retrying with a
    /// closing parenthesis exactly as the ordinal path does.
    fn parse_at(&mut self, i: usize, row: i32, column: i32, formula: &str) -> Node {
        let context = CellReferenceRC {
            sheet: self.workbook.worksheets[i].get_name(),
            row,
            column,
        };
        let node = self.parser.parse(formula, &context);
        if let Node::ParseErrorKind { .. } = node {
            let retry = self.parser.parse(&format!("{formula})"), &context);
            if !matches!(retry, Node::ParseErrorKind { .. }) {
                return retry;
            }
        }
        node
    }

    /// A formula bound as if typed into (`sheet`, `row`, `column`).
    #[cfg(test)]
    pub(crate) fn bind_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> StableFormula {
        let node = self.parse_at(sheet as usize, row, column, formula);
        self.bind_formula(&node, sheet, row, column, &mut MintPlan::default())
            .expect("test formula binds")
    }

    /// The authored contents `value` denotes and the style the ordinal path would leave behind:
    /// the same classification order — quote prefix, formula, number, boolean, error, text.
    fn classify_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
        mut style: Style,
        plan: &mut MintPlan,
    ) -> Result<(Option<CellInput>, Style), String> {
        if value.is_empty() {
            return Ok((None, style));
        }
        if let Some(text) = value.strip_prefix('\'') {
            style.quote_prefix = true;
            return Ok((Some(CellInput::Text(text.to_string())), style));
        }
        style.quote_prefix = false;
        if let Some(formula) = self.formula_without_prefix(value) {
            let formula = formula.to_string();
            let node = self.parse_at(sheet as usize, row, column, &formula);
            let cell = CellReferenceIndex { sheet, row, column };
            if let Some(units) = self.compute_node_units(&node, &cell) {
                style.num_fmt = units.get_num_fmt();
            }
            let bound = self
                .bind_formula(&node, sheet, row, column, plan)
                .map_err(|err| format!("Invalid formula: {err}"))?;
            return Ok((Some(CellInput::Formula(bound)), style));
        }
        // The list of currencies is '$', '€' and the local currency
        let mut currencies = vec!["$", "€"];
        let currency = &self.locale.currency.symbol;
        if !currencies.iter().any(|e| e == currency) {
            currencies.push(currency);
        }
        if let Ok((v, number_format)) = parse_formatted_number(value, &currencies, self.locale) {
            if let Some(num_fmt) = number_format {
                // A date written into an already date-formatted cell keeps the cell's format.
                if !(is_likely_date_number_format(&style.num_fmt)
                    && is_likely_date_number_format(&num_fmt))
                {
                    style.num_fmt = num_fmt;
                }
            }
            return Ok((Some(CellInput::Number(v)), style));
        }
        if let Ok(v) = value.to_lowercase().parse::<bool>() {
            return Ok((Some(CellInput::Boolean(v)), style));
        }
        match get_error_by_name(&value.to_uppercase(), self.language) {
            Some(error) => Ok((Some(CellInput::Error(error)), style)),
            None => Ok((Some(CellInput::Text(value.to_string())), style)),
        }
    }

    /// The patches writing `input` into `(row, column)`, style included, and nothing else.
    ///
    /// A clear takes the cell with it, and the cell style is a register of its own, so the style is
    /// written back to keep the formatting a cleared cell keeps under ordinal addressing.
    fn write_patches(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        input: Option<CellInput>,
        style: Style,
    ) -> Result<Vec<Patch>, String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        match input {
            Some(input) => {
                let prev = self.cell_input(i, &at);
                patches.push(Patch::SetCellValue {
                    sheet: id,
                    at: at.clone(),
                    value: Some(input),
                    ts: None,
                    prev: Box::new(prev),
                });
                let stored = self.get_cell_style_or_none(sheet, row, column)?;
                if Some(&style) != stored.as_ref() {
                    patches.push(Patch::SetCellStyle {
                        sheet: id,
                        at,
                        style: Some(Box::new(style)),
                        ts: None,
                        prev: Box::new(stored),
                    });
                }
                Ok(patches)
            }
            None => {
                // clear cell
                patches.extend(self.clear_patches(i, id, at, false));
                Ok(patches)
            }
        }
    }

    fn cell_style_at(&self, i: usize, at: &StableCellAddress) -> Option<Style> {
        let cell = self.workbook.worksheets[i]
            .sheet_data
            .get(&at.0)?
            .get(&at.1)?;
        self.workbook.styles.get_style(cell.get_style()).ok()
    }

    /// The patches clearing the cell at `at`: the value tombstone, and the style either written
    /// back (the value write takes the cell with it) or cleared alongside it when `all`.
    ///
    /// Both registers are always written, even where there is nothing to clear: the stamps are what
    /// make a concurrent older write to the cell lose whichever order it arrives in.
    fn clear_patches(&self, i: usize, id: SheetId, at: StableCellAddress, all: bool) -> Vec<Patch> {
        let prev = self.cell_input(i, &at);
        let stored = self.cell_style_at(i, &at);
        vec![
            Patch::SetCellValue {
                sheet: id,
                at: at.clone(),
                value: None,
                ts: None,
                prev: Box::new(prev),
            },
            Patch::SetCellStyle {
                sheet: id,
                at,
                style: if all { None } else { stored.clone() }.map(Box::new),
                ts: None,
                prev: Box::new(stored),
            },
        ]
    }

    /// The patches clearing every cell the `area` actually holds, styles included when `all`.
    ///
    /// Only cells that exist are written, so a clear outranks a concurrent older write only where it
    /// could see one; a write to a coordinate absent here has no clear patch to lose to, anywhere.
    fn range_clear_patches(&self, area: &Area, all: bool) -> Result<Vec<Patch>, String> {
        if !self.can_clear_range(area)? {
            return Err("Cannot clear the range because it contains array formulas".to_string());
        }
        let id = self.sheet_of(area.sheet)?;
        let i = area.sheet as usize;
        let sheet = &self.workbook.worksheets[i];
        let mut patches = Vec::new();
        for row in area.row..area.row + area.height {
            let Some(row_key) = Stable::row_at(&sheet.index, row) else {
                continue;
            };
            for column in area.column..area.column + area.width {
                let Some(col_key) = Stable::col_at(&sheet.index, column) else {
                    continue;
                };
                if sheet.cell(row, column).is_none() {
                    continue;
                }
                patches.extend(self.clear_patches(i, id, (row_key.clone(), col_key), all));
            }
        }
        Ok(patches)
    }

    /// Validates that `(sheet, row, column)` is addressable, as the ordinal writers do.
    fn check_cell(&self, sheet: u32, row: i32, column: i32) -> Result<usize, String> {
        self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_row(row) || !is_valid_column_number(column) {
            return Err("Incorrect row or column".to_string());
        }
        Ok(i)
    }
}

/// Document mutation: the collaborative twin of the ordinal writers, patch by patch.
impl CollabModel<'_> {
    /// Adds a sheet with an automatically generated name.
    pub fn new_sheet(&mut self) -> (String, u32) {
        let base_name = self.get_sheet_name();
        let base_name_uppercase = base_name.to_uppercase();
        let mut index = 1;
        while self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == format!("{base_name_uppercase}{index}"))
        {
            index += 1;
        }
        let name = format!("{base_name}{index}");
        let id = self.new_sheet_id();
        self.commit_local(vec![Patch::AddSheet {
            id,
            name: name.clone(),
            position: self.sheet_position(),
            content: None,
        }]);
        let at = self.get_sheet_index_by_sheet_id(id).unwrap_or_default();
        self.evaluate();
        (name, at)
    }

    pub fn add_sheet(&mut self, name: &str) -> Result<(), String> {
        self.insert_sheet(name, self.workbook.worksheets.len() as u32, None)
    }

    pub fn insert_sheet(
        &mut self,
        name: &str,
        sheet_index: u32,
        sheet_id: Option<u32>,
    ) -> Result<(), String> {
        if !is_valid_sheet_name(name) {
            return Err(format!("Invalid name for a sheet: '{name}'"));
        }
        if self
            .workbook
            .get_worksheet_names()
            .iter()
            .map(|s| s.to_uppercase())
            .any(|x| x == name.to_uppercase())
        {
            return Err("A worksheet already exists with that name".to_string());
        }
        if sheet_index as usize > self.workbook.worksheets.len() {
            return Err("Sheet index out of range".to_string());
        }
        let id = match sheet_id {
            // Existence guards outlive their sheet, so a dead id can never be handed out again.
            Some(id)
                if id == 0
                    || self.workbook.worksheets.iter().any(|ws| ws.sheet_id == id)
                    || self.workbook.meta.sheet_existence.contains_key(&id) =>
            {
                return Err(format!("Sheet id {id} is not available"));
            }
            Some(id) => id,
            None => self.new_sheet_id(),
        };
        let position = self.insert_position(sheet_index as usize)?;
        self.commit_local(vec![Patch::AddSheet {
            id,
            name: name.to_string(),
            position,
            content: None,
        }]);
        self.evaluate();
        Ok(())
    }

    /// Sets a cell as if a user had typed `value` into it.
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let style = self.get_style_for_cell(sheet, row, column)?;
        let mut plan = MintPlan::default();
        let (input, style) = self.classify_input(sheet, row, column, &value, style, &mut plan)?;
        // Whatever the formula names has to exist before the write that names it.
        let mut patches = self.mint_patches(&plan);
        patches.extend(self.write_patches(sheet, row, column, input, style)?);
        self.commit_local(patches);
        Ok(())
    }

    /// Clears the contents of a cell, keeping its formatting.
    pub fn cell_clear_contents(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let style = self.get_style_for_cell(sheet, row, column)?;
        let patches = self.write_patches(sheet, row, column, None, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with text, quoting it if it would otherwise read as something else.
    pub fn update_cell_with_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = common::value_needs_quoting(value, self.language);
        let input = Some(CellInput::Text(value.to_string()));
        let patches = self.write_patches(sheet, row, column, input, style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a boolean. The style is unchanged bar the quote prefix.
    pub fn update_cell_with_bool(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: bool,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let patches =
            self.write_patches(sheet, row, column, Some(CellInput::Boolean(value)), style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a number. The style is unchanged bar the quote prefix.
    pub fn update_cell_with_number(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: f64,
    ) -> Result<(), String> {
        self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let patches =
            self.write_patches(sheet, row, column, Some(CellInput::Number(value)), style)?;
        self.commit_local(patches);
        Ok(())
    }

    /// Updates a cell with a formula, which must start with `=`.
    pub fn update_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: String,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let mut style = self.get_style_for_cell(sheet, row, column)?;
        style.quote_prefix = false;
        let Some(body) = self.formula_without_prefix(&formula) else {
            return Err(format!("\"{formula}\" is not a valid formula"));
        };
        let body = body.to_string();
        let node = self.parse_at(i, row, column, &body);
        let mut plan = MintPlan::default();
        let bound = self
            .bind_formula(&node, sheet, row, column, &mut plan)
            .map_err(|err| format!("Invalid formula: {err}"))?;
        let mut patches = self.mint_patches(&plan);
        patches.extend(self.write_patches(
            sheet,
            row,
            column,
            Some(CellInput::Formula(bound)),
            style,
        )?);
        self.commit_local(patches);
        Ok(())
    }

    /// Clears a cell's contents *and* its formatting.
    pub fn cell_clear_all(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let id = self.sheet_of(sheet)?;
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        patches.extend(self.clear_patches(i, id, at, true));
        self.commit_local(patches);
        Ok(())
    }

    /// Removes the content of every cell in the range but leaves the style.
    ///
    /// A coordinate that held nothing stays absent, where upstream materializes an `EmptyCell` for
    /// it: display-identical, but the two models hold different cell counts.
    pub fn range_clear_contents(&mut self, area: &Area) -> Result<(), String> {
        let patches = self.range_clear_patches(area, false)?;
        if !patches.is_empty() {
            self.commit_local(patches);
        }
        Ok(())
    }

    /// Removes both content and style from every cell in the range.
    pub fn range_clear_all(&mut self, area: &Area) -> Result<(), String> {
        let patches = self.range_clear_patches(area, true)?;
        if !patches.is_empty() {
            self.commit_local(patches);
        }
        Ok(())
    }

    /// Removes the style of every materialized cell in the range but keeps the content. A
    /// whole-column or whole-row range also clears the column's or row's own style, as upstream
    /// does.
    ///
    /// A coordinate that holds nothing is left alone, where upstream materializes a
    /// default-styled cell for it.
    pub fn range_clear_formatting(&mut self, area: &Area) -> Result<(), String> {
        let id = self.sheet_of(area.sheet)?;
        let i = area.sheet as usize;
        let mut patches = Vec::new();
        let entire_column = area.row == 1 && area.height == LAST_ROW;
        let entire_row = area.column == 1 && area.width == LAST_COLUMN;
        if entire_column {
            for column in area.column..area.column + area.width {
                let index = &self.workbook.worksheets[i].index;
                let Some(key) = Stable::col_at(index, column) else {
                    continue;
                };
                let prev = self.col_prev(i, &key, ColPropKind::Style);
                if matches!(prev, Some(ColProperty::Style(Some(_)))) {
                    patches.push(Patch::SetColumnSpan {
                        sheet: id,
                        span: (key.clone(), key),
                        property: ColProperty::Style(None),
                        ts: None,
                        prev,
                    });
                }
            }
        } else if entire_row {
            for row in area.row..area.row + area.height {
                let index = &self.workbook.worksheets[i].index;
                let Some(key) = Stable::row_at(index, row) else {
                    continue;
                };
                let prev = self.row_prev(i, Some(&key), RowPropKind::Style);
                if matches!(prev, Some(RowProperty::Style(Some(_)))) {
                    patches.push(Patch::SetRowProperty {
                        sheet: id,
                        row: key,
                        property: RowProperty::Style(None),
                        ts: None,
                        prev,
                    });
                }
            }
        }
        // Only the cells that exist: a whole-column range covers the sheet, not the grid.
        let sheet = &self.workbook.worksheets[i];
        let mut cells: Vec<StableCellAddress> = Vec::new();
        for (row_key, row_data) in &sheet.sheet_data {
            match Stable::row_ordinal(&sheet.index, row_key) {
                Some(row) if (area.row..area.row + area.height).contains(&row) => {}
                _ => continue,
            }
            for column_key in row_data.keys() {
                match Stable::col_ordinal(&sheet.index, column_key) {
                    Some(column) if (area.column..area.column + area.width).contains(&column) => {}
                    _ => continue,
                }
                cells.push((row_key.clone(), column_key.clone()));
            }
        }
        // `sheet_data` is a hash map: sort, so the commit does not depend on its iteration.
        cells.sort();
        for at in cells {
            let prev = self.cell_style_at(i, &at);
            if prev.is_none() {
                continue; // nothing to clear
            }
            patches.push(Patch::SetCellStyle {
                sheet: id,
                at,
                style: None,
                ts: None,
                prev: Box::new(prev),
            });
        }
        if !patches.is_empty() {
            self.commit_local(patches);
        }
        Ok(())
    }

    /// Sets the style of a cell.
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let id = self.sheet_of(sheet)?;
        let prev = self.get_cell_style_or_none(sheet, row, column)?;
        match prev {
            Some(prev) if &prev == style => Ok(()), // nothing changed
            prev => {
                let (at, mut patches) = self.resolve_cell(i, id, row, column);
                patches.push(Patch::SetCellStyle {
                    sheet: id,
                    at,
                    style: Some(Box::new(style.clone())),
                    ts: None,
                    prev: Box::new(prev),
                });
                self.commit_local(patches);
                Ok(())
            }
        }
    }

    /// Sets the named style `style_name` on a cell. Named styles are a local table, so the patch
    /// carries the style it resolves to.
    pub fn set_cell_style_by_name(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style_name: &str,
    ) -> Result<(), String> {
        let style = self.workbook.styles.get_style_by_name(style_name)?;
        self.set_cell_style(sheet, row, column, &style)
    }

    /// Copies a cell's style onto another cell. The source style is the effective one, so a style
    /// the source only inherits from its row or column still lands on the destination cell itself.
    pub fn copy_cell_style(
        &mut self,
        source: (u32, i32, i32),
        destination: (u32, i32, i32),
    ) -> Result<(), String> {
        let style = self.get_style_for_cell(source.0, source.1, source.2)?;
        self.set_cell_style(destination.0, destination.1, destination.2, &style)
    }

    /// Changes the height of a row.
    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        if height < 0.0 {
            return Err(format!("Can not set a negative height: {height}"));
        }
        let property = RowProperty::Height(height / ROW_HEIGHT_FACTOR);

        let at = Stable::row_at(&self.workbook.worksheets[i].index, row);
        let prev = match self.row_prev(i, at.as_ref(), RowPropKind::Height) {
            Some(prev) if prev == property => return Ok(()), // identity modification
            prev => prev,
        };
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property,
            ts: None,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Changes the hidden status of a row.
    pub fn set_row_hidden(&mut self, sheet: u32, row: i32, hidden: bool) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let property = RowProperty::Hidden(hidden);
        let at = Stable::row_at(&self.workbook.worksheets[i].index, row);
        let prev = match self.row_prev(i, at.as_ref(), RowPropKind::Hidden) {
            Some(prev) if prev == property => return Ok(()), // identity change
            prev => prev,
        };
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property,
            ts: None,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets the style of a whole row.
    pub fn set_row_style(&mut self, sheet: u32, row: i32, style: &Style) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let mut patches = Vec::new();
        let key = self.row_key(i, id, row, &mut patches);
        let prev = self.row_prev(i, Some(&key), RowPropKind::Style);
        patches.push(Patch::SetRowProperty {
            sheet: id,
            row: key,
            property: RowProperty::Style(Some(Box::new(style.clone()))),
            ts: None,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Resets a row's style to the default, if it has one.
    pub fn delete_row_style(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        // No row validation, mirroring the ordinal model — which validates the column but not the row.
        let Some(key) = Stable::row_at(&self.workbook.worksheets[i].index, row) else {
            return Ok(());
        };
        let prev = self.row_prev(i, Some(&key), RowPropKind::Style);
        if !matches!(prev, Some(RowProperty::Style(Some(_)))) {
            return Ok(());
        }
        self.commit_local(vec![Patch::SetRowProperty {
            sheet: id,
            row: key,
            property: RowProperty::Style(None),
            ts: None,
            prev,
        }]);
        Ok(())
    }

    /// Writes a single-column property. v1 writes points, never spans: see the shattering
    /// paragraph in [`patch`](crate::collab::patch).
    fn commit_column_property(
        &mut self,
        sheet: u32,
        column: i32,
        property: ColProperty,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let curr = self.col_effective(i, column, property.kind());
        if curr == property {
            return Ok(());
        }
        let mut patches = Vec::new();
        let key = self.col_key(i, id, column, &mut patches);
        let prev = self.col_prev(i, &key, property.kind());
        patches.push(Patch::SetColumnSpan {
            sheet: id,
            span: (key.clone(), key),
            property,
            ts: None,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Changes the width of a column.
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), String> {
        if width < 0.0 {
            return Err(format!("Can not set a negative width: {width}"));
        }
        let property = ColProperty::Width(width / COLUMN_WIDTH_FACTOR);
        self.commit_column_property(sheet, column, property)
    }

    /// Changes the hidden status of a column.
    pub fn set_column_hidden(
        &mut self,
        sheet: u32,
        column: i32,
        hidden: bool,
    ) -> Result<(), String> {
        self.commit_column_property(sheet, column, ColProperty::Hidden(hidden))
    }

    /// Sets the style of a whole column.
    pub fn set_column_style(
        &mut self,
        sheet: u32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let property = ColProperty::Style(Some(Box::new(style.clone())));
        self.commit_column_property(sheet, column, property)
    }

    /// Resets a column's style to the default, if it has one.
    pub fn delete_column_style(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        self.sheet_of(sheet)?;
        let i = sheet as usize;
        let Some(key) = Stable::col_at(&self.workbook.worksheets[i].index, column) else {
            return Ok(());
        };
        let prev = self.col_prev(i, &key, ColPropKind::Style);
        if !matches!(prev, Some(ColProperty::Style(Some(_)))) {
            return Ok(());
        }
        self.commit_column_property(sheet, column, ColProperty::Style(None))
    }

    /// One commit writing `property` to every row in `start..=end`. Rows already holding it are
    /// skipped; if that leaves nothing to write, nothing is committed.
    fn commit_row_span(
        &mut self,
        sheet: u32,
        start: i32,
        end: i32,
        property: RowProperty,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        for row in [start, end] {
            if !is_valid_row(row) {
                return Err(format!("Row number '{row}' is not valid."));
            }
        }
        let mut patches = Vec::new();
        let keys = self.row_keys(i, id, start, end, &mut patches);
        let mut writes = Vec::new();
        for key in keys {
            let prev = match self.row_prev(i, Some(&key), property.kind()) {
                Some(prev) if prev == property => continue, // identity modification
                prev => prev,
            };
            writes.push(Patch::SetRowProperty {
                sheet: id,
                row: key,
                property: property.clone(),
                ts: None,
                prev,
            });
        }
        if writes.is_empty() {
            return Ok(());
        }
        patches.extend(writes);
        self.commit_local(patches);
        Ok(())
    }

    /// [`Self::commit_row_span`] for columns. v1 writes points, never spans.
    fn commit_col_span(
        &mut self,
        sheet: u32,
        start: i32,
        end: i32,
        property: ColProperty,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        for column in [start, end] {
            if !is_valid_column_number(column) {
                return Err(format!("Column number '{column}' is not valid."));
            }
        }
        let mut patches = Vec::new();
        let keys = self.col_keys(i, id, start, end, &mut patches);
        let mut writes = Vec::new();
        for (offset, key) in keys.into_iter().enumerate() {
            if self.col_effective(i, start + offset as i32, property.kind()) == property {
                continue; // identity modification
            }
            let prev = self.col_prev(i, &key, property.kind());
            writes.push(Patch::SetColumnSpan {
                sheet: id,
                span: (key.clone(), key),
                property: property.clone(),
                ts: None,
                prev,
            });
        }
        if writes.is_empty() {
            return Ok(());
        }
        patches.extend(writes);
        self.commit_local(patches);
        Ok(())
    }

    /// Changes the height of every row in `row_start..=row_end`, in one commit.
    pub fn set_rows_height(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        height: f64,
    ) -> Result<(), String> {
        if height < 0.0 {
            return Err(format!("Can not set a negative height: {height}"));
        }
        self.commit_row_span(
            sheet,
            row_start,
            row_end,
            RowProperty::Height(height / ROW_HEIGHT_FACTOR),
        )
    }

    /// Changes the hidden status of every row in `row_start..=row_end`, in one commit.
    pub fn set_rows_hidden(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        hidden: bool,
    ) -> Result<(), String> {
        self.commit_row_span(sheet, row_start, row_end, RowProperty::Hidden(hidden))
    }

    /// Changes the width of every column in `column_start..=column_end`, in one commit.
    pub fn set_columns_width(
        &mut self,
        sheet: u32,
        column_start: i32,
        column_end: i32,
        width: f64,
    ) -> Result<(), String> {
        if width < 0.0 {
            return Err(format!("Can not set a negative width: {width}"));
        }
        self.commit_col_span(
            sheet,
            column_start,
            column_end,
            ColProperty::Width(width / COLUMN_WIDTH_FACTOR),
        )
    }

    /// Changes the hidden status of every column in `column_start..=column_end`, in one commit.
    pub fn set_columns_hidden(
        &mut self,
        sheet: u32,
        column_start: i32,
        column_end: i32,
        hidden: bool,
    ) -> Result<(), String> {
        self.commit_col_span(sheet, column_start, column_end, ColProperty::Hidden(hidden))
    }

    /// Writes a sheet-scoped property, validating nothing beyond the sheet existing.
    fn commit_sheet_property(&mut self, sheet: u32, property: SheetProperty) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let prev = match self.sheet_prev(i, property.kind()) {
            Some(prev) if prev == property => return Ok(()), // identity change
            prev => prev,
        };
        self.commit_local(vec![Patch::SetSheetProperty {
            sheet: id,
            property,
            prev,
        }]);
        Ok(())
    }

    /// Sets the color of the sheet tab.
    pub fn set_sheet_color(&mut self, sheet: u32, color: &Color) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::Color(color.clone()))
    }

    /// Changes the visibility of a sheet.
    pub fn set_sheet_state(&mut self, sheet: u32, state: SheetState) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::State(state))
    }

    /// Makes the grid lines visible (`true`) or hidden (`false`).
    pub fn set_show_grid_lines(&mut self, sheet: u32, show: bool) -> Result<(), String> {
        self.commit_sheet_property(sheet, SheetProperty::ShowGridLines(show))
    }

    /// Freezes the first `frozen_rows` rows of a sheet.
    pub fn set_frozen_rows(&mut self, sheet: u32, frozen_rows: i32) -> Result<(), String> {
        if frozen_rows < 0 {
            return Err("Frozen rows cannot be negative".to_string());
        }
        if frozen_rows >= LAST_ROW {
            return Err("Too many rows".to_string());
        }
        self.commit_sheet_property(sheet, SheetProperty::FrozenRows(frozen_rows))
    }

    /// Freezes the first `frozen_columns` columns of a sheet.
    pub fn set_frozen_columns(&mut self, sheet: u32, frozen_columns: i32) -> Result<(), String> {
        if frozen_columns < 0 {
            return Err("Frozen columns cannot be negative".to_string());
        }
        if frozen_columns >= LAST_COLUMN {
            return Err("Too many columns".to_string());
        }
        self.commit_sheet_property(sheet, SheetProperty::FrozenColumns(frozen_columns))
    }

    /// Merges or unmerges the cells an ordinal rectangle covers.
    pub fn set_merged_range(
        &mut self,
        sheet: u32,
        range: &RangeRef,
        merged: bool,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        // The current state, resolved without minting: a corner the index does not hold cannot be
        // part of a stored merge, so the range is simply not merged.
        let prev = match self.resolved_range(i, range) {
            Some(r) => self.workbook.worksheets[i].merged_cells.contains(&r),
            _ => false,
        };
        // Merging what is already merged, or unmerging what is not, changes nothing: emit nothing,
        // and — having checked before resolving — mint nothing.
        if merged == prev {
            return Ok(());
        }
        let mut patches = Vec::new();
        let range = self.stable_range(i, id, range, &mut patches);
        patches.push(Patch::SetMergedRange {
            sheet: id,
            range,
            merged,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Sets or removes the comment on a cell.
    pub fn set_comment(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        comment: Option<(String, String)>,
    ) -> Result<(), String> {
        let i = self.check_cell(sheet, row, column)?;
        let id = self.sheet_of(sheet)?;
        // What the cell holds now, without minting: an unmaterialized cell holds no comment.
        let w = &self.workbook.worksheets[i];
        let prev = match Stable::row_at(&w.index, row).zip(Stable::col_at(&w.index, column)) {
            Some(cell_ref) => w
                .comments
                .iter()
                .find(move |c| c.cell_ref == cell_ref)
                .cloned(),
            _ => None,
        };
        // Writing back the comment the cell already has — or removing one it does not have —
        // changes nothing, so it emits nothing and mints nothing. `author_id` is not compared: a
        // locally authored comment carries none.
        let unchanged = match (&comment, &prev) {
            (None, None) => true,
            (Some((text, author_name)), Some(p)) => {
                &p.text == text && &p.author_name == author_name
            }
            _ => false,
        };
        if unchanged {
            return Ok(());
        }
        let (at, mut patches) = self.resolve_cell(i, id, row, column);
        let comment = comment.map(|(text, author_name)| Comment::<Stable> {
            text,
            author_name,
            author_id: None,
            cell_ref: at.clone(),
        });
        patches.push(Patch::SetComment {
            sheet: id,
            at,
            comment,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Writes a workbook-global property.
    fn commit_workbook_property(&mut self, property: WorkbookProperty) {
        let prev = match property.kind() {
            WorkbookPropKind::Theme => {
                WorkbookProperty::Theme(Box::new(self.workbook.theme.clone()))
            }
            WorkbookPropKind::Locale => {
                WorkbookProperty::Locale(self.workbook.settings.locale.clone())
            }
            WorkbookPropKind::Timezone => {
                WorkbookProperty::Timezone(self.workbook.settings.tz.clone())
            }
        };
        if prev == property {
            return; // nothing changed
        }
        self.commit_local(vec![Patch::SetWorkbookProperty {
            property,
            prev: Some(prev),
        }]);
    }

    /// Sets the workbook theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.commit_workbook_property(WorkbookProperty::Theme(Box::new(theme)));
        self.evaluate_conditional_formatting();
    }

    /// Sets the workbook locale.
    pub fn set_locale(&mut self, locale_id: &str) -> Result<(), String> {
        let Ok(locale) = get_locale(locale_id) else {
            return Err(format!("Invalid locale: {locale_id}"));
        };
        self.parser.set_locale(locale);
        self.locale = locale;
        self.commit_workbook_property(WorkbookProperty::Locale(locale_id.to_string()));
        self.evaluate();
        Ok(())
    }

    /// Sets the workbook timezone.
    pub fn set_timezone(&mut self, timezone: &str) -> Result<(), String> {
        let Ok(tz) = Tz::parse(timezone) else {
            return Err(format!("Invalid timezone: {timezone}"));
        };
        self.tz = tz;
        self.commit_workbook_property(WorkbookProperty::Timezone(timezone.to_string()));
        self.evaluate();
        Ok(())
    }
}

/// Structural edits.
///
/// Nothing rewrites a formula any more: a bound reference names rows, columns and sheets by
/// identity, so an insert, a delete or a move changes what it resolves to and never its storage.
impl CollabModel<'_> {
    /// The keys `count` rows or columns inserted at ordinal `at` take, together with whatever had
    /// to be materialized to reach that far. Empty when the axis has no room to name them.
    fn insert_keys(index: &FractionalIndex, at: i32, count: i32) -> Vec<FractionalKey> {
        let (at, count) = (at as usize, count as usize);
        if at > index.len() {
            return index.plan_virtual(at - 1 + count);
        }
        index.create_keys(at - 1, count).collect()
    }

    /// Everything the rows `keys` name is about to lose, so that undo can put it back — including
    /// the stamp each register holds, which is what the restore replays at.
    fn row_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<RowSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        let registers = &sheet.index.registers;
        keys.iter()
            .map(|key| {
                let row = sheet.rows.iter().find(|r| &r.r == key);
                let state = match row {
                    Some(row) => RowState {
                        height: row.height,
                        hidden: row.hidden,
                        style: if row.custom_format {
                            self.workbook.styles.get_style(row.s).ok().map(Box::new)
                        } else {
                            None
                        },
                        custom_height: row.custom_height,
                        custom_format: row.custom_format,
                    },
                    None => RowState::default(),
                };
                let (cell_values, cell_styles) = self.cell_snapshots(
                    i,
                    sheet
                        .sheet_data
                        .get(key)
                        .into_iter()
                        .flatten()
                        .map(|(col, cell)| {
                            ((key.clone(), col.clone()), col.clone(), cell.get_style())
                        }),
                );
                let prop_ts = [RowPropKind::Style, RowPropKind::Height, RowPropKind::Hidden]
                    .into_iter()
                    .filter_map(|kind| {
                        let ts = registers.rows.get(&(key.clone(), kind))?;
                        Some((kind, *ts))
                    })
                    .collect();
                RowSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                    prop_ts,
                }
            })
            .collect()
    }

    /// [`Self::row_snapshots`] for columns; the cells are keyed by row instead.
    fn column_snapshots(&self, i: usize, keys: &[FractionalKey]) -> Vec<ColumnSnapshot> {
        let sheet = &self.workbook.worksheets[i];
        let registers = &sheet.index.registers;
        keys.iter()
            .map(|key| {
                let col = sheet.cols.iter().find(|c| &c.min == key && &c.max == key);
                let state = match col {
                    Some(col) => ColState {
                        width: col.width,
                        hidden: col.hidden,
                        style: match col.style {
                            Some(s) => self.workbook.styles.get_style(s).ok().map(Box::new),
                            None => None,
                        },
                        custom_width: col.custom_width,
                    },
                    None => ColState::default(),
                };
                let (cell_values, cell_styles) = self.cell_snapshots(
                    i,
                    sheet.sheet_data.iter().filter_map(|(row, cells)| {
                        let cell = cells.get(key)?;
                        Some(((row.clone(), key.clone()), row.clone(), cell.get_style()))
                    }),
                );
                let span = (key.clone(), key.clone());
                let prop_ts = [ColPropKind::Style, ColPropKind::Width, ColPropKind::Hidden]
                    .into_iter()
                    .filter_map(|kind| {
                        let ts = registers.col_spans.get(&(span.clone(), kind))?;
                        Some((kind, *ts))
                    })
                    .collect();
                ColumnSnapshot {
                    key: key.clone(),
                    state,
                    cell_values,
                    cell_styles,
                    prop_ts,
                }
            })
            .collect()
    }

    /// The authored contents and the styles of the cells `cells` names, keyed and sorted by the
    /// axis the caller is snapshotting along.
    #[allow(clippy::type_complexity)]
    fn cell_snapshots(
        &self,
        i: usize,
        cells: impl Iterator<Item = (StableCellAddress, FractionalKey, i32)>,
    ) -> (
        Vec<(FractionalKey, CellInput, Timestamp)>,
        Vec<(FractionalKey, Style, Timestamp)>,
    ) {
        let registers = &self.workbook.worksheets[i].index.registers;
        let mut values = Vec::new();
        let mut styles = Vec::new();
        let mut cells: Vec<_> = cells.collect();
        cells.sort_by(|(_, a_axis, _), (_, b_axis, _)| a_axis.cmp(b_axis));
        for (at, key, style) in cells {
            if let Some(input) = self.cell_input(i, &at) {
                let ts = registers.cell_values.get(&at).copied().unwrap_or_default();
                values.push((key.clone(), input, ts));
            }
            if style != 0 {
                if let Ok(style) = self.workbook.styles.get_style(style) {
                    let ts = registers.cell_styles.get(&at).copied().unwrap_or_default();
                    styles.push((key, style, ts));
                }
            }
        }
        (values, styles)
    }

    /// The whole of worksheet `i` as a payload: what an `AddSheet` seeds a copy from, and what the
    /// undo of a delete puts back. Keys are carried as they are — the copy addresses the same rows.
    fn sheet_content(&self, i: usize) -> SheetContent {
        let sheet = &self.workbook.worksheets[i];
        let mut cell_values = Vec::new();
        let mut cell_styles = Vec::new();
        for (row, cells) in &sheet.sheet_data {
            for (col, cell) in cells {
                let at = (row.clone(), col.clone());
                if let Some(input) = self.cell_input(i, &at) {
                    cell_values.push((at.clone(), input));
                }
                // Only a cell holding a style of its own: 0 is the table's default entry.
                if cell.get_style() != 0 {
                    if let Ok(style) = self.workbook.styles.get_style(cell.get_style()) {
                        cell_styles.push((at, style));
                    }
                }
            }
        }
        // `sheet_data` is a hash map, and the payload travels: sort so it does not depend on it.
        cell_values.sort_by(|(a, _), (b, _)| a.cmp(b));
        cell_styles.sort_by(|(a, _), (b, _)| a.cmp(b));
        SheetContent {
            state: sheet.state.clone(),
            color: sheet.color.clone(),
            show_grid_lines: sheet.show_grid_lines,
            frozen_rows: sheet.frozen_rows,
            frozen_columns: sheet.frozen_columns,
            rows: sheet
                .rows
                .iter()
                .map(|row| {
                    let state = RowState {
                        height: row.height,
                        hidden: row.hidden,
                        style: match row.custom_format {
                            true => self.workbook.styles.get_style(row.s).ok().map(Box::new),
                            false => None,
                        },
                        custom_height: row.custom_height,
                        custom_format: row.custom_format,
                    };
                    (row.r.clone(), state)
                })
                .collect(),
            columns: sheet
                .cols
                .iter()
                .map(|col| {
                    let state = ColState {
                        width: col.width,
                        hidden: col.hidden,
                        style: col
                            .style
                            .and_then(|s| self.workbook.styles.get_style(s).ok())
                            .map(Box::new),
                        custom_width: col.custom_width,
                    };
                    ((col.min.clone(), col.max.clone()), state)
                })
                .collect(),
            cell_values,
            cell_styles,
            merge_cells: sheet.merged_cells.clone(),
            comments: sheet.comments.clone(),
            // `cf_order` is kept aligned with the rules themselves, entry by entry.
            conditional_formatting: sheet
                .index
                .registers
                .cf_order
                .iter()
                .cloned()
                .zip(
                    sheet
                        .conditional_formatting
                        .iter()
                        .map(|cf| ConditionalFormatState {
                            rule: cf.cf_rule.clone(),
                            ranges: cf.ranges.clone(),
                        }),
                )
                .collect(),
        }
    }

    /// Inserts `row_count` rows above `row`, displacing the formulas that referenced across it.
    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if row_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let keys = Self::insert_keys(&self.workbook.worksheets[i].index.rows, row, row_count);
        if keys.is_empty() {
            return Err("Cannot insert rows there".to_string());
        }
        let patches = vec![Patch::InsertRows { sheet: id, keys }];
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes `row_count` rows starting at `row`, displacing the formulas that referenced them.
    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if row_count <= 0 {
            return Err("Please use insert rows instead".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        let index = &self.workbook.worksheets[i].index;
        let keys: Vec<FractionalKey> = (row..row + row_count)
            .filter_map(|r| Stable::row_at(index, r))
            .collect();
        if keys.is_empty() {
            return Ok(());
        }
        let prev = self.row_snapshots(i, &keys);
        let patches = vec![Patch::DeleteRows {
            sheet: id,
            keys,
            prev,
        }];
        self.commit_local(patches);
        Ok(())
    }

    /// Inserts `column_count` columns before `column`.
    pub fn insert_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if column_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let keys = Self::insert_keys(
            &self.workbook.worksheets[i].index.cols,
            column,
            column_count,
        );
        if keys.is_empty() {
            return Err("Cannot insert columns there".to_string());
        }
        let patches = vec![Patch::InsertColumns { sheet: id, keys }];
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes `column_count` columns starting at `column`.
    pub fn delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if column_count <= 0 {
            return Err("Please use insert columns instead".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        let index = &self.workbook.worksheets[i].index;
        let keys: Vec<FractionalKey> = (column..column + column_count)
            .filter_map(|c| Stable::col_at(index, c))
            .collect();
        if keys.is_empty() {
            return Ok(());
        }
        let prev = self.column_snapshots(i, &keys);
        let patches = vec![Patch::DeleteColumns {
            sheet: id,
            keys,
            prev,
        }];
        self.commit_local(patches);
        Ok(())
    }

    /// `plan_move`'s inputs for a block of `count` elements at ordinal `first` moving by `delta`:
    /// the source positions, and the destination in the pre-drain coordinates it reads.
    fn move_span(first: i32, count: i32, delta: i32) -> (std::ops::Range<usize>, usize) {
        let source = (first - 1) as usize..(first - 1 + count) as usize;
        // Past the block, the drained entries are what the planner subtracts back off.
        let dest = if delta > 0 {
            first + count - 1 + delta
        } else {
            first + delta - 1
        };
        (source, dest as usize)
    }

    /// Moves the rows `[row, row + row_count)` by `delta`, rewriting the formulas that referenced
    /// across them. Content, properties and ranges are keyed by identity, so they follow the rows
    /// without a patch of their own.
    ///
    /// Arrays are a later phase: upstream's array-split guard is deferred with them.
    pub fn move_rows_action(
        &mut self,
        sheet: u32,
        row: i32,
        row_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if row_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = row + delta;
        let target_last = row + row_count - 1 + delta;
        if !(1..=LAST_ROW).contains(&target_first) || !(1..=LAST_ROW).contains(&target_last) {
            return Err("Target row out of boundaries".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) || !(1..=LAST_ROW).contains(&(row + row_count - 1)) {
            return Err("Initial row out of boundaries".to_string());
        }
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let index = &self.workbook.worksheets[i].index.rows;
        let (source, dest) = Self::move_span(row, row_count, delta);
        // `plan_move` rebuilds this same tail internally, deterministically, to plan over it.
        let keys = index.plan_virtual(target_last.max(row + row_count - 1) as usize);
        let moves = index.plan_move(source, dest);
        if moves.is_empty() {
            return Err("Cannot move rows there".to_string());
        }
        // A row named past the tail holds nothing yet, so its inverse files it back onto its identity.
        let prev = moves
            .iter()
            .map(|(id, _)| index.held_key(id).unwrap_or_else(|| id.clone()))
            .collect();
        let mut patches = Vec::new();
        if !keys.is_empty() {
            patches.push(Patch::InsertRows { sheet: id, keys });
        }
        patches.push(Patch::MoveRows {
            sheet: id,
            moves,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// [`Self::move_rows_action`] for columns.
    pub fn move_columns_action(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if column_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = column + delta;
        let target_last = column + column_count - 1 + delta;
        if !(1..=LAST_COLUMN).contains(&target_first) || !(1..=LAST_COLUMN).contains(&target_last) {
            return Err("Target column out of boundaries".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column)
            || !(1..=LAST_COLUMN).contains(&(column + column_count - 1))
        {
            return Err("Initial column out of boundaries".to_string());
        }
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let index = &self.workbook.worksheets[i].index.cols;
        let (source, dest) = Self::move_span(column, column_count, delta);
        // `plan_move` rebuilds this same tail internally, deterministically, to plan over it.
        let keys = index.plan_virtual(target_last.max(column + column_count - 1) as usize);
        let moves = index.plan_move(source, dest);
        if moves.is_empty() {
            return Err("Cannot move columns there".to_string());
        }
        // A column named past the tail holds nothing yet, so its inverse files it back onto its identity.
        let prev = moves
            .iter()
            .map(|(id, _)| index.held_key(id).unwrap_or_else(|| id.clone()))
            .collect();
        let mut patches = Vec::new();
        if !keys.is_empty() {
            patches.push(Patch::InsertColumns { sheet: id, keys });
        }
        patches.push(Patch::MoveColumns {
            sheet: id,
            moves,
            prev,
        });
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes a sheet by index. Fails if it is the last one.
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), String> {
        if self.workbook.worksheets.len() == 1 {
            return Err("Cannot delete only sheet".to_string());
        }
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        // The authored name and the filed key, so the undo revives the sheet as it was filed.
        let prev = match (
            self.authored_name(id).cloned(),
            self.workbook.meta.sheet_positions.get(&id),
        ) {
            (Some(name), Some(position)) => Some(Box::new(SheetRestore {
                name,
                position: position.value.clone(),
                content: Some(Box::new(self.sheet_content(i))),
            })),
            _ => None,
        };
        self.commit_local(vec![Patch::DeleteSheet { sheet: id, prev }]);
        self.evaluate();
        Ok(())
    }

    /// Deletes a sheet by name. Fails if it does not exist or it is the last one.
    pub fn delete_sheet_by_name(&mut self, name: &str) -> Result<(), String> {
        match self.get_sheet_index_by_name(name) {
            Some(sheet_index) => self.delete_sheet(sheet_index),
            None => Err("Sheet not found".to_string()),
        }
    }

    pub fn duplicate_sheet(&mut self, source: u32) -> Result<(String, u32), String> {
        let source_id = self.sheet_of(source)?;
        let i = source as usize;
        // The repair pass names sheets; asking it here gives the copy the name it would keep
        // anyway. A peer authoring the same name concurrently falls through to that pass.
        let mut taken =
            SHEET_NAMES.taken(self.workbook.worksheets.iter().map(|ws| ws.name.as_str()));
        let new_name = SHEET_NAMES.free_name(self.workbook.worksheets[i].name.clone(), &mut taken);
        let id = self.new_sheet_id();
        let position = self.insert_position(i + 1)?;

        // The copy's keys are the source's, so an own-sheet reference keeps working as it is; only
        // one naming the source *by id* has to be pointed at the copy instead.
        let mut content = self.sheet_content(i);
        for (_, input) in &mut content.cell_values {
            if let CellInput::Formula(formula) = input {
                formula.retarget_sheet(source_id, id);
            }
        }
        let mut patches = vec![Patch::AddSheet {
            id,
            name: new_name.clone(),
            position,
            content: Some(Box::new(content)),
        }];

        // Names local to the source are always copied; a global one only when it names the source.
        // Locals come first so that, with the de-dup below, they win over a global of the same name.
        let mut names: Vec<(bool, String, DefinedNameBody)> = self
            .defined_name_display()
            .into_iter()
            .filter(|(_, scope, _)| *scope == Some(source_id) || scope.is_none())
            .filter_map(|(entry, scope, name)| {
                Some((scope.is_none(), name, self.formula_of(entry)?))
            })
            .collect();
        names.sort_by_key(|(is_global, _, _)| *is_global);
        let mut copied: Vec<String> = Vec::new();
        for (is_global, name, mut body) in names {
            if copied.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                continue;
            }
            // The retarget must run for every copied name, not only when the result is inspected.
            let changed = body.formula.retarget_sheet(source_id, id);
            if is_global && !changed {
                continue;
            }
            copied.push(name.clone());
            let new_id = self.new_defined_name_id(Some(id), &name);
            patches.push(Patch::SetDefinedName {
                id: new_id,
                property: DefinedNameProperty::Definition(Some(body)),
                prev: Some(DefinedNameProperty::Definition(None)),
            });
            patches.push(Patch::SetDefinedName {
                id: new_id,
                property: DefinedNameProperty::Name((Some(id), name)),
                prev: None,
            });
        }
        self.commit_local(patches);
        let at = self.get_sheet_index_by_sheet_id(id).unwrap_or_default();
        self.evaluate();
        Ok((new_name, at))
    }

    /// Renames a sheet. Formulas naming it follow by themselves.
    pub fn rename_sheet_by_index(&mut self, sheet: u32, new_name: &str) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        if !is_valid_sheet_name(new_name) {
            return Err(format!("Invalid name for a sheet: '{new_name}'."));
        }
        if self
            .get_sheet_index_by_name(new_name)
            .is_some_and(|found| found != sheet)
        {
            return Err(format!("Sheet already exists: '{new_name}'."));
        }
        if let Some(name) = self.authored_name(id) {
            if name == new_name {
                return Ok(()); // name hasn't changed
            }
        }
        let prev = self.sheet_prev(i, SheetPropKind::Name);
        let patches = vec![Patch::SetSheetProperty {
            sheet: id,
            property: SheetProperty::Name(new_name.to_string()),
            prev,
        }];
        // Nothing else to write: a bound reference names the sheet by id, so it renders under the
        // new name the moment the register does.
        self.commit_local(patches);
        self.evaluate();
        Ok(())
    }

    /// Renames a sheet found by its current name.
    pub fn rename_sheet(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        match self.get_sheet_index_by_name(old_name) {
            Some(sheet) => self.rename_sheet_by_index(sheet, new_name),
            None => Err(format!("Could not find sheet {old_name}")),
        }
    }
}

/// Defined names and conditional formatting.
impl CollabModel<'_> {
    /// The sheet id `scope` names, and an error if it names nothing.
    fn scope_id(&self, scope: Option<u32>) -> Result<Option<SheetId>, String> {
        match scope {
            Some(index) => Ok(Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            )),
            None => Ok(None),
        }
    }

    /// The id a name authored as `(scope, name)` takes: the hash of both, salted past any name
    /// already live here — so creating one under a name a rename freed does not land on the
    /// renamed name.
    fn new_defined_name_id(&self, scope: Option<SheetId>, name: &str) -> DefinedNameId {
        let live = |id: &DefinedNameId| {
            self.workbook
                .meta
                .defined_names
                .get(id)
                .is_some_and(|state| state.formula.value.is_some())
        };
        (0..)
            .map(|salt| defined_name_id(scope, name, salt))
            .find(|id| !live(id))
            .expect("a free salt")
    }

    /// The name showing `name` in `scope`, matched case-insensitively as every upstream lookup is.
    pub(crate) fn defined_name_id_of(
        &self,
        scope: Option<SheetId>,
        name: &str,
    ) -> Option<DefinedNameId> {
        let upper = name.to_uppercase();
        self.defined_name_display()
            .into_iter()
            .find(|(_, s, display)| *s == scope && display.to_uppercase() == upper)
            .map(|(id, ..)| id)
    }

    /// The body `id` currently holds, `None` once it has been deleted.
    pub(crate) fn formula_of(&self, id: DefinedNameId) -> Option<DefinedNameBody> {
        let state = self.workbook.meta.defined_names.get(&id)?;
        state.formula.value.clone()
    }

    /// Parses a defined-name body as the user typed it and binds it against the name context —
    /// which is also what a `Current` sheet reference inside it resolves to.
    fn bind_defined_name(
        &mut self,
        formula: &str,
        plan: &mut MintPlan,
    ) -> Result<DefinedNameBody, String> {
        let context = self.defined_name_context();
        let (node, equals) = self.user_formula_to_node(formula, &context)?;
        // The context is the first worksheet's A1, so that is the host — and what a reference
        // carrying no sheet prefix binds to.
        let formula = self
            .bind_formula(&node, 0, 1, 1, plan)
            .map_err(|err| format!("Invalid formula: {err}"))?;
        Ok(DefinedNameBody { formula, equals })
    }

    /// Adds a defined name, global when `scope` is `None`.
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let sheet_id = self.scope_id(scope)?;
        if self.defined_name_id_of(sheet_id, name).is_some() {
            return Err("Name: Defined name already exists".to_string());
        }
        let mut plan = MintPlan::default();
        let formula = self.bind_defined_name(formula, &mut plan)?;
        let id = self.new_defined_name_id(sheet_id, name);
        let mut patches = self.mint_patches(&plan);
        patches.extend([
            Patch::SetDefinedName {
                id,
                // Nothing was defined under this id, which is what undoing the create puts back.
                property: DefinedNameProperty::Definition(Some(formula)),
                prev: Some(DefinedNameProperty::Definition(None)),
            },
            Patch::SetDefinedName {
                id,
                property: DefinedNameProperty::Name((sheet_id, name.to_string())),
                prev: None,
            },
        ]);
        self.commit_local(patches);
        Ok(())
    }

    /// Deletes a defined name. The entry and its address survive, so an undo revives it.
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let sheet_id = self.scope_id(scope)?;
        let Some(id) = self.defined_name_id_of(sheet_id, name) else {
            return Err("Defined name not found".to_string());
        };
        let prev = self.formula_of(id);
        self.commit_local(vec![Patch::SetDefinedName {
            id,
            property: DefinedNameProperty::Definition(None),
            prev: Some(DefinedNameProperty::Definition(prev)),
        }]);
        Ok(())
    }

    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let sheet_id = self.scope_id(scope)?;
        let new_sheet_id = self.scope_id(new_scope)?;
        let renaming = name.to_uppercase() != new_name.to_uppercase() || scope != new_scope;
        if renaming && self.defined_name_id_of(new_sheet_id, new_name).is_some() {
            return Err("Name: Defined name already exists".to_string());
        }
        let Some(id) = self.defined_name_id_of(sheet_id, name) else {
            return Err("Defined name not found".to_string());
        };
        let old_formula = self.formula_of(id);
        let mut plan = MintPlan::default();
        let formula = self.bind_defined_name(new_formula, &mut plan)?;
        let mut patches = self.mint_patches(&plan);
        if renaming {
            let prev = self
                .workbook
                .meta
                .defined_names
                .get(&id)
                .map(|state| DefinedNameProperty::Name(state.name.value.clone()));
            patches.push(Patch::SetDefinedName {
                id,
                property: DefinedNameProperty::Name((new_sheet_id, new_name.to_string())),
                prev,
            });
            // Nothing else to write: a bound formula names the entry by id, not by text.
        }
        if old_formula.as_ref() != Some(&formula) {
            patches.push(Patch::SetDefinedName {
                id,
                property: DefinedNameProperty::Definition(Some(formula)),
                prev: Some(DefinedNameProperty::Definition(old_formula)),
            });
        }
        // An update that changed neither register is not an edit, so it authors no commit.
        if !patches.is_empty() {
            self.commit_local(patches);
        }
        Ok(())
    }

    /// Adds a conditional formatting rule to `sheet`. The key it is minted with is both its
    /// identity and its priority, so the returned priority is its position in storage order.
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: CfRuleInput,
    ) -> Result<u32, String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let ordinal = RangeRef::parse_sqref(range);
        if ordinal.is_empty() {
            return Err(format!("Invalid conditional formatting range: '{range}'"));
        }
        let mut rule = rule;
        self.cf_rule_input_to_internal(&mut rule, sheet)?;
        let rule = self.cf_rule_from_input(rule);
        let mut patches = Vec::new();
        let ranges = ordinal
            .iter()
            .map(|r| self.stable_range(i, id, r, &mut patches))
            .collect();
        let key = self.cf_key(i);
        patches.push(Patch::AddConditionalFormat {
            sheet: id,
            key: key.clone(),
            rule: Box::new(rule),
            ranges,
        });
        self.commit_local(patches);
        let order = &self.workbook.worksheets[i].index.registers.cf_order;
        Ok(order.binary_search(&key).map_or(0, |at| at as u32 + 1))
    }

    /// A rule key past every rule this sheet holds, suffixed with this replica's session.
    fn cf_key(&self, i: usize) -> FractionalKey {
        let order = &self.workbook.worksheets[i].index.registers.cf_order;
        let mut buf = KeyBuf::from(&(2 * (order.len() as u32 + 1)).to_be_bytes()[1..]);
        buf.extend_from_slice(&self.suffix());
        FractionalKey::try_from_bytes(&buf).expect("position and suffix are 7 bytes")
    }

    /// Removes the conditional formatting rule at `index`.
    pub fn delete_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let ws = &self.workbook.worksheets[i];
        let Some(key) = ws.index.registers.cf_order.get(index).cloned() else {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        };
        let prev = ws
            .conditional_formatting
            .get(index)
            .map(|cf| ConditionalFormatState {
                rule: cf.cf_rule.clone(),
                ranges: cf.ranges.clone(),
            })
            .map(Box::new);
        self.commit_local(vec![Patch::DeleteConditionalFormat {
            sheet: id,
            key,
            prev,
        }]);
        Ok(())
    }

    /// Replaces the range and the rule of the conditional formatting entry at `index`.
    pub fn update_conditional_formatting(
        &mut self,
        sheet: u32,
        index: usize,
        new_range: &str,
        new_rule: CfRuleInput,
    ) -> Result<(), String> {
        let id = self.sheet_of(sheet)?;
        let i = sheet as usize;
        let ordinal = RangeRef::parse_sqref(new_range);
        if ordinal.is_empty() {
            return Err(format!(
                "Invalid conditional formatting range: '{new_range}'"
            ));
        }
        let Some(key) = self.workbook.worksheets[i]
            .index
            .registers
            .cf_order
            .get(index)
            .cloned()
        else {
            return Err(format!(
                "Conditional formatting index {index} out of bounds"
            ));
        };
        let mut new_rule = new_rule;
        self.cf_rule_input_to_internal(&mut new_rule, sheet)?;
        let rule = self.cf_rule_from_input(new_rule);
        let mut patches = Vec::new();
        let ranges = ordinal
            .iter()
            .map(|r| self.stable_range(i, id, r, &mut patches))
            .collect();
        let old = &self.workbook.worksheets[i].conditional_formatting[index];
        let (old_rule, old_ranges) = (old.cf_rule.clone(), old.ranges.clone());
        patches.push(Patch::SetConditionalFormat {
            sheet: id,
            key: key.clone(),
            property: CfProperty::Rule(Box::new(rule)),
            prev: Some(CfProperty::Rule(Box::new(old_rule))),
        });
        patches.push(Patch::SetConditionalFormat {
            sheet: id,
            key,
            property: CfProperty::Ranges(ranges),
            prev: Some(CfProperty::Ranges(old_ranges)),
        });
        self.commit_local(patches);
        Ok(())
    }
}

/// Reads the ordinal model answers from plain worksheet fields, which stable addressing answers
/// the same way.
impl CollabModel<'_> {
    pub fn get_named_style(&self, name: &str) -> Result<Style, String> {
        self.workbook.styles.get_style_by_name(name)
    }

    pub fn get_named_style_list(&self) -> Vec<String> {
        self.workbook.styles.get_named_style_list()
    }
}

/// Named styles: one register per style, keyed by a hash of the name it was created under, with the
/// style table derived from them.
impl CollabModel<'_> {
    /// The id a style authored as `name` takes: the hash of the name, salted past any style already
    /// live here — so creating one under a name a rename freed does not land on the renamed style.
    fn new_style_id(&self, name: &str) -> NamedStyleId {
        let live = |id: &NamedStyleId| {
            self.workbook
                .meta
                .named_styles
                .get(id)
                .is_some_and(|state| state.definition.value.is_some())
        };
        (0u32..)
            .map(|salt| stable_id([name.as_bytes(), salt.to_le_bytes().as_ref()]))
            .find(|id| !live(id))
            .expect("a free salt")
    }

    /// The style showing `name`, erroring as the ordinal model's lookup does.
    fn style_id_by_name(&self, name: &str) -> Result<NamedStyleId, String> {
        self.named_style_display()
            .into_iter()
            .find(|(_, display)| display == name)
            .map(|(id, _)| id)
            .ok_or_else(|| format!("Style '{name}' not found"))
    }

    /// Creates a named style. Fails if a style already shows that name.
    pub fn create_named_style(&mut self, name: &str, style: &Style) -> Result<(), String> {
        if self.workbook.styles.get_xf_id_by_name(name).is_ok() {
            return Err("A style with that name already exists".to_string());
        }
        let id = self.new_style_id(name);
        self.commit_local(vec![
            Patch::SetNamedStyle {
                id,
                property: NamedStyleProperty::Definition(Some(Box::new(NamedStyle {
                    style: style.clone(),
                    builtin_id: 0,
                }))),
                // Nothing was defined under this id, which is what undoing the create puts back.
                prev: Some(NamedStyleProperty::Definition(None)),
            },
            Patch::SetNamedStyle {
                id,
                property: NamedStyleProperty::Name(name.to_string()),
                prev: None,
            },
        ]);
        Ok(())
    }

    /// Deletes a named style. Cells that used it keep their formatting; only the name goes.
    pub fn delete_named_style(&mut self, name: &str) -> Result<(), String> {
        if self.workbook.styles.is_builtin_style(name) {
            return Err(format!("Cannot delete built-in style '{name}'"));
        }
        let id = self.style_id_by_name(name)?;
        let prev = self.definition_of(id);
        self.commit_local(vec![Patch::SetNamedStyle {
            id,
            property: NamedStyleProperty::Definition(None),
            prev: Some(NamedStyleProperty::Definition(prev)),
        }]);
        Ok(())
    }

    /// Updates a named style's formatting and, when `new_name` differs, its name. Everything drawn
    /// with the old style follows in the same commit.
    pub fn update_named_style(
        &mut self,
        name: &str,
        new_name: &str,
        style: &Style,
    ) -> Result<(), String> {
        if self.workbook.styles.is_builtin_style(name) {
            return Err(format!("Cannot modify built-in style '{name}'"));
        }
        let id = self.style_id_by_name(name)?;
        if name != new_name && self.workbook.styles.get_xf_id_by_name(new_name).is_ok() {
            return Err(format!("A style named '{new_name}' already exists"));
        }
        let old_style = self.workbook.styles.get_style_by_name(name)?;
        // Cells styled by name carry the style itself (patches resolve names locally), so what
        // draws the old style is its anonymous format index — if any cell ever took it.
        let old_xf_id = self.workbook.styles.get_style_index(&old_style);
        let prev = self.definition_of(id);
        let mut patches = vec![Patch::SetNamedStyle {
            id,
            property: NamedStyleProperty::Definition(Some(Box::new(NamedStyle {
                style: style.clone(),
                builtin_id: prev.as_ref().map(|d| d.builtin_id).unwrap_or(0),
            }))),
            prev: Some(NamedStyleProperty::Definition(prev)),
        }];
        if name != new_name {
            patches.push(Patch::SetNamedStyle {
                id,
                property: NamedStyleProperty::Name(new_name.to_string()),
                prev: Some(NamedStyleProperty::Name(name.to_string())),
            });
        }
        if let Some(old_xf_id) = old_xf_id {
            if self.workbook.styles.get_style_index(style) != Some(old_xf_id) {
                patches.extend(self.restyle_patches(old_xf_id, style));
            }
        }
        self.commit_local(patches);
        Ok(())
    }

    fn definition_of(&self, id: NamedStyleId) -> Option<Box<NamedStyle>> {
        let state = self.workbook.meta.named_styles.get(&id)?;
        state.definition.value.clone()
    }

    /// Repoints everything drawn with `old_xf` at `style`. Style *index* equality is the test, as in
    /// the ordinal model: a cell styled the same way by hand moves with the named style.
    fn restyle_patches(&self, old_xf: i32, style: &Style) -> Vec<Patch> {
        let prev = self.workbook.styles.get_style(old_xf).ok();
        let mut patches = Vec::new();
        for sheet in &self.workbook.worksheets {
            let id = sheet.sheet_id;
            // `sheet_data` is a hash map: sort, so the commit does not depend on its iteration.
            let mut cells: Vec<StableCellAddress> = sheet
                .sheet_data
                .iter()
                .flat_map(|(row, row_data)| {
                    row_data
                        .iter()
                        .filter(|(_, cell)| cell.get_style() == old_xf)
                        .map(move |(column, _)| (row.clone(), column.clone()))
                })
                .collect();
            cells.sort();
            patches.extend(cells.into_iter().map(|at| Patch::SetCellStyle {
                sheet: id,
                at,
                style: Some(Box::new(style.clone())),
                ts: None,
                prev: Box::new(prev.clone()),
            }));
            // A row's index only draws anything when it is its custom format.
            for row in sheet
                .rows
                .iter()
                .filter(|r| r.custom_format && r.s == old_xf)
            {
                patches.push(Patch::SetRowProperty {
                    sheet: id,
                    row: row.r.clone(),
                    property: RowProperty::Style(Some(Box::new(style.clone()))),
                    ts: None,
                    prev: Some(RowProperty::Style(prev.clone().map(Box::new))),
                });
            }
            for col in sheet.cols.iter().filter(|c| c.style == Some(old_xf)) {
                patches.push(Patch::SetColumnSpan {
                    sheet: id,
                    span: (col.min.clone(), col.max.clone()),
                    property: ColProperty::Style(Some(Box::new(style.clone()))),
                    ts: None,
                    prev: Some(ColProperty::Style(prev.clone().map(Box::new))),
                });
            }
        }
        patches
    }
}

/// Calls phase 5b does not answer. They validate nothing and change nothing: the caller gets an
/// error rather than an edit that would never reach a peer.
macro_rules! unsupported {
    (&self $( $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty; )*) => {
        impl CollabModel<'_> {
            $(
                #[allow(unused_variables, clippy::too_many_arguments)]
                pub fn $name(&self, $( $arg: $ty ),*) -> Result<$ret, String> {
                    Err(UNSUPPORTED.to_string())
                }
            )*
        }
    };
    (&mut self $( $name:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty; )*) => {
        impl CollabModel<'_> {
            $(
                #[allow(unused_variables, clippy::too_many_arguments)]
                pub fn $name(&mut self, $( $arg: $ty ),*) -> Result<$ret, String> {
                    Err(UNSUPPORTED.to_string())
                }
            )*
        }
    };
}

/// What every stub above returns.
const UNSUPPORTED: &str = "unsupported in collab mode";

unsupported! { &self
    get_sheet_markup(sheet: u32) -> String;
    get_dxf_for_conditional_formatting(sheet: u32, index: usize) -> Option<Dxf>;
    get_conditional_formatting_list(sheet: u32) -> Vec<ConditionalFormattingView>;
}

unsupported! { &mut self
    set_language(language_id: &str) -> ();
    set_user_array_formula(sheet: u32, row: i32, column: i32, width: i32, height: i32, value: &str) -> ();
    raise_conditional_formatting_priority(sheet: u32, index: usize) -> ();
    lower_conditional_formatting_priority(sheet: u32, index: usize) -> ();
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::collab::log::{Commit, CommitId, Consumer, SessionId};
    use crate::collab::patch::invert_patches;

    /// Replays what a framework would transport: each [`LocalCommit`] as one commit, carrying the
    /// stamp its author minted.
    fn deliver(model: &mut CollabModel<'_>, session: SessionId, commits: &[LocalCommit]) {
        for (i, commit) in commits.iter().enumerate() {
            model
                .apply(Commit {
                    id: &CommitId::from([i as u8].as_slice()),
                    session: &session,
                    hlc: commit.hlc,
                    patches: &commit.patches,
                })
                .unwrap();
        }
    }

    /// The full loop: emit locally, ship, converge. Both replicas must end up with the same
    /// document — registers included — and evaluate to the same values.
    #[test]
    fn emission_loop() {
        let mut a = CollabModel::new(1);
        let (name, sheet) = a.new_sheet();
        assert_eq!((name.as_str(), sheet), ("Sheet1", 0));

        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "20".to_string()).unwrap();
        a.set_user_input(0, 3, 1, "text".to_string()).unwrap();
        a.set_user_input(0, 4, 1, "TRUE".to_string()).unwrap();
        a.set_user_input(0, 1, 2, "=A1+A2".to_string()).unwrap();
        a.set_user_input(0, 2, 2, "=B1*2".to_string()).unwrap();
        a.set_user_input(0, 3, 2, "=CONCAT(A3, \"!\")".to_string())
            .unwrap();
        // Far past anything materialized: the write carries the rows it needs with it.
        a.set_user_input(0, 500, 3, "=B1".to_string()).unwrap();
        a.update_cell_with_number(0, 5, 1, 7.5).unwrap();
        a.update_cell_with_text(0, 6, 1, "=not a formula").unwrap();
        a.update_cell_with_bool(0, 7, 1, false).unwrap();
        a.update_cell_with_formula(0, 8, 1, "=A1*3".to_string())
            .unwrap();
        a.set_user_input(0, 9, 1, "1".to_string()).unwrap();
        a.cell_clear_contents(0, 9, 1).unwrap();
        a.evaluate();

        // Ordinal reads: the drop-in accessors see what a user typed.
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 2), Ok("60".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 3, 2), Ok("text!".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 500, 3), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 4, 1), Ok("TRUE".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 5, 1), Ok("7.5".to_string()));
        // Quoted: it reads back as text, not as a formula.
        assert_eq!(
            a.get_formatted_cell_value(0, 6, 1),
            Ok("=not a formula".to_string())
        );
        assert_eq!(a.get_formatted_cell_value(0, 7, 1), Ok("FALSE".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 8, 1), Ok("30".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 9, 1), Ok("".to_string()));
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 500);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), 3);

        // Every mutator call is exactly one commit.
        let commits = a.flush();
        assert_eq!(commits.len(), 15);
        assert!(a.flush().is_empty());

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &commits);
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
        assert_eq!(b.workbook.meta, a.workbook.meta);
        for (row, column) in [(1, 2), (2, 2), (3, 2), (500, 3), (4, 1), (8, 1)] {
            assert_eq!(
                b.get_formatted_cell_value(0, row, column),
                a.get_formatted_cell_value(0, row, column),
                "value at ({row}, {column})"
            );
        }

        // Redelivering the same commits changes nothing.
        deliver(&mut b, 1, &commits);
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// Assert that every shared formula is parsed.
    fn assert_parsed_covers_shared(model: &CollabModel<'_>) {
        assert_eq!(model.parsed_formulas.len(), model.workbook.worksheets.len());
        for (i, sheet) in model.workbook.worksheets.iter().enumerate() {
            assert_eq!(
                model.parsed_formulas[i].len(),
                sheet.shared_formulas.len(),
                "parse table of sheet {i}"
            );
        }
    }

    /// A content-only commit lowers only the entry it appends.
    #[test]
    fn content_only_keeps_old_formulas() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("20".to_string()));

        // Content-only: a cell write into an existing row, then one into a fresh one.
        a.set_user_input(0, 1, 1, "5".to_string()).unwrap();
        a.set_user_input(0, 30, 1, "=A1+1".to_string()).unwrap();
        a.evaluate();

        assert_eq!(a.get_formatted_cell_value(0, 30, 1), Ok("6".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("10".to_string()));
        assert_parsed_covers_shared(&a);
    }

    /// The gate on `resync_derived`, counted: content-only commits must never take the full path,
    /// structural ones always must — locally and on the remote side.
    #[test]
    fn full_resync_gate_is_counted() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap();
        a.evaluate();

        // A second replica, caught up on the setup, to watch the same gate on `Consumer::apply`.
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());

        // Content-only: nothing here may reach the full path.
        let before = a.local.full_resyncs;
        let style = Style {
            quote_prefix: true,
            ..Default::default()
        };
        a.set_user_input(0, 1, 1, "5".to_string()).unwrap();
        // A fresh far row mints tail keys — not affecting prior formulas
        a.set_user_input(0, 60, 1, "=A1+1".to_string()).unwrap();
        a.set_cell_style(0, 1, 1, &style).unwrap();
        a.set_merged_range(0, &RangeRef::parse_a1("A1:A2").unwrap(), true)
            .unwrap();
        a.set_comment(0, 1, 1, Some(("note".to_string(), "me".to_string())))
            .unwrap();
        assert_eq!(a.local.full_resyncs, before);

        // And the document is still honest: the fresh formula evaluates, every formula is parsed.
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 60, 1), Ok("6".to_string()));
        assert_parsed_covers_shared(&a);

        // The same commits, delivered: the remote side must not take the full path either.
        let content = a.flush();
        assert!(!content.is_empty());
        let before = b.local.full_resyncs;
        deliver(&mut b, 1, &content);
        assert_eq!(b.local.full_resyncs, before);
        b.evaluate();
        assert_eq!(b.get_formatted_cell_value(0, 60, 1), Ok("6".to_string()));
        assert_parsed_covers_shared(&b);

        // Structural: each of these makes one commit, and each must trigger re-evaluation
        macro_rules! bumps {
            ($what:literal, $call:expr) => {{
                let before = a.local.full_resyncs;
                $call;
                assert_eq!(a.local.full_resyncs, before + 1, $what);
            }};
        }
        bumps!("insert_rows", a.insert_rows(0, 1, 1).unwrap());
        bumps!("delete_rows", a.delete_rows(0, 1, 1).unwrap());
        bumps!("rename_sheet", a.rename_sheet_by_index(0, "Data").unwrap());
        bumps!("new_sheet", a.new_sheet());
        bumps!(
            "new_defined_name",
            a.new_defined_name("total", None, "Data!$A$1").unwrap()
        );
        assert_parsed_covers_shared(&a);

        // Delivered, they take the full path on the remote side too: one per commit.
        let structural = a.flush();
        assert_eq!(structural.len(), 5);
        let before = b.local.full_resyncs;
        deliver(&mut b, 1, &structural);
        assert_eq!(b.local.full_resyncs, before + 5);
    }

    /// The structural kinds all move something a lowered node embeds, so each must still resync.
    #[test]
    fn structural_commits_resync() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "7".to_string()).unwrap();
        a.set_user_input(0, 1, 2, "=A2".to_string()).unwrap();
        a.new_defined_name("total", None, "Sheet1!$A$2").unwrap();
        a.set_user_input(1, 1, 1, "=Sheet1!A2".to_string()).unwrap();
        a.set_user_input(1, 2, 1, "=total".to_string()).unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("7".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("7".to_string()));

        // A mid-sheet insert shifts what row 2 is: the reference follows the row, not the ordinal.
        a.insert_rows(0, 2, 1).unwrap();
        a.evaluate();
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=A3".to_string())));
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("7".to_string()));

        // A rename re-renders the cross-sheet reference.
        a.rename_sheet_by_index(0, "Data").unwrap();
        a.evaluate();
        assert_eq!(
            a.get_cell_formula(1, 1, 1),
            Ok(Some("=Data!A3".to_string()))
        );
        assert_eq!(a.get_formatted_cell_value(1, 1, 1), Ok("7".to_string()));

        // Redefining a name changes what the cells reading it evaluate to.
        a.update_defined_name("total", None, "total", None, "Data!$A$1")
            .unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("1".to_string()));
        assert_parsed_covers_shared(&a);
    }

    #[test]
    fn remote_content_only_commit() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());

        // remote (a) brings formula that needs to be evaluated even if it's content-only
        a.set_user_input(0, 40, 1, "=A1*3".to_string()).unwrap();
        deliver(&mut b, 1, &a.flush());
        b.evaluate();

        assert_eq!(b.get_formatted_cell_value(0, 40, 1), Ok("30".to_string()));
        assert_parsed_covers_shared(&b);
    }

    /// Sheet ids are minted per session, not per state: two replicas holding the same document and
    /// creating a sheet at the same time must keep both sheets, not race for one id.
    #[test]
    fn concurrent_sheet_ids() {
        let mut a = CollabModel::new(1);
        let mut b = CollabModel::new(2);
        a.new_sheet();
        b.new_sheet();
        assert_ne!(
            a.workbook.worksheets[0].sheet_id,
            b.workbook.worksheets[0].sheet_id
        );
        // A second sheet on the same replica moves the slot on, so ids stay distinct locally too.
        a.new_sheet();
        assert_ne!(
            a.workbook.worksheets[0].sheet_id,
            a.workbook.worksheets[1].sheet_id
        );

        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        assert_eq!(a.workbook.worksheets.len(), 3);
        assert_eq!(a.workbook, b.workbook);
        assert_eq!(a.workbook.meta, b.workbook.meta);
    }

    /// We use macro, since script is running against both `TestModel` and `CollabModel`, which atm.
    /// don't share common API.
    macro_rules! script {
        ($m:expr) => {{
            let m = &mut $m;
            for row in 1..=5 {
                m.set_user_input(0, row, 1, format!("{row}")).unwrap();
            }
            m.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
            // Spans the rows the insert splits.
            m.set_user_input(0, 5, 2, "=SUM(A1:A5)".to_string())
                .unwrap();
            m.set_user_input(0, 1, 3, "=A3+1".to_string()).unwrap();
            m.set_user_input(0, 1, 4, "100".to_string()).unwrap();
            // Points at the column the delete removes: it has to end up as #REF!.
            m.set_user_input(0, 1, 5, "=D1+1".to_string()).unwrap();
            // Absolute, and above every edit, so it means the same thing afterwards.
            m.new_defined_name("top", None, "Sheet1!$A$1").unwrap();
            m.set_user_input(0, 2, 5, "=top*10".to_string()).unwrap();

            m.insert_rows(0, 3, 2).unwrap();
            m.delete_columns(0, 4, 1).unwrap();
            // Both blocks cross formulas already in the sheet, so the rewrites are exercised.
            // Neither crosses A1, which `top` names: upstream leaves defined names alone on a
            // structural edit and emission does not, so the script keeps off that divergence.
            m.move_rows_action(0, 2, 2, 4).unwrap();
            m.move_columns_action(0, 2, 1, 1).unwrap();
            m.move_rows_action(0, 6, 2, 3).unwrap();
            m.evaluate();
        }};
    }

    #[test]
    fn structural_rewrite() {
        let mut oracle = crate::test::util::new_empty_model();
        script!(oracle);

        let mut a = CollabModel::new(1);
        a.new_sheet();
        script!(a);

        // The oracle populates every cell it touched; the collaborative model must agree on all
        // of them, and hold nothing the oracle does not.
        let cells = oracle.get_all_cells();
        assert!(cells.len() > 8);
        for cell in &cells {
            let (sheet, row, column) = (cell.index, cell.row, cell.column);
            assert_eq!(
                a.get_formatted_cell_value(sheet, row, column),
                oracle.get_formatted_cell_value(sheet, row, column),
                "value at ({sheet}, {row}, {column})"
            );
            assert_eq!(
                a.get_cell_formula(sheet, row, column),
                oracle.get_cell_formula(sheet, row, column),
                "formula at ({sheet}, {row}, {column})"
            );
        }
        assert_eq!(a.get_all_cells().len(), cells.len());
        // The rewrite really did happen. The reference across the insert grew to A1:A7; the row
        // move then sent row 7 to 5 and pulled the range's top end down with it, and the column
        // move carried the formula itself from B to C.
        assert_eq!(
            a.get_cell_formula(0, 5, 3),
            Ok(Some("=SUM(A1:A5)".to_string()))
        );
        assert_eq!(a.get_formatted_cell_value(0, 5, 3), Ok("13".to_string()));
        // The one into the deleted column broke, and column D was left where it was.
        assert_eq!(
            a.get_cell_formula(0, 1, 4),
            Ok(Some("=#REF!+1".to_string()))
        );
        // The last move reaches past every materialized row, so rows 6 and 7 land on 9 and 10 —
        // ordinals that only exist because the commit carried them.
        assert_eq!(a.get_formatted_cell_value(0, 9, 4), Ok("10".to_string()));
        // The moved rows carried their contents: row 2 held 2, and now row 9 does.
        assert_eq!(a.get_formatted_cell_value(0, 9, 1), Ok("2".to_string()));
        // A reference into the block follows the row it names, not the ordinal it had.
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=A3+1".to_string())));

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// A move racing an edit inside the block it moves. B addresses the cell by the ordinal it
    /// still sees, and identity addressing has to land that edit on the row's new ordinal.
    #[test]
    fn concurrent_move_and_edit() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        for row in 1..=4 {
            a.set_user_input(0, row, 1, format!("{row}")).unwrap();
        }
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Rows 1 and 2 go to 3 and 4, landing on the last ordinal; B, still on the old view, edits
        // the second of them.
        a.move_rows_action(0, 1, 2, 2).unwrap();
        b.set_user_input(0, 2, 1, "edited".to_string()).unwrap();

        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        a.evaluate();
        b.evaluate();

        // The edit traveled with the row rather than staying on the ordinal it named.
        for m in [&a, &b] {
            assert_eq!(
                m.get_formatted_cell_value(0, 4, 1),
                Ok("edited".to_string())
            );
            assert_eq!(m.get_formatted_cell_value(0, 3, 1), Ok("1".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("3".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("4".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);

        // The move above left minted keys on the tail. A second move reaching *past* that tail has
        // to materialize rows that still sort last, and a write far below has to land where it says.
        a.move_rows_action(0, 1, 1, 4).unwrap();
        a.set_user_input(0, 8, 1, "far".to_string()).unwrap();
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // Row 1 went to 5, the rest shifted up one.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("4".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("1".to_string()));
            assert_eq!(
                m.get_formatted_cell_value(0, 3, 1),
                Ok("edited".to_string())
            );
            assert_eq!(m.get_formatted_cell_value(0, 4, 1), Ok("".to_string()));
            assert_eq!(m.get_formatted_cell_value(0, 5, 1), Ok("3".to_string()));
            // Named row 8, reads back at row 8 — not at whatever ordinal a stale mint would give.
            assert_eq!(m.get_formatted_cell_value(0, 8, 1), Ok("far".to_string()));
            assert_eq!(m.workbook.worksheets[0].index.rows.len(), 8);
        }
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn concurrent_tail_move() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        for row in 1..=3 {
            a.set_user_input(0, row, 1, format!("{row}")).unwrap(); // A1:A3=1..3
        }
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Row 6 is past the last materialized row on both replicas.
        // - A: moves it up one
        // - B: on the stale view, writes into a row that move materializes
        //      and moves row 1 down past it.
        a.move_rows_action(0, 6, 1, -1).unwrap(); // 6 -> 5
        b.set_user_input(0, 5, 1, "b".to_string()).unwrap(); // A5=b
        b.move_rows_action(0, 1, 1, 1).unwrap(); // 1 -> 2

        let (from_a, from_b) = (a.flush(), b.flush());
        // The same commits on two fresh replicas, delivered in opposite orders.
        let mut ab = CollabModel::new(3);
        let mut ba = CollabModel::new(4);
        deliver(&mut ab, 1, &setup);
        deliver(&mut ab, 1, &from_a);
        deliver(&mut ab, 2, &from_b);
        deliver(&mut ba, 1, &setup);
        deliver(&mut ba, 2, &from_b);
        deliver(&mut ba, 1, &from_a);
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        for m in [&mut a, &mut b, &mut ab, &mut ba] {
            m.evaluate();
        }

        // Row 6 landed between the two rows the commit minted, so it reads at 5; B's write went to
        // the row it named, which the move pushed down one; row 1 followed B's move to row 2.
        for m in [&a, &b, &ab, &ba] {
            // B moved row 1 -> 2: so A1 and A2 are swapped in places
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("2".to_string())); // A1=2
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("1".to_string())); // A2=1

            assert_eq!(m.get_formatted_cell_value(0, 3, 1), Ok("3".to_string())); // A3=3

            // B written A5=b, but A moved row 6 -> 5, so A5 is also swapped with A6
            assert_eq!(m.get_formatted_cell_value(0, 6, 1), Ok("b".to_string())); // A6=b
            assert_eq!(m.workbook.worksheets[0].index.rows.len(), 6);
        }
        assert_eq!(b.workbook, a.workbook);
        assert_eq!(ab.workbook, a.workbook);
        assert_eq!(ba.workbook, a.workbook);

        // Redelivery of the move commit to a replica that already has it is a no-op.
        let settled = projection(&b);
        deliver(&mut b, 1, &from_a);
        b.evaluate();
        assert_eq!(projection(&b), settled);
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn undo_tail_move() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        for row in 1..=3 {
            a.set_user_input(0, row, 1, format!("{row}")).unwrap(); //A1:A3=1..3
        }
        a.flush();
        let before = projection(&a);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 3); // total rows: 3

        a.move_rows_action(0, 6, 1, -1).unwrap(); // row 6 -> 5
        let moved = a.flush();
        let after = projection(&a);
        assert_ne!(after, before);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 6); // total rows: 6 (after move)

        let invert = |commits: &[LocalCommit]| -> Vec<Patch> {
            commits
                .iter()
                .rev()
                .flat_map(|commit| invert_patches(&commit.patches))
                .collect()
        };
        a.commit_local(invert(&moved));
        let undone = a.flush();
        assert_eq!(projection(&a), before);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 3); // total rows: 3 (after undo)

        // Redo is the inverse of the inverse.
        a.commit_local(invert(&undone));
        a.flush();
        assert_eq!(projection(&a), after);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 6); // total rows: 6 (after redo)
    }

    /// What a replica shows, which is what an undo has to restore: contents and formatting by
    /// ordinal position, plus the sheet and workbook fields. Deliberately not the CRDT
    /// bookkeeping — guards, tombstones and the append-only intern tables only move forward.
    fn projection(model: &CollabModel<'_>) -> String {
        let mut out = format!("{:?}\n", model.workbook.defined_names);
        for (sheet, ws) in model.workbook.worksheets.iter().enumerate() {
            let sheet = sheet as u32;
            out += &format!(
                "{} {:?} {:?} {} {:?} {:?}\n",
                ws.name, ws.color, ws.state, ws.show_grid_lines, ws.merged_cells, ws.comments
            );
            for column in 1..=Stable::col_count(&ws.index) {
                out += &format!(
                    "c{column} {:?} {:?}\n",
                    model.get_column_width(sheet, column),
                    model.is_column_hidden(sheet, column)
                );
            }
            for row in 1..=Stable::row_count(&ws.index) {
                out += &format!(
                    "r{row} {:?} {:?}\n",
                    model.get_row_height(sheet, row),
                    model.is_row_hidden(sheet, row)
                );
                for column in 1..=Stable::col_count(&ws.index) {
                    out += &format!(
                        "{row},{column} {:?} {:?}\n",
                        model.get_localized_cell_content(sheet, row, column),
                        model.get_cell_style_or_none(sheet, row, column)
                    );
                }
            }
        }
        out
    }

    /// Undo is a new commit carrying the inverses, and it has to land the projection back where it
    /// started.
    #[test]
    fn undo_inverse() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap();
        a.set_row_height(0, 1, 30.0).unwrap();
        a.set_column_width(0, 1, 80.0).unwrap();
        a.new_defined_name("kept", None, "Sheet1!$A$1").unwrap();
        a.flush();
        let before = projection(&a);
        let (height, width) = (
            a.get_row_height(0, 1).unwrap(),
            a.get_column_width(0, 1).unwrap(),
        );

        let mut style = Style::default();
        style.font.b = true;
        a.set_user_input(0, 1, 1, "99".to_string()).unwrap();
        a.set_cell_style(0, 1, 1, &style).unwrap();
        a.update_cell_with_formula(0, 2, 1, "=A1*3".to_string())
            .unwrap();
        a.set_row_height(0, 1, 55.0).unwrap();
        a.set_column_width(0, 1, 120.0).unwrap();
        a.set_row_hidden(0, 2, true).unwrap();
        a.new_defined_name("temp", None, "Sheet1!$A$2").unwrap();
        a.update_defined_name("kept", None, "kept", None, "Sheet1!$A$2")
            .unwrap();
        let merged = RangeRef::parse_a1("A1:B2").unwrap();
        a.set_merged_range(0, &merged, true).unwrap();
        a.set_comment(0, 1, 1, Some(("note".to_string(), "me".to_string())))
            .unwrap();
        a.set_sheet_color(0, &Color::Rgb("#ff0000".to_string()))
            .unwrap();
        assert_ne!(projection(&a), before);

        // One commit, the inverses of every edit, newest first.
        let undo: Vec<Patch> = a
            .flush()
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        assert_eq!(projection(&a), before);
        assert_eq!(a.workbook.defined_names.len(), 1);
        assert_eq!(a.get_row_height(0, 1), Ok(height));
        assert_eq!(a.get_column_width(0, 1), Ok(width));
        assert!(!a.is_row_hidden(0, 2).unwrap());
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("2".to_string()));
    }

    #[test]
    fn delete_beats_older_write() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 1, "anchor".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Stamped below the deletes, but reaching A only after them.
        b.set_user_input(0, 2, 2, "old".to_string()).unwrap();
        // Row 3 holds nothing outside the doomed column, so B's map for it empties.
        b.set_user_input(0, 3, 1, "old-col".to_string()).unwrap();
        a.delete_rows(0, 2, 1).unwrap();
        a.delete_columns(0, 1, 1).unwrap();

        let late = b.flush();
        let deletes = a.flush();
        deliver(&mut b, 1, &deletes);
        deliver(&mut a, 2, &late);

        // Shared strings are interned per replica, so only the sheet itself has to match.
        let (a_ws, b_ws) = (&a.workbook.worksheets[0], &b.workbook.worksheets[0]);
        assert!(a_ws.sheet_data.is_empty()); // the deletes outrank the writes on both sides
        assert_eq!(b_ws.sheet_data, a_ws.sheet_data);
        assert_eq!(b_ws.index, a_ws.index);
    }

    /// Undoing a delete replays the restores at the stamps they were captured with, so a
    /// concurrent newer edit to a restored cell outlives the undo.
    #[test]
    fn undo_delete_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 2, 1, "keep".to_string()).unwrap();
        a.set_user_input(0, 2, 2, "edit-me".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        a.delete_rows(0, 2, 1).unwrap();
        // Later in program order, so the shared clock stamps it above the delete and the restores.
        b.set_user_input(0, 2, 2, "999".to_string()).unwrap();

        // Undo: one commit carrying the inverses of the delete, newest first.
        let deletes = a.flush();
        let undo: Vec<Patch> = deletes
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        let undone = a.flush();
        deliver(&mut b, 1, &deletes);
        // The surviving edit is dead but invisible until the undo puts its row back.
        assert_eq!(b.get_formatted_cell_value(0, 2, 2), Ok("".to_string()));
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        // The uncontested cell came back; the contested one kept B's newer edit.
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("keep".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 2, 2), Ok("999".to_string()));
        assert_eq!(
            b.workbook.worksheets[0].index.registers,
            a.workbook.worksheets[0].index.registers
        );
        assert_eq!(b.get_formatted_cell_value(0, 2, 1), Ok("keep".to_string()));
        assert_eq!(b.get_formatted_cell_value(0, 2, 2), Ok("999".to_string()));
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn undo_move_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap(); // A1=1
        a.set_user_input(0, 2, 1, "2".to_string()).unwrap(); // A2=2
        a.set_user_input(0, 3, 1, "3".to_string()).unwrap(); // A3=3
        a.set_user_input(0, 1, 2, "=A2*10".to_string()).unwrap(); // B1
        let cell_count = a.get_all_cells().len();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Peer A: move row A2 past the tail (to A6)
        a.move_rows_action(0, 2, 1, 4).unwrap();
        a.evaluate();
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=A6*10".to_string())));
        assert_eq!(a.get_formatted_cell_value(0, 6, 1), Ok("2".to_string()));
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("20".to_string()));

        // Peer B never saw the move: the edit names the row's identity and must follow it both ways
        b.set_user_input(0, 2, 1, "20".to_string()).unwrap(); // A2=20

        // Peer A: undo recent move
        let moved = a.flush();
        let undo: Vec<Patch> = moved
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();

        deliver(&mut b, 1, &moved);
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // A2=20 -> Peer B change is still in effect
            assert_eq!(m.get_formatted_cell_value(0, 2, 1), Ok("20".to_string()));
            // B1=A2*10 -> Peer A undo move (A2->A6) is in effect
            assert_eq!(m.get_cell_formula(0, 1, 2), Ok(Some("=A2*10".to_string())));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("200".to_string()));
            // A6 (move destination) has no value
            assert_eq!(m.get_formatted_cell_value(0, 6, 1), Ok("".to_string()));
        }
        assert_eq!(a.get_all_cells().len(), cell_count);
        assert_eq!(b.workbook, a.workbook);

        // Redo: the inverse of the inverse.
        let redo: Vec<Patch> = undone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(redo);
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // Peer A redo moves A2 back to A6, but that doesn't conflict with Peer's B change A2=20
            assert_eq!(m.get_formatted_cell_value(0, 6, 1), Ok("20".to_string()));
            assert_eq!(m.get_cell_formula(0, 1, 2), Ok(Some("=A6*10".to_string())));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("200".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);
    }

    fn area(row: i32, column: i32, width: i32, height: i32) -> Area {
        Area {
            sheet: 0,
            row,
            column,
            width,
            height,
        }
    }

    fn bold() -> Style {
        let mut style = Style::default();
        style.font.b = true;
        style
    }

    fn italic() -> Style {
        let mut style = Style::default();
        style.font.i = true;
        style
    }

    fn struck() -> Style {
        let mut style = Style::default();
        style.font.strike = true;
        style
    }

    /// How many patches of one kind a commit carries.
    fn count(commit: &LocalCommit, kind: fn(&Patch) -> bool) -> usize {
        commit.patches.iter().filter(|p| kind(p)).count()
    }

    #[test]
    fn style_deletion_and_copy() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        let plain = a.get_style_for_cell(0, 1, 5).unwrap();

        a.set_row_style(0, 2, &bold()).unwrap();
        a.set_column_style(0, 3, &italic()).unwrap();
        a.set_cell_style(0, 2, 1, &struck()).unwrap(); // own style inside the bold row
        a.set_cell_style(0, 7, 3, &struck()).unwrap(); // own style inside the italic column
        let mut wire = a.flush();

        // What each cell inherits before anything is deleted.
        assert_eq!(a.get_style_for_cell(0, 2, 5), Ok(bold()));
        assert_eq!(a.get_style_for_cell(0, 6, 3), Ok(italic()));
        assert_eq!(a.get_style_for_cell(0, 2, 1), Ok(struck()));

        // A copy reads the source's effective style: its own here, its row's below.
        a.copy_cell_style((0, 7, 3), (0, 9, 9)).unwrap();
        a.copy_cell_style((0, 2, 5), (0, 10, 9)).unwrap();
        assert_eq!(a.get_cell_style_or_none(0, 9, 9), Ok(Some(struck())));
        assert_eq!(a.get_cell_style_or_none(0, 10, 9), Ok(Some(bold())));
        wire.extend(a.flush());

        // The row's style goes; the cell that had its own, and the copy taken off the row, do not.
        a.delete_row_style(0, 2).unwrap();
        assert_eq!(a.get_style_for_cell(0, 2, 5), Ok(plain.clone()));
        assert_eq!(a.get_style_for_cell(0, 2, 1), Ok(struck()));
        assert_eq!(a.get_cell_style_or_none(0, 10, 9), Ok(Some(bold())));
        let deleted_row = a.flush();
        assert_eq!(deleted_row.len(), 1); // one user action, one commit

        // Same for the column.
        a.delete_column_style(0, 3).unwrap();
        assert_eq!(a.get_style_for_cell(0, 6, 3), Ok(plain.clone()));
        assert_eq!(a.get_style_for_cell(0, 7, 3), Ok(struck()));
        let deleted_col = a.flush();
        assert_eq!(deleted_col.len(), 1);

        // Nothing to reset is not an edit — the style just cleared, one never set on a row and a
        // column the sheet does hold, and one on a row and a column it has no key for at all.
        a.delete_row_style(0, 2).unwrap();
        a.delete_column_style(0, 3).unwrap();
        a.delete_row_style(0, 7).unwrap();
        a.delete_column_style(0, 1).unwrap();
        a.delete_row_style(0, 400).unwrap();
        a.delete_column_style(0, 40).unwrap();
        assert!(a.flush().is_empty());

        assert_eq!(a.delete_column_style(0, 0), Ok(()));
        assert!(a.flush().is_empty());

        // Upstream's validation, error strings included; a rejected call emits nothing.
        let bad_sheet = Err("Invalid sheet index".to_string());
        assert_eq!(a.delete_row_style(9, 1), bad_sheet);
        assert_eq!(a.delete_column_style(9, 1), bad_sheet);
        assert_eq!(a.copy_cell_style((9, 1, 1), (0, 1, 1)), bad_sheet);
        assert_eq!(a.copy_cell_style((0, 1, 1), (9, 1, 1)), bad_sheet);
        assert_eq!(
            a.copy_cell_style((0, 1, 1), (0, 0, 1)),
            Err("Incorrect row or column".to_string())
        );
        assert!(a.flush().is_empty());

        // The deletion's prev is all undo needs: no inversion of its own.
        let undo: Vec<Patch> = invert_patches(&deleted_row[0].patches);
        a.commit_local(undo);
        assert_eq!(a.get_style_for_cell(0, 2, 5), Ok(bold()));
        assert_eq!(a.get_style_for_cell(0, 6, 3), Ok(plain));

        wire.extend(deleted_row);
        wire.extend(deleted_col);
        wire.extend(a.flush());
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &wire);
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn named_styles() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        let mut wire = a.flush();

        // create, apply, read back
        a.create_named_style("bold", &bold()).unwrap();
        assert_eq!(a.get_named_style("bold"), Ok(bold()));
        assert_eq!(a.get_named_style_list(), ["normal", "bold"]);
        assert_eq!(
            a.create_named_style("bold", &italic()),
            Err("A style with that name already exists".to_string())
        );
        a.set_cell_style_by_name(0, 1, 1, "bold").unwrap();
        a.set_row_style(0, 2, &bold()).unwrap();
        a.set_column_style(0, 3, &bold()).unwrap();
        // Never named, styled the same way by hand: the sweep below cannot tell the two apart.
        a.set_cell_style(0, 5, 5, &bold()).unwrap();
        assert_eq!(a.get_style_for_cell(0, 1, 1), Ok(bold()));

        // modify named style
        let mut bolder = bold();
        bolder.font.i = true;
        a.update_named_style("bold", "bold", &bolder).unwrap();
        assert_eq!(a.get_named_style("bold"), Ok(bolder.clone()));
        // The cell, the row, the column — and the hand-styled cell: style index equality is the
        // test (see `restyle_patches`), unlike upstream, which only moves cells parented to the style.
        for (row, column) in [(1, 1), (2, 5), (6, 3), (5, 5)] {
            assert_eq!(
                a.get_style_for_cell(0, row, column),
                Ok(bolder.clone()),
                "cell ({row}, {column})"
            );
        }

        // rename to 'strong': now 'bold' should be free to be reused
        a.update_named_style("bold", "strong", &bolder).unwrap();
        assert_eq!(a.get_named_style_list(), ["normal", "strong"]);
        a.create_named_style("bold", &italic()).unwrap();
        assert_eq!(a.get_named_style("bold"), Ok(italic()));
        assert_eq!(a.get_named_style("strong"), Ok(bolder.clone()));
        // Two registers, not one: the second "bold" salted its way past the live id.
        assert_eq!(a.workbook.meta.named_styles.len(), 2);

        // delete named style 'strong'
        wire.extend(a.flush());
        let strong = a.style_id_by_name("strong").unwrap();
        a.delete_named_style("strong").unwrap();
        // style is no longer reachable by name, but it's still referenced by cell
        // this behavior differs from Excel, but it's how IronCalc works atm.
        assert!(a.get_named_style("strong").is_err());
        assert_eq!(a.get_style_for_cell(0, 1, 1), Ok(bolder.clone()));
        let deleted = a.flush();
        a.commit_local(invert_patches(&deleted[0].patches));
        assert_eq!(a.get_named_style("strong"), Ok(bolder.clone()));
        assert_eq!(a.style_id_by_name("strong"), Ok(strong));
        wire.extend(deleted);
        wire.extend(a.flush());

        assert!(
            a.delete_named_style("normal").is_err(),
            "can't delete built-in style"
        );
        assert!(
            a.update_named_style("normal", "plain", &bold()).is_err(),
            "can't modify built-in styles"
        );
        assert!(a.delete_named_style("nope").is_err(), "style missing");
        assert!(
            a.update_named_style("nope", "x", &bold()).is_err(),
            "style missing"
        );
        assert!(
            a.update_named_style("bold", "strong", &italic()).is_err(),
            "rename to existing"
        );
        assert!(a.flush().is_empty());

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &wire);
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);

        // two peers creating the same style give it the same id
        a.create_named_style("shared", &bold()).unwrap();
        b.create_named_style("shared", &italic()).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut a, 2, &from_b);
        deliver(&mut b, 1, &from_a);
        for peer in [&a, &b] {
            // The later definition stands, and there is only ever one name to hold it.
            assert_eq!(peer.get_named_style("shared"), Ok(italic()));
            assert_eq!(
                peer.get_named_style_list()
                    .iter()
                    .filter(|n| n.starts_with("shared"))
                    .count(),
                1
            );
        }
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);

        // two peers rename the same style and update separate properties
        a.update_named_style("shared", "renamed", &italic())
            .unwrap();
        b.update_named_style("shared", "shared", &bold()).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut a, 2, &from_b);
        deliver(&mut b, 1, &from_a);
        for peer in [&a, &b] {
            // rename last-write-wins conflict resolution
            assert_eq!(peer.get_named_style("renamed"), Ok(bold()));
        }
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);

        // two peers rename different styles to the same name
        a.update_named_style("renamed", "same", &bold()).unwrap();
        b.update_named_style("bold", "same", &italic()).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut a, 2, &from_b);
        deliver(&mut b, 1, &from_a);
        for peer in [&a, &b] {
            // same rename conflict resolution -> add " (1)" to differentiate
            assert_eq!(peer.get_named_style("same"), Ok(bold()));
            assert_eq!(peer.get_named_style("same (1)"), Ok(italic()));
        }
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    /// Ships each replica's pending commits to the other, then checks they converged.
    fn exchange(a: &mut CollabModel<'_>, b: &mut CollabModel<'_>) {
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(a, 2, &from_b);
        deliver(b, 1, &from_a);
        a.evaluate();
        b.evaluate();
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn defined_name_identity() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Other").unwrap();
        a.set_user_input(0, 3, 1, "7".to_string()).unwrap(); // A3=7
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);
        let (sheet1_id, other_id) = (
            a.workbook.worksheets[0].sheet_id,
            a.workbook.worksheets[1].sheet_id,
        );

        // same name on both peers => same entity (case-insensitive)
        a.new_defined_name("total", None, "Sheet1!$A$1").unwrap();
        b.new_defined_name("Total", None, "Sheet1!$A$2").unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.workbook.meta.defined_names.len(), 1);
        // Both of the later commit's registers won, its spelling of the name included.
        assert_eq!(
            defined(&a),
            [("Total".to_string(), None, "Sheet1!$A$2".to_string())]
        );

        // names are unique per scope
        a.new_defined_name("total", Some(1), "Other!$A$1").unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(
            defined(&a),
            [
                ("Total".to_string(), None, "Sheet1!$A$2".to_string()),
                (
                    "total".to_string(),
                    Some(other_id),
                    "Other!$A$1".to_string()
                ),
            ]
        );

        // A renames, B changes definition -> both succeed (grand=Sheet1!$A$3)
        a.update_defined_name("Total", None, "grand", None, "Sheet1!$A$2")
            .unwrap();
        b.update_defined_name("Total", None, "Total", None, "Sheet1!$A$3")
            .unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.workbook.meta.defined_names.len(), 2);
        assert!(defined(&a).contains(&("grand".to_string(), None, "Sheet1!$A$3".to_string())));

        // 2 peers, 2 different names for the same formula -> 2 different entities
        a.new_defined_name("alpha", None, "Sheet1!$A$1").unwrap();
        a.new_defined_name("beta", None, "Sheet1!$A$2").unwrap();
        exchange(&mut a, &mut b);
        // renamed 2 defined names to the same one should trigger name repair
        a.update_defined_name("alpha", None, "merged", None, "Sheet1!$A$1")
            .unwrap();
        b.update_defined_name("beta", None, "merged", None, "Sheet1!$A$2")
            .unwrap();
        exchange(&mut a, &mut b);
        for peer in [&a, &b] {
            let names = defined(peer);
            // name repair
            assert!(names.contains(&("merged".to_string(), None, "Sheet1!$A$1".to_string())));
            assert!(names.contains(&("merged (1)".to_string(), None, "Sheet1!$A$2".to_string())));
        }

        // 'alpha' was renamed in the past, reusing the name creates new entity
        a.new_defined_name("alpha", None, "Other!$A$1").unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.workbook.meta.defined_names.len(), 5);
        let names = defined(&a);
        assert!(names.contains(&("alpha".to_string(), None, "Other!$A$1".to_string())));
        assert!(names.contains(&("merged".to_string(), None, "Sheet1!$A$1".to_string())));

        a.update_defined_name("alpha", None, "alpha", Some(0), "Other!$A$1")
            .unwrap();
        b.update_defined_name("alpha", None, "alpha", None, "Sheet1!$A$9")
            .unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.workbook.meta.defined_names.len(), 5);
        assert!(defined(&a).contains(&(
            "alpha".to_string(),
            Some(sheet1_id),
            "Sheet1!$A$9".to_string()
        )));

        // delete keeps the entry, so the undo revives id, address and formula
        let before = defined(&a);
        a.delete_defined_name("merged", None).unwrap();
        let gone = a.flush();
        deliver(&mut b, 1, &gone);
        let after_delete = defined(&a);
        assert_eq!(after_delete.len(), before.len() - 1);
        // Repair is derived, not authored: with the winner gone the loser stops being renumbered.
        assert!(after_delete.contains(&("merged".to_string(), None, "Sheet1!$A$2".to_string())));
        assert!(!after_delete.iter().any(|(name, ..)| name == "merged (1)"));
        let undo: Vec<Patch> = gone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        exchange(&mut a, &mut b);
        assert_eq!(defined(&a), before);
        assert_eq!(a.workbook.meta.defined_names.len(), 5);

        // ---- formulas still name a defined name by display text, so a rename rewrites them.
        //      Pre-`Node<A: Position>` semantics, pinned as such ----
        a.new_defined_name("rate", None, "Sheet1!$A$3").unwrap();
        a.set_user_input(0, 1, 2, "=rate*2".to_string()).unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.get_formatted_cell_value(0, 1, 2), Ok("14".to_string()));
        a.update_defined_name("rate", None, "fee", None, "Sheet1!$A$3")
            .unwrap();
        exchange(&mut a, &mut b);
        assert_eq!(a.get_cell_formula(0, 1, 2), Ok(Some("=fee*2".to_string())));
        assert_eq!(b.get_formatted_cell_value(0, 1, 2), Ok("14".to_string()));

        // an update that changes neither register produces no changes
        a.update_defined_name("fee", None, "fee", None, "Sheet1!$A$3")
            .unwrap();
        assert!(a.flush().is_empty());

        // upstream's validation, word for word
        assert_eq!(
            a.new_defined_name("1bad", None, "Sheet1!$A$1"),
            Err("Name: Invalid defined name".to_string())
        );
        assert_eq!(
            a.new_defined_name("FEE", None, "Sheet1!$A$1"),
            Err("Name: Defined name already exists".to_string())
        );
        assert_eq!(
            a.new_defined_name("x", Some(9), "Sheet1!$A$1"),
            Err("Scope: Invalid sheet index".to_string())
        );
        assert_eq!(
            a.delete_defined_name("nope", None),
            Err("Defined name not found".to_string())
        );
        assert_eq!(
            a.update_defined_name("nope", None, "y", None, "Sheet1!$A$1"),
            Err("Defined name not found".to_string())
        );
        assert_eq!(
            a.update_defined_name("fee", None, "1bad", None, "Sheet1!$A$1"),
            Err("Name: Invalid defined name".to_string())
        );
        assert_eq!(
            a.update_defined_name("fee", None, "alpha", Some(0), "Sheet1!$A$1"),
            Err("Name: Defined name already exists".to_string())
        );
        assert!(a.flush().is_empty());
    }

    /// Clearing a range: contents go, formatting stays or goes with them, and the coordinates the
    /// range names but the sheet does not hold are left alone — nothing is materialized.
    #[test]
    fn range_clear() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap(); // A1=1
        a.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap(); // A2=A1*2
        a.set_cell_style(0, 1, 2, &bold()).unwrap(); // B1=(bold)
        a.set_user_input(0, 2, 2, "keep".to_string()).unwrap(); // B2="keep"
        a.set_cell_style(0, 2, 2, &italic()).unwrap(); // B2=(italic)
        a.set_user_input(0, 1, 3, "100".to_string()).unwrap(); // C1=100
        a.flush();

        // A1:B4 — rows 3 and 4 do not exist yet, and A3..B4 were never written.
        a.range_clear_contents(&area(1, 1, 2, 4)).unwrap();
        a.evaluate();

        let commits = a.flush();
        assert_eq!(commits.len(), 1); // one user action, one commit
        let clear = &commits[0];
        // Only the four cells the sheet holds are written, and nothing is materialized for the rest.
        assert_eq!(count(clear, |p| matches!(p, Patch::SetCellValue { .. })), 4);
        assert_eq!(count(clear, |p| matches!(p, Patch::SetCellStyle { .. })), 4);
        assert_eq!(clear.patches.len(), 8);
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 2);

        // Contents gone, formatting kept.
        for (row, column) in [(1, 1), (2, 1), (2, 2)] {
            assert_eq!(
                a.get_formatted_cell_value(0, row, column),
                Ok(String::new()),
                "value at ({row}, {column})"
            );
        }
        assert_eq!(a.get_cell_formula(0, 2, 1), Ok(None));
        assert_eq!(a.get_cell_style_or_none(0, 1, 2), Ok(Some(bold())));
        assert_eq!(a.get_cell_style_or_none(0, 2, 2), Ok(Some(italic())));
        // Outside the range nothing moved.
        assert_eq!(a.get_formatted_cell_value(0, 1, 3), Ok("100".to_string()));

        // The clear left the four cells behind, holding just their styles, so they are cleared again.
        a.range_clear_all(&area(1, 1, 2, 4)).unwrap();
        let commits = a.flush();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].patches.len(), 8);
        assert_eq!(a.get_cell_style_or_none(0, 1, 2), Ok(None));
        assert_eq!(a.get_cell_style_or_none(0, 2, 2), Ok(None));
        // Nothing left in the range at all — only C1 survives.
        assert_eq!(a.get_all_cells().len(), 1);
        assert_eq!(a.get_formatted_cell_value(0, 1, 3), Ok("100".to_string()));

        // A range the sheet holds nothing in is not an edit at all: no commit, no rows minted.
        a.range_clear_all(&area(6, 4, 2, 2)).unwrap();
        a.range_clear_contents(&area(1, 1, 2, 4)).unwrap();
        assert!(a.flush().is_empty());
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), 2);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), 3);

        // Upstream's validation, error string included; a rejected call emits nothing.
        let bad = "Row or column is outside valid range.".to_string();
        assert_eq!(a.range_clear_contents(&area(0, 1, 1, 1)), Err(bad.clone()));
        assert_eq!(a.range_clear_all(&area(1, 0, 1, 1)), Err(bad));
        let off_sheet = Area {
            sheet: 9,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        };
        assert_eq!(
            a.range_clear_all(&off_sheet),
            Err("Invalid sheet index".to_string())
        );
        assert!(a.flush().is_empty());
    }

    #[test]
    fn range_clear_converges() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_user_input(0, 1, 2, "2".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // Both stamped below the clear and reaching A only after it. A1 is a cell the clear can
        // see; B3 is not — row 3 does not exist when the clear is authored.
        b.set_user_input(0, 1, 1, "11".to_string()).unwrap();
        b.set_user_input(0, 3, 2, "33".to_string()).unwrap();
        a.range_clear_contents(&area(1, 1, 2, 3)).unwrap();
        // Later in program order, so the shared clock stamps it above the clear.
        b.set_user_input(0, 1, 2, "42".to_string()).unwrap();

        let clear = a.flush();
        let from_b = b.flush();
        deliver(&mut b, 1, &clear);
        deliver(&mut a, 2, &from_b);
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // The older write to a cell the clear saw lost; the newer one to the same cell won.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok(String::new()));
            assert_eq!(m.get_formatted_cell_value(0, 1, 2), Ok("42".to_string()));
            // The clear never named B3, so nothing there was ever contested.
            assert_eq!(m.get_formatted_cell_value(0, 3, 2), Ok("33".to_string()));
        }
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn range_clear_undo_concurrent() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.set_cell_style(0, 1, 1, &bold()).unwrap();
        a.set_user_input(0, 2, 1, "edit-me".to_string()).unwrap();
        a.set_user_input(0, 3, 1, "3".to_string()).unwrap();
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // A clear is nothing but register writes, so its inverse is nothing but register writes.
        a.range_clear_all(&area(1, 1, 1, 3)).unwrap();
        b.set_user_input(0, 2, 1, "999".to_string()).unwrap();

        let cleared = a.flush();
        let undo: Vec<Patch> = cleared
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);

        let undone = a.flush();
        deliver(&mut b, 1, &cleared);
        deliver(&mut b, 1, &undone);
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();

        for m in [&a, &b] {
            // The uncontested cells came back, style included.
            assert_eq!(m.get_formatted_cell_value(0, 1, 1), Ok("1".to_string()));
            assert_eq!(m.get_cell_style_or_none(0, 1, 1), Ok(Some(bold())));
            assert_eq!(m.get_formatted_cell_value(0, 3, 1), Ok("3".to_string()));
            // The contested one too: the undo's stamp is above B's edit, so the restore wins.
            //TODO: should contesting undo win over concurrent edit?
            assert_eq!(
                m.get_formatted_cell_value(0, 2, 1),
                Ok("edit-me".to_string())
            );
        }
        assert_eq!(b.workbook, a.workbook);
    }

    /// The tab order a replica shows.
    fn names(model: &CollabModel<'_>) -> Vec<String> {
        model
            .get_worksheets_properties()
            .into_iter()
            .map(|p| p.name)
            .collect()
    }

    /// Sheet lifecycle: named creation, insertion anywhere in the tab order, deletion by name and
    /// the undo of an add — all of it converging on a second replica.
    #[test]
    fn sheet_lifecycle() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Beta").unwrap();
        a.add_sheet("Gamma").unwrap();
        assert_eq!(names(&a), ["Sheet1", "Beta", "Gamma"]);
        // A named add lands at the end, and the ordinal writers can address it there.
        a.set_user_input(2, 1, 1, "gamma".to_string()).unwrap();

        a.insert_sheet("Mid", 1, None).unwrap();
        assert_eq!(names(&a), ["Sheet1", "Mid", "Beta", "Gamma"]);
        a.insert_sheet("First", 0, None).unwrap();
        a.insert_sheet("Last", 5, None).unwrap();
        assert_eq!(
            names(&a),
            ["First", "Sheet1", "Mid", "Beta", "Gamma", "Last"]
        );
        // Two inserts into the same gap still order by the index each named.
        a.insert_sheet("Mid2", 3, None).unwrap();
        assert_eq!(
            names(&a),
            ["First", "Sheet1", "Mid", "Mid2", "Beta", "Gamma", "Last"]
        );

        // Validation, with the ordinal writers' messages verbatim.
        assert_eq!(
            a.add_sheet("bad/name"),
            Err("Invalid name for a sheet: 'bad/name'".to_string())
        );
        assert_eq!(
            a.add_sheet("bETA"),
            Err("A worksheet already exists with that name".to_string())
        );
        assert_eq!(
            a.insert_sheet("Nope", 99, None),
            Err("Sheet index out of range".to_string())
        );
        // An id already spoken for is refused: reusing one would address that sheet, not create one.
        let taken = a.workbook.worksheets[0].sheet_id;
        assert_eq!(
            a.insert_sheet("Nope", 0, Some(taken)),
            Err(format!("Sheet id {taken} is not available"))
        );
        assert_eq!(
            a.insert_sheet("Nope", 0, Some(0)),
            Err("Sheet id 0 is not available".to_string())
        );
        a.insert_sheet("Chosen", 0, Some(4242)).unwrap();
        assert_eq!(a.workbook.worksheets[0].sheet_id, 4242);

        // Deletion by name is case insensitive, and an unknown name is an error, not a no-op.
        a.delete_sheet_by_name("mid2").unwrap();
        assert_eq!(
            a.delete_sheet_by_name("Mid2"),
            Err("Sheet not found".to_string())
        );
        assert_eq!(
            names(&a),
            ["Chosen", "First", "Sheet1", "Mid", "Beta", "Gamma", "Last"]
        );

        // A dead sheet's key is still filed, so an insert into the gap it left must not re-mint it.
        a.insert_sheet("Revived", 3, None).unwrap();
        assert_eq!(names(&a)[3], "Revived");
        assert_eq!(a.workbook.worksheets.len(), 8);

        // Undo of an add removes the sheet again.
        let setup = a.flush();
        a.add_sheet("Doomed").unwrap();
        assert_eq!(names(&a).last().map(String::as_str), Some("Doomed"));
        let added = a.flush();
        let undo: Vec<Patch> = added
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        assert!(!names(&a).contains(&"Doomed".to_string()));
        assert_eq!(a.workbook.worksheets.len(), 8);

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);
        deliver(&mut b, 1, &added);
        deliver(&mut b, 1, &a.flush());
        a.evaluate();
        b.evaluate();
        assert_eq!(names(&b), names(&a));
        let gamma = names(&b).iter().position(|n| n == "Gamma").unwrap() as u32;
        assert_eq!(
            b.get_formatted_cell_value(gamma, 1, 1),
            Ok("gamma".to_string())
        );
        assert_eq!(b.workbook, a.workbook);

        // Concurrent inserts at the same index: both survive, in the same order on both replicas.
        a.insert_sheet("FromA", 1, None).unwrap();
        b.insert_sheet("FromB", 1, None).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        assert!(names(&a).contains(&"FromA".to_string()));
        assert!(names(&a).contains(&"FromB".to_string()));
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);
    }

    /// The sheet each display name ended up on. Equality of this across replicas is the real
    /// convergence claim: agreeing on the *set* of names would not stop two replicas swapping them.
    fn naming<'a>(model: &'a CollabModel<'_>) -> Vec<(&'a str, SheetId)> {
        model
            .workbook
            .worksheets
            .iter()
            .map(|ws| (ws.name.as_str(), ws.sheet_id))
            .collect()
    }

    /// The id of the sheet a single-`AddSheet` commit created.
    fn added_id(commits: &[LocalCommit]) -> SheetId {
        match commits.first().and_then(|c| c.patches.first()) {
            Some(Patch::AddSheet { id, .. }) => *id,
            other => panic!("expected an AddSheet, got {other:?}"),
        }
    }

    /// The display name `model` gave the sheet with this id.
    fn sheet_name<'a>(model: &'a CollabModel<'_>, id: SheetId) -> &'a str {
        model
            .workbook
            .worksheets
            .iter()
            .find(|ws| ws.sheet_id == id)
            .map(|ws| ws.name.as_str())
            .unwrap()
    }

    fn sorted_names(model: &CollabModel<'_>) -> Vec<String> {
        let mut out = names(model);
        out.sort();
        out
    }

    /// Concurrent adds and renames can author the same name on different replicas. Excel needs them
    /// case-insensitively unique, so the losers are renumbered — the same way everywhere.
    #[test]
    fn sheet_name_repair() {
        // One Sheet1, shared by three replicas.
        let mut p1 = CollabModel::new(1);
        p1.new_sheet();
        let setup = p1.flush();
        let mut p2 = CollabModel::new(2);
        let mut p3 = CollabModel::new(3);
        deliver(&mut p2, 1, &setup);
        deliver(&mut p3, 1, &setup);

        // All three add "Data" without having seen each other. Local validation cannot see it.
        p1.add_sheet("Data").unwrap();
        p2.add_sheet("Data").unwrap();
        p3.add_sheet("Data").unwrap();
        let (c1, c2, c3) = (p1.flush(), p2.flush(), p3.flush());
        let (id1, id2, id3) = (added_id(&c1), added_id(&c2), added_id(&c3));

        deliver(&mut p1, 2, &c2);
        deliver(&mut p1, 3, &c3);
        deliver(&mut p2, 1, &c1);
        deliver(&mut p2, 3, &c3);
        deliver(&mut p3, 1, &c1);
        deliver(&mut p3, 2, &c2);

        let expected = ["Data", "Data (1)", "Data (2)", "Sheet1"];
        for peer in [&p1, &p2, &p3] {
            assert_eq!(peer.workbook.worksheets.len(), 4);
            assert_eq!(sorted_names(peer), expected);
            // First write wins: p1 authored first, so p1's sheet keeps the name unsuffixed.
            assert_eq!(sheet_name(peer, id1), "Data");
        }
        // Same names *on the same sheets*, not merely the same set of names.
        assert_eq!(naming(&p1), naming(&p2));
        assert_eq!(naming(&p2), naming(&p3));

        // Commutativity: delivery order cannot change what a replica ends up showing.
        let authored = [(1u32, &c1), (2, &c2), (3, &c3)];
        let orders = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];
        let replays: Vec<CollabModel<'static>> = orders
            .iter()
            .map(|order| {
                let mut consumer = CollabModel::new(9);
                deliver(&mut consumer, 1, &setup);
                for &i in order {
                    let (session, commits) = authored[i];
                    deliver(&mut consumer, session, commits);
                }
                consumer
            })
            .collect();
        for (order, replay) in orders.iter().zip(&replays) {
            assert_eq!(naming(replay), naming(&replays[0]), "order {order:?}");
            assert_eq!(replay.workbook, replays[0].workbook, "order {order:?}");
        }
        assert_eq!(naming(&replays[0]), naming(&p1));

        // A replica that never saw the collision authors "Data (1)" itself. The trailing " (1)" is
        // a number to continue, not a base to suffix again, so it lands on "Data (3)".
        let mut p4 = CollabModel::new(4);
        deliver(&mut p4, 1, &setup);
        p4.add_sheet("Data (1)").unwrap();
        let c4 = p4.flush();
        let id4 = added_id(&c4);
        deliver(&mut p4, 1, &c1);
        deliver(&mut p4, 2, &c2);
        deliver(&mut p4, 3, &c3);
        deliver(&mut p1, 4, &c4);
        for peer in [&p1, &p4] {
            assert_eq!(
                sorted_names(peer),
                ["Data", "Data (1)", "Data (2)", "Data (3)", "Sheet1"]
            );
            assert_eq!(sheet_name(peer, id4), "Data (3)");
            assert_eq!(sheet_name(peer, id1), "Data");
        }
        assert_eq!(naming(&p1), naming(&p4));
        // The other two sheets kept whatever the first pass gave them.
        assert_eq!(sheet_name(&p1, id2), sheet_name(&p2, id2));
        assert_eq!(sheet_name(&p1, id3), sheet_name(&p2, id3));

        // Two replicas rename *different* sheets to the same name.
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Other").unwrap();
        let base = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &base);
        let (sheet1, other) = (
            a.workbook.worksheets[0].sheet_id,
            a.workbook.worksheets[1].sheet_id,
        );
        a.rename_sheet_by_index(0, "X").unwrap();
        b.rename_sheet_by_index(1, "X").unwrap();
        let (ra, rb) = (a.flush(), b.flush());
        deliver(&mut a, 2, &rb);
        deliver(&mut b, 1, &ra);
        for peer in [&a, &b] {
            assert_eq!(sorted_names(peer), ["X", "X (1)"]);
            assert_eq!(sheet_name(peer, sheet1), "X");
            assert_eq!(sheet_name(peer, other), "X (1)");
        }
        assert_eq!(naming(&a), naming(&b));

        // The auto-generated name races too: both replicas mint "Sheet2" off the same tab list.
        let mut c = CollabModel::new(1);
        c.new_sheet();
        let one = c.flush();
        let mut d = CollabModel::new(2);
        deliver(&mut d, 1, &one);
        assert_eq!(c.new_sheet().0, "Sheet2");
        assert_eq!(d.new_sheet().0, "Sheet2");
        let (two_c, two_d) = (c.flush(), d.flush());
        deliver(&mut c, 2, &two_d);
        deliver(&mut d, 1, &two_c);
        for peer in [&c, &d] {
            assert_eq!(sorted_names(peer), ["Sheet1", "Sheet2", "Sheet2 (1)"]);
            assert_eq!(sheet_name(peer, added_id(&two_c)), "Sheet2");
        }
        assert_eq!(naming(&c), naming(&d));
    }

    /// Every defined name, as `(name, scope, formula)`. Sorted: storage order is by identity hash,
    /// which says nothing about the document.
    fn defined(model: &CollabModel<'_>) -> Vec<(String, Option<SheetId>, String)> {
        let mut names: Vec<_> = model
            .workbook
            .defined_names
            .iter()
            .map(|dn| (dn.name.clone(), dn.sheet_id, dn.formula.clone()))
            .collect();
        names.sort();
        names
    }

    /// A copy carries the whole sheet: contents, formatting, sheet properties and the defined names
    /// that named it — with every reference to the original retargeted at the copy.
    #[test]
    fn duplicate_sheet_scenario() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Other").unwrap();
        a.set_user_input(1, 1, 1, "5".to_string()).unwrap();

        a.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(0, 2, 1, "=Sheet1!A1*2".to_string())
            .unwrap();
        a.set_user_input(0, 3, 1, "=A1+1".to_string()).unwrap();
        a.set_user_input(0, 4, 1, "=Other!A1".to_string()).unwrap();
        let mut style = Style::default();
        style.font.b = true;
        a.set_cell_style(0, 1, 1, &style).unwrap();
        a.set_row_height(0, 2, 42.0).unwrap();
        let merged = RangeRef::parse_a1("B1:C2").unwrap();
        a.set_merged_range(0, &merged, true).unwrap();
        a.new_defined_name("loc", Some(0), "Sheet1!$A$1").unwrap();
        a.new_defined_name("glob", None, "Sheet1!$A$1").unwrap();
        a.new_defined_name("elsewhere", None, "Other!$A$1").unwrap();
        a.evaluate();

        let (name, at) = a.duplicate_sheet(0).unwrap();
        assert_eq!((name.as_str(), at), ("Sheet1 (1)", 1));
        // The copy sits right after the original, which keeps its own name.
        assert_eq!(names(&a), ["Sheet1", "Sheet1 (1)", "Other"]);
        a.evaluate();

        // An explicit self-reference now names the copy; an implicit one follows its host, and a
        // reference to a third sheet is left where it pointed.
        assert_eq!(
            a.get_cell_formula(1, 2, 1),
            Ok(Some("='Sheet1 (1)'!A1*2".to_string()))
        );
        assert_eq!(a.get_cell_formula(1, 3, 1), Ok(Some("=A1+1".to_string())));
        assert_eq!(
            a.get_cell_formula(1, 4, 1),
            Ok(Some("=Other!A1".to_string()))
        );
        // The source is untouched.
        assert_eq!(
            a.get_cell_formula(0, 2, 1),
            Ok(Some("=Sheet1!A1*2".to_string()))
        );
        for sheet in [0, 1] {
            assert_eq!(
                a.get_formatted_cell_value(sheet, 1, 1),
                Ok("10".to_string())
            );
            assert_eq!(
                a.get_formatted_cell_value(sheet, 2, 1),
                Ok("20".to_string())
            );
            assert_eq!(
                a.get_formatted_cell_value(sheet, 3, 1),
                Ok("11".to_string())
            );
            assert_eq!(a.get_formatted_cell_value(sheet, 4, 1), Ok("5".to_string()));
        }
        // Formatting, row properties and merges came along.
        assert_eq!(a.get_cell_style_or_none(1, 1, 1), Ok(Some(style)));
        assert_eq!(a.get_row_height(1, 2), Ok(42.0));
        assert_eq!(
            a.workbook.worksheets[1].merged_cells,
            a.workbook.worksheets[0].merged_cells
        );

        // Upstream's naming rules: the local name is copied as a local of the copy, the global one
        // that named the source gets a local copy beside it, the unrelated global is left alone.
        let copy_id = a.workbook.worksheets[1].sheet_id;
        let source_id = a.workbook.worksheets[0].sheet_id;
        assert_eq!(
            defined(&a),
            [
                ("elsewhere".to_string(), None, "Other!$A$1".to_string()),
                ("glob".to_string(), None, "Sheet1!$A$1".to_string()),
                (
                    "glob".to_string(),
                    Some(copy_id),
                    "'Sheet1 (1)'!$A$1".to_string()
                ),
                (
                    "loc".to_string(),
                    Some(copy_id),
                    "'Sheet1 (1)'!$A$1".to_string()
                ),
                (
                    "loc".to_string(),
                    Some(source_id),
                    "Sheet1!$A$1".to_string()
                ),
            ]
        );

        // The two sheets are independent documents from here on.
        a.set_user_input(0, 1, 1, "100".to_string()).unwrap();
        a.set_user_input(1, 1, 1, "7".to_string()).unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("200".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("14".to_string()));

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());
        b.evaluate();
        assert_eq!(names(&b), names(&a));
        assert_eq!(defined(&b), defined(&a));
        for (sheet, row) in [(0, 1), (0, 2), (1, 1), (1, 2), (1, 3), (1, 4)] {
            assert_eq!(
                b.get_formatted_cell_value(sheet, row, 1),
                a.get_formatted_cell_value(sheet, row, 1),
                "value at ({sheet}, {row})"
            );
        }
        assert_eq!(b.workbook, a.workbook);

        // Out of range is an error, with the ordinal writer's message.
        assert_eq!(
            a.duplicate_sheet(99),
            Err("Invalid sheet index".to_string())
        );
    }

    #[test]
    fn delete_sheet_undo() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Tail").unwrap();
        a.insert_sheet("Data", 1, None).unwrap();
        a.set_user_input(1, 1, 1, "10".to_string()).unwrap();
        a.set_user_input(1, 2, 1, "=A1*2".to_string()).unwrap();
        let mut style = Style::default();
        style.font.b = true;
        a.set_cell_style(1, 1, 1, &style).unwrap();
        a.set_row_height(1, 1, 42.0).unwrap();
        a.set_sheet_color(1, &Color::Rgb("#00ff00".to_string()))
            .unwrap();
        // Scoped to another sheet: the delete must leave it alone.
        a.new_defined_name("kept", Some(0), "Sheet1!$A$1").unwrap();
        a.evaluate();
        let data_id = a.workbook.worksheets[1].sheet_id;
        let data_key = a.workbook.meta.sheet_positions[&data_id].value.clone();
        let before = projection(&a);
        let setup = a.flush();

        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        // B edits the sheet A is deleting. Reaching A after the delete, the edit is dropped there —
        // and the restore holds what A captured, so it is lost on B too rather than resurrected.
        b.set_user_input(1, 1, 1, "999".to_string()).unwrap();
        a.delete_sheet(1).unwrap();
        let deleted = a.flush();
        deliver(&mut a, 2, &b.flush());
        deliver(&mut b, 1, &deleted);
        assert_eq!(names(&a), ["Sheet1", "Tail"]);
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // Undo: everything the sheet held comes back.
        let undo: Vec<Patch> = deleted
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();
        deliver(&mut b, 1, &undone);
        a.evaluate();
        b.evaluate();
        assert_eq!(projection(&a), before);
        assert_eq!(a.workbook.worksheets[1].sheet_id, data_id);
        assert_eq!(a.get_formatted_cell_value(1, 1, 1), Ok("10".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 2, 1), Ok("20".to_string()));
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // Redo takes it away again.
        let redo: Vec<Patch> = undone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(redo);
        let redone = a.flush();
        deliver(&mut b, 1, &redone);
        assert_eq!(names(&a), ["Sheet1", "Tail"]);
        assert_eq!(names(&b), names(&a));
        assert_eq!(b.workbook, a.workbook);

        // While it is dead B takes its name, and A fills the gap it left in the tab order.
        b.add_sheet("Data").unwrap();
        let grabbed = b.flush();
        a.insert_sheet("Filler", 1, None).unwrap();
        let filler_id = a.workbook.worksheets[1].sheet_id;
        // The gap re-mints the very key the dead sheet still holds: only the id separates them.
        assert_eq!(a.workbook.meta.sheet_positions[&filler_id].value, data_key);
        let filled = a.flush();
        deliver(&mut a, 2, &grabbed);
        deliver(&mut b, 1, &filled);

        let undo: Vec<Patch> = redone
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let revived = a.flush();
        deliver(&mut b, 1, &revived);

        // A third replica meets the two sheets sharing a key in the opposite order, which only the
        // id tiebreak keeps from showing them in the opposite order too.
        let mut c = CollabModel::new(3);
        for (session, commits) in [
            (1, &setup),
            (1, &deleted),
            (1, &undone),
            (1, &redone),
            (2, &grabbed),
            (1, &revived),
            (1, &filled),
        ] {
            deliver(&mut c, session, commits);
        }
        a.evaluate();
        b.evaluate();
        c.evaluate();
        for peer in [&a, &b, &c] {
            // The revival authors the name anew, so B's earlier "Data" keeps it and the sheet
            // coming back gives way.
            assert_eq!(
                sorted_names(peer),
                ["Data", "Data (1)", "Filler", "Sheet1", "Tail"]
            );
            assert_eq!(sheet_name(peer, data_id), "Data (1)");
            let data = peer.get_sheet_index_by_sheet_id(data_id).unwrap();
            assert_eq!(
                peer.get_formatted_cell_value(data, 2, 1),
                Ok("20".to_string())
            );
        }
        assert_eq!(naming(&a), naming(&b));
        assert_eq!(naming(&a), naming(&c));
        assert_eq!(b.workbook, a.workbook);
    }

    #[test]
    fn late_edit_to_revived_sheet() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.add_sheet("Data").unwrap();
        a.set_user_input(1, 1, 1, "10".to_string()).unwrap(); // A: Data!A1=10
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        b.set_user_input(1, 5, 1, "999".to_string()).unwrap(); // B: Data!A5=999
        let early = b.flush();
        a.delete_sheet(1).unwrap(); // A: delete 'Data'
        let deleted = a.flush();
        b.set_user_input(1, 6, 1, "888".to_string()).unwrap(); // B (concurrently): Data!A6=888
        let late = b.flush();
        let undo: Vec<Patch> = deleted
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        a.commit_local(undo);
        let undone = a.flush();

        // A meets both edits after the revive; B meets the delete after making them.
        deliver(&mut a, 2, &early);
        deliver(&mut a, 2, &late);
        deliver(&mut b, 1, &deleted);
        deliver(&mut b, 1, &undone);
        a.evaluate();
        b.evaluate();
        // Dropped on both, not merely agreed on: the revive stamp outranks either edit's.
        assert_eq!(a.get_formatted_cell_value(1, 5, 1), Ok("".to_string()));
        assert_eq!(b.get_formatted_cell_value(1, 5, 1), Ok("".to_string()));
        assert_eq!(a.get_formatted_cell_value(1, 1, 1), Ok("10".to_string()));
        // A resurrected row would re-enter the index right after the snapshot's, so the ordinal the
        // edit was authored at says nothing: what proves it gone is the sheet still being one row.
        let rows = |m: &CollabModel<'_>| m.workbook.worksheets[1].sheet_data.len();
        assert_eq!((rows(&a), rows(&b)), (1, 1));
        assert_eq!(b.workbook, a.workbook);

        // Only writes from before the revive die: new work on the revived sheet applies as usual.
        b.set_user_input(1, 5, 1, "42".to_string()).unwrap();
        deliver(&mut a, 2, &b.flush());
        a.evaluate();
        b.evaluate();
        assert_eq!(a.get_formatted_cell_value(1, 5, 1), Ok("42".to_string()));
        assert_eq!(b.get_formatted_cell_value(1, 5, 1), Ok("42".to_string()));
        assert_eq!(b.workbook, a.workbook);
    }

    /// Divergence from ordinal, by design: a reference has to bind to something, so naming a sheet
    /// that is not there is refused at write time rather than stored and evaluated to `#REF!`.
    #[test]
    fn formula_naming_an_absent_sheet_is_refused() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "7".to_string()).unwrap();
        assert!(a.set_user_input(0, 2, 1, "=Sheet2!C3".to_string()).is_err());
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("".to_string()));

        // Once the sheet is there the same input binds and evaluates.
        a.new_sheet();
        a.set_user_input(0, 2, 1, "=Sheet2!C3".to_string()).unwrap();
        a.evaluate();
        assert_eq!(a.get_formatted_cell_value(0, 2, 1), Ok("0".to_string()));
    }

    /// Two cells at different positions can author the identical binding — same keys, same
    /// relative flags — so they intern one entry.
    #[test]
    fn shared_binding_serves_every_host() {
        let mut model = CollabModel::new(1);
        model.new_sheet();
        model.set_user_input(0, 1, 1, "5".to_string()).unwrap();
        model.set_user_input(0, 1, 4, "=A1".to_string()).unwrap();
        model.set_user_input(0, 1, 5, "=A1".to_string()).unwrap();
        model.evaluate();
        // One entry, because both cells bound to the same key with the same flags.
        assert_eq!(model.workbook.worksheets[0].shared_formulas.len(), 1);
        assert_eq!(model.get_formatted_cell_value(0, 1, 4), Ok("5".to_string()));
        assert_eq!(model.get_formatted_cell_value(0, 1, 5), Ok("5".to_string()));

        // A row above everything: the target and both hosts slide down together.
        model.insert_rows(0, 1, 1).unwrap();
        model.evaluate();
        assert_eq!(model.get_formatted_cell_value(0, 2, 4), Ok("5".to_string()));
        assert_eq!(model.get_formatted_cell_value(0, 2, 5), Ok("5".to_string()));
        assert_eq!(model.get_cell_formula(0, 2, 4), Ok(Some("=A2".to_string())));
    }

    /// The `$` flags are display metadata the binding carries: a relative reference is shown
    /// relative, an absolute one with its dollars, whatever the evaluator is handed.
    #[test]
    fn display_keeps_the_dollars_it_was_given() {
        let mut model = CollabModel::new(1);
        model.new_sheet();
        model.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        model.set_user_input(0, 3, 2, "=A1".to_string()).unwrap();
        model.set_user_input(0, 4, 2, "=$A$1".to_string()).unwrap();
        model
            .set_user_input(0, 5, 2, "=SUM($A1:A$1)".to_string())
            .unwrap();
        assert_eq!(model.get_cell_formula(0, 3, 2), Ok(Some("=A1".to_string())));
        assert_eq!(
            model.get_cell_formula(0, 4, 2),
            Ok(Some("=$A$1".to_string()))
        );
        assert_eq!(
            model.get_cell_formula(0, 5, 2),
            Ok(Some("=SUM($A1:A$1)".to_string()))
        );
    }

    /// A rename writes one register and nothing else. A peer that bound `=Sheet1!A1` before the
    /// rename landed still points at the same sheet, and shows it under its new name.
    #[test]
    fn rename_needs_no_formula_rewrite() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.new_sheet();
        a.set_user_input(0, 1, 1, "7".to_string()).unwrap();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &a.flush());

        // Concurrent: `a` renames the sheet, `b` writes a formula naming it by its old name.
        a.rename_sheet_by_index(0, "Data").unwrap();
        b.set_user_input(1, 1, 1, "=Sheet1!A1".to_string()).unwrap();
        let (from_a, from_b) = (a.flush(), b.flush());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);
        a.evaluate();
        b.evaluate();

        assert_eq!(a.workbook, b.workbook);
        for model in [&a, &b] {
            assert_eq!(model.workbook.worksheets[0].name, "Data");
            assert_eq!(
                model.get_cell_formula(1, 1, 1),
                Ok(Some("=Data!A1".to_string()))
            );
            assert_eq!(model.get_formatted_cell_value(1, 1, 1), Ok("7".to_string()));
        }
    }

    /// A defined name's body is bound too, so it follows a sheet rename with no rewrite — the
    /// text `get_defined_name_list` shows is a projection of the binding, not stored state.
    #[test]
    fn defined_name_body_follows_a_rename() {
        let mut model = CollabModel::new(1);
        model.new_sheet();
        model.set_user_input(0, 1, 1, "42".to_string()).unwrap();
        model
            .new_defined_name("MyName", None, "Sheet1!$A$1")
            .unwrap();
        model
            .set_user_input(0, 5, 5, "=MyName".to_string())
            .unwrap();
        model.evaluate();
        assert_eq!(
            model.get_formatted_cell_value(0, 5, 5),
            Ok("42".to_string())
        );

        model.rename_sheet_by_index(0, "Data").unwrap();
        model.evaluate();
        assert_eq!(
            model.get_defined_name_list(),
            vec![("MyName".to_string(), None, "Data!$A$1".to_string())]
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 5, 5),
            Ok("42".to_string())
        );
    }

    /// Divergence from upstream, by design: a reference that cannot be bound is refused at write
    /// time instead of being stored and evaluated to `#REF!`.
    #[test]
    fn unbindable_input_is_refused() {
        let mut model = CollabModel::new(1);
        model.new_sheet();
        assert!(model
            .set_user_input(0, 1, 1, "=NoSuchSheet!A1".to_string())
            .is_err());
        assert!(model
            .update_cell_with_formula(0, 1, 1, "=NoSuchSheet!A1".to_string())
            .is_err());
        assert!(model
            .new_defined_name("Nope", None, "NoSuchSheet!$A$1")
            .is_err());
        // Nothing was written, and nothing was committed.
        assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
        assert!(model.workbook.defined_names.is_empty());
    }

    #[test]
    fn redundant_style_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        a.set_cell_style(0, 1, 1, &bold()).unwrap();
        assert_eq!(a.flush().len(), 1);

        a.set_cell_style(0, 1, 1, &bold()).unwrap();
        assert!(a.flush().is_empty()); // redundant

        a.set_cell_style(0, 2, 2, &bold()).unwrap();
        a.flush();
        a.copy_cell_style((0, 1, 1), (0, 2, 2)).unwrap();
        assert!(a.flush().is_empty());
        // A different style still emits.
        a.set_cell_style(0, 1, 1, &italic()).unwrap();
        assert_eq!(a.flush().len(), 1);
        assert_eq!(a.get_cell_style_or_none(0, 1, 1), Ok(Some(italic())));
    }

    #[test]
    fn redundant_merge_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        let range = RangeRef::parse_a1("A1:B2").unwrap();
        a.set_merged_range(0, &range, true).unwrap();
        assert_eq!(a.flush().len(), 1);

        a.set_merged_range(0, &range, true).unwrap();
        assert!(a.flush().is_empty()); // redundant

        // range outside materialized window of cells, no patch is produced
        let other = RangeRef::parse_a1("D4:E5").unwrap();
        let (rows, cols) = (
            a.workbook.worksheets[0].index.rows.len(),
            a.workbook.worksheets[0].index.cols.len(),
        );
        a.set_merged_range(0, &other, false).unwrap();
        assert!(a.flush().is_empty());
        // Unmaterialized corners stayed unmaterialized: the skipped patch minted no keys.
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), rows);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), cols);

        // Unmerging what is merged still emits, once.
        a.set_merged_range(0, &range, false).unwrap();
        assert_eq!(a.flush().len(), 1);
        assert!(a.workbook.worksheets[0].merged_cells.is_empty());
        a.set_merged_range(0, &range, false).unwrap();
        assert!(a.flush().is_empty());
    }

    #[test]
    fn redundant_comment_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        a.set_comment(0, 1, 1, Some(("note".into(), "me".into())))
            .unwrap();
        assert_eq!(a.flush().len(), 1);

        a.set_comment(0, 1, 1, Some(("note".into(), "me".into())))
            .unwrap();
        assert!(a.flush().is_empty()); // redundant

        // No comment there to remove — on a materialized cell, and on one that does not exist.
        let (rows, cols) = (
            a.workbook.worksheets[0].index.rows.len(),
            a.workbook.worksheets[0].index.cols.len(),
        );
        a.set_comment(0, 1, 2, None).unwrap();
        a.set_comment(0, 40, 40, None).unwrap();
        assert!(a.flush().is_empty());
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), rows);
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), cols);

        // Different text, and then the removal, both change something.
        a.set_comment(0, 1, 1, Some(("other".into(), "me".into())))
            .unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_comment(0, 1, 1, None).unwrap();
        assert_eq!(a.flush().len(), 1);
        assert!(a.workbook.worksheets[0].comments.is_empty());
    }

    #[test]
    fn redundant_rename_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        a.rename_sheet_by_index(0, "Data").unwrap();
        assert_eq!(a.flush().len(), 1);

        a.rename_sheet_by_index(0, "Data").unwrap();
        assert!(a.flush().is_empty()); // redundant
        assert_eq!(a.workbook.worksheets[0].get_name(), "Data");

        // Validation is unchanged, and still runs first.
        a.new_sheet();
        assert!(a.rename_sheet_by_index(1, "Data").is_err());
        assert!(a.rename_sheet_by_index(0, "[").is_err());
        // Renaming a sheet to its own name is a no-op, not a collision error.
        a.flush();
        let own_name = a.workbook.worksheets[1].get_name();
        a.rename_sheet_by_index(1, &own_name).unwrap();
        assert!(a.flush().is_empty());
    }

    #[test]
    fn deleting_unmaterialized_rows_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.flush();

        a.delete_rows(0, 50, 3).unwrap();
        a.delete_columns(0, 50, 3).unwrap();
        assert!(a.flush().is_empty());

        // A range that does hold materialized ordinals still commits.
        a.delete_rows(0, 1, 3).unwrap();
        assert_eq!(a.flush().len(), 1);

        // Validation is unchanged.
        assert!(a.delete_rows(0, 1, 0).is_err());
        assert!(a.delete_columns(0, 1, 0).is_err());
    }

    #[test]
    fn redundant_row_property_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.flush();

        a.set_row_height(0, 1, 40.0).unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_row_height(0, 1, 40.0).unwrap();
        assert!(a.flush().is_empty()); // redundant

        a.set_row_hidden(0, 1, true).unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_row_hidden(0, 1, true).unwrap();
        assert!(a.flush().is_empty()); // redundant

        // An unmaterialized row already holds the defaults, so writing them mints no key.
        let rows = a.workbook.worksheets[0].index.rows.len();
        a.set_row_height(0, 40, DEFAULT_ROW_HEIGHT).unwrap();
        a.set_row_hidden(0, 40, false).unwrap();
        assert!(a.flush().is_empty());
        assert_eq!(a.workbook.worksheets[0].index.rows.len(), rows);

        // Validation is unchanged, and still runs first.
        assert!(a.set_row_height(0, 1, -1.0).is_err());
        assert!(a.set_row_hidden(0, 0, true).is_err());

        // Different values still emit.
        a.set_row_height(0, 1, 60.0).unwrap();
        a.set_row_hidden(0, 1, false).unwrap();
        assert_eq!(a.flush().len(), 2);
        assert_eq!(a.get_row_height(0, 1), Ok(60.0));
        assert_eq!(a.is_row_hidden(0, 1), Ok(false));
    }

    #[test]
    fn redundant_column_property_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_input(0, 1, 1, "1".to_string()).unwrap();
        a.flush();

        a.set_column_width(0, 1, 120.0).unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_column_width(0, 1, 120.0).unwrap();
        assert!(a.flush().is_empty()); // redundant

        a.set_column_hidden(0, 1, true).unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_column_hidden(0, 1, true).unwrap();
        assert!(a.flush().is_empty()); // redundant

        // An unmaterialized column already holds the defaults, so writing them mints no key.
        let cols = a.workbook.worksheets[0].index.cols.len();
        a.set_column_width(0, 30, DEFAULT_COLUMN_WIDTH).unwrap();
        a.set_column_hidden(0, 30, false).unwrap();
        assert!(a.flush().is_empty());
        assert_eq!(a.workbook.worksheets[0].index.cols.len(), cols);

        // Validation is unchanged, and still runs first.
        assert!(a.set_column_width(0, 1, -1.0).is_err());
        assert!(a.set_column_hidden(0, 0, true).is_err());

        // Different values still emit.
        a.set_column_width(0, 1, 200.0).unwrap();
        a.set_column_hidden(0, 1, false).unwrap();
        assert_eq!(a.flush().len(), 2);
        assert_eq!(a.get_column_width(0, 1), Ok(200.0));
        assert_eq!(a.is_column_hidden(0, 1), Ok(false));
    }

    #[test]
    fn redundant_sheet_property_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        let red = Color::Rgb("#FF0000".to_string());
        a.set_sheet_color(0, &red).unwrap();
        assert_eq!(a.flush().len(), 1);
        a.set_sheet_color(0, &red).unwrap();
        assert!(a.flush().is_empty()); // redundant

        a.set_show_grid_lines(0, false).unwrap();
        a.set_frozen_rows(0, 2).unwrap();
        assert_eq!(a.flush().len(), 2);
        a.set_show_grid_lines(0, false).unwrap();
        a.set_frozen_rows(0, 2).unwrap();
        assert!(a.flush().is_empty()); // redundant

        // The values the sheet already has, never written by anyone.
        a.set_frozen_columns(0, 0).unwrap();
        a.set_sheet_state(0, SheetState::Visible).unwrap();
        assert!(a.flush().is_empty());

        // Validation is unchanged, and still runs first.
        assert!(a.set_frozen_rows(0, -1).is_err());
        assert!(a.set_frozen_columns(0, -1).is_err());

        // A different value still emits.
        a.set_frozen_rows(0, 3).unwrap();
        assert_eq!(a.flush().len(), 1);
        assert_eq!(a.workbook.worksheets[0].frozen_rows, 3);
    }

    #[test]
    fn redundant_workbook_property_emits_nothing() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.flush();
        a.set_locale("en-GB").unwrap();
        a.set_timezone("Europe/Berlin").unwrap();
        assert_eq!(a.flush().len(), 2);
        a.set_locale("en-GB").unwrap();
        a.set_timezone("Europe/Berlin").unwrap();
        assert!(a.flush().is_empty()); // redundant

        // The theme the workbook already has.
        let theme = a.workbook.theme.clone();
        a.set_theme(theme);
        assert!(a.flush().is_empty());

        // Validation is unchanged, and still runs first.
        assert!(a.set_locale("nope").is_err());
        assert!(a.set_timezone("Nowhere/Nothing").is_err());

        // A different value still emits.
        a.set_locale("es").unwrap();
        assert_eq!(a.flush().len(), 1);
        assert_eq!(a.workbook.settings.locale, "es");
    }

    #[test]
    fn redundant_height_loses_to_a_concurrent_edit() {
        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_row_height(0, 1, 40.0).unwrap();
        let setup = a.flush();
        let mut b = CollabModel::new(2);
        deliver(&mut b, 1, &setup);

        b.set_row_height(0, 1, 80.0).unwrap();
        a.set_row_height(0, 1, 40.0).unwrap(); // redundant, and later than B's write
        let (from_a, from_b) = (a.flush(), b.flush());
        assert!(from_a.is_empty());
        deliver(&mut b, 1, &from_a);
        deliver(&mut a, 2, &from_b);

        for m in [&a, &b] {
            assert_eq!(m.get_row_height(0, 1), Ok(80.0));
        }
        assert_eq!(b.workbook, a.workbook);
    }
}
