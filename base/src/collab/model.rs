use crate::collab::fractional_index::{FractionalIndex, FractionalKey, SESSION_SUFFIX_LEN};
use crate::collab::hlc::Hlc;
use crate::collab::log::{Lww, SessionId, Timestamp};
use crate::collab::patch::{
    CfPropKind, ColPropKind, DefinedNameId, NamedStyle, NamedStyleId, Patch, RowPropKind, SheetId,
    SheetPropKind, WorkbookPropKind,
};
use crate::constants::{
    COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, ROW_HEIGHT_FACTOR,
};
use crate::expressions::parser::Parser;
use crate::expressions::utils::{is_valid_column_number, is_valid_row};
use crate::language::get_default_language;
use crate::locale::get_default_locale;
use crate::model::Model;
use crate::new_empty::{APPLICATION, APP_VERSION, IRONCALC_USER};
use crate::types::{
    sealed::Sealed, CellAddr, Col, Metadata, Position, RangeRef, Row, Style, Workbook,
    WorkbookSettings, Worksheet,
};
use crate::tz::Tz;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What a replica assumes until a peer writes the workbook's locale and timezone registers.
const DEFAULT_LOCALE: &str = "en";
const DEFAULT_TIMEZONE: &str = "UTC";

/// Stable addressing: rows and columns are named by the [`FractionalKey`] they were minted with, and
/// where they currently sit lives in the sheet's [`SheetIndexes`] rather than in the name.
///
/// The derives mirror [`Ordinal`](crate::types::Ordinal)'s: they are what the `#[derive]`s on the
/// generic containers, which emit `A: Trait` bounds, ask of the marker.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize,
)]
pub struct Stable;

impl Sealed for Stable {}

impl Position for Stable {
    type Key = FractionalKey;
    type SheetIndex = SheetIndexes;
    type MergedCell = StableRange;
    type WorkbookMeta = WorkbookMeta;
    type Local = CollabSession;
    // Flips to `StableFormula` once formula bind/lower land.
    type Formula = String;

    fn row_ordinal(idx: &SheetIndexes, key: &FractionalKey) -> Option<i32> {
        idx.rows.position_of(key).map(|p| p as i32 + 1)
    }

    fn col_ordinal(idx: &SheetIndexes, key: &FractionalKey) -> Option<i32> {
        idx.cols.position_of(key).map(|p| p as i32 + 1)
    }

    fn row_at(idx: &SheetIndexes, ordinal: i32) -> Option<FractionalKey> {
        if ordinal < 1 {
            return None;
        }
        idx.rows.key(ordinal as usize - 1).cloned()
    }

    fn col_at(idx: &SheetIndexes, ordinal: i32) -> Option<FractionalKey> {
        if ordinal < 1 {
            return None;
        }
        idx.cols.key(ordinal as usize - 1).cloned()
    }

    fn row_count(idx: &SheetIndexes) -> i32 {
        idx.rows.len() as i32
    }

    fn col_count(idx: &SheetIndexes) -> i32 {
        idx.cols.len() as i32
    }

    fn resolve_merged(merged: &StableRange, idx: &SheetIndexes) -> Option<(i32, i32, i32, i32)> {
        Self::resolve_range(merged, idx)
    }

    fn resolve_range(range: &StableRange, idx: &SheetIndexes) -> Option<(i32, i32, i32, i32)> {
        range.resolve(idx)
    }
}

/// The two orderings a sheet's keys resolve against, plus the sheet's write registers.
#[derive(Clone, Debug, Default, PartialEq, Encode, Decode)]
pub struct SheetIndexes {
    pub rows: FractionalIndex,
    pub cols: FractionalIndex,
    pub registers: SheetRegisters,
}

