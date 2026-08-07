//! Replicated operations.
//!
//! A [`Patch`] is an assignment `register ← value`. The register is addressed by a scope (workbook,
//! sheet, row, column, cell, defined name, ...) together with a *property discriminant*, and removal
//! is expressed as `None` rather than a dedicated operation.
//!
//! The property discriminant being part of the register address is what makes concurrent edits to
//! different properties of the same object survive: renaming a sheet and recolouring it are writes
//! to two different registers, so neither loses. Collapsing a property sub-enum into a single
//! register would silently change that.
//!
//! Column spans are the one register address that is not a single object. A sequential edit shatters
//! any wide span it partly overlaps eagerly — the wide register is removed and the narrower ones
//! written in the same commit — so locally spans never overlap. Only concurrency can make them, and
//! overlapping spans are then resolved per position by register write timestamp: LWW, newest
//! covering span wins. The resolution machinery is phase-5 work. Open-ended spans are spelled two
//! ways on purpose: `Option` axes in the generic [`RangeRef`](crate::types::RangeRef) world, where
//! [`Ordinal`](crate::types::Ordinal) has no sentinel to spare, and [`FractionalKey::NULL`]
//! brackets in this FractionalKey-native patch and storage layer.
//!
//! Patches must be **index-free**: they may not carry values whose meaning depends on a replica's
//! local tables. Style indices into `Styles.cell_xfs`, shared-string indices and shared-formula
//! indices are all assigned by local insertion order, so two replicas can mint the same index for
//! different content. Patches therefore carry resolved values ([`Style`], the formula text, ...) and
//! each replica interns them locally on apply. Rows, columns and sheets are addressed by
//! [`FractionalKey`] instead, which every replica agrees on by construction.
//!
//! Every `prev` field is undo data. It is `#[serde(skip)]`, so it is populated only on locally
//! generated patches and is absent on anything received from a peer.

use crate::cf_types::CfRule;
use crate::collab::fractional_index::FractionalKey;
use crate::collab::log::Timestamp;
use crate::collab::model::{StableCellAddress, StableRange};
use crate::collab::DynError;
use crate::expressions::token::Error;
use crate::types::{ArrayKind, Color, Comment, SheetState, Style, Theme};
use crate::user_model::history::Diff;
use serde::{Deserialize, Serialize};

pub type SheetId = FractionalKey;

#[derive(Serialize, Deserialize)]
pub enum Patch {
    // ---- Cells ----
    /// `value: None` clears the cell's contents. A range clear fans out to one patch per populated
    /// cell, so that every write stays a single-register assignment.
    SetCellValue {
        sheet: SheetId,
        at: StableCellAddress,
        value: Option<CellInput>,

        #[serde(skip)]
        prev: Box<Option<CellInput>>,
    },
    /// `value` is a [`CellInput::Array`], or `None` to clear the array. `prev` covers the whole
    /// range the array occupied, which is why this is not folded into [`Patch::SetCellValue`].
    SetArrayValue {
        sheet: SheetId,
        anchor: StableCellAddress,
        value: Option<CellInput>,

        #[serde(skip)]
        prev: Vec<Vec<Option<CellInput>>>,
    },
    /// `style: None` clears the cell's formatting.
    SetCellStyle {
        sheet: SheetId,
        at: StableCellAddress,
        style: Option<Box<Style>>,

        #[serde(skip)]
        prev: Box<Option<Style>>,
    },

