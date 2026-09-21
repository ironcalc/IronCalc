use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    hash::{Hash, Hasher},
};

use crate::constants::{COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT};
use crate::expressions::parser::Node;
use crate::model::Model;
use crate::user_model::OrdinalUserState;
use crate::{
    cf_types::ConditionalFormatting,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        token::Error,
        utils::{
            column_to_number, is_valid_column, is_valid_column_number, is_valid_row,
            number_to_column, parse_reference_a1,
        },
    },
    ROW_HEIGHT_FACTOR,
};

fn default_as_false() -> bool {
    false
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, Default)]
#[serde(untagged)]
pub enum Color {
    Rgb(String),
    /// Theme slot index and tint. Tint ∈ [-1, 1]: positive lightens, negative darkens.
    Theme(i32, f64),
    /// No color — equivalent to OOXML `<color auto="1"/>` or absence of `<color>`.
    #[default]
    None,
}

impl Color {
    /// Bit pattern of a theme tint, normalised so that equality and hashing agree.
    ///
    /// `f64`'s own `==` cannot back an [`Eq`] implementation: it is not reflexive, because
    /// `NaN != NaN`. A `Color` holding a NaN tint would therefore never compare equal to itself and
    /// could never be found again once used as a hash key. Normalising collapses every NaN to one
    /// bit pattern, and `-0.0` to `0.0` so that the two spellings of "no tint" stay equal as they
    /// are under `f64` comparison.
    fn tint_key(tint: f64) -> u64 {
        if tint.is_nan() {
            f64::NAN.to_bits()
        } else {
            (tint + 0.0).to_bits()
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Color::Rgb(a), Color::Rgb(b)) => a == b,
            (Color::Theme(a_slot, a_tint), Color::Theme(b_slot, b_tint)) => {
                a_slot == b_slot && Color::tint_key(*a_tint) == Color::tint_key(*b_tint)
            }
            (Color::None, Color::None) => true,
            _ => false,
        }
    }
}

impl Eq for Color {}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Color::Rgb(rgb) => rgb.hash(state),
            Color::Theme(slot, tint) => {
                slot.hash(state);
                Color::tint_key(*tint).hash(state);
            }
            Color::None => {}
        }
    }
}

/// Valid hex colors are #FFAABB
/// #fff is not valid
fn is_valid_hex_color(color: &str) -> bool {
    if color.chars().count() != 7 {
        return false;
    }
    if !color.starts_with('#') {
        return false;
    }
    if let Ok(z) = i32::from_str_radix(&color[1..], 16) {
        if (0..=0xffffff).contains(&z) {
            return true;
        }
    }
    false
}

impl Color {
    pub fn is_none(&self) -> bool {
        matches!(self, Color::None)
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, Color::None)
    }

    /// Resolves the color to a `#RRGGBB` string, consulting the workbook theme when needed.
    /// Returns an empty string for `Color::None`.
    pub fn to_rgb(&self, theme: &Theme) -> String {
        match self {
            Color::Rgb(s) => s.clone(),
            Color::Theme(idx, tint) => theme.resolve(*idx, *tint),
            Color::None => String::new(),
        }
    }

    pub fn from_rgb(color: &str) -> Result<Self, String> {
        if is_valid_hex_color(color) {
            return Ok(Color::Rgb(color.to_string()));
        }
        Err(format!("Invalid color: '{}'.", color))
    }

    /// Parses a color from the JS/WASM parameter format:
    /// - `""` => `Color::None`
    /// - `"#RRGGBB"` => `Color::Rgb(...)`
    /// - `"[index, tint]"` => `Color::Theme(index, tint)`
    pub fn from_param(s: &str) -> Result<Self, String> {
        if s.is_empty() {
            return Ok(Color::None);
        }
        if is_valid_hex_color(s) {
            return Ok(Color::Rgb(s.to_string()));
        }
        if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let mut parts = inner.splitn(2, ',');
            if let (Some(idx_str), Some(tint_str)) = (parts.next(), parts.next()) {
                if let (Ok(idx), Ok(tint)) = (
                    idx_str.trim().parse::<i32>(),
                    tint_str.trim().parse::<f64>(),
                ) {
                    return Ok(Color::Theme(idx, tint));
                }
            }
        }
        Err(format!("Invalid color: '{}'.", s))
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub application: String,
    pub app_version: String,
    pub creator: String,
    pub last_modified_by: String,
    pub created: String,       // "2020-08-06T21:20:53Z",
    pub last_modified: String, //"2020-11-20T16:24:35"
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct WorkbookSettings {
    pub tz: String,
    pub locale: String,
}

/// A Workbook View tracks of the selected sheet for each view
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct WorkbookView {
    /// The index of the currently selected sheet.
    pub sheet: u32,
    /// The current width of the window
    pub window_width: i64,
    /// The current height of the window
    pub window_height: i64,
}

/// An internal representation of an IronCalc Workbook
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Workbook<A: Position = Ordinal> {
    pub shared_strings: Vec<String>,
    pub defined_names: Vec<DefinedName>,
    pub worksheets: Vec<Worksheet<A>>,
    pub styles: Styles,
    pub name: String,
    pub settings: WorkbookSettings,
    pub metadata: Metadata,
    pub tables: HashMap<String, Table>,
    /// Per-user viewport state, never encoded: it decodes back as empty.
    #[bitcode(skip)]
    pub views: HashMap<u32, WorkbookView>,
    pub theme: Theme,
    /// CRDT metadata riding with the document; `()` encodes to zero bytes.
    pub meta: A::WorkbookMeta,
}

/// A defined name. The `sheet_id` is the sheet index in case the name is local
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DefinedName {
    pub name: String,
    pub formula: String,
    pub sheet_id: Option<u32>,
}