/// The last-write guard of every register a sheet owns: only the [`Timestamp`] that last won each,
/// never the value — values stay unwrapped in the worksheet's ordinary fields, as under ordinal
/// addressing.
///
/// An entry outlives its subject: deleting a row keeps its cells' guards, so a concurrent write to
/// one of them loses to the delete instead of resurrecting the row.
#[derive(Clone, Debug, Default, PartialEq, Encode, Decode)]
pub struct SheetRegisters {
    pub cell_values: HashMap<StableCellAddress, Timestamp>,
    pub cell_styles: HashMap<StableCellAddress, Timestamp>,
    pub arrays: HashMap<StableCellAddress, Timestamp>,
    pub rows: HashMap<(FractionalKey, RowPropKind), Timestamp>,
    pub col_spans: HashMap<((FractionalKey, FractionalKey), ColPropKind), Timestamp>,
    pub props: HashMap<SheetPropKind, Timestamp>,
    pub merges: HashMap<StableRange, Timestamp>,
    pub comments: HashMap<StableCellAddress, Timestamp>,
    pub cf: HashMap<(FractionalKey, CfPropKind), Timestamp>,
    /// CF rule identity ↔ storage order; kept sorted, position = priority.
    pub cf_order: Vec<FractionalKey>,
    /// Where a rule sits, for the rules that were ever moved. Position keys are CRDT-only state,
    /// so the value sits with its guard, as in [`WorkbookMeta::sheet_positions`].
    pub cf_positions: HashMap<FractionalKey, Lww<FractionalKey>>,
}

/// Workbook-wide registers: those outliving the sheet they talk about, and those no sheet owns.
#[derive(Clone, Debug, Default, PartialEq, Encode, Decode)]
pub struct WorkbookMeta {
    /// AddSheet/DeleteSheet LWW; entries survive deletion (resurrection guard).
    pub sheet_existence: HashMap<u32, Timestamp>,
    /// Tab-order register; the position key is CRDT-only state, so value sits with its guard.
    pub sheet_positions: HashMap<u32, Lww<FractionalKey>>,
    /// Authored sheet names. `Worksheet::name` is the *display* name, derived from these and
    /// repaired for collisions, so the value sits with its guard.
    pub sheet_names: HashMap<u32, Lww<String>>,
    pub props: HashMap<WorkbookPropKind, Timestamp>,
    /// Defined names; entries survive deletion (resurrection guard).
    pub defined_names: HashMap<DefinedNameId, DefinedNameState>,
    /// Named styles; entries survive deletion (resurrection guard).
    pub named_styles: HashMap<NamedStyleId, NamedStyleState>,
}

#[derive(Clone, Debug, Default, PartialEq, Encode, Decode)]
pub struct DefinedNameState {
    /// The authored scope and name. A scope move is address-shaped like a rename, so one register
    /// carries both and a concurrent redefinition of the formula still survives.
    pub name: Lww<(Option<SheetId>, String)>,
    /// `None` is a deleted name: the entry and its address survive, so an undo can revive it.
    pub formula: Lww<Option<String>>,
}

/// A named style's two registers. Both are CRDT-only state — the style table shows a *display* name
/// and a locally interned `xf_id` — so each value sits with its own guard, as in
/// [`WorkbookMeta::sheet_positions`].
#[derive(Clone, Debug, Default, PartialEq, Encode, Decode)]
pub struct NamedStyleState {
    /// The authored name, which the display name is derived from and repaired for collisions.
    pub name: Lww<String>,
    /// `None` is a deleted style: the entry and its name survive, so an undo can revive it.
    pub definition: Lww<Option<Box<NamedStyle>>>,
}

/// A description of a continuous range of cells, described using stable identifiers, which can be
/// used to keep track of cell position under various concurrent operations (ex. adding/removing
/// rows or columns).
pub type StableRange = RangeRef<Stable>;

/// A cell position, described using stable identifiers, which can be used to keep track of cell
/// position under various concurrent operations (ex. adding/removing rows or columns).
pub type StableCellAddress = CellAddr<Stable>;

