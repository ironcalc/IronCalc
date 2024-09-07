#![deny(missing_docs)]

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    constants,
    expressions::{
        types::Area,
        utils::{is_valid_column_number, is_valid_row},
    },
    model::Model,
    types::{
        Alignment, BorderItem, BorderStyle, CellType, Col, HorizontalAlignment, SheetProperties,
        Style, VerticalAlignment,
    },
    utils::is_valid_hex_color,
};

use crate::user_model::history::{
    ColumnData, Diff, DiffList, DiffType, History, QueueDiffs, RowData,
};

#[derive(Serialize, Deserialize)]
pub enum BorderType {
    All,
    Inner,
    Outer,
    Top,
    Right,
    Bottom,
    Left,
    CenterH,
    CenterV,
    None,
}

/// This is the struct for a border area
#[derive(Serialize, Deserialize)]
pub struct BorderArea {
    item: BorderItem,
    r#type: BorderType,
}

fn boolean(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("Invalid value for boolean: '{value}'.")),
    }
}

fn color(value: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    if !is_valid_hex_color(value) {
        return Err(format!("Invalid color: '{value}'."));
    }
    Ok(Some(value.to_owned()))
}

fn border(value: &str) -> Result<Option<BorderItem>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    let parts = value.split(',');
    let values = parts.collect::<Vec<&str>>();
    match values[..] {
        [border_style, color_str] => {
            let style = match border_style {
                "thin" => BorderStyle::Thin,
                "medium" => BorderStyle::Medium,
                "thick" => BorderStyle::Thick,
                "double" => BorderStyle::Double,
                "dotted" => BorderStyle::Dotted,
                "slantDashDot" => BorderStyle::SlantDashDot,
                "mediumDashed" => BorderStyle::MediumDashed,
                "mediumDashDotDot" => BorderStyle::MediumDashDotDot,
                "mediumDashDot" => BorderStyle::MediumDashDot,
                _ => {
                    return Err(format!("Invalid border style: '{border_style}'."));
                }
            };
            Ok(Some(BorderItem {
                style,
                color: color(color_str)?,
            }))
        }
        _ => Err(format!("Invalid border value: '{value}'.")),
    }
}

fn horizontal(value: &str) -> Result<HorizontalAlignment, String> {
    match value {
        "center" => Ok(HorizontalAlignment::Center),
        "centerContinuous" => Ok(HorizontalAlignment::CenterContinuous),
        "distributed" => Ok(HorizontalAlignment::Distributed),
        "fill" => Ok(HorizontalAlignment::Fill),
        "general" => Ok(HorizontalAlignment::General),
        "justify" => Ok(HorizontalAlignment::Justify),
        "left" => Ok(HorizontalAlignment::Left),
        "right" => Ok(HorizontalAlignment::Right),
        _ => Err(format!(
            "Invalid value for horizontal alignment: '{value}'."
        )),
    }
}

fn vertical(value: &str) -> Result<VerticalAlignment, String> {
    match value {
        "bottom" => Ok(VerticalAlignment::Bottom),
        "center" => Ok(VerticalAlignment::Center),
        "distributed" => Ok(VerticalAlignment::Distributed),
        "justify" => Ok(VerticalAlignment::Justify),
        "top" => Ok(VerticalAlignment::Top),
        _ => Err(format!("Invalid value for vertical alignment: '{value}'.")),
    }
}

/// # A wrapper around [`Model`] for a spreadsheet end user.
/// UserModel is a wrapper around Model with undo/redo history, _diffs_, automatic evaluation and view management.
///
/// A diff in this context (or more correctly a _user diff_) is a change created by a user.
///
/// Automatic evaluation means that actions like setting a value on a cell or deleting a column
/// will evaluate the model if needed.
///
/// It is meant to be used by UI applications like Web IronCalc or TironCalc.
///
///
/// # Examples
///
/// ```rust
/// # use ironcalc_base::UserModel;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut model = UserModel::new_empty("model", "en", "UTC")?;
/// model.set_user_input(0, 1, 1, "=1+1")?;
/// assert_eq!(model.get_formatted_cell_value(0, 1, 1)?, "2");
/// model.undo()?;
/// assert_eq!(model.get_formatted_cell_value(0, 1, 1)?, "");
/// model.redo()?;
/// assert_eq!(model.get_formatted_cell_value(0, 1, 1)?, "2");
/// # Ok(())
/// # }
/// ```
pub struct UserModel {
    pub(crate) model: Model,
    history: History,
    send_queue: Vec<QueueDiffs>,
    pause_evaluation: bool,
}

