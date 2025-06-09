use std::collections::HashMap;

use bitcode::{Decode, Encode};
use serde::Serialize;

use crate::types::{Cell, Col, Row, SheetState, Style, Worksheet};

#[derive(Clone, Encode, Decode, Serialize)]
pub struct RowData {
    pub(crate) row: Option<Row>,
    pub(crate) data: HashMap<i32, Cell>,
}

#[derive(Clone, Encode, Decode, Serialize)]
pub struct ColumnData {
    pub(crate) column: Option<Col>,
    pub(crate) data: HashMap<i32, Cell>,
}

/// Represents the type of a diff operation
#[derive(Clone, Encode, Decode, Serialize)]
pub enum DiffType {
    /// An undo operation
    Undo,
    /// A redo operation
    Redo,
}

#[derive(Clone, Encode, Decode, Serialize)]
pub enum Diff {
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
    InsertRow {
        sheet: u32,
        row: i32,
    },
    DeleteRow {
        sheet: u32,
        row: i32,
        old_data: Box<RowData>,
    },
    InsertColumn {
        sheet: u32,
        column: i32,
    },
    DeleteColumn {
        sheet: u32,
        column: i32,
        old_data: Box<ColumnData>,
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

/// A collection of diffs that can be applied to a model.
/// This represents a single operation that can be undone or redone.
#[derive(Clone, Encode, Decode, Serialize)]
pub struct QueueDiffs {
    /// The type of operation this represents (Undo or Redo)
    pub r#type: DiffType,
    /// The list of individual diffs that make up this operation
    pub list: DiffList,
}
