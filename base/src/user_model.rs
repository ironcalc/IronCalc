#![deny(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    constants,
    expressions::utils::{is_valid_column_number, is_valid_row},
    model::Model,
    types::{Cell, Col, Row, Style},
};

#[derive(Clone, Serialize, Deserialize)]
struct RowData {
    row: Option<Row>,
    data: HashMap<i32, Cell>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ColumnData {
    column: Option<Col>,
    data: HashMap<i32, Cell>,
}

#[derive(Clone, Serialize, Deserialize)]
enum Diff {
    // Cell diffs
    SetCellValue {
        sheet: u32,
        row: i32,
        column: i32,
        new_value: String,
        old_value: Box<Option<Cell>>,
    },
    DeleteCell {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Option<Cell>>,
    },
    RemoveCell {
        sheet: u32,
        row: i32,
        column: i32,
        old_value: Box<Option<Cell>>,
        old_style: i32,
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
}

type DiffList = Vec<Diff>;

#[derive(Default)]
struct History {
    undo_stack: Vec<DiffList>,
    redo_stack: Vec<DiffList>,
}

impl History {
    fn push(&mut self, diff_list: DiffList) {
        self.undo_stack.push(diff_list);
        self.redo_stack = vec![];
    }

    fn undo(&mut self) -> Option<Vec<Diff>> {
        match self.undo_stack.pop() {
            Some(diff_list) => {
                self.redo_stack.push(diff_list.clone());
                Some(diff_list)
            }
            None => None,
        }
    }

    fn redo(&mut self) -> Option<Vec<Diff>> {
        match self.redo_stack.pop() {
            Some(diff_list) => {
                self.undo_stack.push(diff_list.clone());
                Some(diff_list)
            }
            None => None,
        }
    }
}

/// # UserModel
/// UserModel is a wrapper around Model with undo/redo history and _diffs_.
///
/// A diff in this context (or more correctly a _user diff_) is a change created by a user.
/// It is meant to be used by UI applications like Web IronCalc or TironCalc
pub struct UserModel {
    model: Model,
    history: History,
    send_queue: Vec<DiffList>,
    pause_evaluation: bool,
}

impl UserModel {
    /// Creates a user model from a model
    pub fn from_model(model: Model) -> UserModel {
        UserModel {
            model,
            history: History::default(),
            send_queue: vec![],
            pause_evaluation: false,
        }
    }

    /// Undoes last change if any
    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(diff_list) = self.history.undo() {
            self.apply_undo_diff_list(&diff_list)?;
        };
        Ok(())
    }

    /// Redoes the last undone change
    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(diff_list) = self.history.redo() {
            self.apply_diff_list(&diff_list)?;
        };
        Ok(())
    }

    /// Returns true if there are items to be undone
    pub fn can_undo(&self) -> bool {
        !self.history.undo_stack.is_empty()
    }

    /// Returns true if there are items to be redone
    pub fn can_redo(&self) -> bool {
        !self.history.redo_stack.is_empty()
    }

    /// Pauses or unpauses automatic evaluation
    pub fn set_pause_evaluation(&mut self, pause_evaluation: bool) {
        self.pause_evaluation = pause_evaluation;
    }

    /// Forces an evaluation of the model
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    /// Returns the list of pending diffs and removes them from the queue
    pub fn flush_send_queue(&mut self) -> String {
        // This can never fail :O:
        let q = serde_json::to_string(&self.send_queue).unwrap();
        self.send_queue = vec![];
        q
    }

    /// This are external diffs that need to be applied to the model
    pub fn apply_external_diffs(&mut self, diff_list_str: &str) -> Result<(), String> {
        if let Ok(diff_list) = serde_json::from_str::<DiffList>(diff_list_str) {
            self.apply_diff_list(&diff_list)?;
        };
        Ok(())
    }

    /// set user input
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(), String> {
        if !is_valid_column_number(column) {
            return Err("Invalid column".to_string());
        }
        if !is_valid_row(row) {
            return Err("Invalid row".to_string());
        }
        let old_value = self
            .model
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .cloned();
        self.model
            .set_user_input(sheet, row, column, value.to_string());

        self.evaluate_if_not_paused();

        let diff_list = vec![Diff::SetCellValue {
            sheet,
            row,
            column,
            new_value: value.to_string(),
            old_value: Box::new(old_value),
        }];
        self.history.push(diff_list);
        Ok(())
    }

    /// Returns the content
    #[inline]
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String, String> {
        self.model.get_cell_content(sheet, row, column)
    }

