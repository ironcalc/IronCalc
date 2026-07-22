use crate::cf_types::CfRule;
use crate::collab::fractional_index::FractionalKey;
use crate::collab::model::{StableCellAddress, StableRange};
use crate::collab::workbook::CollaborativeWorkbook;
use crate::collab::DynError;
use crate::types::{Cell, Color, Style, Theme};
use crate::user_model::history::{ColumnData, Diff, RowData};
use serde::{Deserialize, Serialize};

pub type SheetId = FractionalKey;

#[derive(Serialize, Deserialize)]
pub enum Patch {
    SetCell {
        sheet: SheetId,
        at: StableCellAddress,
        input: String, //TODO: evaluate it to final value?

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Box<Option<Cell>>,
    },
    SetArrayValue {
        sheet: SheetId,
        anchor: StableCellAddress,
        range: StableRange,
        input: String, //TODO: evaluate it to final value?

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Vec<Vec<Option<Cell>>>,
    },
    SetCellStyle {
        sheet: SheetId,
        at: StableCellAddress,
        style: Option<Box<Style>>, //TODO: change style into something that can cover individual properties

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Box<Option<Style>>,
    },
    SetColumnStyle {
        sheet: SheetId,
        col: FractionalKey,
        style: Option<Box<Style>>,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Box<Option<Style>>,
    },
    SetRowStyle {
        sheet: SheetId,
        row: FractionalKey,
        style: Option<Box<Style>>,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Box<Option<Style>>,
    },
    InsertRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Vec<RowData>,
    },
    InsertColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
    },
    DeleteColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Vec<ColumnData>,
    },
    NewSheet {
        /// Position + identity of the new sheet in `sheet_order` (its alias is
        /// derived locally on apply).
        key: FractionalKey,
        name: String,
    },
    DuplicateSheet {
        source: FractionalKey,
        new_key: FractionalKey,
    },
    RenameSheet {
        sheet: SheetId,
        name: String,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: String,
    },
    SetSheetColor {
        sheet: SheetId,
        value: Color,
        prev: Color,
    },
    SetShowGridLines {
        sheet: SheetId,
        value: bool,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: bool,
    },
    SetFrozenRowsCount {
        sheet: SheetId,
        value: i32,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: i32,
    },
    SetFrozenColumnsCount {
        sheet: SheetId,
        value: i32,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: i32,
    },

    // ---- Workbook-global LWW registers ----
    SetTheme {
        value: Box<Theme>,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: Box<Theme>,
    },
    SetLocale {
        value: String,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: String,
    },
    SetTimezone {
        value: String,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: String,
    },

    // ---- Defined names (LWW-map keyed by `(scope, name)`) ----
    CreateDefinedName {
        scope: Option<FractionalKey>,
        name: String,
        formula: String,
    },
    DeleteDefinedName {
        scope: Option<FractionalKey>,
        name: String,

        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev: String,
    },
    UpdateDefinedName {
        scope: Option<FractionalKey>,
        name: String,
        new_name: String,
        new_scope: Option<FractionalKey>,
        new_formula: String,
        /// Previous value. Used only for undo/redo.
        #[serde(skip)]
        prev_formula: String,
    },

    // ---- Column / row moves (re-key to a new fractional position) ----
    MoveColumns {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        /// Destination anchor; moved keys are regenerated relative to it.
        dest: FractionalKey,
    },
    MoveRows {
        sheet: SheetId,
        keys: Vec<FractionalKey>,
        dest: FractionalKey,
    },

    // ---- Named styles (LWW-map keyed by name) ----
    CreateNamedStyle {
        name: String,
        xf_id: i32,
    },
    DeleteNamedStyle {
        name: String,
        prev_xf_id: i32,
    },
    UpdateNamedStyle {
        name: String,
        new_name: String,
        prev_xf_id: i32,
        new_xf_id: i32,
    },

    // ---- Conditional formatting ----
    // Each rule gets a stable `cf_id` (the `OpId` of its creating patch) in place
    // of the positional `index`, and a fractional `priority` in place of `u32`,
    // so ordering survives concurrent inserts and a "swap" is just a re-priority.
    AddConditionalFormatting {
        sheet: SheetId,
        ranges: Vec<StableRange>,
        rule: Box<CfRule>,
        priority: FractionalKey,
    },
    DeleteConditionalFormatting {
        sheet: SheetId,
        prev_ranges: Vec<StableRange>,
        prev_rule: Box<CfRule>,
        prev_priority: FractionalKey,
    },
    UpdateConditionalFormatting {
        sheet: SheetId,
        prev_ranges: Vec<StableRange>,
        prev_rule: Box<CfRule>,
        ranges: Vec<StableRange>,
        rule: Box<CfRule>,
    },
    /// Replaces `Diff::SwapConditionalFormattingPriority`: with fractional
    /// priorities a swap is two independent re-priority ops, so `diff_to_patch`
    /// emits one `SetConditionalFormattingPriority` per affected rule.
    SetConditionalFormattingPriority {
        sheet: SheetId,
        priority: FractionalKey,
        prev_priority: FractionalKey,
    },
}