/// * state:
///   18.18.68 ST_SheetState (Sheet Visibility Types)
///   hidden, veryHidden, visible
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum SheetState {
    Visible,
    Hidden,
    VeryHidden,
}

impl Display for SheetState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SheetState::Visible => write!(formatter, "visible"),
            SheetState::Hidden => write!(formatter, "hidden"),
            SheetState::VeryHidden => write!(formatter, "veryHidden"),
        }
    }
}

/// Represents the state of the worksheet as seen by the user. This includes
/// details such as the currently selected cell, the visible range, and the
/// position of the viewport.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct WorksheetView {
    /// The row index of the currently selected cell (the anchor of the selection).
    pub row: i32,
    /// The column index of the currently selected cell (the anchor of the selection).
    pub column: i32,
    /// The selected range in the worksheet, specified as [start_row, start_column, end_row, end_column].
    /// Always normalized (start <= end). This is derived state: the bounding box of the
    /// anchor and the focus, grown so it never covers a merged cell partially.
    pub range: [i32; 4],
    /// The row of the focus: the moving corner of the selection, the one that
    /// keyboard- or pointer-extending the selection displaces. It is not
    /// necessarily a corner of `range` (extending over merged cells grows the
    /// range beyond the anchor-focus bounding box).
    pub focus_row: i32,
    /// The column of the focus.
    pub focus_column: i32,
    /// The row index of the topmost visible cell in the worksheet view.
    pub top_row: i32,
    /// The column index of the leftmost visible cell in the worksheet view.
    pub left_column: i32,
}

/// Represents a hyperlink in the worksheet, which can be either external or internal.
/// The display text is not part of the link, it is the content of the cell the link is
/// attached to. Links are just cell metadata.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum Link {
    /// A link to a resource outside the workbook: an URL, a mailto: URI or a file.
    /// If the target points to a location inside another document it is written
    /// as `target#location` (e.g. `file.xlsx#Sheet1!A1`).
    External {
        target: String,
        tooltip: Option<String>,
    },
    /// A link to a location in this workbook: a cell reference like `Sheet1!A30`
    /// or a defined name.
    Internal {
        location: String,
        tooltip: Option<String>,
    },
}

/// A rectangular range of cells that is displayed as a single cell.
/// The anchor (top-left) cell holds the content and is the only cell of the
/// range users can edit; every other cell of the range is "covered" and must
/// stay empty of content, although it can hold styles.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
pub struct MergedCell {
    /// Row of the anchor (top-left) cell
    pub row: i32,
    /// Column of the anchor (top-left) cell
    pub column: i32,
    /// Number of columns of the merged range (>= 1)
    pub width: i32,
    /// Number of rows of the merged range (>= 1)
    pub height: i32,
}

impl MergedCell {
    /// Last (bottom) row of the merged range
    pub fn last_row(&self) -> i32 {
        self.row + self.height - 1
    }

    /// Last (rightmost) column of the merged range
    pub fn last_column(&self) -> i32 {
        self.column + self.width - 1
    }

    /// Returns true if (row, column) is inside the merged range
    pub fn contains(&self, row: i32, column: i32) -> bool {
        row >= self.row
            && row <= self.last_row()
            && column >= self.column
            && column <= self.last_column()
    }

    /// Returns true if the merged range intersects the rectangle of `width` columns
    /// and `height` rows anchored at (row, column)
    pub fn intersects(&self, row: i32, column: i32, width: i32, height: i32) -> bool {
        row <= self.last_row()
            && row + height > self.row
            && column <= self.last_column()
            && column + width > self.column
    }
}

impl From<&MergedCell> for RangeRef {
    fn from(m: &MergedCell) -> RangeRef {
        RangeRef {
            rows: Some((m.row, m.last_row())),
            cols: Some((m.column, m.last_column())),
        }
    }
}

impl From<&RangeRef> for MergedCell {
    /// Unbounded axes span the whole grid.
    fn from(r: &RangeRef) -> MergedCell {
        let (row, column, last_row, last_column) = r.resolve();
        MergedCell {
            row,
            column,
            width: last_column - column + 1,
            height: last_row - row + 1,
        }
    }
}

pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The addressing scheme of the workbook data model: how rows and columns are named.
///
/// `Clone` is a supertrait so that the `#[derive]`s on the generic containers, which emit
/// `A: Clone` bounds, are satisfied by a bare `A: Position`.
pub trait Position: sealed::Sealed + Sized + Clone {
    /// Row/column identifier. Bounds are the union of what the containers' derives need.
    type Key: Clone + Ord + Hash + std::fmt::Debug + Encode + bitcode::DecodeOwned;
    /// Per-sheet ordering context; `()` for [`Ordinal`].
    type SheetIndex: Clone + Default + std::fmt::Debug + PartialEq + Encode + bitcode::DecodeOwned;
    /// How a merged range is stored: an anchor plus a size ([`MergedCell`]) when
    /// the addressing is positional, a keyed [`RangeRef`] when it is stable.
    type MergedCell: Clone + std::fmt::Debug + PartialEq + Encode + bitcode::DecodeOwned;
    /// Workbook-wide replication metadata; `()` for [`Ordinal`].
    type WorkbookMeta: Clone + Default + std::fmt::Debug + PartialEq + Encode + bitcode::DecodeOwned;
    /// Replica-local model state, never serialized; `()` for [`Ordinal`].
    type Local: Default;
    /// Replica-local state of the [`UserModel`](crate::UserModel) wrapper: undo/redo and whatever
    /// else the wrapper keeps outside the workbook.
    type UserState: Default;
    /// Storage form of a shared formula: R1C1 text under [`Ordinal`], a bound token stream under
    /// [`Stable`](crate::collab::model::Stable).
    type Formula: Clone + PartialEq + std::fmt::Debug + Encode + bitcode::DecodeOwned;
    /// Storage form of a cell hyperlink: a plain [`Link`] under [`Ordinal`], a link whose internal
    /// location is a bound stream under [`Stable`](crate::collab::model::Stable).
    type Link: Clone + PartialEq + std::fmt::Debug + Encode + bitcode::DecodeOwned;