    /// Returns formatted value
    #[inline]
    pub fn get_formatted_cell_value(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<String, String> {
        self.model.get_formatted_cell_value(sheet, row, column)
    }

    /// Inserts a row
    pub fn insert_row(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        let diff_list = vec![Diff::InsertRow { sheet, row }];
        self.history.push(diff_list);
        self.model.insert_rows(sheet, row, 1)
    }

    /// Deletes a row
    pub fn delete_row(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        let mut row_data = None;
        let worksheet = self.model.workbook.worksheet(sheet)?;
        for rd in &worksheet.rows {
            if rd.r == row {
                row_data = Some(rd.clone());
                break;
            }
        }
        let data = worksheet.sheet_data.get(&row).unwrap().clone();
        let old_data = Box::new(RowData {
            row: row_data,
            data,
        });
        let diff_list = vec![Diff::DeleteRow {
            sheet,
            row,
            old_data,
        }];
        self.history.push(diff_list);
        self.model.delete_rows(sheet, row, 1)
    }

    /// Inserts a column
    pub fn insert_column(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        let diff_list = vec![Diff::InsertColumn { sheet, column }];
        self.history.push(diff_list);
        self.model.insert_columns(sheet, column, 1)
    }

    /// Deletes a column
    pub fn delete_column(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        let worksheet = self.model.workbook.worksheet(sheet)?;
        if !is_valid_column_number(column) {
            return Err(format!("Column number '{column}' is not valid."));
        }

        let mut column_data = None;
        for col in &worksheet.cols {
            let min = col.min;
            let max = col.max;
            if column >= min && column <= max {
                column_data = Some(Col {
                    min: column,
                    max: column,
                    width: col.width,
                    custom_width: col.custom_width,
                    style: col.style,
                });
                break;
            }
        }

        let mut data = HashMap::new();
        for (row, row_data) in &worksheet.sheet_data {
            if let Some(cell) = row_data.get(&column) {
                data.insert(*row, cell.clone());
            }
        }

        let diff_list = vec![Diff::DeleteColumn {
            sheet,
            column,
            old_data: Box::new(ColumnData {
                column: column_data,
                data,
            }),
        }];
        self.history.push(diff_list);
        self.model.delete_columns(sheet, column, 1)
    }

    /// Sets the width of a column
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), String> {
        let old_value = self.model.get_column_width(sheet, column)?;
        self.history.push(vec![Diff::SetColumnWidth {
            sheet,
            column,
            new_value: width,
            old_value,
        }]);
        self.model.set_column_width(sheet, column, width)
    }

    /// Sets the height of a row
    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<(), String> {
        let old_value = self.model.get_row_height(sheet, row)?;
        self.history.push(vec![Diff::SetRowHeight {
            sheet,
            row,
            new_value: height,
            old_value,
        }]);
        self.model.set_row_height(sheet, row, height)
    }

    /// Gets the height of a row
    #[inline]
    pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64, String> {
        self.model.get_row_height(sheet, row)
    }

    /// Gets the width of a column
    #[inline]
    pub fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64, String> {
        self.model.get_column_width(sheet, column)
    }

    /// Returns the number of frozen rows in the sheet
    ///
    /// See also:
    /// * [Model::get_frozen_rows_count()]
    #[inline]
    pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32, String> {
        self.model.get_frozen_rows_count(sheet)
    }

    /// Returns the number of frozen columns in the sheet
    ///
    /// See also:
    /// * [Model::get_frozen_columns_count()]
    #[inline]
    pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32, String> {
        self.model.get_frozen_columns_count(sheet)
    }

    /// Sets the number of frozen rows in sheet
    ///
    /// See also:
    /// * [Model::set_frozen_rows()]
    pub fn set_frozen_rows_count(&mut self, sheet: u32, frozen_rows: i32) -> Result<(), String> {
        let old_value = self.model.get_frozen_rows_count(sheet)?;
        self.history.push(vec![Diff::SetFrozenRowsCount {
            sheet,
            new_value: frozen_rows,
            old_value,
        }]);
        self.model.set_frozen_rows(sheet, frozen_rows)
    }

    /// Sets the number of frozen columns in sheet
    ///
    /// See also:
    /// * [Model::set_frozen_columns()]
    pub fn set_frozen_columns_count(
        &mut self,
        sheet: u32,
        frozen_columns: i32,
    ) -> Result<(), String> {
        let old_value = self.model.get_frozen_columns_count(sheet)?;
        self.history.push(vec![Diff::SetFrozenColumnsCount {
            sheet,
            new_value: frozen_columns,
            old_value,
        }]);
        self.model.set_frozen_columns(sheet, frozen_columns)
    }

    // **** Private methods ****** //

    fn evaluate_if_not_paused(&mut self) {
        if !self.pause_evaluation {
            self.model.evaluate();
        }
    }