    // ---- Rows ----
    InsertRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        #[serde(skip)]
        prev: Vec<RowSnapshot>,
    },
    /// Moved keys are regenerated relative to `dest`.
    MoveRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        dest: FractionalKey,
    },
    SetRowProperty {
        sheet: SheetId,
        row: FractionalKey,
        property: RowProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[serde(skip)]
        prev: Option<RowProperty>,
    },

    // ---- Columns ----
    InsertColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        #[serde(skip)]
        prev: Vec<ColumnSnapshot>,
    },
    MoveColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        dest: FractionalKey,
    },
    /// A property write over a column *span*, addressed by its corner keys — see
    /// [`Col`](crate::types::Col) for what a span covers. A [`FractionalKey::NULL`] corner is an
    /// open end: `(NULL, k)` and `(k, NULL)` are half-open, and `(NULL, NULL)` is every column,
    /// including ones no replica has materialized yet — the register whole-sheet styling writes to,
    /// which ordinal code spells as a `(1, LAST_COLUMN)` record and stable-land cannot, having no
    /// "last" key.
    SetColumnSpan {
        sheet: SheetId,
        span: (FractionalKey, FractionalKey),
        property: ColProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[serde(skip)]
        prev: Option<ColProperty>,
    },

    // ---- Sheets ----
    /// `content: None` creates a blank sheet. `Some(..)` covers both duplicating an existing sheet
    /// and undoing a [`Patch::DeleteSheet`] — in the latter case `key` is the deleted sheet's own
    /// key, so existing [`SheetId`] references resolve again.
    ///
    /// The content is captured on the authoring replica at commit time, never resolved at apply
    /// time. Resolving on apply would make the result depend on which concurrent edits to the source
    /// sheet a replica had already seen, and replicas would diverge.
    AddSheet {
        key: FractionalKey,
        name: String,
        content: Option<Box<SheetContent>>,
    },
    DeleteSheet {
        sheet: SheetId,

        #[serde(skip)]
        prev: Option<Box<SheetContent>>,
    },
    SetSheetProperty {
        sheet: SheetId,
        property: SheetProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[serde(skip)]
        prev: Option<SheetProperty>,
    },

    // ---- Workbook ----
    SetWorkbookProperty {
        property: WorkbookProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[serde(skip)]
        prev: Option<WorkbookProperty>,
    },

    // ---- Defined names (keyed by `(scope, name)`) ----
    /// `formula: None` deletes the name. A rename fans out to two patches — a delete of the old name
    /// and a write of the new one — so two peers renaming the same name concurrently end up with
    /// both new names present.
    SetDefinedName {
        scope: Option<SheetId>,
        name: String,
        formula: Option<String>,

        #[serde(skip)]
        prev: Option<String>,
    },

    // ---- Named styles (keyed by name) ----
    /// `definition: None` deletes the style.
    SetNamedStyle {
        name: String,
        definition: Option<Box<NamedStyle>>,

        #[serde(skip)]
        prev: Option<Box<NamedStyle>>,
    },

    // ---- Conditional formatting ----
    // A rule is identified by the `FractionalKey` minted when it was created, and its priority is
    // its position in the worksheet's conditional formatting index. Reordering is therefore a move,
    // not a property write, and the `u32` priority together with the swap operation it required both
    // disappear.
    AddConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,
        rule: Box<CfRule>,
        ranges: Vec<StableRange>,
    },
    DeleteConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,

        #[serde(skip)]
        prev: Option<Box<ConditionalFormatState>>,
    },
    /// Raising or lowering a rule's priority: the moved keys are regenerated relative to `dest`.
    MoveConditionalFormats {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        dest: FractionalKey,
    },
    SetConditionalFormat {
        sheet: SheetId,
        key: FractionalKey,
        property: CfProperty,

        /// Same discriminant as `property`, holding the value it replaced.
        #[serde(skip)]
        prev: Option<CfProperty>,
    },
}

/// A property of a single row. Each variant is a distinct register.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RowProperty {
    /// `None` deletes the row style.
    Style(Option<Box<Style>>),
    Height(f64),
    Hidden(bool),
}

/// A property of a single column. Each variant is a distinct register.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColProperty {
    /// `None` deletes the column style.
    Style(Option<Box<Style>>),
    Width(f64),
    Hidden(bool),
}

/// A property of a single worksheet. Each variant is a distinct register.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SheetProperty {
    Name(String),
    Color(Color),
    State(SheetState),
    ShowGridLines(bool),
    FrozenRows(i32),
    FrozenColumns(i32),
}

/// A workbook-global property. Each variant is a distinct register.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorkbookProperty {
    Theme(Box<Theme>),
    Locale(String),
    Timezone(String),
}