impl CollaborativeWorkbook {
    /// Convert an undo/redo [`Diff`] into the collaborative patches that realise
    /// it. Returns a `Vec` because some diffs fan out (`UpdateDefinedName`
    /// rename, `SwapConditionalFormattingPriority`, `DuplicateSheet`); an
    /// undo-only artifact may yield `[]`.
    ///
    /// Takes `&mut self` to (a) resolve `i32`/`u32` positions to stable keys via
    /// the collaborative indices and (b) mint fresh [`OpId`]s from the session
    /// clock. `prev` values are read straight off the `Diff`, so no separate
    /// workbook lookup is needed for them.
    fn diff_to_patch(&mut self, diff: Diff) -> Vec<Patch> {
        match diff {
            Diff::SetCellValue { .. } => todo!(),
            Diff::SetArrayValue { .. } => todo!(),
            Diff::RangeClearContents { .. } => todo!(),
            Diff::RangeClearAll { .. } => todo!(),
            Diff::CellClearFormatting { .. } => todo!(),
            Diff::SetCellStyle { .. } => todo!(),
            Diff::SetColumnWidth { .. } => todo!(),
            Diff::SetColumnHidden { .. } => todo!(),
            Diff::SetRowHeight { .. } => todo!(),
            Diff::SetRowHidden { .. } => todo!(),
            Diff::SetColumnStyle { .. } => todo!(),
            Diff::SetRowStyle { .. } => todo!(),
            Diff::DeleteColumnStyle { .. } => todo!(),
            Diff::DeleteRowStyle { .. } => todo!(),
            Diff::InsertRows { .. } => todo!(),
            Diff::DeleteRows { .. } => todo!(),
            Diff::InsertColumns { .. } => todo!(),
            Diff::DeleteColumns { .. } => todo!(),
            Diff::DeleteSheet { .. } => todo!(),
            Diff::SetFrozenRowsCount { .. } => todo!(),
            Diff::SetFrozenColumnsCount { .. } => todo!(),
            Diff::NewSheet { .. } => todo!(),
            Diff::DuplicateSheet { .. } => todo!(),
            Diff::RenameSheet { .. } => todo!(),
            Diff::SetSheetColor { .. } => todo!(),
            Diff::SetSheetState { .. } => todo!(),
            Diff::SetShowGridLines { .. } => todo!(),
            Diff::SetTheme { .. } => todo!(),
            Diff::CreateDefinedName { .. } => todo!(),
            Diff::DeleteDefinedName { .. } => todo!(),
            Diff::UpdateDefinedName { .. } => todo!(),
            Diff::MoveColumns { .. } => todo!(),
            Diff::MoveRows { .. } => todo!(),
            Diff::SetLocale { .. } => todo!(),
            Diff::SetTimezone { .. } => todo!(),
            Diff::CreateNamedStyle { .. } => todo!(),
            Diff::DeleteNamedStyle { .. } => todo!(),
            Diff::UpdateNamedStyle { .. } => todo!(),
            Diff::AddConditionalFormatting { .. } => todo!(),
            Diff::DeleteConditionalFormatting { .. } => todo!(),
            Diff::UpdateConditionalFormatting { .. } => todo!(),
            Diff::SwapConditionalFormattingPriority { .. } => todo!(),
        }
    }

    /// Apply a patch, resolving it against this workbook's CRDT state:
    /// LWW registers keep the write iff `patch.id` beats the stored `OpId`;
    /// sequence ops insert/tombstone keys in the relevant `FractionalIndex`;
    /// map ops do per-entry LWW put/remove.
    fn apply_patch(&mut self, patch: Patch) -> Result<(), DynError> {
        match patch {
            Patch::SetCell { .. } => todo!(),
            Patch::SetArrayValue { .. } => todo!(),
            Patch::SetCellStyle { .. } => todo!(),
            Patch::SetColumnStyle { .. } => todo!(),
            Patch::SetRowStyle { .. } => todo!(),
            Patch::InsertRows { .. } => todo!(),
            Patch::DeleteRows { .. } => todo!(),
            Patch::InsertColumns { .. } => todo!(),
            Patch::DeleteColumns { .. } => todo!(),
            Patch::NewSheet { .. } => todo!(),
            Patch::DuplicateSheet { .. } => todo!(),
            Patch::RenameSheet { .. } => todo!(),
            Patch::SetSheetColor { .. } => todo!(),
            Patch::SetShowGridLines { .. } => todo!(),
            Patch::SetFrozenRowsCount { .. } => todo!(),
            Patch::SetFrozenColumnsCount { .. } => todo!(),
            Patch::SetTheme { .. } => todo!(),
            Patch::SetLocale { .. } => todo!(),
            Patch::SetTimezone { .. } => todo!(),
            Patch::CreateDefinedName { .. } => todo!(),
            Patch::DeleteDefinedName { .. } => todo!(),
            Patch::UpdateDefinedName { .. } => todo!(),
            Patch::MoveColumns { .. } => todo!(),
            Patch::MoveRows { .. } => todo!(),
            Patch::CreateNamedStyle { .. } => todo!(),
            Patch::DeleteNamedStyle { .. } => todo!(),
            Patch::UpdateNamedStyle { .. } => todo!(),
            Patch::AddConditionalFormatting { .. } => todo!(),
            Patch::DeleteConditionalFormatting { .. } => todo!(),
            Patch::UpdateConditionalFormatting { .. } => todo!(),
            Patch::SetConditionalFormattingPriority { .. } => todo!(),
        }
    }
}