/// Where one axis of a [`StableRange`] currently sits, as a 1-based closed interval.
///
/// A corner that still resolves keeps its identity. One that does not — deleted, or never in this
/// index — clamps to where it would now sit: the `lo` corner takes the element that took its place,
/// the `hi` corner the one just before it. A [`FractionalKey::NULL`] corner is not a position at
/// all but an open end. `None` means clamping collapsed the span.
fn resolve_axis(
    index: &FractionalIndex,
    span: &Option<(FractionalKey, FractionalKey)>,
) -> Option<(i32, i32)> {
    // Open on both ends is the whole axis, exactly as `None` is.
    let span = span
        .as_ref()
        .filter(|(lo, hi)| !lo.is_empty() || !hi.is_empty());
    let Some((lo, hi)) = span else {
        return Some((1, index.len() as i32));
    };
    // An open end takes the axis' extreme, and has to do so before the clamping path below: NULL
    // sorts under every real key, so `lower_bound` would put an open *upper* bracket at 0.
    let lo_ord = if lo.is_empty() {
        Some(1)
    } else {
        index.position_of(lo).map(|p| p as i32 + 1)
    };
    let hi_ord = if hi.is_empty() {
        Some(index.len() as i32)
    } else {
        index.position_of(hi).map(|p| p as i32 + 1)
    };
    match (lo_ord, hi_ord) {
        // Concurrent moves can invert the corners; the rectangle they bound is still the same one.
        (Some(lo), Some(hi)) => Some((lo.min(hi), lo.max(hi))),
        (lo_ord, hi_ord) => {
            let lo = lo_ord.unwrap_or_else(|| index.active_index(lo) as i32 + 1);
            let hi = hi_ord.unwrap_or_else(|| index.active_index(hi) as i32);
            (lo <= hi).then_some((lo, hi))
        }
    }
}

impl StableRange {
    /// The 1-based ordinal rectangle `(row1, column1, row2, column2)` this currently denotes, or
    /// `None` if it collapsed. An unbounded axis spans whatever the index holds right now.
    pub fn resolve(&self, idx: &SheetIndexes) -> Option<(i32, i32, i32, i32)> {
        let (row1, row2) = resolve_axis(&idx.rows, &self.rows)?;
        let (column1, column2) = resolve_axis(&idx.cols, &self.cols)?;
        Some((row1, column1, row2, column2))
    }

    pub fn contains(&self, idx: &SheetIndexes, row: &FractionalKey, col: &FractionalKey) -> bool {
        let Some((row1, column1, row2, column2)) = self.resolve(idx) else {
            return false;
        };
        match (Stable::row_ordinal(idx, row), Stable::col_ordinal(idx, col)) {
            (Some(r), Some(c)) => (row1..=row2).contains(&r) && (column1..=column2).contains(&c),
            _ => false,
        }
    }
}

/// A column-property span under stable addressing is a *region*, not a fixed set of columns: it
/// covers whatever currently sits between its two corner keys, exactly as [`StableRange`]'s column
/// axis does. Consequences:
///
/// - A column moved out of the region loses the span's properties, and one moved in gains them.
///   That is not Excel's behaviour, but Excel has no concurrent semantics to be faithful to.
/// - A single-column span `(k, k)` follows its column wherever it moves.
/// - A [`FractionalKey::NULL`] corner is an open end, so `(NULL, NULL)` is the storable whole-axis
///   span — the register `set_style` over the whole sheet writes to.
/// - Sequential (local) edits shatter wide spans eagerly, exactly as the ordinal code does today:
///   the wide record is removed and narrower ones written in the same commit.
/// - Only concurrency can produce overlapping spans; those resolve per position by register write
///   timestamp — LWW, newest covering span wins. That machinery is phase-5 work.
impl Col<Stable> {
    /// The 1-based ordinal interval this span currently covers, or `None` if it collapsed.
    /// Same corner resolution and clamping as [`StableRange`]: see [`resolve_axis`].
    pub fn resolve(&self, idx: &SheetIndexes) -> Option<(i32, i32)> {
        resolve_axis(&idx.cols, &Some((self.min.clone(), self.max.clone())))
    }
}

impl Worksheet<Stable> {
    /// The column record that owns property `kind` at ordinal `column`: of the spans covering it,
    /// the one whose register was written last. Only concurrency makes them overlap; a record with
    /// no register entry for `kind` never wrote it and does not compete.
    fn covering_col(&self, column: i32, kind: ColPropKind) -> Option<&Col<Stable>> {
        let mut best: Option<(&Timestamp, &Col<Stable>)> = None;
        for col in &self.cols {
            match col.resolve(&self.index) {
                Some((min, max)) if (min..=max).contains(&column) => {}
                _ => continue,
            }
            let span = (col.min.clone(), col.max.clone());
            let Some(ts) = self.index.registers.col_spans.get(&(span, kind)) else {
                continue;
            };
            if best.is_none_or(|(stored, _)| stored < ts) {
                best = Some((ts, col));
            }
        }
        best.map(|(_, col)| col)
    }