/// A property of a single conditional formatting rule. Each variant is a distinct register.
///
/// Priority is deliberately absent: it is the rule's position in the worksheet's conditional
/// formatting index, changed with [`Patch::MoveConditionalFormats`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CfProperty {
    Rule(Box<CfRule>),
    Ranges(Vec<StableRange>),
}

/// A named cell style, carried by value rather than as an `xf_id` index into
/// `Styles.cell_style_xfs`, which is assigned per replica. `builtin_id` is an OOXML constant and is
/// safe to replicate as-is.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamedStyle {
    pub style: Style,
    pub builtin_id: i32,
}

/// A conditional formatting rule, without its priority — priority is the rule's position in the
/// worksheet's conditional formatting index.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConditionalFormatState {
    pub rule: CfRule,
    pub ranges: Vec<StableRange>,
}

/// A complete worksheet payload, used to seed [`Patch::AddSheet`] and to restore a
/// [`Patch::DeleteSheet`]. The sheet's name is carried by `AddSheet` itself and so is absent here.
///
/// Every field is index-free — see the module documentation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SheetContent {
    pub state: SheetState,
    pub color: Color,
    pub show_grid_lines: bool,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    /// Ordered by [`FractionalKey`].
    pub rows: Vec<(FractionalKey, RowState)>,
    /// Column spans, addressed by corner keys; a [`FractionalKey::NULL`] corner is an open end, so
    /// `(NULL, NULL)` is the whole-sheet span.
    pub columns: Vec<((FractionalKey, FractionalKey), ColState)>,
    /// Cell contents and cell styles are kept apart because most cells carry no style of their own,
    /// and they are separate registers in any case.
    pub cell_values: Vec<(StableCellAddress, CellInput)>,
    pub cell_styles: Vec<(StableCellAddress, Style)>,
    pub merge_cells: Vec<StableRange>,
    pub comments: Vec<Comment>,
    /// Ordered by [`FractionalKey`], which is both each rule's identity and its priority.
    pub conditional_formatting: Vec<(FractionalKey, ConditionalFormatState)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RowState {
    pub height: f64,
    pub hidden: bool,
    pub style: Option<Box<Style>>,
    pub custom_height: bool,
    pub custom_format: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColState {
    pub width: f64,
    pub hidden: bool,
    pub style: Option<Box<Style>>,
    pub custom_width: bool,
}

/// Everything a [`Patch::DeleteRows`] removed, so that undo can put it back. Local-only undo data.
#[derive(Clone, Debug)]
pub struct RowSnapshot {
    pub key: FractionalKey,
    pub state: RowState,
    /// Keyed by column.
    pub cell_values: Vec<(FractionalKey, CellInput)>,
    /// Keyed by column.
    pub cell_styles: Vec<(FractionalKey, Style)>,
}

/// Everything a [`Patch::DeleteColumns`] removed, so that undo can put it back. Local-only undo
/// data.
#[derive(Clone, Debug)]
pub struct ColumnSnapshot {
    pub key: FractionalKey,
    pub state: ColState,
    /// Keyed by row.
    pub cell_values: Vec<(FractionalKey, CellInput)>,
    /// Keyed by row.
    pub cell_styles: Vec<(FractionalKey, Style)>,
}

/// The contents of a cell as authored, never as evaluated.
///
/// Literals are carried already parsed rather than as the text the user typed: parsing depends on
/// the workbook locale, which is itself a replicated register, so re-parsing on each replica could
/// resolve differently. Formulas are carried in the internal (English) form for the same reason.
///
/// Evaluated results — [`FormulaValue`](crate::types::FormulaValue), spill values and spill cells —
/// are derived state. They are recomputed locally and never replicated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CellInput {
    Number(f64),
    Boolean(bool),
    Text(String),
    Error(Error),
    Formula(String),
    /// The anchor of an array formula. The cells it spills into are derived, not stored.
    Array {
        formula: String,
        range: StableRange,
        kind: ArrayKind,
    },
}