    // Key ⇄ ordinal resolution. Ordinals are the 1-based `i32` the rest of the codebase uses;
    // `None` means the key names nothing in this index any more.
    fn row_ordinal(idx: &Self::SheetIndex, key: &Self::Key) -> Option<i32>;
    fn col_ordinal(idx: &Self::SheetIndex, key: &Self::Key) -> Option<i32>;
    fn row_at(idx: &Self::SheetIndex, ordinal: i32) -> Option<Self::Key>;
    fn col_at(idx: &Self::SheetIndex, ordinal: i32) -> Option<Self::Key>;

    /// How many rows/columns this index currently addresses. The whole grid under [`Ordinal`].
    fn row_count(idx: &Self::SheetIndex) -> i32;
    fn col_count(idx: &Self::SheetIndex) -> i32;

    /// The 1-based ordinal rectangle `(row1, column1, row2, column2)` a range currently denotes,
    /// or `None` if it collapsed.
    fn resolve_range(
        range: &RangeRef<Self>,
        idx: &Self::SheetIndex,
    ) -> Option<(i32, i32, i32, i32)>;

    /// The range as ordinals: each corner is the 1-based index its key currently sits at, and an
    /// unbounded axis stays unbounded. `None` when a corner names nothing any more.
    fn to_ordinal_range(range: &RangeRef<Self>, idx: &Self::SheetIndex) -> Option<RangeRef>;

    /// The 1-based ordinal rectangle `(row1, column1, row2, column2)` a merged range currently
    /// covers, or `None` if it collapsed.
    fn resolve_merged(
        merged: &Self::MergedCell,
        idx: &Self::SheetIndex,
    ) -> Option<(i32, i32, i32, i32)>;

    // Row/column metrics. Genuinely per-representation: [`Ordinal`] reads the ranged `Col`/`Row`
    // records by index, stable addressing resolves a key and its covering spans.
    fn column_width(sheet: &Worksheet<Self>, column: i32) -> Result<f64, String>;
    fn is_column_hidden(sheet: &Worksheet<Self>, column: i32) -> Result<bool, String>;
    fn row_height(sheet: &Worksheet<Self>, row: i32) -> Result<f64, String>;
    fn is_row_hidden(sheet: &Worksheet<Self>, row: i32) -> Result<bool, String>;

    /// The cell at `(row, column)`: what `sheet_data` holds.
    fn cell(sheet: &Worksheet<Self>, row: i32, column: i32) -> Option<&Cell>;
    /// Stores a spilled cell at `(row, column)`.
    fn write_spill(
        sheet: &mut Worksheet<Self>,
        row: i32,
        column: i32,
        cell: Cell,
    ) -> Result<(), String>;
    /// Drops every spilled cell of the sheet, before a full re-evaluation rebuilds them.
    fn drop_spills(_sheet: &mut Worksheet<Self>) {}

    /// The AST the formula interned at `index` on `sheet` is *shown* as:
    /// 1. For [Ordinal] is pretty much identity function.
    /// 2. For [Stable] is a lowered stable references to construct a specific node.
    fn materialize_formula<'b>(
        model: &'b Model<Self>,
        sheet: u32,
        row: i32,
        column: i32,
        index: i32,
    ) -> Option<std::borrow::Cow<'b, Node>>;
}

/// Positional addressing: rows and columns are 1-based indices.
///
/// A unit marker deriving everything, so that the `#[derive]`s on the generic
/// containers (which emit `A: Trait` bounds) are satisfiable.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize,
)]
pub struct Ordinal;

impl sealed::Sealed for Ordinal {}

impl Position for Ordinal {
    type Key = i32;
    type SheetIndex = ();
    type MergedCell = MergedCell;
    type WorkbookMeta = ();
    type Local = ();
    type UserState = OrdinalUserState;
    type Formula = String;
    type Link = Link;

    // The key *is* the ordinal, so resolution is the identity and there is nothing to bound-check.
    #[inline]
    fn row_ordinal(_idx: &(), key: &i32) -> Option<i32> {
        Some(*key)
    }
    #[inline]
    fn col_ordinal(_idx: &(), key: &i32) -> Option<i32> {
        Some(*key)
    }
    #[inline]
    fn row_at(_idx: &(), ordinal: i32) -> Option<i32> {
        Some(ordinal)
    }
    #[inline]
    fn col_at(_idx: &(), ordinal: i32) -> Option<i32> {
        Some(ordinal)
    }

    #[inline]
    fn row_count(_idx: &()) -> i32 {
        LAST_ROW
    }
    #[inline]
    fn col_count(_idx: &()) -> i32 {
        LAST_COLUMN
    }
    #[inline]
    fn resolve_merged(merged: &MergedCell, _idx: &()) -> Option<(i32, i32, i32, i32)> {
        Some((
            merged.row,
            merged.column,
            merged.last_row(),
            merged.last_column(),
        ))
    }

    fn resolve_range(range: &RangeRef, _idx: &()) -> Option<(i32, i32, i32, i32)> {
        Some(range.resolve())
    }

    #[inline]
    fn to_ordinal_range(range: &RangeRef, _idx: &()) -> Option<RangeRef> {
        Some(range.clone())
    }

    fn column_width(sheet: &Worksheet, column: i32) -> Result<f64, String> {
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        for col in &sheet.cols {
            if column >= col.min && column <= col.max {
                if col.hidden {
                    return Ok(0.0);
                }
                if col.custom_width {
                    return Ok(col.width * COLUMN_WIDTH_FACTOR);
                }
                break;
            }
        }
        Ok(DEFAULT_COLUMN_WIDTH)
    }

