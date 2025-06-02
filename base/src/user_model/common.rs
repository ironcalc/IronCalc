#![deny(missing_docs)]

use std::{collections::HashMap, fmt::Debug, io::Cursor};

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{self, LAST_COLUMN, LAST_ROW},
    expressions::{
        types::{Area, CellReferenceIndex},
        utils::{is_valid_column_number, is_valid_row},
    },
    model::Model,
    types::{
        Alignment, BorderItem, CellType, Col, HorizontalAlignment, SheetProperties, SheetState,
        Style, VerticalAlignment, WebUser,
    },
    utils::is_valid_hex_color,
};

use crate::user_model::history::{
    ColumnData, Diff, DiffList, DiffType, History, QueueDiffs, RowData,
};

use super::border_utils::is_max_border;
/// Data for the clipboard
pub type ClipboardData = HashMap<i32, HashMap<i32, ClipboardCell>>;

pub type ClipboardTuple = (i32, i32, i32, i32);

#[derive(Serialize, Deserialize)]
pub struct ClipboardCell {
    text: String,
    style: Style,
}

#[derive(Serialize, Deserialize)]
pub struct Clipboard {
    pub(crate) csv: String,
    pub(crate) data: ClipboardData,
    pub(crate) sheet: u32,
    pub(crate) range: (i32, i32, i32, i32),
}

#[derive(Serialize, Deserialize, PartialEq)]
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
    pub(crate) item: BorderItem,
    pub(crate) r#type: BorderType,
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