    pub fn get_column_width(&self, column: i32) -> Result<f64, String> {
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        Ok(match self.covering_col(column, ColPropKind::Width) {
            Some(col) => col.width * COLUMN_WIDTH_FACTOR,
            None => DEFAULT_COLUMN_WIDTH,
        })
    }

    pub fn is_column_hidden(&self, column: i32) -> Result<bool, String> {
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        Ok(self
            .covering_col(column, ColPropKind::Hidden)
            .is_some_and(|col| col.hidden))
    }

    pub fn get_column_style(&self, column: i32) -> Result<Option<i32>, String> {
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        Ok(self
            .covering_col(column, ColPropKind::Style)
            .and_then(|col| col.style))
    }

    /// Rows are addressed one key at a time, so there is nothing to resolve between.
    fn row_record(&self, row: i32) -> Option<&Row<Stable>> {
        let key = Stable::row_at(&self.index, row)?;
        self.rows.iter().find(|r| r.r == key)
    }

    pub fn row_height(&self, row: i32) -> Result<f64, String> {
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        Ok(match self.row_record(row) {
            Some(record) => record.height * ROW_HEIGHT_FACTOR,
            None => DEFAULT_ROW_HEIGHT,
        })
    }

    pub fn is_row_hidden(&self, row: i32) -> Result<bool, String> {
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        Ok(self.row_record(row).is_some_and(|record| record.hidden))
    }
}

/// A collaborative model: the evaluation engine running directly on stably addressed storage.
pub type CollabModel<'a> = Model<'a, Stable>;

/// Reads that stable addressing has to answer for itself, because a column property is a span
/// register rather than a record per column.
impl CollabModel<'_> {
    pub fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64, String> {
        self.workbook.worksheet(sheet)?.get_column_width(column)
    }

    pub fn is_column_hidden(&self, sheet: u32, column: i32) -> Result<bool, String> {
        self.workbook.worksheet(sheet)?.is_column_hidden(column)
    }

    pub fn get_column_style(&self, sheet: u32, column: i32) -> Result<Option<Style>, String> {
        match self.workbook.worksheet(sheet)?.get_column_style(column)? {
            Some(index) => self.workbook.styles.get_style(index).map(Some),
            None => Ok(None),
        }
    }

    pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64, String> {
        self.workbook.worksheet(sheet)?.row_height(row)
    }

    pub fn is_row_hidden(&self, sheet: u32, row: i32) -> Result<bool, String> {
        self.workbook.worksheet(sheet)?.is_row_hidden(row)
    }
}

/// One mutator call's worth of patches, stamped once and already applied locally.
///
/// Contract with the hosting framework: it transports each of these as a single commit, carrying
/// `hlc` unchanged as [`Commit::hlc`](crate::collab::log::Commit::hlc). Anything else and the
/// author's register timestamps stop matching its peers'.
#[derive(Debug)]
pub struct LocalCommit {
    pub hlc: Hlc,
    pub patches: Vec<Patch>,
}

/// The replica-local half of a [`CollabModel`]: who we are, and what we have not shipped yet.
#[derive(Debug, Default)]
pub struct CollabSession {
    /// This replica's identity: the suffix of every [`FractionalKey`] it mints.
    pub session: SessionId,
    /// Commits produced locally and not yet handed to the log.
    pub pending: Vec<LocalCommit>,
}