impl Debug for UserModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserModel").finish()
    }
}

impl UserModel {
    /// Creates a user model from an existing model
    pub fn from_model(model: Model) -> UserModel {
        UserModel {
            model,
            history: History::default(),
            send_queue: vec![],
            pause_evaluation: false,
        }
    }

    /// Creates a new UserModel.
    ///
    /// See also:
    /// * [Model::new_empty]
    pub fn new_empty(name: &str, locale_id: &str, timezone: &str) -> Result<UserModel, String> {
        let model = Model::new_empty(name, locale_id, timezone)?;
        Ok(UserModel {
            model,
            history: History::default(),
            send_queue: vec![],
            pause_evaluation: false,
        })
    }

    /// Creates a model from it's internal representation
    ///
    /// See also:
    /// * [Model::from_bytes]
    pub fn from_bytes(s: &[u8]) -> Result<UserModel, String> {
        let model = Model::from_bytes(s)?;
        Ok(UserModel {
            model,
            history: History::default(),
            send_queue: vec![],
            pause_evaluation: false,
        })
    }

    /// Returns the internal representation of a model
    ///
    /// See also:
    ///  * [Model::to_json_str]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.model.to_bytes()
    }

    /// Undoes last change if any, places the change in the redo list and evaluates the model if needed
    ///
    /// See also:
    /// * [UserModel::redo]
    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(diff_list) = self.history.undo() {
            self.apply_undo_diff_list(&diff_list)?;
            self.send_queue.push(QueueDiffs {
                r#type: DiffType::Undo,
                list: diff_list.clone(),
            });
        };
        Ok(())
    }

    /// Redoes the last undone change, places the change in the undo list and evaluates the model if needed
    ///
    /// See also:
    /// * [UserModel::redo]
    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(diff_list) = self.history.redo() {
            self.apply_diff_list(&diff_list)?;
            self.send_queue.push(QueueDiffs {
                r#type: DiffType::Redo,
                list: diff_list.clone(),
            });
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

    /// Pauses automatic evaluation.
    ///
    /// See also:
    /// * [UserModel::evaluate]
    /// * [UserModel::resume_evaluation]
    pub fn pause_evaluation(&mut self) {
        self.pause_evaluation = true;
    }

    /// Resumes automatic evaluation.
    ///
    /// See also:
    /// * [UserModel::evaluate]
    /// * [UserModel::pause_evaluation]
    pub fn resume_evaluation(&mut self) {
        self.pause_evaluation = false;
    }

    /// Forces an evaluation of the model
    ///
    /// See also:
    /// * [Model::evaluate]
    /// * [UserModel::pause_evaluation]
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    /// Returns the list of pending diffs and removes them from the queue
    ///
    /// This is used together with [apply_external_diffs](UserModel::apply_external_diffs) to keep two remote models
    /// in sync.
    ///
    /// See also:
    /// * [UserModel::apply_external_diffs]
    pub fn flush_send_queue(&mut self) -> Vec<u8> {
        // This can never fail :O:
        let q = bitcode::encode(&self.send_queue);
        self.send_queue = vec![];
        q
    }

    /// This are external diffs that need to be applied to the model
    ///
    /// This is used together with [flush_send_queue](UserModel::flush_send_queue) to keep two remote models in sync
    ///
    /// See also:
    /// * [UserModel::flush_send_queue]
    pub fn apply_external_diffs(&mut self, diff_list_str: &[u8]) -> Result<(), String> {
        if let Ok(queue_diffs_list) = bitcode::decode::<Vec<QueueDiffs>>(diff_list_str) {
            for queue_diff in queue_diffs_list {
                if matches!(queue_diff.r#type, DiffType::Redo) {
                    self.apply_diff_list(&queue_diff.list)?;
                } else {
                    self.apply_undo_diff_list(&queue_diff.list)?;
                }
            }
        } else {
            return Err("Error parsing diff list".to_string());
        }
        Ok(())
    }

    /// Set the input in a cell
    ///
    /// See also:
    /// * [Model::set_user_input]
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
            .set_user_input(sheet, row, column, value.to_string())?;

        self.evaluate_if_not_paused();

        let diff_list = vec![Diff::SetCellValue {
            sheet,
            row,
            column,
            new_value: value.to_string(),
            old_value: Box::new(old_value),
        }];
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Returns the content of a cell
    ///
    /// See also:
    /// * [Model::get_cell_content]
    #[inline]
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String, String> {
        self.model.get_cell_content(sheet, row, column)
    }

    /// Returns the formatted value of a cell
    ///
    /// See also:
    /// * [Model::get_formatted_cell_value]
    #[inline]
    pub fn get_formatted_cell_value(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<String, String> {
        self.model.get_formatted_cell_value(sheet, row, column)
    }

    /// Returns the type of the cell
    ///
    /// See also
    /// * [Model::get_cell_type]
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<CellType, String> {
        self.model.get_cell_type(sheet, row, column)
    }

    /// Adds new sheet
    ///
    /// See also:
    /// * [Model::new_sheet]
    pub fn new_sheet(&mut self) {
        let (name, index) = self.model.new_sheet();
        self.push_diff_list(vec![Diff::NewSheet { index, name }]);
    }

    /// Deletes sheet by index
    ///
    /// See also:
    /// * [Model::delete_sheet]
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), String> {
        self.push_diff_list(vec![Diff::DeleteSheet { sheet }]);
        // There is no coming back
        self.history.clear();
        let sheet_count = self.model.workbook.worksheets.len() as u32;
        // If we are deleting the last sheet we need to change the selected sheet
        if sheet == sheet_count - 1 && sheet_count > 1 {
            if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
                view.sheet = sheet_count - 2;
            };
        }

        self.model.delete_sheet(sheet)?;
        Ok(())
    }

    /// Renames a sheet by index
    ///
    /// See also:
    /// * [Model::rename_sheet_by_index]
    pub fn rename_sheet(&mut self, sheet: u32, new_name: &str) -> Result<(), String> {
        let old_value = self.model.workbook.worksheet(sheet)?.name.clone();
        self.model.rename_sheet_by_index(sheet, new_name)?;
        self.push_diff_list(vec![Diff::RenameSheet {
            index: sheet,
            old_value,
            new_value: new_name.to_string(),
        }]);
        Ok(())
    }

    /// Sets sheet color
    ///
    /// Note: an empty string will remove the color
    ///
    /// See also
    /// * [Model::set_sheet_color]
    /// * [UserModel::get_worksheets_properties]
    pub fn set_sheet_color(&mut self, sheet: u32, color: &str) -> Result<(), String> {
        let old_value = match &self.model.workbook.worksheet(sheet)?.color {
            Some(c) => c.clone(),
            None => "".to_string(),
        };
        self.model.set_sheet_color(sheet, color)?;
        self.push_diff_list(vec![Diff::SetSheetColor {
            index: sheet,
            old_value,
            new_value: color.to_string(),
        }]);
        Ok(())
    }

    /// Removes cells contents and style
    ///
    /// See also:
    /// * [Model::cell_clear_all]
    pub fn range_clear_all(&mut self, range: &Area) -> Result<(), String> {
        let sheet = range.sheet;
        let mut diff_list = Vec::new();
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                let old_value = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .cell(row, column)
                    .cloned();
                let old_style = self.model.get_style_for_cell(sheet, row, column)?;
                self.model.cell_clear_all(sheet, row, column)?;
                diff_list.push(Diff::CellClearAll {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    old_style: Box::new(old_style),
                });
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Deletes the content in cells, but keeps the style
    ///
    /// See also:
    /// * [Model::cell_clear_contents]
    pub fn range_clear_contents(&mut self, range: &Area) -> Result<(), String> {
        let sheet = range.sheet;
        let mut diff_list = Vec::new();
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                let old_value = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .cell(row, column)
                    .cloned();
                self.model.cell_clear_contents(sheet, row, column)?;
                diff_list.push(Diff::CellClearContents {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                });
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Inserts a row
    ///
    /// See also:
    /// * [Model::insert_rows]
    pub fn insert_row(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        let diff_list = vec![Diff::InsertRow { sheet, row }];
        self.push_diff_list(diff_list);
        self.model.insert_rows(sheet, row, 1)
    }

    /// Deletes a row
    ///
    /// See also:
    /// * [Model::delete_rows]
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
        self.push_diff_list(diff_list);
        self.model.delete_rows(sheet, row, 1)
    }

    /// Inserts a column
    ///
    /// See also:
    /// * [Model::insert_columns]
    pub fn insert_column(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        let diff_list = vec![Diff::InsertColumn { sheet, column }];
        self.push_diff_list(diff_list);
        self.model.insert_columns(sheet, column, 1)
    }

    /// Deletes a column
    ///
    /// See also:
    /// * [Model::delete_columns]
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
        self.push_diff_list(diff_list);
        self.model.delete_columns(sheet, column, 1)
    }

    /// Sets the width of a column
    ///
    /// See also:
    /// * [Model::set_column_width]
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), String> {
        let old_value = self.model.get_column_width(sheet, column)?;
        self.push_diff_list(vec![Diff::SetColumnWidth {
            sheet,
            column,
            new_value: width,
            old_value,
        }]);
        self.model.set_column_width(sheet, column, width)
    }

    /// Sets the height of a row
    ///
    /// See also:
    /// * [Model::set_row_height]
    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<(), String> {
        let old_value = self.model.get_row_height(sheet, row)?;
        self.push_diff_list(vec![Diff::SetRowHeight {
            sheet,
            row,
            new_value: height,
            old_value,
        }]);
        self.model.set_row_height(sheet, row, height)
    }

    /// Gets the height of a row
    ///
    /// See also:
    /// * [Model::get_row_height]
    #[inline]
    pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64, String> {
        self.model.get_row_height(sheet, row)
    }

    /// Gets the width of a column
    ///
    /// See also:
    /// * [Model::get_column_width]
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
        self.push_diff_list(vec![Diff::SetFrozenRowsCount {
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
        self.push_diff_list(vec![Diff::SetFrozenColumnsCount {
            sheet,
            new_value: frozen_columns,
            old_value,
        }]);
        self.model.set_frozen_columns(sheet, frozen_columns)
    }

    /// Paste `styles` in the selected area
    pub fn on_paste_styles(&mut self, styles: &[Vec<Style>]) -> Result<(), String> {
        let styles_heigh = styles.len() as i32;
        let styles_width = styles[0].len() as i32;
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Ok(());
        };
        let range = if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
            if let Some(view) = worksheet.views.get(&self.model.view_id) {
                view.range
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // If the pasted area is smaller than the selected area we increase it
        let [row_start, column_start, row_end, column_end] = range;
        let last_row = row_end.max(row_start + styles_heigh - 1);
        let last_column = column_end.max(column_start + styles_width - 1);

        let mut diff_list = Vec::new();
        for row in row_start..=last_row {
            for column in column_start..=last_column {
                let row_index = ((row - row_start) % styles_heigh) as usize;
                let column_index = ((column - column_start) % styles_width) as usize;
                let style = &styles[row_index][column_index];
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                self.model.set_cell_style(sheet, row, column, style)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(style.clone()),
                });
            }
        }
        self.push_diff_list(diff_list);

        // select the pasted range
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.range = [row_start, column_start, last_row, last_column];
            }
        }
        Ok(())
    }

    /// Sets the border
    pub fn set_area_with_border(
        &mut self,
        range: &Area,
        border_area: &BorderArea,
    ) -> Result<(), String> {
        // FIXME: We need to set the border also in neighbouring cells.
        let sheet = range.sheet;
        let mut diff_list = Vec::new();
        let last_row = range.row + range.height - 1;
        let last_column = range.column + range.width - 1;
        for row in range.row..=last_row {
            for column in range.column..=last_column {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                let mut style = old_value.clone();

                // First remove all existing borders
                style.border.top = None;
                style.border.right = None;
                style.border.bottom = None;
                style.border.left = None;

                match border_area.r#type {
                    BorderType::All => {
                        style.border.top = Some(border_area.item.clone());
                        style.border.right = Some(border_area.item.clone());
                        style.border.bottom = Some(border_area.item.clone());
                        style.border.left = Some(border_area.item.clone());
                    }
                    BorderType::Inner => {
                        if row != range.row {
                            style.border.top = Some(border_area.item.clone());
                        }
                        if row != last_row {
                            style.border.bottom = Some(border_area.item.clone());
                        }
                        if column != range.column {
                            style.border.left = Some(border_area.item.clone());
                        }
                        if column != last_column {
                            style.border.right = Some(border_area.item.clone());
                        }
                    }
                    BorderType::Outer => {
                        if row == range.row {
                            style.border.top = Some(border_area.item.clone());
                        }
                        if row == last_row {
                            style.border.bottom = Some(border_area.item.clone());
                        }
                        if column == range.column {
                            style.border.left = Some(border_area.item.clone());
                        }
                        if column == last_column {
                            style.border.right = Some(border_area.item.clone());
                        }
                    }
                    BorderType::Top => style.border.top = Some(border_area.item.clone()),
                    BorderType::Right => style.border.right = Some(border_area.item.clone()),
                    BorderType::Bottom => style.border.bottom = Some(border_area.item.clone()),
                    BorderType::Left => style.border.left = Some(border_area.item.clone()),
                    BorderType::CenterH => {
                        if row != range.row {
                            style.border.top = Some(border_area.item.clone());
                        }
                        if row != last_row {
                            style.border.bottom = Some(border_area.item.clone());
                        }
                    }
                    BorderType::CenterV => {
                        if column != range.column {
                            style.border.left = Some(border_area.item.clone());
                        }
                        if column != last_column {
                            style.border.right = Some(border_area.item.clone());
                        }
                    }
                    BorderType::None => {
                        // noop, we already removed all the borders
                    }
                }

                self.model.set_cell_style(sheet, row, column, &style)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(style),
                });
            }
        }

        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Updates the range with a cell style.
    /// See also:
    /// * [Model::set_cell_style]
    pub fn update_range_style(
        &mut self,
        range: &Area,
        style_path: &str,
        value: &str,
    ) -> Result<(), String> {
        let sheet = range.sheet;
        let mut diff_list = Vec::new();
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                let old_value = self.model.get_style_for_cell(sheet, row, column)?;
                let mut style = old_value.clone();
                match style_path {
                    "font.b" => {
                        style.font.b = boolean(value)?;
                    }
                    "font.i" => {
                        style.font.i = boolean(value)?;
                    }
                    "font.u" => {
                        style.font.u = boolean(value)?;
                    }
                    "font.strike" => {
                        style.font.strike = boolean(value)?;
                    }
                    "font.color" => {
                        style.font.color = color(value)?;
                    }
                    "fill.bg_color" => {
                        style.fill.bg_color = color(value)?;
                    }
                    "fill.fg_color" => {
                        style.fill.fg_color = color(value)?;
                    }
                    "num_fmt" => {
                        value.clone_into(&mut style.num_fmt);
                    }
                    "border.left" => {
                        style.border.left = border(value)?;
                    }
                    "border.right" => {
                        style.border.right = border(value)?;
                    }
                    "border.top" => {
                        style.border.top = border(value)?;
                    }
                    "border.bottom" => {
                        style.border.bottom = border(value)?;
                    }
                    "alignment" => {
                        if !value.is_empty() {
                            return Err(format!("Alignment must be empty, but found: '{value}'."));
                        }
                        style.alignment = None;
                    }
                    "alignment.horizontal" => match style.alignment {
                        Some(ref mut s) => s.horizontal = horizontal(value)?,
                        None => {
                            let alignment = Alignment {
                                horizontal: horizontal(value)?,
                                ..Default::default()
                            };
                            style.alignment = Some(alignment)
                        }
                    },
                    "alignment.vertical" => match style.alignment {
                        Some(ref mut s) => s.vertical = vertical(value)?,
                        None => {
                            let alignment = Alignment {
                                vertical: vertical(value)?,
                                ..Default::default()
                            };
                            style.alignment = Some(alignment)
                        }
                    },
                    "alignment.wrap_text" => match style.alignment {
                        Some(ref mut s) => s.wrap_text = boolean(value)?,
                        None => {
                            let alignment = Alignment {
                                wrap_text: boolean(value)?,
                                ..Default::default()
                            };
                            style.alignment = Some(alignment)
                        }
                    },
                    _ => {
                        return Err(format!("Invalid style path: '{style_path}'."));
                    }
                }
                self.model.set_cell_style(sheet, row, column, &style)?;
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_value),
                    new_value: Box::new(style),
                });
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Returns the style for a cell
    ///
    /// See also:
    /// * [Model::get_style_for_cell]
    #[inline]
    pub fn get_cell_style(&mut self, sheet: u32, row: i32, column: i32) -> Result<Style, String> {
        self.model.get_style_for_cell(sheet, row, column)
    }

    /// Fills the cells from `source_area` until `to_row`.
    /// This simulates the user clicking on the cell outline handle and dragging it downwards (or upwards)
    pub fn auto_fill_rows(&mut self, source_area: &Area, to_row: i32) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let sheet = source_area.sheet;
        let row1 = source_area.row;
        let column1 = source_area.column;
        let width1 = source_area.width;
        let height1 = source_area.height;

        // Check first all parameters are valid
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index: '{sheet}'"));
        }

        if !is_valid_column_number(column1) {
            return Err(format!("Invalid column: '{column1}'"));
        }
        if !is_valid_row(row1) {
            return Err(format!("Invalid row: '{row1}'"));
        }
        if !is_valid_column_number(column1 + width1 - 1) {
            return Err(format!("Invalid column: '{}'", column1 + width1 - 1));
        }
        if !is_valid_row(row1 + height1 - 1) {
            return Err(format!("Invalid row: '{}'", row1 + height1 - 1));
        }

        if !is_valid_row(to_row) {
            return Err(format!("Invalid row: '{to_row}'"));
        }

        // anchor_row is the first row that repeats in each case.
        let anchor_row;
        let sign;
        // this is the range of rows we are going to fill
        let row_range: Vec<i32>;

        if to_row >= row1 + height1 {
            // we go downwards, we start from `row1 + height1` to `to_row`,
            anchor_row = row1;
            sign = 1;
            row_range = (row1 + height1..to_row + 1).collect();
        } else if to_row < row1 {
            // we go upwards, starting from `row1 - `` all the way to `to_row`
            anchor_row = row1 + height1 - 1;
            sign = -1;
            row_range = (to_row..row1).rev().collect();
        } else {
            return Err("Invalid parameters for autofill".to_string());
        }

        for column in column1..column1 + width1 {
            let mut index = 0;
            for row_ref in &row_range {
                // Save value and style first
                let row = *row_ref;
                let old_value = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .cell(row, column)
                    .cloned();
                let old_style = self.model.get_style_for_cell(sheet, row, column)?;

                // compute the new value and set it
                let source_row = anchor_row + index;
                let target_value = self
                    .model
                    .extend_to(sheet, source_row, column, row, column)?;
                self.model
                    .set_user_input(sheet, row, column, target_value.to_string())?;

                // Compute the new style and set it
                let new_style = self.model.get_style_for_cell(sheet, source_row, column)?;
                self.model.set_cell_style(sheet, row, column, &new_style)?;

                // Add the diffs
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_style),
                    new_value: Box::new(new_style),
                });
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: target_value.to_string(),
                    old_value: Box::new(old_value),
                });

                index = (index + sign) % height1;
            }
        }
        self.push_diff_list(diff_list);
        self.evaluate();
        Ok(())
    }

    /// Fills the cells from `source_area` until `to_column`.
    /// This simulates the user clicking on the cell outline handle and dragging it to the right (or to the left)
    pub fn auto_fill_columns(&mut self, source_area: &Area, to_column: i32) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let sheet = source_area.sheet;
        let row1 = source_area.row;
        let column1 = source_area.column;
        let width1 = source_area.width;
        let height1 = source_area.height;

        // Check first all parameters are valid
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index: '{sheet}'"));
        }

        if !is_valid_column_number(column1) {
            return Err(format!("Invalid column: '{column1}'"));
        }
        if !is_valid_row(row1) {
            return Err(format!("Invalid row: '{row1}'"));
        }
        if !is_valid_column_number(column1 + width1 - 1) {
            return Err(format!("Invalid column: '{}'", column1 + width1 - 1));
        }
        if !is_valid_row(row1 + height1 - 1) {
            return Err(format!("Invalid row: '{}'", row1 + height1 - 1));
        }

        if !is_valid_row(to_column) {
            return Err(format!("Invalid row: '{to_column}'"));
        }

        // anchor_column is the first column that repeats in each case.
        let anchor_column;
        let sign;
        // this is the range of columns we are going to fill
        let column_range: Vec<i32>;

        if to_column >= column1 + width1 {
            // we go right, we start from `1 + width` to `to_column`,
            anchor_column = column1;
            sign = 1;
            column_range = (column1 + width1..to_column + 1).collect();
        } else if to_column < column1 {
            // we go left, starting from `column1 - `` all the way to `to_column`
            anchor_column = column1 + width1 - 1;
            sign = -1;
            column_range = (to_column..column1).rev().collect();
        } else {
            return Err("Invalid parameters for autofill".to_string());
        }

        for row in row1..row1 + height1 {
            let mut index = 0;
            for column_ref in &column_range {
                let column = *column_ref;
                // Save value and style first
                let old_value = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .cell(row, column)
                    .cloned();
                let old_style = self.model.get_style_for_cell(sheet, row, column)?;

                // compute the new value and set it
                let source_column = anchor_column + index;
                let target_value = self
                    .model
                    .extend_to(sheet, row, source_column, row, column)?;
                self.model
                    .set_user_input(sheet, row, column, target_value.to_string())?;

                // Compute the new style and set it
                let new_style = self.model.get_style_for_cell(sheet, row, source_column)?;
                self.model.set_cell_style(sheet, row, column, &new_style)?;

                // Add the diffs
                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row,
                    column,
                    old_value: Box::new(old_style),
                    new_value: Box::new(new_style),
                });
                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row,
                    column,
                    new_value: target_value.to_string(),
                    old_value: Box::new(old_value),
                });

                index = (index + sign) % width1;
            }
        }
        self.push_diff_list(diff_list);
        self.evaluate();
        Ok(())
    }

    /// Returns information about the sheets
    ///
    /// See also:
    /// * [Model::get_worksheets_properties]
    #[inline]
    pub fn get_worksheets_properties(&self) -> Vec<SheetProperties> {
        self.model.get_worksheets_properties()
    }

    /// Set the gid lines in the worksheet to visible (`true`) or hidden (`false`)
    pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> Result<(), String> {
        let old_value = self.model.workbook.worksheet(sheet)?.show_grid_lines;
        self.model.set_show_grid_lines(sheet, show_grid_lines)?;

        self.push_diff_list(vec![Diff::SetShowGridLines {
            sheet,
            new_value: show_grid_lines,
            old_value,
        }]);
        Ok(())
    }

    /// Returns true in the grid lines for
    pub fn get_show_grid_lines(&self, sheet: u32) -> Result<bool, String> {
        Ok(self.model.workbook.worksheet(sheet)?.show_grid_lines)
    }

    // **** Private methods ****** //

    fn push_diff_list(&mut self, diff_list: DiffList) {
        self.send_queue.push(QueueDiffs {
            r#type: DiffType::Redo,
            list: diff_list.clone(),
        });
        self.history.push(diff_list);
    }

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
                                .update_cell(*row, *column, value)?;
                        }
                        None => {
                            self.model.cell_clear_all(*sheet, *row, *column)?;
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
                Diff::CellClearContents {
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
                            .update_cell(*row, *column, value)?;
                    }
                }
                Diff::CellClearAll {
                    sheet,
                    row,
                    column,
                    old_value,
                    old_style,
                } => {
                    needs_evaluation = true;
                    if let Some(value) = *old_value.clone() {
                        self.model
                            .workbook
                            .worksheet_mut(*sheet)?
                            .update_cell(*row, *column, value)?;
                        self.model
                            .set_cell_style(*sheet, *row, *column, old_style)?;
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
                        worksheet.update_cell(*row, *column, cell.clone())?;
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
                Diff::DeleteSheet { sheet: _ } => {
                    // do nothing
                }
                Diff::NewSheet { index, name: _ } => {
                    self.model.delete_sheet(*index)?;
                }
                Diff::RenameSheet {
                    index,
                    old_value,
                    new_value: _,
                } => {
                    self.model.rename_sheet_by_index(*index, old_value)?;
                }
                Diff::SetSheetColor {
                    index,
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_sheet_color(*index, old_value)?;
                }
                Diff::SetShowGridLines {
                    sheet,
                    old_value,
                    new_value: _,
                } => {
                    self.model.set_show_grid_lines(*sheet, *old_value)?;
                }
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
                        .set_user_input(*sheet, *row, *column, new_value.to_string())?;
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
                Diff::CellClearContents {
                    sheet,
                    row,
                    column,
                    old_value: _,
                } => {
                    self.model.cell_clear_contents(*sheet, *row, *column)?;
                    needs_evaluation = true;
                }
                Diff::CellClearAll {
                    sheet,
                    row,
                    column,
                    old_value: _,
                    old_style: _,
                } => {
                    self.model.cell_clear_all(*sheet, *row, *column)?;
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
                Diff::DeleteSheet { sheet } => self.model.delete_sheet(*sheet)?,
                Diff::NewSheet { index, name } => {
                    self.model.insert_sheet(name, *index, None)?;
                }
                Diff::RenameSheet {
                    index,
                    old_value: _,
                    new_value,
                } => {
                    self.model.rename_sheet_by_index(*index, new_value)?;
                }
                Diff::SetSheetColor {
                    index,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_sheet_color(*index, new_value)?;
                }
                Diff::SetShowGridLines {
                    sheet,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_show_grid_lines(*sheet, *new_value)?;
                }
            }
        }

        if needs_evaluation {
            self.evaluate_if_not_paused();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        types::{HorizontalAlignment, VerticalAlignment},
        user_model::common::{horizontal, vertical},
    };

    #[test]
    fn test_vertical() {
        let all = vec![
            VerticalAlignment::Bottom,
            VerticalAlignment::Center,
            VerticalAlignment::Distributed,
            VerticalAlignment::Justify,
            VerticalAlignment::Top,
        ];
        for a in all {
            assert_eq!(vertical(&format!("{}", a)), Ok(a));
        }
    }

    #[test]
    fn test_horizontal() {
        let all = vec![
            HorizontalAlignment::Center,
            HorizontalAlignment::CenterContinuous,
            HorizontalAlignment::Distributed,
            HorizontalAlignment::Fill,
            HorizontalAlignment::General,
            HorizontalAlignment::Justify,
            HorizontalAlignment::Left,
            HorizontalAlignment::Right,
        ];
        for a in all {
            assert_eq!(horizontal(&format!("{}", a)), Ok(a));
        }
    }
}