fn update_style(old_value: &Style, style_path: &str, value: &str) -> Result<Style, String> {
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
        "font.size_delta" => {
            // This is a special case, we need to add the value to the current size
            let size_delta: i32 = value
                .parse()
                .map_err(|_| format!("Invalid value for font size: '{value}'."))?;
            let new_size = style.font.sz + size_delta;
            if new_size < 1 {
                return Err(format!("Invalid value for font size: '{new_size}'."));
            }
            style.font.sz = new_size;
        }
        "fill.bg_color" => {
            style.fill.bg_color = color(value)?;
            style.fill.pattern_type = "solid".to_string();
        }
        "fill.fg_color" => {
            style.fill.fg_color = color(value)?;
            style.fill.pattern_type = "solid".to_string();
        }
        "num_fmt" => {
            value.clone_into(&mut style.num_fmt);
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
    Ok(style)
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

    /// Returns the internal model
    pub fn get_model(&self) -> &Model {
        &self.model
    }

    /// Returns the workbook name
    pub fn get_name(&self) -> String {
        self.model.workbook.name.clone()
    }

    /// Sets the name of a workbook
    pub fn set_name(&mut self, name: &str) {
        self.model.workbook.name = name.to_string();
    }

    /// Set users
    pub fn set_users(&mut self, users: &[WebUser]) {
        self.model.workbook.users = users.to_vec();
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

        let mut diff_list = vec![Diff::SetCellValue {
            sheet,
            row,
            column,
            new_value: value.to_string(),
            old_value: Box::new(old_value),
        }];
        let style = self.model.get_style_for_cell(sheet, row, column)?;

        let line_count = value.split('\n').count() as f64;
        let row_height = self.model.get_row_height(sheet, row)?;
        // This is in sync with the front-end auto fit row
        let font_size = style.font.sz as f64;
        let line_height = font_size * 1.5;
        let cell_height = (line_count - 1.0) * line_height + 8.0 + font_size;
        if cell_height > row_height {
            diff_list.push(Diff::SetRowHeight {
                sheet,
                row,
                new_value: cell_height,
                old_value: row_height,
            });
            self.model.set_row_height(sheet, row, cell_height)?;
        }

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
    pub fn new_sheet(&mut self) -> Result<(), String> {
        let (name, index) = self.model.new_sheet();
        self.set_selected_sheet(index)?;
        self.push_diff_list(vec![Diff::NewSheet { index, name }]);
        Ok(())
    }

    /// Deletes sheet by index
    ///
    /// See also:
    /// * [Model::delete_sheet]
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), String> {
        let worksheet = self.model.workbook.worksheet(sheet)?;

        self.push_diff_list(vec![Diff::DeleteSheet {
            sheet,
            old_data: Box::new(worksheet.clone()),
        }]);

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
        if old_value == new_name {
            return Ok(());
        }
        self.model.rename_sheet_by_index(sheet, new_name)?;
        self.push_diff_list(vec![Diff::RenameSheet {
            index: sheet,
            old_value,
            new_value: new_name.to_string(),
        }]);
        Ok(())
    }

    /// Hides sheet by index
    ///
    /// See also:
    /// * [Model::set_sheet_state]
    /// * [UserModel::unhide_sheet]
    pub fn hide_sheet(&mut self, sheet: u32) -> Result<(), String> {
        let sheet_count = self.model.workbook.worksheets.len() as u32;
        for index in 1..sheet_count {
            let sheet_index = (sheet + index) % sheet_count;
            if self.model.workbook.worksheet(sheet_index)?.state == SheetState::Visible {
                if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
                    view.sheet = sheet_index;
                };
                break;
            }
        }
        let old_value = self.model.workbook.worksheet(sheet)?.state.clone();
        self.push_diff_list(vec![Diff::SetSheetState {
            index: sheet,
            new_value: SheetState::Hidden,
            old_value,
        }]);
        self.model.set_sheet_state(sheet, SheetState::Hidden)?;
        Ok(())
    }

    /// Un hides sheet by index
    ///
    /// See also:
    /// * [Model::set_sheet_state]
    /// * [UserModel::hide_sheet]
    pub fn unhide_sheet(&mut self, sheet: u32) -> Result<(), String> {
        let old_value = self.model.workbook.worksheet(sheet)?.state.clone();
        self.push_diff_list(vec![Diff::SetSheetState {
            index: sheet,
            new_value: SheetState::Visible,
            old_value,
        }]);
        self.model.set_sheet_state(sheet, SheetState::Visible)?;
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
        // TODO: full rows/columns
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
        // TODO: full rows/columns
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

    fn clear_column_formatting(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let old_value = self.model.get_column_style(sheet, column)?;
        self.model.delete_column_style(sheet, column)?;
        diff_list.push(Diff::DeleteColumnStyle {
            sheet,
            column,
            old_value: Box::new(old_value),
        });

        let data_rows: Vec<i32> = self
            .model
            .workbook
            .worksheet(sheet)?
            .sheet_data
            .keys()
            .copied()
            .collect();
        let styled_rows = &self.model.workbook.worksheet(sheet)?.rows.clone();

        // Delete the formatting in all non empty cells
        for row in data_rows {
            if let Some(old_style) = self.model.get_cell_style_or_none(sheet, row, column)? {
                // We can always assume that style with style_index 0 exists and it is the default
                self.model
                    .workbook
                    .worksheet_mut(sheet)?
                    .set_cell_style(row, column, 0)?;
                diff_list.push(Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style: Box::new(Some(old_style)),
                });
            } else {
                let old_style = self.model.get_style_for_cell(sheet, row, column)?;
                if old_style != Style::default() {
                    self.model
                        .workbook
                        .worksheet_mut(sheet)?
                        .set_cell_style(row, column, 0)?;
                    diff_list.push(Diff::CellClearFormatting {
                        sheet,
                        row,
                        column,
                        old_style: Box::new(None),
                    });
                }
            }
        }
        // Delete the formatting in all cells with a row style
        for row in styled_rows {
            if let Some(old_style) = self.model.get_cell_style_or_none(sheet, row.r, column)? {
                // We can always assume that style with style_index 0 exists and it is the default
                self.model
                    .workbook
                    .worksheet_mut(sheet)?
                    .set_cell_style(row.r, column, 0)?;
                diff_list.push(Diff::CellClearFormatting {
                    sheet,
                    row: row.r,
                    column,
                    old_style: Box::new(Some(old_style)),
                });
            } else {
                let old_style = self.model.get_style_for_cell(sheet, row.r, column)?;
                if old_style != Style::default() {
                    self.model
                        .workbook
                        .worksheet_mut(sheet)?
                        .set_cell_style(row.r, column, 0)?;
                    diff_list.push(Diff::CellClearFormatting {
                        sheet,
                        row: row.r,
                        column,
                        old_style: Box::new(None),
                    });
                }
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    fn clear_row_formatting(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let old_value = self.model.get_row_style(sheet, row)?;
        self.model.delete_row_style(sheet, row)?;
        diff_list.push(Diff::DeleteRowStyle {
            sheet,
            row,
            old_value: Box::new(old_value),
        });

        // Delete the formatting in all non empty cells
        let columns: Vec<i32> = self
            .model
            .workbook
            .worksheet(sheet)?
            .sheet_data
            .get(&row)
            .map(|row_data| row_data.keys().copied().collect())
            .unwrap_or_default();
        for column in columns {
            if let Some(old_style) = self.model.get_cell_style_or_none(sheet, row, column)? {
                // We can always assume that style with style_index 0 exists and it is the default
                self.model
                    .workbook
                    .worksheet_mut(sheet)?
                    .set_cell_style(row, column, 0)?;
                diff_list.push(Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style: Box::new(Some(old_style)),
                });
            } else {
                let old_style = self.model.get_style_for_cell(sheet, row, column)?;
                if old_style != Style::default() {
                    self.model
                        .workbook
                        .worksheet_mut(sheet)?
                        .set_cell_style(row, column, 0)?;
                    diff_list.push(Diff::CellClearFormatting {
                        sheet,
                        row,
                        column,
                        old_style: Box::new(None),
                    });
                }
            }
        }
        self.push_diff_list(diff_list);

        Ok(())
    }

    /// Removes cells styles and formatting, but keeps the content
    ///
    /// See also:
    /// * [UserModel::range_clear_all]
    /// * [UserModel::range_clear_contents]
    pub fn range_clear_formatting(&mut self, range: &Area) -> Result<(), String> {
        let sheet = range.sheet;
        if range.row == 1 && range.height == LAST_ROW {
            for column in range.column..range.column + range.width {
                self.clear_column_formatting(sheet, column)?;
            }
            return Ok(());
        }
        if range.column == 1 && range.width == LAST_COLUMN {
            for row in range.row..range.row + range.height {
                self.clear_row_formatting(sheet, row)?;
            }
            return Ok(());
        }
        let mut diff_list = Vec::new();
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                if let Some(old_style) = self.model.get_cell_style_or_none(sheet, row, column)? {
                    // We can always assume that style with style_index 0 exists and it is the default
                    self.model
                        .workbook
                        .worksheet_mut(sheet)?
                        .set_cell_style(row, column, 0)?;
                    diff_list.push(Diff::CellClearFormatting {
                        sheet,
                        row,
                        column,
                        old_style: Box::new(Some(old_style)),
                    });
                } else {
                    let old_style = self.model.get_style_for_cell(sheet, row, column)?;
                    if old_style != Style::default() {
                        self.model
                            .workbook
                            .worksheet_mut(sheet)?
                            .set_cell_style(row, column, 0)?;
                        diff_list.push(Diff::CellClearFormatting {
                            sheet,
                            row,
                            column,
                            old_style: Box::new(None),
                        });
                    }
                }
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
        self.model.insert_rows(sheet, row, 1)?;
        self.evaluate_if_not_paused();
        Ok(())
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
        let data = match worksheet.sheet_data.get(&row) {
            Some(s) => s.clone(),
            None => return Err(format!("Row number '{row}' is not valid.")),
        };
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
        self.model.delete_rows(sheet, row, 1)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Inserts a column
    ///
    /// See also:
    /// * [Model::insert_columns]
    pub fn insert_column(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        let diff_list = vec![Diff::InsertColumn { sheet, column }];
        self.push_diff_list(diff_list);
        self.model.insert_columns(sheet, column, 1)?;
        self.evaluate_if_not_paused();
        Ok(())
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
        self.model.delete_columns(sheet, column, 1)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Sets the width of a group of columns in a single diff list
    ///
    /// See also:
    /// * [Model::set_column_width]
    pub fn set_columns_width(
        &mut self,
        sheet: u32,
        column_start: i32,
        column_end: i32,
        width: f64,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        for column in column_start..=column_end {
            let old_value = self.model.get_column_width(sheet, column)?;
            diff_list.push(Diff::SetColumnWidth {
                sheet,
                column,
                new_value: width,
                old_value,
            });
            self.model.set_column_width(sheet, column, width)?;
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Sets the height of a range of rows in a single diff list
    ///
    /// See also:
    /// * [Model::set_row_height]
    pub fn set_rows_height(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        height: f64,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        for row in row_start..=row_end {
            let old_value = self.model.get_row_height(sheet, row)?;
            diff_list.push(Diff::SetRowHeight {
                sheet,
                row,
                new_value: height,
                old_value,
            });
            self.model.set_row_height(sheet, row, height)?;
        }
        self.push_diff_list(diff_list);
        Ok(())
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
        let styles_height = styles.len() as i32;
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
        let last_row = row_end.max(row_start + styles_height - 1);
        let last_column = column_end.max(column_start + styles_width - 1);

        let mut diff_list = Vec::new();
        for row in row_start..=last_row {
            for column in column_start..=last_column {
                let row_index = ((row - row_start) % styles_height) as usize;
                let column_index = ((column - column_start) % styles_width) as usize;
                let style = &styles[row_index][column_index];
                let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;
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

    // Updates the style of a cell, adding the new style to the diff list
    fn update_single_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style_path: &str,
        value: &str,
        diff_list: &mut Vec<Diff>,
    ) -> Result<(), String> {
        // This is the value in the cell itself
        let old_value = self.model.get_cell_style_or_none(sheet, row, column)?;

        // This takes into account row or column styles. If none of those are present, it will return the default style
        let old_style = self.get_cell_style(sheet, row, column)?;
        let new_style = update_style(&old_style, style_path, value)?;
        self.model.set_cell_style(sheet, row, column, &new_style)?;
        diff_list.push(Diff::SetCellStyle {
            sheet,
            row,
            column,
            old_value: Box::new(old_value),
            new_value: Box::new(new_style),
        });
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
        if range.row == 1 && range.height == LAST_ROW {
            // Full columns
            let styled_rows = &self.model.workbook.worksheet(sheet)?.rows.clone();
            // We need all the rows in the column to update the style
            // NB: This is too much, this is all the rows that have values
            let data_rows: Vec<i32> = self
                .model
                .workbook
                .worksheet(sheet)?
                .sheet_data
                .keys()
                .copied()
                .collect();
            for column in range.column..range.column + range.width {
                // we set the style of the full column
                let old_style = self.model.get_column_style(sheet, column)?;
                let style = match old_style.as_ref() {
                    Some(s) => s,
                    None => &Style::default(),
                };
                let style = update_style(style, style_path, value)?;
                self.model.set_column_style(sheet, column, &style)?;
                diff_list.push(Diff::SetColumnStyle {
                    sheet,
                    column,
                    old_value: Box::new(old_style),
                    new_value: Box::new(style),
                });

                // We need to update the styles in all cells that have a row style
                for row_s in styled_rows.iter() {
                    let row = row_s.r;
                    self.update_single_cell_style(
                        sheet,
                        row,
                        column,
                        style_path,
                        value,
                        &mut diff_list,
                    )?;
                }

                // Update style in all cells that have different styles
                // FIXME: We need a better way to transverse of cells in a column
                for &row in &data_rows {
                    if let Some(data_row) =
                        self.model.workbook.worksheet(sheet)?.sheet_data.get(&row)
                    {
                        if data_row.get(&column).is_some() {
                            // If the cell has non empty content it will always have some style
                            self.update_single_cell_style(
                                sheet,
                                row,
                                column,
                                style_path,
                                value,
                                &mut diff_list,
                            )?;
                        }
                    }
                }
            }
        } else if range.column == 1 && range.width == LAST_COLUMN {
            // Full rows
            let styled_columns = &self.model.workbook.worksheet(sheet)?.cols.clone();
            for row in range.row..range.row + range.height {
                // Now update style in all cells that are not empty
                let columns: Vec<i32> = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .sheet_data
                    .get(&row)
                    .map(|row_data| row_data.keys().copied().collect())
                    .unwrap_or_default();
                for column in columns {
                    self.update_single_cell_style(
                        sheet,
                        row,
                        column,
                        style_path,
                        value,
                        &mut diff_list,
                    )?;
                }

                // We need to go through all the cells that have a column style and merge the styles
                for col in styled_columns.iter() {
                    for column in col.min..col.max + 1 {
                        self.update_single_cell_style(
                            sheet,
                            row,
                            column,
                            style_path,
                            value,
                            &mut diff_list,
                        )?;
                    }
                }

                // Finally update the style of the row
                let old_style = self.model.get_row_style(sheet, row)?;
                let style = match old_style.as_ref() {
                    Some(s) => s,
                    None => &Style::default(),
                };
                let style = update_style(style, style_path, value)?;
                self.model.set_row_style(sheet, row, &style)?;
                diff_list.push(Diff::SetRowStyle {
                    sheet,
                    row,
                    old_value: Box::new(old_style),
                    new_value: Box::new(style),
                });
            }
        } else {
            for row in range.row..range.row + range.height {
                for column in range.column..range.column + range.width {
                    self.update_single_cell_style(
                        sheet,
                        row,
                        column,
                        style_path,
                        value,
                        &mut diff_list,
                    )?;
                }
            }
        }
        self.push_diff_list(diff_list);
        Ok(())
    }

    /// Returns the style for a cell
    ///
    /// Cells share a border, so the left border of B1 is the right border of A1
    /// In the object structure the borders of the cells might be difference,
    /// We always pick the "heaviest" border.
    ///
    /// See also:
    /// * [Model::get_style_for_cell]
    pub fn get_cell_style(&self, sheet: u32, row: i32, column: i32) -> Result<Style, String> {
        let mut style = self.model.get_style_for_cell(sheet, row, column)?;

        // We need to check if the adjacent cells have a "heavier" border
        let border_top = if row > 1 {
            self.model
                .get_style_for_cell(sheet, row - 1, column)?
                .border
                .bottom
        } else {
            None
        };

        let border_right = if column < LAST_COLUMN {
            self.model
                .get_style_for_cell(sheet, row, column + 1)?
                .border
                .left
        } else {
            None
        };

        let border_bottom = if row < LAST_ROW {
            self.model
                .get_style_for_cell(sheet, row + 1, column)?
                .border
                .top
        } else {
            None
        };

        let border_left = if column > 1 {
            self.model
                .get_style_for_cell(sheet, row, column - 1)?
                .border
                .right
        } else {
            None
        };

        if is_max_border(style.border.top.as_ref(), border_top.as_ref()) {
            style.border.top = border_top;
        }

        if is_max_border(style.border.right.as_ref(), border_right.as_ref()) {
            style.border.right = border_right;
        }

        if is_max_border(style.border.bottom.as_ref(), border_bottom.as_ref()) {
            style.border.bottom = border_bottom;
        }

        if is_max_border(style.border.left.as_ref(), border_left.as_ref()) {
            style.border.left = border_left;
        }

        Ok(style)
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
                let old_style = self.model.get_cell_style_or_none(sheet, row, column)?;

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
        let first_row = source_area.row;
        let first_column = source_area.column;
        let last_column = first_column + source_area.width - 1;
        let last_row = first_row + source_area.height - 1;

        // Check first all parameters are valid
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index: '{sheet}'"));
        }

        if !is_valid_column_number(first_column) {
            return Err(format!("Invalid column: '{first_column}'"));
        }
        if !is_valid_row(first_row) {
            return Err(format!("Invalid row: '{first_row}'"));
        }
        if !is_valid_column_number(last_column) {
            return Err(format!("Invalid column: '{}'", last_column));
        }
        if !is_valid_row(last_row) {
            return Err(format!("Invalid row: '{}'", last_row));
        }

        if !is_valid_row(to_column) {
            return Err(format!("Invalid row: '{to_column}'"));
        }

        // anchor_column is the first column that repeats in each case.
        let anchor_column;
        let sign;
        // this is the range of columns we are going to fill
        let column_range: Vec<i32>;

        if to_column > last_column {
            // we go right, we start from `1 + width` to `to_column`,
            anchor_column = first_column;
            sign = 1;
            column_range = (last_column + 1..to_column + 1).collect();
        } else if to_column < first_column {
            // we go left, starting from `column1 - `` all the way to `to_column`
            anchor_column = last_column;
            sign = -1;
            column_range = (to_column..first_column).rev().collect();
        } else {
            return Err("Invalid parameters for autofill".to_string());
        }

        for row in first_row..=last_row {
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
                let old_style = self.model.get_cell_style_or_none(sheet, row, column)?;

                // compute the new value and set it
                let source_column = anchor_column + index;
                let target_value = self
                    .model
                    .extend_to(sheet, row, source_column, row, column)?;
                self.model
                    .set_user_input(sheet, row, column, target_value.to_string())?;

                let new_style = self.model.get_style_for_cell(sheet, row, source_column)?;
                // Compute the new style and set it

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

                index = (index + sign) % source_area.width;
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

    /// Returns a copy of the selected area
    pub fn copy_to_clipboard(&self) -> Result<Clipboard, String> {
        let selected_area = self.get_selected_view();
        let sheet = selected_area.sheet;
        let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(vec![]);

        let mut data = HashMap::new();
        let [row_start, column_start, row_end, column_end] = selected_area.range;
        let dimension = self.model.workbook.worksheet(sheet)?.dimension();
        let row_end = row_end.min(dimension.max_row);
        let column_end = column_end.min(dimension.max_column);
        for row in row_start..=row_end {
            let mut data_row = HashMap::new();
            let mut text_row = Vec::new();
            for column in column_start..=column_end {
                let text = self.get_formatted_cell_value(sheet, row, column)?;
                let content = self.get_cell_content(sheet, row, column)?;
                let style = self.model.get_style_for_cell(sheet, row, column)?;
                data_row.insert(
                    column,
                    ClipboardCell {
                        text: content,
                        style,
                    },
                );
                text_row.push(text);
            }
            wtr.write_record(text_row)
                .map_err(|e| format!("Error while processing csv: {}", e))?;
            data.insert(row, data_row);
        }

        let csv = String::from_utf8(
            wtr.into_inner()
                .map_err(|e| format!("Processing error: '{}'", e))?,
        )
        .map_err(|e| format!("Error converting from utf8: '{}'", e))?;

        Ok(Clipboard {
            csv,
            data,
            sheet,
            range: (row_start, column_start, row_end, column_end),
        })
    }

    /// Paste text that we copied
    pub fn paste_from_clipboard(
        &mut self,
        source_sheet: u32,
        source_range: ClipboardTuple,
        clipboard: &ClipboardData,
        is_cut: bool,
    ) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let view = self.get_selected_view();
        let (source_first_row, source_first_column, source_last_row, source_last_column) =
            source_range;
        let sheet = view.sheet;
        let [selected_row, selected_column, _, _] = view.range;
        let mut max_row = selected_row;
        let mut max_column = selected_column;
        let area = &Area {
            sheet,
            row: source_first_row,
            column: source_first_column,
            width: source_last_column - source_first_column + 1,
            height: source_last_row - source_first_row + 1,
        };
        for (source_row, data_row) in clipboard {
            let delta_row = source_row - source_first_row;
            let target_row = selected_row + delta_row;
            max_row = max_row.max(target_row);
            for (source_column, value) in data_row {
                let delta_column = source_column - source_first_column;
                let target_column = selected_column + delta_column;
                max_column = max_column.max(target_column);

                // We are copying the value in
                // (source_row, source_column) to (target_row , target_column)
                // References in formulas are displaced

                // remain in the copied area
                let source = &CellReferenceIndex {
                    sheet,
                    column: *source_column,
                    row: *source_row,
                };
                let target = &CellReferenceIndex {
                    sheet,
                    column: target_column,
                    row: target_row,
                };
                let new_value = if is_cut {
                    self.model
                        .move_cell_value_to_area(&value.text, source, target, area)?
                } else {
                    self.model
                        .extend_copied_value(&value.text, source, target)?
                };

                let old_value = self
                    .model
                    .workbook
                    .worksheet(sheet)?
                    .cell(target_row, target_column)
                    .cloned();

                let old_style =
                    self.model
                        .get_cell_style_or_none(sheet, target_row, target_column)?;

                self.model
                    .set_user_input(sheet, target_row, target_column, new_value.clone())?;
                self.model
                    .set_cell_style(sheet, target_row, target_column, &value.style)?;

                diff_list.push(Diff::SetCellValue {
                    sheet,
                    row: target_row,
                    column: target_column,
                    new_value,
                    old_value: Box::new(old_value),
                });

                diff_list.push(Diff::SetCellStyle {
                    sheet,
                    row: target_row,
                    column: target_column,
                    old_value: Box::new(old_style),
                    new_value: Box::new(value.style.clone()),
                });
            }
        }
        if is_cut {
            for row in source_first_row..=source_last_row {
                for column in source_first_column..=source_last_column {
                    let old_value = self
                        .model
                        .workbook
                        .worksheet(source_sheet)?
                        .cell(row, column)
                        .cloned();

                    diff_list.push(Diff::CellClearContents {
                        sheet: source_sheet,
                        row,
                        column,
                        old_value: Box::new(old_value),
                    });
                    self.model.cell_clear_contents(source_sheet, row, column)?;
                }
            }
        }
        self.push_diff_list(diff_list);
        // select the pasted area
        self.set_selected_range(selected_row, selected_column, max_row, max_column)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Paste a csv-string into the model
    pub fn paste_csv_string(&mut self, area: &Area, csv: &str) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let sheet = area.sheet;
        let mut row = area.row;
        let mut column = area.column;
        let mut csv_reader = Cursor::new(csv);
        csv_reader.set_position(0);
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_reader(csv_reader);
        for record in reader.records() {
            match record {
                Ok(r) => {
                    column = area.column;
                    for value in &r {
                        let old_value = self
                            .model
                            .workbook
                            .worksheet(sheet)?
                            .cell(row, column)
                            .cloned();
                        // let old_style = self.model.get_style_for_cell(sheet, row, column)?;
                        self.model
                            .set_user_input(sheet, row, column, value.to_string())?;

                        diff_list.push(Diff::SetCellValue {
                            sheet,
                            row,
                            column,
                            new_value: value.to_string(),
                            old_value: Box::new(old_value),
                        });
                        column += 1;
                    }
                }
                Err(_) => {
                    // skip
                    continue;
                }
            };
            row += 1;
        }
        self.push_diff_list(diff_list);
        // select the pasted area
        self.set_selected_range(area.row, area.column, row, column)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Returns the list of defined names
    pub fn get_defined_name_list(&self) -> Vec<(String, Option<u32>, String)> {
        self.model.workbook.get_defined_names_with_scope()
    }

    /// Delete an existing defined name
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let old_value = self.model.get_defined_name_formula(name, scope)?;
        let diff_list = vec![Diff::DeleteDefinedName {
            name: name.to_string(),
            scope,
            old_value,
        }];
        self.push_diff_list(diff_list);
        self.model.delete_defined_name(name, scope)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Create a new defined name
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.model.new_defined_name(name, scope, formula)?;
        let diff_list = vec![Diff::CreateDefinedName {
            name: name.to_string(),
            scope,
            value: formula.to_string(),
        }];
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Updates a defined name
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        let old_formula = self.model.get_defined_name_formula(name, scope)?;
        let diff_list = vec![Diff::UpdateDefinedName {
            name: name.to_string(),
            scope,
            old_formula: old_formula.to_string(),
            new_name: new_name.to_string(),
            new_scope,
            new_formula: new_formula.to_string(),
        }];
        self.push_diff_list(diff_list);
        self.model
            .update_defined_name(name, scope, new_name, new_scope, new_formula)?;
        self.evaluate_if_not_paused();
        Ok(())
    }

    // **** Private methods ****** //

    pub(crate) fn push_diff_list(&mut self, diff_list: DiffList) {
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
        for diff in diff_list.iter().rev() {
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
                } => {
                    if let Some(old_style) = old_value.as_ref() {
                        self.model
                            .set_cell_style(*sheet, *row, *column, old_style)?;
                    } else {
                        // If the cell did not have a style there was nothing on it
                        self.model.cell_clear_all(*sheet, *row, *column)?;
                    }
                }
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
                Diff::NewSheet { index, name: _ } => {
                    self.model.delete_sheet(*index)?;
                    if *index > 0 {
                        self.set_selected_sheet(*index - 1)?;
                    }
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
                Diff::CreateDefinedName {
                    name,
                    scope,
                    value: _,
                } => {
                    self.model.delete_defined_name(name, *scope)?;
                }
                Diff::DeleteDefinedName {
                    name,
                    scope,
                    old_value,
                } => {
                    self.model.new_defined_name(name, *scope, old_value)?;
                }
                Diff::UpdateDefinedName {
                    name,
                    scope,
                    old_formula,
                    new_name,
                    new_scope,
                    new_formula: _,
                } => {
                    self.model.update_defined_name(
                        new_name,
                        *new_scope,
                        name,
                        *scope,
                        old_formula,
                    )?;
                }
                Diff::SetSheetState {
                    index,
                    old_value,
                    new_value: _,
                } => self.model.set_sheet_state(*index, old_value.clone())?,
                Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style,
                } => {
                    if let Some(value) = old_style.as_ref() {
                        self.model.set_cell_style(*sheet, *row, *column, value)?;
                    } else {
                        self.model.cell_clear_all(*sheet, *row, *column)?;
                    }
                }
                Diff::DeleteSheet { sheet, old_data } => {
                    needs_evaluation = true;
                    let sheet_name = &old_data.name.clone();
                    let sheet_index = *sheet;
                    let sheet_id = old_data.sheet_id;
                    self.model
                        .insert_sheet(sheet_name, sheet_index, Some(sheet_id))?;
                    let worksheet = self.model.workbook.worksheet_mut(*sheet)?;
                    for (row, row_data) in &old_data.sheet_data {
                        for (column, cell) in row_data {
                            worksheet.update_cell(*row, *column, cell.clone())?;
                        }
                    }
                    worksheet.rows = old_data.rows.clone();
                    worksheet.cols = old_data.cols.clone();
                    worksheet.show_grid_lines = old_data.show_grid_lines;
                    worksheet.frozen_columns = old_data.frozen_columns;
                    worksheet.frozen_rows = old_data.frozen_rows;
                    worksheet.state = old_data.state.clone();
                    worksheet.color = old_data.color.clone();
                    worksheet.merge_cells = old_data.merge_cells.clone();
                    worksheet.shared_formulas = old_data.shared_formulas.clone();
                    self.model.reset_parsed_structures();

                    self.set_selected_sheet(sheet_index)?;
                }
                Diff::SetColumnStyle {
                    sheet,
                    column,
                    old_value,
                    new_value: _,
                } => match old_value.as_ref() {
                    Some(s) => self.model.set_column_style(*sheet, *column, s)?,
                    None => {
                        self.model.delete_column_style(*sheet, *column)?;
                    }
                },
                Diff::SetRowStyle {
                    sheet,
                    row,
                    old_value,
                    new_value: _,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_row_style(*sheet, *row, s)?;
                    } else {
                        self.model.delete_row_style(*sheet, *row)?;
                    }
                }
                Diff::DeleteColumnStyle {
                    sheet,
                    column,
                    old_value,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_column_style(*sheet, *column, s)?;
                    } else {
                        self.model.delete_column_style(*sheet, *column)?;
                    }
                }
                Diff::DeleteRowStyle {
                    sheet,
                    row,
                    old_value,
                } => {
                    if let Some(s) = old_value.as_ref() {
                        self.model.set_row_style(*sheet, *row, s)?;
                    } else {
                        self.model.delete_row_style(*sheet, *row)?;
                    }
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
                Diff::DeleteSheet { sheet, old_data: _ } => {
                    self.model.delete_sheet(*sheet)?;
                    if *sheet > 0 {
                        self.set_selected_sheet(*sheet - 1)?;
                    }
                }
                Diff::NewSheet { index, name } => {
                    self.model.insert_sheet(name, *index, None)?;
                    self.set_selected_sheet(*index)?;
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
                Diff::CreateDefinedName { name, scope, value } => {
                    self.model.new_defined_name(name, *scope, value)?
                }
                Diff::DeleteDefinedName {
                    name,
                    scope,
                    old_value: _,
                } => self.model.delete_defined_name(name, *scope)?,
                Diff::UpdateDefinedName {
                    name,
                    scope,
                    old_formula: _,
                    new_name,
                    new_scope,
                    new_formula,
                } => self.model.update_defined_name(
                    name,
                    *scope,
                    new_name,
                    *new_scope,
                    new_formula,
                )?,
                Diff::SetSheetState {
                    index,
                    old_value: _,
                    new_value,
                } => self.model.set_sheet_state(*index, new_value.clone())?,
                Diff::CellClearFormatting {
                    sheet,
                    row,
                    column,
                    old_style: _,
                } => {
                    self.model
                        .workbook
                        .worksheet_mut(*sheet)?
                        .set_cell_style(*row, *column, 0)?;
                }
                Diff::SetColumnStyle {
                    sheet,
                    column,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_column_style(*sheet, *column, new_value)?;
                }
                Diff::SetRowStyle {
                    sheet,
                    row,
                    old_value: _,
                    new_value,
                } => {
                    self.model.set_row_style(*sheet, *row, new_value)?;
                }
                Diff::DeleteColumnStyle {
                    sheet,
                    column,
                    old_value: _,
                } => {
                    self.model.delete_column_style(*sheet, *column)?;
                }
                Diff::DeleteRowStyle {
                    sheet,
                    row,
                    old_value: _,
                } => {
                    self.model.delete_row_style(*sheet, *row)?;
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