impl CollabModel<'static> {
    /// An empty replica: a workbook with **no sheets at all**, since every sheet arrives as a
    /// [`Patch::AddSheet`](crate::collab::patch::Patch::AddSheet) like any other write.
    ///
    /// `session` is this replica's identity and the suffix of every [`FractionalKey`] it mints, so
    /// it must be non-zero: the all-zero suffix is
    /// [`virtual_key`](crate::collab::fractional_index::virtual_key)'s.
    pub fn new(session: SessionId) -> Self {
        debug_assert!(session != 0, "session 0 is reserved for virtual keys");
        let locale = get_default_locale();
        let language = get_default_language();
        let workbook = Workbook {
            shared_strings: vec![],
            defined_names: vec![],
            worksheets: vec![],
            styles: Default::default(),
            name: String::new(),
            settings: WorkbookSettings {
                tz: DEFAULT_TIMEZONE.to_string(),
                locale: DEFAULT_LOCALE.to_string(),
            },
            // Not a replicated register: blank rather than clock-stamped, so two replicas of the
            // same log stay byte-for-byte equal.
            metadata: Metadata {
                application: APPLICATION.to_string(),
                app_version: APP_VERSION.to_string(),
                creator: IRONCALC_USER.to_string(),
                last_modified_by: IRONCALC_USER.to_string(),
                created: String::new(),
                last_modified: String::new(),
            },
            tables: HashMap::new(),
            // Viewports are local UI state and never travel in a snapshot.
            views: HashMap::new(),
            theme: Default::default(),
            meta: Default::default(),
        };
        CollabModel {
            workbook,
            parsed_formulas: Vec::new(),
            parsed_defined_names: HashMap::new(),
            shared_strings: HashMap::new(),
            parser: Parser::new(vec![], vec![], HashMap::new(), locale, language),
            cells: HashMap::new(),
            locale,
            language,
            tz: Tz::parse(DEFAULT_TIMEZONE).expect("UTC is a valid timezone"),
            view_id: 0,
            variable_stack: HashMap::new(),
            last_variable_id: 0,
            lambdas: HashMap::new(),
            last_lambda_id: 0,
            spill_cells: Vec::new(),
            support: HashMap::new(),
            cf_cache: HashMap::new(),
            links: HashMap::new(),
            local: CollabSession {
                session,
                pending: Vec::new(),
            },
        }
    }
}

impl CollabModel<'_> {
    /// The suffix this replica mints [`FractionalKey`]s with.
    pub(crate) fn suffix(&self) -> [u8; SESSION_SUFFIX_LEN] {
        self.local.session.to_be_bytes()
    }

    /// Ordering context for a brand new sheet, minting with this replica's session.
    pub(crate) fn new_indexes(&self) -> SheetIndexes {
        let suffix = self.suffix();
        SheetIndexes {
            rows: FractionalIndex::new(vec![], vec![], suffix),
            cols: FractionalIndex::new(vec![], vec![], suffix),
            registers: Default::default(),
        }
    }
}

// Two of these build stable storage out of an ordinal model, through a helper `collab-test` gates.
#[cfg(all(test, not(feature = "collab-test")))]
mod test {
    use super::*;
    use crate::cf_types::{CfRuleInput, ValueOperator};
    use crate::test::test_stable_projection::stable_from_ordinal;
    use crate::types::{Cell, Color, Comment, Dxf, Fill, Row, SheetState, Worksheet};

    /// Explicit session suffixes: the default is all zeroes, which is the suffix reserved for
    /// [`virtual_key`](crate::collab::fractional_index::virtual_key).
    fn new_indexes() -> SheetIndexes {
        SheetIndexes {
            rows: FractionalIndex::new(vec![], vec![], [b'r', 0, 0, 0]),
            cols: FractionalIndex::new(vec![], vec![], [b'c', 0, 0, 0]),
            registers: Default::default(),
        }
    }

    fn mint(index: &mut FractionalIndex, count: usize) -> Vec<FractionalKey> {
        (0..count)
            .map(|i| index.create_key(i).expect("index has room").clone())
            .collect()
    }

