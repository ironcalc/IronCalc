use std::collections::HashMap;

use bitcode::{Decode, Encode};

use crate::types::{Cell, Col, Row, Style};

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
    SetCellStyle {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Style>,
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
    DeleteSheet {
        sheet: u32,
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
    SetShowGridLines {
        sheet: u32,
        old_value: bool,
        new_value: bool,
    }, // FIXME: we are missing SetViewDiffs
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

    pub fn clear(&mut self) {
        self.redo_stack = vec![];
        self.undo_stack = vec![];
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