    fn is_column_hidden(sheet: &Worksheet, column: i32) -> Result<bool, String> {
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        for col in &sheet.cols {
            if column >= col.min && column <= col.max {
                return Ok(col.hidden);
            }
        }
        Ok(false)
    }

    fn row_height(sheet: &Worksheet, row: i32) -> Result<f64, String> {
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        for r in &sheet.rows {
            if r.r == row {
                if r.hidden {
                    return Ok(0.0);
                }
                return Ok(r.height * ROW_HEIGHT_FACTOR);
            }
        }
        Ok(DEFAULT_ROW_HEIGHT)
    }

    fn is_row_hidden(sheet: &Worksheet, row: i32) -> Result<bool, String> {
        if !is_valid_row(row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        for r in &sheet.rows {
            if r.r == row {
                return Ok(r.hidden);
            }
        }
        Ok(false)
    }

    #[inline]
    fn cell(sheet: &Worksheet, row: i32, column: i32) -> Option<&Cell> {
        sheet.stored_cell(row, column)
    }
    #[inline]
    fn write_spill(sheet: &mut Worksheet, row: i32, column: i32, cell: Cell) -> Result<(), String> {
        sheet.update_cell(row, column, cell)
    }

    /// The stored node already carries offsets from wherever the formula sits, so it *is* the
    /// display form.
    fn materialize_formula<'b>(
        model: &'b Model,
        sheet: u32,
        _row: i32,
        _column: i32,
        index: i32,
    ) -> Option<std::borrow::Cow<'b, Node>> {
        let (node, _) = model
            .parsed_formulas
            .get(sheet as usize)?
            .get(index as usize)?;
        Some(std::borrow::Cow::Borrowed(node))
    }
}

/// A cell position: (row, column), 1-based.
pub type CellAddr<A = Ordinal> = (<A as Position>::Key, <A as Position>::Key);

/// A rectangular reference. An axis is a closed 1-based interval, or `None`
/// meaning the whole axis (full-column `D:D`, full-row `5:7`).
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(bound(
    serialize = "A::Key: Serialize",
    deserialize = "A::Key: serde::Deserialize<'de>"
))]
pub struct RangeRef<A: Position = Ordinal> {
    pub rows: Option<(A::Key, A::Key)>,
    pub cols: Option<(A::Key, A::Key)>,
}

/// One side of an A1 reference: a column index, a row index, or both.
fn parse_a1_part(s: &str) -> Option<(Option<i32>, Option<i32>)> {
    let s = s.replace('$', "");
    if let Some(r) = parse_reference_a1(&s) {
        return Some((Some(r.column), Some(r.row)));
    }
    if is_valid_column(&s) {
        return Some((Some(column_to_number(&s).ok()?), None));
    }
    let row = s.parse::<i32>().ok()?;
    is_valid_row(row).then_some((None, Some(row)))
}

/// `None` when the interval spans the whole axis, so that a bounded storage ref
/// and its formula shorthand parse to the same value.
fn unbounded_if_full(span: (i32, i32), last: i32) -> Option<(i32, i32)> {
    (span != (1, last)).then_some(span)
}