    #[test]
    fn stable_worksheet_round_trip() {
        let mut index = new_indexes();
        let rows = mint(&mut index.rows, 5);
        let cols = mint(&mut index.cols, 3);

        let mut sheet_data = crate::types::SheetData::<Stable>::default();
        for (r, row_key) in rows.iter().enumerate() {
            for (c, col_key) in cols.iter().enumerate() {
                sheet_data.entry(row_key.clone()).or_default().insert(
                    col_key.clone(),
                    Cell::NumberCell {
                        v: (10 * r + c) as f64,
                        s: 0,
                    },
                );
            }
        }

        let mut ws = Worksheet::<Stable> {
            dimension: "A1:C5".to_string(),
            cols: vec![Col::<Stable> {
                min: cols[1].clone(),
                max: cols[1].clone(),
                width: 42.0,
                custom_width: true,
                hidden: false,
                style: Some(0),
            }],
            rows: vec![Row::<Stable> {
                r: rows[2].clone(),
                height: 21.0,
                custom_format: false,
                custom_height: true,
                s: 0,
                hidden: false,
            }],
            name: "Stable".to_string(),
            sheet_data,
            shared_formulas: vec![],
            sheet_id: 1,
            state: SheetState::Visible,
            color: Color::None,
            merged_cells: vec![StableRange {
                rows: Some((rows[0].clone(), rows[1].clone())),
                cols: Some((cols[0].clone(), cols[1].clone())),
            }],
            comments: vec![Comment::<Stable> {
                text: "note".to_string(),
                author_name: "me".to_string(),
                author_id: None,
                cell_ref: (rows[3].clone(), cols[2].clone()),
            }],
            frozen_rows: 0,
            frozen_columns: 0,
            views: HashMap::new(),
            show_grid_lines: true,
            conditional_formatting: vec![],
            links: HashMap::new(),
            index,
        };

        // 1. Resolution is a round trip on both axes, and answers nothing for a key it never saw.
        for (i, key) in rows.iter().enumerate() {
            let ordinal = i as i32 + 1;
            assert_eq!(Stable::row_ordinal(&ws.index, key), Some(ordinal));
            assert_eq!(Stable::row_at(&ws.index, ordinal).as_ref(), Some(key));
        }
        for (i, key) in cols.iter().enumerate() {
            let ordinal = i as i32 + 1;
            assert_eq!(Stable::col_ordinal(&ws.index, key), Some(ordinal));
            assert_eq!(Stable::col_at(&ws.index, ordinal).as_ref(), Some(key));
        }
        let stranger = FractionalKey::from([0xffu8, 0, 0, 0, 0].as_slice());
        assert_eq!(Stable::row_ordinal(&ws.index, &stranger), None);
        assert_eq!(Stable::col_ordinal(&ws.index, &stranger), None);
        assert_eq!(Stable::row_at(&ws.index, 0), None);
        assert_eq!(Stable::row_at(&ws.index, 6), None);

        // 2. An ordinal read is a resolution followed by a lookup by identity.
        let cell_at = |ws: &Worksheet<Stable>, row: i32, col: i32| {
            let r = Stable::row_at(&ws.index, row)?;
            let c = Stable::col_at(&ws.index, col)?;
            ws.sheet_data.get(&r)?.get(&c).cloned()
        };
        assert_eq!(cell_at(&ws, 2, 3), Some(Cell::NumberCell { v: 12.0, s: 0 }));

        // 3. A move renames positions, never identities: the cells stay exactly where they were
        //    filed and only the ordinals they answer to change.
        let before = ws.sheet_data.clone();
        ws.index.rows.move_to(0..1, 5); // first row to the end
        assert_eq!(ws.sheet_data, before);
        assert_eq!(Stable::row_ordinal(&ws.index, &rows[0]), Some(5));
        assert_eq!(Stable::row_ordinal(&ws.index, &rows[1]), Some(1));
        assert_eq!(cell_at(&ws, 5, 3), Some(Cell::NumberCell { v: 2.0, s: 0 }));
        assert_eq!(cell_at(&ws, 1, 3), Some(Cell::NumberCell { v: 12.0, s: 0 }));

        // 4. A removal takes the key out of the ordering and shifts everything after it up.
        ws.index.rows.remove_key(&rows[1]);
        assert_eq!(Stable::row_ordinal(&ws.index, &rows[1]), None);
        assert_eq!(Stable::row_ordinal(&ws.index, &rows[2]), Some(1));
        assert_eq!(Stable::row_ordinal(&ws.index, &rows[0]), Some(4));

        // 5. The whole sheet survives bitcode, ordering context included.
        let decoded: Worksheet<Stable> = bitcode::decode(&bitcode::encode(&ws)).unwrap();
        assert_eq!(decoded, ws);
        assert_eq!(Stable::row_ordinal(&decoded.index, &rows[2]), Some(1));
        assert_eq!(Stable::col_at(&decoded.index, 3).as_ref(), Some(&cols[2]));
        assert_eq!(
            cell_at(&decoded, 1, 3),
            Some(Cell::NumberCell { v: 22.0, s: 0 })
        );
    }