    fn apply_undo_diff_list(&mut self, diff_list: &DiffList) -> Result<(), String> {
        let mut needs_evaluation = false;
        for diff in diff_list {
            match diff {
                Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: _,
                    old_value,
                } => {
                    needs_evaluation = true;
                    match *old_value.clone() {
                        Some(value) => {
                            self.model
                                .workbook
                                .worksheet_mut(*sheet)?
                                .update_cell(*row, *column, value);
                        }
                        None => {
                            self.model.delete_cell(*sheet, *row, *column)?;
                        }
                    }
                }
                Diff::SetColumnWidth {
                    sheet,
                    column,
                    new_value: _,
                    old_value,
                } => self.model.set_column_width(*sheet, *column, *old_value)?,
                Diff::SetRowHeight {
                    sheet,
                    row,
                    new_value: _,
                    old_value,
                } => self.model.set_row_height(*sheet, *row, *old_value)?,
                Diff::DeleteCell {
                    sheet,
                    row,
                    column,
                    old_value,
                } => {
                    needs_evaluation = true;
                    if let Some(value) = *old_value.clone() {
                        self.model
                            .workbook
                            .worksheet_mut(*sheet)?
                            .update_cell(*row, *column, value);
                    }
                }
                Diff::RemoveCell {
                    sheet,
                    row,
                    column,
                    old_value,
                    old_style: _,
                } => {
                    needs_evaluation = true;
                    if let Some(value) = *old_value.clone() {
                        self.model
                            .workbook
                            .worksheet_mut(*sheet)?
                            .update_cell(*row, *column, value);
                    }
                }
                Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value,
                    new_value: _,
                } => self
                    .model
                    .set_cell_style(*sheet, *row, *column, old_value)?,
                Diff::InsertRow { sheet, row } => {
                    self.model.delete_rows(*sheet, *row, 1)?;
                    needs_evaluation = true;
                }
                Diff::DeleteRow {
                    sheet,
                    row,
                    old_data,
                } => {
                    needs_evaluation = true;
                    self.model.insert_rows(*sheet, *row, 1)?;
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    if let Some(row_data) = old_data.row.clone() {
                        worksheet.rows.push(row_data);
                    }
                    worksheet.sheet_data.insert(*row, old_data.data.clone());
                }
                Diff::InsertColumn { sheet, column } => {
                    self.model.delete_columns(*sheet, *column, 1)?;
                    needs_evaluation = true;
                }
                Diff::DeleteColumn {
                    sheet,
                    column,
                    old_data,
                } => {
                    needs_evaluation = true;
                    // inserts an empty column
                    self.model.insert_columns(*sheet, *column, 1)?;
                    // puts all the data back
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    for (row, cell) in &old_data.data {
                        worksheet.update_cell(*row, *column, cell.clone());
                    }
                    // makes sure that the width and style is correct
                    if let Some(col) = &old_data.column {
                        let width = col.width * constants::COLUMN_WIDTH_FACTOR;
                        let style = col.style;
                        worksheet.set_column_width_and_style(*column, width, style)?;
                    }
                }
                Diff::SetFrozenRowsCount {
                    sheet,
                    new_value: _,
                    old_value,
                } => self.model.set_frozen_rows(*sheet, *old_value)?,
                Diff::SetFrozenColumnsCount {
                    sheet,
                    new_value: _,
                    old_value,
                } => self.model.set_frozen_columns(*sheet, *old_value)?,
            }
        }
        if needs_evaluation {
            self.evaluate_if_not_paused();
        }
        Ok(())
    }

    /// Applies diff list
    fn apply_diff_list(&mut self, diff_list: &DiffList) -> Result<(), String> {
        let mut needs_evaluation = false;
        for diff in diff_list {
            match diff {
                Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value,
                    old_value: _,
                } => {
                    needs_evaluation = true;
                    self.model
                        .set_user_input(*sheet, *row, *column, new_value.to_string());
                }
                Diff::SetColumnWidth {
                    sheet,
                    column,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_column_width(*sheet, *column, *new_value)?;
                }
                Diff::SetRowHeight {
                    sheet,
                    row,
                    new_value,
                    old_value: _,
                } => {
                    self.model.set_row_height(*sheet, *row, *new_value)?;
                }
                Diff::DeleteCell {
                    sheet,
                    row,
                    column,
                    old_value: _,
                } => {
                    self.model.delete_cell(*sheet, *row, *column)?;
                    needs_evaluation = true;
                }
                Diff::RemoveCell {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    old_style: _,
                } => {
                    self.model.set_cell_empty(*sheet, *row, *column)?;
                    needs_evaluation = true;
                }
                Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    new_value,
                } => self
                    .model
                    .set_cell_style(*sheet, *row, *column, new_value)?,
                Diff::InsertRow { sheet, row } => {
                    self.model.insert_rows(*sheet, *row, 1)?;
                    needs_evaluation = true;
                }
                Diff::DeleteRow {
                    sheet,
                    row,
                    old_data: _,
                } => {
                    self.model.delete_rows(*sheet, *row, 1)?;
                    needs_evaluation = true;
                }
                Diff::InsertColumn { sheet, column } => {
                    needs_evaluation = true;
                    self.model.insert_columns(*sheet, *column, 1)?;
                }
                Diff::DeleteColumn {
                    sheet,
                    column,
                    old_data: _,
                } => {
                    self.model.delete_columns(*sheet, *column, 1)?;
                    needs_evaluation = true;
                }
                Diff::SetFrozenRowsCount {
                    sheet,
                    new_value,
                    old_value: _,
                } => self.model.set_frozen_rows(*sheet, *new_value)?,
                Diff::SetFrozenColumnsCount {
                    sheet,
                    new_value,
                    old_value: _,
                } => self.model.set_frozen_columns(*sheet, *new_value)?,
            }
        }

        if needs_evaluation {
            self.evaluate_if_not_paused();
        }
        Ok(())
    }
}