fn ordered(a: i32, b: i32) -> (i32, i32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

impl RangeRef {
    pub fn cell(row: i32, column: i32) -> RangeRef {
        RangeRef {
            rows: Some((row, row)),
            cols: Some((column, column)),
        }
    }

    /// Parses `A1`, `A1:B2`, full-column `D:F` or full-row `5:7`.
    /// Case-insensitive, `$` markers are ignored, out-of-bounds is rejected.
    pub fn parse_a1(s: &str) -> Option<RangeRef> {
        let s = s.to_uppercase();
        let Some((left, right)) = s.split_once(':') else {
            let (column, row) = parse_a1_part(&s)?;
            return Some(RangeRef::cell(row?, column?));
        };
        match (parse_a1_part(left)?, parse_a1_part(right)?) {
            // A full-span axis is the storage spelling of an unbounded one (`D1:D1048576` = `D:D`).
            ((Some(c1), Some(r1)), (Some(c2), Some(r2))) => Some(RangeRef {
                rows: unbounded_if_full(ordered(r1, r2), LAST_ROW),
                cols: unbounded_if_full(ordered(c1, c2), LAST_COLUMN),
            }),
            ((Some(c1), None), (Some(c2), None)) => Some(RangeRef {
                rows: None,
                cols: Some(ordered(c1, c2)),
            }),
            ((None, Some(r1)), (None, Some(r2))) => Some(RangeRef {
                rows: Some(ordered(r1, r2)),
                cols: None,
            }),
            _ => None,
        }
    }

    /// Canonical A1 rendering: uppercase, no `$`.
    pub fn to_a1(&self) -> String {
        match (self.rows, self.cols) {
            (Some((r1, r2)), None) => format!("{r1}:{r2}"),
            (None, Some((c1, c2))) => format!(
                "{}:{}",
                number_to_column(c1).unwrap_or_default(),
                number_to_column(c2).unwrap_or_default()
            ),
            // Bounded on both axes; a fully unbounded ref renders as the whole grid.
            _ => self.to_a1_bounded(),
        }
    }

    /// A1 rendering for xlsx storage attributes (`ST_Ref`): both corners always
    /// carry a column and a row, so unbounded axes are expanded to the whole grid.
    pub fn to_a1_bounded(&self) -> String {
        let (row1, column1, row2, column2) = self.resolve();
        let c1 = number_to_column(column1).unwrap_or_default();
        let c2 = number_to_column(column2).unwrap_or_default();
        if row1 == row2 && c1 == c2 {
            format!("{c1}{row1}")
        } else {
            format!("{c1}{row1}:{c2}{row2}")
        }
    }

    /// Parses a whitespace-separated list of references, skipping invalid parts.
    pub fn parse_sqref(s: &str) -> Vec<RangeRef> {
        s.split_whitespace()
            .filter_map(RangeRef::parse_a1)
            .collect()
    }

    pub fn to_sqref(ranges: &[RangeRef]) -> String {
        ranges
            .iter()
            .map(RangeRef::to_a1)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// [`RangeRef::to_a1_bounded`] over a list, for the xlsx `sqref` attribute.
    pub fn to_sqref_bounded(ranges: &[RangeRef]) -> String {
        ranges
            .iter()
            .map(RangeRef::to_a1_bounded)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// `(row1, column1, row2, column2)` with unbounded axes expanded to the whole grid.
    pub fn resolve(&self) -> (i32, i32, i32, i32) {
        let (row1, row2) = self.rows.unwrap_or((1, LAST_ROW));
        let (column1, column2) = self.cols.unwrap_or((1, LAST_COLUMN));
        (row1, column1, row2, column2)
    }

    pub fn contains(&self, row: i32, column: i32) -> bool {
        let (row1, column1, row2, column2) = self.resolve();
        (row1..=row2).contains(&row) && (column1..=column2).contains(&column)
    }
}

/// Internal representation of a worksheet Excel object
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Worksheet<A: Position = Ordinal> {
    pub dimension: String,
    pub cols: Vec<Col<A>>,
    pub rows: Vec<Row<A>>,
    pub name: String,
    pub sheet_data: SheetData<A>,
    pub shared_formulas: Vec<A::Formula>,
    pub sheet_id: u32,
    pub state: SheetState,
    pub color: Color,
    pub merged_cells: Vec<A::MergedCell>,
    pub comments: Vec<Comment<A>>,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    /// Per-user viewport state, never encoded: it decodes back as empty.
    #[bitcode(skip)]
    pub views: HashMap<u32, WorksheetView>,
    /// Whether or not to show the grid lines in the worksheet
    pub show_grid_lines: bool,
    pub conditional_formatting: Vec<ConditionalFormatting<A>>,
    /// Hyperlinks in the worksheet, keyed by (row, column) of the cell they are attached to
    pub links: HashMap<CellAddr<A>, A::Link>,
    /// The ordering context every key in this sheet resolves against; `()` for [`Ordinal`].
    pub index: A::SheetIndex,
}

/// Internal representation of Excel's sheet_data
/// It is row first and because of this all of our API's should be row first
pub type SheetData<A = Ordinal> =
    HashMap<<A as Position>::Key, HashMap<<A as Position>::Key, Cell>>;

// ECMA-376-1:2016 section 18.3.1.73
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Row<A: Position = Ordinal> {
    /// Row index
    pub r: A::Key,
    pub height: f64,
    pub custom_format: bool,
    pub custom_height: bool,
    pub s: i32,
    pub hidden: bool,
}

impl<A: Position> Row<A> {
    pub fn is_empty(&self) -> bool {
        self.s == 0
            && !self.custom_format
            && !self.custom_height
            && !self.hidden
            && self.height == DEFAULT_ROW_HEIGHT / ROW_HEIGHT_FACTOR
    }
}

// ECMA-376-1:2016 section 18.3.1.13
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Col<A: Position = Ordinal> {
    // Column definitions are defined on ranges, unlike rows which store unique, per-row entries.
    /// First column affected by this record. Settings apply to column in \[min, max\] range.
    pub min: A::Key,
    /// Last column affected by this record. Settings apply to column in \[min, max\] range.
    pub max: A::Key,
    pub width: f64,
    pub custom_width: bool,
    pub hidden: bool,
    pub style: Option<i32>,
}

/// Cell type enum matching Excel TYPE() function values.
#[derive(Debug, Eq, PartialEq)]
pub enum CellType {
    Number = 1,
    Text = 2,
    LogicalValue = 4,
    ErrorValue = 16,
    Array = 64,
    CompoundData = 128,
}

/// The evaluated value stored in a formula cell.
/// `Unevaluated` is a transient state that only exists during evaluation.
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum FormulaValue {
    Unevaluated,
    Boolean(bool),
    Number(f64),
    Text(String),
    Error {
        ei: Error,
        // Origin cell reference, e.g. "Sheet3!C4"
        o: String,
        // Human-readable error message, e.g. "Not implemented function"
        m: String,
    },
}

/// The value stored in a spill cell (no formula, no origin tracking).
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum SpillValue {
    Boolean(bool),
    Number(f64),
    Text(String),
    Error(Error),
}

/// Whether an array formula is a CSE (Ctrl+Shift+Enter) formula or a dynamic formula.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq)]
pub enum ArrayKind {
    /// Ctrl+Shift+Enter array formula: fills a fixed declared range.
    Cse,
    /// Dynamic array formula: spills into adjacent cells automatically.
    Dynamic,
}

// A cell in a worksheet.
// Every cell has a style index (s) pointing to cell_xfs in the workbook styles.
// Other fields:
// * `f`    — formula index into the sheet's shared_formulas list
// * `si`   — shared string index (SharedString cells only)
// * `v`    — evaluated value (formula/spill cells)
// * `r`    — spill range (width, height) for array/dynamic formula anchors
// * `kind` — Cse or Dynamic for array formula anchors
// * `a`    — anchor cell (row, column) for spill cells
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Cell {
    EmptyCell {
        s: i32,
    },
    BooleanCell {
        v: bool,
        s: i32,
    },
    NumberCell {
        v: f64,
        s: i32,
    },
    // Maybe we should not have this type. In Excel this is just a string
    ErrorCell {
        ei: Error,
        s: i32,
    },
    // Always a shared string
    SharedString {
        si: i32,
        s: i32,
    },
    // A regular (non-array) formula cell.
    // `v` is `Unevaluated` transiently during evaluation, then holds the result.
    CellFormula {
        f: i32,
        s: i32,
        v: FormulaValue,
    },
    // The anchor of an array or dynamic formula.
    // `kind` distinguishes CSE from dynamic; `r` is the spill range (width, height).
    // `v` is `Unevaluated` transiently during evaluation, then holds the anchor cell result.
    ArrayFormula {
        f: i32,
        s: i32,
        r: (i32, i32),
        kind: ArrayKind,
        v: FormulaValue,
    },
    // A spill cell: holds a value produced by an array/dynamic formula at `a` (row, column).
    SpillCell {
        s: i32,
        a: (i32, i32),
        v: SpillValue,
    },
}

impl Default for Cell {
    fn default() -> Self {
        Cell::EmptyCell { s: 0 }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(bound(
    serialize = "A::Key: Serialize",
    deserialize = "A::Key: serde::Deserialize<'de>"
))]
pub struct Comment<A: Position = Ordinal> {
    pub text: String,
    pub author_name: String,
    pub author_id: Option<String>,
    pub cell_ref: CellAddr<A>,
}

// ECMA-376-1:2016 section 18.5.1.2
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Table {
    pub name: String,
    pub display_name: String,
    pub sheet_name: String,
    pub reference: String,
    pub totals_row_count: u32,
    pub header_row_count: u32,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub columns: Vec<TableColumn>,
    pub style_info: TableStyleInfo,
    pub has_filters: bool,
}

// totals_row_label vs totals_row_function might be mutually exclusive. Use an enum?
// the totals_row_function is an enum not String methinks
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TableColumn {
    pub id: u32,
    pub name: String,
    pub totals_row_label: Option<String>,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub totals_row_function: Option<String>,
}

impl Default for TableColumn {
    fn default() -> Self {
        TableColumn {
            id: 0,
            name: "Column".to_string(),
            totals_row_label: None,
            totals_row_function: None,
            data_dxf_id: None,
            header_row_dxf_id: None,
            totals_row_dxf_id: None,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct TableStyleInfo {
    pub name: Option<String>,
    pub show_first_column: bool,
    pub show_last_column: bool,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct DxfFont {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub u: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sz: Option<i32>,
    #[serde(skip_serializing_if = "Color::is_none")]
    #[serde(default)]
    pub color: Color,
}

// Dxf stands for "Differential Formatting". It is used in places like:
// * conditional formatting
// * tables
// to specify partial formatting that overrides the cell formatting.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone, Default)]
pub struct Dxf {
    pub font: Option<DxfFont>,
    pub fill: Option<Fill>,
    pub border: Option<Border>,
    pub num_fmt: Option<NumFmt>,
    pub alignment: Option<Alignment>,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Styles {
    pub num_fmts: Vec<NumFmt>,
    pub fonts: Vec<Font>,
    pub fills: Vec<Fill>,
    pub borders: Vec<Border>,
    pub cell_style_xfs: Vec<CellStyleXfs>,
    pub cell_xfs: Vec<CellXfs>,
    pub cell_styles: Vec<CellStyles>,
    pub dxfs: Vec<Dxf>,
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            num_fmts: vec![],
            fonts: vec![Default::default()],
            fills: vec![Default::default(), Default::default()],
            borders: vec![Default::default()],
            cell_style_xfs: vec![Default::default()],
            cell_xfs: vec![Default::default()],
            cell_styles: vec![Default::default()],
            dxfs: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    pub num_fmt: String,
    pub fill: Fill,
    pub font: Font,
    pub border: Border,
    pub quote_prefix: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            alignment: None,
            num_fmt: "general".to_string(),
            fill: Fill::default(),
            font: Font::default(),
            border: Border::default(),
            quote_prefix: false,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct NumFmt {
    pub num_fmt_id: i32,
    pub format_code: String,
}

impl Default for NumFmt {
    fn default() -> Self {
        NumFmt {
            num_fmt_id: 0,
            format_code: "general".to_string(),
        }
    }
}

// ST_FontScheme simple type (§18.18.33).
// Usually major fonts are used for styles like headings,
// and minor fonts are used for body and paragraph text.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FontScheme {
    #[default]
    Minor,
    Major,
    None,
}

impl Display for FontScheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FontScheme::Minor => write!(formatter, "minor"),
            FontScheme::Major => write!(formatter, "major"),
            FontScheme::None => write!(formatter, "none"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Font {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub strike: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub u: bool, // seems that Excel supports a bit more - double underline / account underline etc.
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub b: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub i: bool,
    pub sz: i32,
    #[serde(skip_serializing_if = "Color::is_none")]
    #[serde(default)]
    pub color: Color,
    pub name: String,
    // This is the font family fallback
    // 1 -> serif
    // 2 -> sans serif
    // 3 -> monospaced
    // ...
    pub family: i32,
    pub scheme: FontScheme,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            strike: false,
            u: false,
            b: false,
            i: false,
            sz: 12,
            color: Color::None,
            name: "Inter".to_string(),
            family: 2,
            scheme: FontScheme::Minor,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Fill {
    #[serde(skip_serializing_if = "Color::is_none")]
    #[serde(default)]
    pub color: Color,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum HorizontalAlignment {
    Center,
    CenterContinuous,
    Distributed,
    Fill,
    #[default]
    General,
    Justify,
    Left,
    Right,
}

// Note that alignment in "General" depends on type

impl HorizontalAlignment {
    fn is_default(&self) -> bool {
        self == &HorizontalAlignment::default()
    }
}

// FIXME: Is there a way to generate this automatically?
impl Display for HorizontalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HorizontalAlignment::Center => write!(formatter, "center"),
            HorizontalAlignment::CenterContinuous => write!(formatter, "centerContinuous"),
            HorizontalAlignment::Distributed => write!(formatter, "distributed"),
            HorizontalAlignment::Fill => write!(formatter, "fill"),
            HorizontalAlignment::General => write!(formatter, "general"),
            HorizontalAlignment::Justify => write!(formatter, "justify"),
            HorizontalAlignment::Left => write!(formatter, "left"),
            HorizontalAlignment::Right => write!(formatter, "right"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VerticalAlignment {
    #[default]
    Bottom,
    Center,
    Distributed,
    Justify,
    Top,
}

impl VerticalAlignment {
    fn is_default(&self) -> bool {
        self == &VerticalAlignment::default()
    }
}

impl Display for VerticalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VerticalAlignment::Bottom => write!(formatter, "bottom"),
            VerticalAlignment::Center => write!(formatter, "center"),
            VerticalAlignment::Distributed => write!(formatter, "distributed"),
            VerticalAlignment::Justify => write!(formatter, "justify"),
            VerticalAlignment::Top => write!(formatter, "top"),
        }
    }
}

// 1762
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Alignment {
    #[serde(default)]
    #[serde(skip_serializing_if = "HorizontalAlignment::is_default")]
    pub horizontal: HorizontalAlignment,
    #[serde(skip_serializing_if = "VerticalAlignment::is_default")]
    #[serde(default)]
    pub vertical: VerticalAlignment,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub wrap_text: bool,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CellStyleXfs {
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    pub apply_number_format: bool,
    pub apply_border: bool,
    pub apply_alignment: bool,
    pub apply_protection: bool,
    pub apply_font: bool,
    pub apply_fill: bool,
}

impl Default for CellStyleXfs {
    fn default() -> Self {
        CellStyleXfs {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
            apply_number_format: true,
            apply_border: true,
            apply_alignment: true,
            apply_protection: true,
            apply_font: true,
            apply_fill: true,
        }
    }
}

/// The formatting categories a named style includes — Excel's "Style Includes"
/// checkboxes, stored as the `apply*` attributes of the style's `cellStyleXfs`
/// record. Applying the style to a cell only stamps the included categories.
/// The default (like "Normal") includes everything; the built-in "Percent",
/// for example, includes only the number format.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(default)]
pub struct StyleIncludes {
    pub number_format: bool,
    pub font: bool,
    pub fill: bool,
    pub border: bool,
    pub alignment: bool,
    pub protection: bool,
}

impl Default for StyleIncludes {
    fn default() -> Self {
        StyleIncludes {
            number_format: true,
            font: true,
            fill: true,
            border: true,
            alignment: true,
            protection: true,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct CellXfs {
    pub xf_id: i32,
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    pub apply_number_format: bool,
    pub apply_border: bool,
    pub apply_alignment: bool,
    pub apply_protection: bool,
    pub apply_font: bool,
    pub apply_fill: bool,
    pub quote_prefix: bool,
    pub alignment: Option<Alignment>,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CellStyles {
    pub name: String,
    pub xf_id: i32,
    pub builtin_id: i32,
}

impl Default for CellStyles {
    fn default() -> Self {
        CellStyles {
            name: "normal".to_string(),
            xf_id: 0,
            builtin_id: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, PartialOrd, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    Thin,
    Medium,
    Thick,
    Double,
    Dotted,
    SlantDashDot,
    MediumDashed,
    MediumDashDotDot,
    MediumDashDot,
}

impl Display for BorderStyle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BorderStyle::Thin => write!(formatter, "thin"),
            BorderStyle::Thick => write!(formatter, "thick"),
            BorderStyle::SlantDashDot => write!(formatter, "slantdashdot"),
            BorderStyle::MediumDashed => write!(formatter, "mediumdashed"),
            BorderStyle::MediumDashDotDot => write!(formatter, "mediumdashdotdot"),
            BorderStyle::MediumDashDot => write!(formatter, "mediumdashdot"),
            BorderStyle::Medium => write!(formatter, "medium"),
            BorderStyle::Double => write!(formatter, "double"),
            BorderStyle::Dotted => write!(formatter, "dotted"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone)]
pub struct BorderItem {
    pub style: BorderStyle,
    #[serde(skip_serializing_if = "Color::is_none")]
    #[serde(default)]
    pub color: Color,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Border {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_up: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_down: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagonal: Option<BorderItem>,
}

/// Information need to show a sheet tab in the UI
/// The color is serialized only if it is not Color::None
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SheetProperties {
    pub name: String,
    pub state: String,
    pub sheet_id: u32,
    #[serde(skip_serializing_if = "Color::is_none")]
    #[serde(default)]
    pub color: Color,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Theme {
    pub name: String,
    pub dk1: String,
    pub lt1: String,
    pub dk2: String,
    pub lt2: String,
    pub accent1: String,
    pub accent2: String,
    pub accent3: String,
    pub accent4: String,
    pub accent5: String,
    pub accent6: String,
    pub hlink: String,
    pub fol_hlink: String,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "Office".to_string(),
            dk1: "#000000".to_string(),
            lt1: "#FFFFFF".to_string(),
            dk2: "#44546A".to_string(),
            lt2: "#E7E6E6".to_string(),
            accent1: "#4472C4".to_string(),
            accent2: "#ED7D31".to_string(),
            accent3: "#A5A5A5".to_string(),
            accent4: "#FFC000".to_string(),
            accent5: "#5B9BD5".to_string(),
            accent6: "#70AD47".to_string(),
            hlink: "#0563C1".to_string(),
            fol_hlink: "#954F72".to_string(),
        }
    }
}

impl Theme {
    /// Resolves a `theme="N"` attribute (and optional `tint`) to an `#RRGGBB` string.
    /// Applies the OOXML dk/lt swap for indices 0–3.
    pub fn resolve(&self, theme_index: i32, tint: f64) -> String {
        use crate::colors::hex_with_tint_to_rgb;
        let color = match theme_index {
            0 => &self.lt1,
            1 => &self.dk1,
            2 => &self.lt2,
            3 => &self.dk2,
            4 => &self.accent1,
            5 => &self.accent2,
            6 => &self.accent3,
            7 => &self.accent4,
            8 => &self.accent5,
            9 => &self.accent6,
            10 => &self.hlink,
            11 => &self.fol_hlink,
            _ => &self.dk1,
        };
        hex_with_tint_to_rgb(color, tint)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_valid_hex_color() {
        assert!(is_valid_hex_color("#000000"));
        assert!(is_valid_hex_color("#ffffff"));

        assert!(!is_valid_hex_color("000000"));
        assert!(!is_valid_hex_color("ffffff"));

        assert!(!is_valid_hex_color("#gggggg"));

        // Not obvious cases unrecognized as colors
        assert!(!is_valid_hex_color("#ffffff "));
        assert!(!is_valid_hex_color("#fff")); // CSS shorthand
        assert!(!is_valid_hex_color("#ffffff00")); // with alpha channel
    }

    /// The bare names are the `Ordinal` instantiation; purely a compile-time check.
    #[test]
    fn types_default_to_ordinal() {
        fn assert_default(w: Workbook<Ordinal>) -> Workbook {
            w
        }
        fn assert_range_default(r: RangeRef<Ordinal>) -> RangeRef {
            r
        }
        let _ = (assert_default, assert_range_default);
    }

    fn round_trip(s: &str) -> String {
        RangeRef::parse_a1(s).unwrap().to_a1()
    }

    #[test]
    fn test_range_ref_parse_a1() {
        assert_eq!(round_trip("A1"), "A1");
        assert_eq!(round_trip("A1:B2"), "A1:B2");
        assert_eq!(round_trip("D:D"), "D:D");
        assert_eq!(round_trip("D:F"), "D:F");
        assert_eq!(round_trip("5:7"), "5:7");
        // $ markers ignored, case-insensitive
        assert_eq!(round_trip("$a$1:$b$2"), "A1:B2");
        assert_eq!(round_trip("$d:$f"), "D:F");
        // corners are normalized so that lo <= hi
        assert_eq!(round_trip("B2:A1"), "A1:B2");
        assert_eq!(round_trip("F:D"), "D:F");
        assert_eq!(round_trip("7:5"), "5:7");
        // a degenerate rect renders without a colon
        assert_eq!(round_trip("A1:A1"), "A1");
        // a full-span bounded axis (the storage form) normalizes to unbounded
        assert_eq!(round_trip("D1:D1048576"), "D:D");
        assert_eq!(round_trip("A5:XFD7"), "5:7");
        assert_eq!(RangeRef::parse_a1("D1:D1048576"), RangeRef::parse_a1("D:D"));
        // the whole grid has no compact form, so it round trips bijectively
        assert_eq!(round_trip("A1:XFD1048576"), "A1:XFD1048576");
        assert_eq!(
            RangeRef::parse_a1("A1:XFD1048576"),
            Some(RangeRef {
                rows: None,
                cols: None
            })
        );
    }

    #[test]
    fn test_range_ref_to_a1_bounded() {
        let bounded = |s: &str| RangeRef::parse_a1(s).unwrap().to_a1_bounded();
        assert_eq!(bounded("D:D"), "D1:D1048576");
        assert_eq!(bounded("5:7"), "A5:XFD7");
        assert_eq!(bounded("A1:B2"), "A1:B2");
        assert_eq!(bounded("C3"), "C3");
    }

    #[test]
    fn test_range_ref_parse_a1_invalid() {
        assert_eq!(RangeRef::parse_a1(""), None);
        assert_eq!(RangeRef::parse_a1("not_a_range"), None);
        assert_eq!(RangeRef::parse_a1("!!!!"), None);
        assert_eq!(RangeRef::parse_a1("A1:"), None);
        // a bare column or row is not a range
        assert_eq!(RangeRef::parse_a1("D"), None);
        assert_eq!(RangeRef::parse_a1("5"), None);
        // mixed axis kinds
        assert_eq!(RangeRef::parse_a1("A1:B"), None);
        assert_eq!(RangeRef::parse_a1("D:5"), None);
        // out of bounds
        assert_eq!(RangeRef::parse_a1("A0"), None);
        assert_eq!(RangeRef::parse_a1("XFE1"), None);
        assert_eq!(RangeRef::parse_a1("A1048577"), None);
        assert_eq!(RangeRef::parse_a1("0:3"), None);
    }

    #[test]
    fn test_range_ref_sqref() {
        let ranges = RangeRef::parse_sqref(" a1:b2   $D:$D  garbage 5:7 ");
        assert_eq!(RangeRef::to_sqref(&ranges), "A1:B2 D:D 5:7");
        assert!(RangeRef::parse_sqref("").is_empty());
    }

    #[test]
    fn test_range_ref_resolve() {
        assert_eq!(RangeRef::cell(3, 2).resolve(), (3, 2, 3, 2));
        assert_eq!(
            RangeRef::parse_a1("D:F").unwrap().resolve(),
            (1, 4, LAST_ROW, 6)
        );
        assert_eq!(
            RangeRef::parse_a1("5:7").unwrap().resolve(),
            (5, 1, 7, LAST_COLUMN)
        );
    }

    #[test]
    fn test_range_ref_contains() {
        let rect = RangeRef::parse_a1("B2:C3").unwrap();
        assert!(rect.contains(2, 2));
        assert!(rect.contains(3, 3));
        assert!(!rect.contains(1, 2));
        assert!(!rect.contains(2, 4));

        let column = RangeRef::parse_a1("D:D").unwrap();
        assert!(column.contains(1, 4));
        assert!(column.contains(LAST_ROW, 4));
        assert!(!column.contains(1, 5));

        let row = RangeRef::parse_a1("5:7").unwrap();
        assert!(row.contains(5, 1));
        assert!(row.contains(7, LAST_COLUMN));
        assert!(!row.contains(8, 1));
    }
}