    #[test]
    fn stable_range_resolve_and_clamp() {
        let mut index = new_indexes();
        let rows = mint(&mut index.rows, 6);
        let cols = mint(&mut index.cols, 4);

        let rect = StableRange {
            rows: Some((rows[1].clone(), rows[4].clone())),
            cols: Some((cols[0].clone(), cols[2].clone())),
        };
        assert_eq!(rect.resolve(&index), Some((2, 1, 5, 3)));
        assert!(rect.contains(&index, &rows[2], &cols[1]));
        assert!(!rect.contains(&index, &rows[0], &cols[1])); // above the rectangle

        // An unbounded axis is whatever the index holds right now, so it grows with the index.
        let full_rows = StableRange {
            rows: None,
            cols: Some((cols[0].clone(), cols[2].clone())),
        };
        assert_eq!(full_rows.resolve(&index), Some((1, 1, 6, 3)));
        index.rows.create_key(6);
        assert_eq!(full_rows.resolve(&index), Some((1, 1, 7, 3)));
        assert_eq!(rect.resolve(&index), Some((2, 1, 5, 3)));

        // Concurrent moves can drag the lo corner past the hi one; the resolved rectangle stays
        // ordered, because two corners that still resolve bound the same rectangle either way.
        index.rows.move_to(1..2, 5);
        assert_eq!(rect.resolve(&index), Some((4, 1, 5, 3)));
        index.rows.move_to(4..5, 1);
        assert_eq!(rect.resolve(&index), Some((2, 1, 5, 3)));

        // Deleting the hi corner clamps it to the element just before where it used to sit.
        index.rows.remove_key(&rows[4]);
        assert_eq!(rect.resolve(&index), Some((2, 1, 4, 3)));

        // Deleting everything the rectangle covered collapses it.
        for key in [&rows[1], &rows[2], &rows[3]] {
            index.rows.remove_key(key);
        }
        assert_eq!(rect.resolve(&index), None);
        assert!(!rect.contains(&index, &rows[0], &cols[0]));
    }

    #[test]
    fn stable_col_span_semantics() {
        let mut index = new_indexes();
        let cols = mint(&mut index.cols, 6);
        let span = |min: &FractionalKey, max: &FractionalKey| Col::<Stable> {
            min: min.clone(),
            max: max.clone(),
            width: 20.0,
            custom_width: true,
            hidden: false,
            style: None,
        };
        let wide = span(&cols[0], &cols[3]);
        assert_eq!(wide.resolve(&index), Some((1, 4)));

        // A span is a region between its corners; the degenerate `(k, k)` follows its column.
        let single = span(&cols[4], &cols[4]);
        assert_eq!(single.resolve(&index), Some((5, 5)));
        index.cols.move_to(4..5, 6);
        assert_eq!(single.resolve(&index), Some((6, 6)));
        assert_eq!(wide.resolve(&index), Some((1, 4)));

        // `(NULL, NULL)` is the whole axis — the whole-sheet styling register — and tracks it as it
        // grows.
        let all = span(&FractionalKey::NULL, &FractionalKey::NULL);
        assert_eq!(all.resolve(&index), Some((1, 6)));
        index.cols.create_key(6).expect("index has room");
        assert_eq!(all.resolve(&index), Some((1, 7)));

        // A half-open span runs from its one real corner to the end of the axis, and keeps
        // tracking that end as columns are appended past it.
        let tail = span(&cols[2], &FractionalKey::NULL);
        let head = span(&FractionalKey::NULL, &cols[2]);
        assert_eq!(tail.resolve(&index), Some((3, 7)));
        assert_eq!(head.resolve(&index), Some((1, 3)));
        index.cols.create_key(7).expect("index has room");
        assert_eq!(tail.resolve(&index), Some((3, 8)));
        assert_eq!(head.resolve(&index), Some((1, 3)));
    }

    /// The stable twin of an ordinal workbook: the same document, addressed by key.
    fn stable_twin(model: &Model) -> Workbook<Stable> {
        let wb = &model.workbook;
        Workbook {
            shared_strings: wb.shared_strings.clone(),
            defined_names: wb.defined_names.clone(),
            worksheets: wb.worksheets.iter().map(stable_from_ordinal).collect(),
            styles: wb.styles.clone(),
            name: wb.name.clone(),
            settings: wb.settings.clone(),
            metadata: wb.metadata.clone(),
            tables: wb.tables.clone(),
            views: wb.views.clone(),
            theme: wb.theme.clone(),
            meta: Default::default(),
        }
    }

