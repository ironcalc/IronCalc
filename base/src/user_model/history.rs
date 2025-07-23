#![allow(deprecated)]

use std::collections::HashMap;

use bitcode::{Decode, Encode};

use crate::types::{Cell, Col, Row, SheetState, Style, Worksheet};

#[derive(Clone, Encode, Decode)]
pub(crate) struct RowData {
    pub(crate) row: Option<Row>,
    pub(crate) data: HashMap<i32, Cell>,
}

#[derive(Clone, Encode, Decode)]
pub(crate) struct ColumnData {
    pub(crate) column: Option<Col>,
    pub(crate) data: HashMap<i32, Cell>,
}

#[derive(Clone, Encode, Decode)]
#[allow(deprecated)]
pub(crate) enum Diff {
    // Cell diffs
    SetCellValue {
        sheet: u32,
        row: i32,
        column: i32,
        new_value: String,
        old_value: Box<Option<Cell>>,
    },
    CellClearContents {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Option<Cell>>,
    },
    CellClearAll {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Option<Cell>>,
        old_style: Box<Style>,
    },
    CellClearFormatting {
        sheet: u32,
        row: i32,
        column: i32,
        old_style: Box<Option<Style>>,
    },
    SetCellStyle {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Option<Style>>,
        new_value: Box<Style>,
    },
    // Column and Row diffs
    SetColumnWidth {
        sheet: u32,
        column: i32,
        new_value: f64,
        old_value: f64,
    },
    SetRowHeight {
        sheet: u32,
        row: i32,
        new_value: f64,
        old_value: f64,
    },
    SetColumnStyle {
        sheet: u32,
        column: i32,
        old_value: Box<Option<Style>>,
        new_value: Box<Style>,
    },
    SetRowStyle {
        sheet: u32,
        row: i32,
        old_value: Box<Option<Style>>,
        new_value: Box<Style>,
    },
    DeleteColumnStyle {
        sheet: u32,
        column: i32,
        old_value: Box<Option<Style>>,
    },
    DeleteRowStyle {
        sheet: u32,
        row: i32,
        old_value: Box<Option<Style>>,
    },
    /// **DEPRECATED**: Use `InsertRows` with count=1 instead.
    ///
    /// This variant is kept for backward compatibility to handle old persisted diffs.
    /// New code should always use `InsertRows` even for single row insertions.
    #[deprecated(since = "0.5.0", note = "Use InsertRows with count=1 instead")]
    #[allow(dead_code)]
    #[allow(deprecated)]
    InsertRow {
        #[allow(deprecated)]
        sheet: u32,
        #[allow(deprecated)]
        row: i32,
    },
    /// **DEPRECATED**: Use `DeleteRows` with count=1 instead.
    ///
    /// This variant is kept for backward compatibility to handle old persisted diffs.
    /// New code should always use `DeleteRows` even for single row deletions.
    #[deprecated(since = "0.5.0", note = "Use DeleteRows with count=1 instead")]
    #[allow(dead_code)]
    #[allow(deprecated)]
    DeleteRow {
        #[allow(deprecated)]
        sheet: u32,
        #[allow(deprecated)]
        row: i32,
        #[allow(deprecated)]
        old_data: Box<RowData>,
    },
    /// **DEPRECATED**: Use `InsertColumns` with count=1 instead.
    ///
    /// This variant is kept for backward compatibility to handle old persisted diffs.
    /// New code should always use `InsertColumns` even for single column insertions.
    #[deprecated(since = "0.5.0", note = "Use InsertColumns with count=1 instead")]
    #[allow(dead_code)]
    #[allow(deprecated)]
    InsertColumn {
        #[allow(deprecated)]
        sheet: u32,
        #[allow(deprecated)]
        column: i32,
    },
    /// **DEPRECATED**: Use `DeleteColumns` with count=1 instead.
    ///
    /// This variant is kept for backward compatibility to handle old persisted diffs.
    /// New code should always use `DeleteColumns` even for single column deletions.
    #[deprecated(since = "0.5.0", note = "Use DeleteColumns with count=1 instead")]
    #[allow(dead_code)]
    #[allow(deprecated)]
    DeleteColumn {
        #[allow(deprecated)]
        sheet: u32,
        #[allow(deprecated)]
        column: i32,
        #[allow(deprecated)]
        old_data: Box<ColumnData>,
    },
    InsertRows {
        sheet: u32,
        row: i32,
        count: i32,
    },
    DeleteRows {
        sheet: u32,
        row: i32,
        count: i32,
        old_data: Vec<RowData>,
    },
    InsertColumns {
        sheet: u32,
        column: i32,
        count: i32,
    },
    DeleteColumns {
        sheet: u32,
        column: i32,
        count: i32,
        old_data: Vec<ColumnData>,
    },
    DeleteSheet {
        sheet: u32,
        old_data: Box<Worksheet>,
    },
    SetFrozenRowsCount {
        sheet: u32,
        new_value: i32,
        old_value: i32,
    },
    SetFrozenColumnsCount {
        sheet: u32,
        new_value: i32,
        old_value: i32,
    },
    NewSheet {
        index: u32,
        name: String,
    },
    RenameSheet {
        index: u32,
        old_value: String,
        new_value: String,
    },
    SetSheetColor {
        index: u32,
        old_value: String,
        new_value: String,
    },
    SetSheetState {
        index: u32,
        old_value: SheetState,
        new_value: SheetState,
    },
    SetShowGridLines {
        sheet: u32,
        old_value: bool,
        new_value: bool,
    },
    CreateDefinedName {
        name: String,
        scope: Option<u32>,
        value: String,
    },
    DeleteDefinedName {
        name: String,
        scope: Option<u32>,
        old_value: String,
    },
    UpdateDefinedName {
        name: String,
        scope: Option<u32>,
        old_formula: String,
        new_name: String,
        new_scope: Option<u32>,
        new_formula: String,
    },
    // FIXME: we are missing SetViewDiffs
}

pub(crate) type DiffList = Vec<Diff>;

#[derive(Default)]
pub(crate) struct History {
    pub(crate) undo_stack: Vec<DiffList>,
    pub(crate) redo_stack: Vec<DiffList>,
}

impl History {
    pub fn push(&mut self, diff_list: DiffList) {
        self.undo_stack.push(diff_list);
        self.redo_stack = vec![];
    }

    pub fn undo(&mut self) -> Option<Vec<Diff>> {
        match self.undo_stack.pop() {
            Some(diff_list) => {
                self.redo_stack.push(diff_list.clone());
                Some(diff_list)
            }
            None => None,
        }
    }

    pub fn redo(&mut self) -> Option<Vec<Diff>> {
        match self.redo_stack.pop() {
            Some(diff_list) => {
                self.undo_stack.push(diff_list.clone());
                Some(diff_list)
            }
            None => None,
        }
    }
}

#[derive(Clone, Encode, Decode)]
pub enum DiffType {
    Undo,
    Redo,
}

#[derive(Clone, Encode, Decode)]
pub struct QueueDiffs {
    pub r#type: DiffType,
    pub list: DiffList,
}