    /// The engine is the same engine: evaluating a workbook and its stable twin has to give the
    /// same values and the same conditional formatting, cell for cell.
    #[test]
    fn stable_eval_matches_ordinal() {
        let mut ordinal = Model::new_empty("model", "en", "UTC", "en").unwrap();
        ordinal.set_user_input(0, 1, 1, "10".to_string()).unwrap();
        ordinal.set_user_input(0, 2, 1, "20".to_string()).unwrap();
        ordinal.set_user_input(0, 3, 1, "text".to_string()).unwrap();
        ordinal
            .set_user_input(0, 1, 2, "=A1+A2".to_string())
            .unwrap();
        ordinal
            .set_user_input(0, 2, 2, "=B1*2".to_string())
            .unwrap();
        ordinal
            .set_user_input(0, 3, 2, "=CONCAT(A3, \"!\")".to_string())
            .unwrap();
        ordinal
            .add_conditional_formatting(
                0,
                "A1:B3",
                CfRuleInput::CellIs {
                    operator: ValueOperator::GreaterThan,
                    formula: "15".to_string(),
                    formula2: None,
                    format: Dxf {
                        fill: Some(Fill {
                            color: Color::Rgb("#FF0000".to_string()),
                        }),
                        ..Default::default()
                    },
                    stop_if_true: false,
                },
            )
            .unwrap();
        ordinal.evaluate();

        let mut stable = CollabModel::new(1);
        stable.workbook = stable_twin(&ordinal);
        // Parses and evaluates: no projection and no copy, the engine reads the stable storage.
        stable.reset_parsed_structures();

        let cells = ordinal.get_all_cells();
        assert!(!cells.is_empty());
        // The rule fired on the stable side, so the comparison below is not vacuous.
        assert!(!stable.cf_cache.is_empty());
        for cell in cells {
            let (sheet, row, column) = (cell.index, cell.row, cell.column);
            assert_eq!(
                stable.get_formatted_cell_value(sheet, row, column),
                ordinal.get_formatted_cell_value(sheet, row, column),
                "value at ({sheet}, {row}, {column})"
            );
            assert_eq!(
                stable
                    .get_extended_style_for_cell(sheet, row, column)
                    .map(|s| s.style),
                ordinal
                    .get_extended_style_for_cell(sheet, row, column)
                    .map(|s| s.style),
                "conditional formatting at ({sheet}, {row}, {column})"
            );
        }
    }

    /// Formulas are stored in R1C1 with a fixed parse anchor, so a row move needs no reparse: the
    /// relative offsets resolve against wherever the formula cell now sits.
    #[test]
    fn stable_eval_tracks_moves() {
        let mut ordinal = Model::new_empty("model", "en", "UTC", "en").unwrap();
        for (row, value) in [(1, "10"), (2, "20"), (3, "30"), (4, "40")] {
            ordinal
                .set_user_input(0, row, 1, value.to_string())
                .unwrap();
        }
        // Two rows above its own: 20 now, whatever sits there after the move later.
        ordinal.set_user_input(0, 4, 2, "=A2".to_string()).unwrap();

        let mut stable = CollabModel::new(1);
        stable.workbook = stable_twin(&ordinal);
        stable.reset_parsed_structures();
        assert_eq!(
            stable.get_formatted_cell_value(0, 4, 2),
            Ok("20".to_string())
        );

        // Second row to the end: the rows now read 10, 30, 40, 20 and the formula cell sits third.
        stable.workbook.worksheets[0].index.rows.move_to(1..2, 4);
        stable.evaluate();

        assert_eq!(
            stable.get_formatted_cell_value(0, 1, 1),
            Ok("10".to_string())
        );
        assert_eq!(
            stable.get_formatted_cell_value(0, 2, 1),
            Ok("30".to_string())
        );
        assert_eq!(
            stable.get_formatted_cell_value(0, 3, 1),
            Ok("40".to_string())
        );
        assert_eq!(
            stable.get_formatted_cell_value(0, 4, 1),
            Ok("20".to_string())
        );
        // The formula moved with its row and its offset resolves against the new position.
        assert_eq!(
            stable.get_formatted_cell_value(0, 3, 2),
            Ok("10".to_string())
        );
    }
}
