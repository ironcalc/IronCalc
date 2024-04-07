#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::expressions::utils::{is_valid_column_number, is_valid_row};

use super::common::UserModel;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct SelectedView {
    pub sheet: u32,
    pub row: i32,
    pub column: i32,
    pub range: [i32; 4],
    pub top_row: i32,
    pub left_column: i32,
}

impl UserModel {
    /// Returns the selected sheet index
    pub fn get_selected_sheet(&self) -> u32 {
        if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        }
    }

    /// Returns the selected cell
    pub fn get_selected_cell(&self) -> (u32, i32, i32) {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        };
        if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
            if let Some(view) = worksheet.views.get(&self.model.view_id) {
                return (sheet, view.row, view.column);
            }
        }
        // return a safe default
        (0, 1, 1)
    }

    /// Returns selected view
    pub fn get_selected_view(&self) -> SelectedView {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        };
        if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
            if let Some(view) = worksheet.views.get(&self.model.view_id) {
                return SelectedView {
                    sheet,
                    row: view.row,
                    column: view.column,
                    range: view.range,
                    top_row: view.top_row,
                    left_column: view.left_column,
                };
            }
        }
        // return a safe default
        SelectedView {
            sheet: 0,
            row: 1,
            column: 1,
            range: [1, 1, 1, 1],
            top_row: 1,
            left_column: 1,
        }
    }

    /// Sets the the selected sheet
    pub fn set_selected_sheet(&mut self, sheet: u32) -> Result<(), String> {
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index {}", sheet));
        }
        if let Some(view) = self.model.workbook.views.get_mut(&0) {
            view.sheet = sheet;
        }
        Ok(())
    }

    /// Sets the selected cell for the current view. Note that this also sets the selected range
    pub fn set_selected_cell(&mut self, row: i32, column: i32) -> Result<(), String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        };
        if !is_valid_column_number(column) {
            return Err(format!("Invalid column: '{column}'"));
        }
        if !is_valid_row(row) {
            return Err(format!("Invalid row: '{row}'"));
        }
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index {}", sheet));
        }
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&0) {
                view.row = row;
                view.column = column;
                view.range = [row, column, row, column];
            }
        }
        Ok(())
    }

    /// Sets the selected range. Note that the selected cell must be in one of the corners.
    pub fn set_selected_range(
        &mut self,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Result<(), String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        };

        if !is_valid_column_number(start_column) {
            return Err(format!("Invalid column: '{start_column}'"));
        }
        if !is_valid_row(start_row) {
            return Err(format!("Invalid row: '{start_row}'"));
        }

        if !is_valid_column_number(end_column) {
            return Err(format!("Invalid column: '{end_column}'"));
        }
        if !is_valid_row(end_row) {
            return Err(format!("Invalid row: '{end_row}'"));
        }
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index {}", sheet));
        }
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&0) {
                let selected_row = view.row;
                let selected_column = view.column;
                // The selected cells must be on one of the corners of the selected range:
                if selected_row != start_row && selected_row != end_row {
                    return Err(format!(
                        "The selected cells is not in one of the corners. Row: '{}' and row range '({}, {})'",
                        selected_row, start_row, end_row
                    ));
                }
                if selected_column != start_column && selected_column != end_column {
                    return Err(format!(
                        "The selected cells is not in one of the corners. Column '{}' and column range '({}, {})'",
                        selected_column, start_column, end_column
                    ));
                }
                view.range = [start_row, start_column, end_row, end_column];
            }
        }
        Ok(())
    }

    /// The selected range is expanded with the keyboard
    pub fn on_expand_selected_range(&mut self, key: &str) -> Result<(), String> {
        let (sheet, window_width, window_height) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (
                    view.sheet,
                    view.window_width as f64,
                    view.window_height as f64,
                )
            } else {
                return Ok(());
            };
        let (selected_row, selected_column, range, top_row, left_column) =
            if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
                if let Some(view) = worksheet.views.get(&self.model.view_id) {
                    (
                        view.row,
                        view.column,
                        view.range,
                        view.top_row,
                        view.left_column,
                    )
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            };
        let [row_start, column_start, row_end, column_end] = range;

        match key {
            "ArrowRight" => {
                if selected_column > column_start {
                    let new_column = column_start + 1;
                    if !(is_valid_column_number(new_column)) {
                        return Ok(());
                    }
                    self.set_selected_range(row_start, new_column, row_end, column_end)?;
                } else {
                    let new_column = column_end + 1;
                    if !is_valid_column_number(new_column) {
                        return Ok(());
                    }
                    // if the column is not fully visible we 'scroll' right until it is
                    let mut width = 0.0;
                    let mut c = left_column;
                    while c <= new_column {
                        width += self.model.get_column_width(sheet, c)?;
                        c += 1;
                    }
                    if width > window_width {
                        self.set_top_left_visible_cell(top_row, left_column + 1)?;
                    }
                    self.set_selected_range(row_start, column_start, row_end, column_end + 1)?;
                }
            }
            "ArrowLeft" => {
                if selected_column < column_end {
                    let new_column = column_end - 1;
                    if !is_valid_column_number(new_column) {
                        return Ok(());
                    }
                    if new_column < left_column {
                        self.set_top_left_visible_cell(top_row, new_column)?;
                    }
                    self.set_selected_range(row_start, column_start, row_end, new_column)?;
                } else {
                    let new_column = column_start - 1;
                    if !is_valid_column_number(new_column) {
                        return Ok(());
                    }
                    if new_column < left_column {
                        self.set_top_left_visible_cell(top_row, new_column)?;
                    }
                    self.set_selected_range(row_start, new_column, row_end, column_end)?;
                }
            }
            "ArrowUp" => {
                if selected_row < row_end {
                    let new_row = row_end - 1;
                    if !is_valid_row(new_row) {
                        return Ok(());
                    }
                    self.set_selected_range(row_start, column_start, new_row, column_end)?;
                } else {
                    let new_row = row_start - 1;
                    if !is_valid_row(new_row) {
                        return Ok(());
                    }
                    if new_row < top_row {
                        self.set_top_left_visible_cell(new_row, left_column)?;
                    }
                    self.set_selected_range(new_row, column_start, row_end, column_end)?;
                }
            }
            "ArrowDown" => {
                if selected_row > row_start {
                    let new_row = row_start + 1;
                    if !is_valid_row(new_row) {
                        return Ok(());
                    }
                    self.set_selected_range(new_row, column_start, row_end, column_end)?;
                } else {
                    let new_row = row_end + 1;
                    if !is_valid_row(new_row) {
                        return Ok(());
                    }
                    let mut height = 0.0;
                    let mut r = top_row;
                    while r <= new_row + 1 {
                        height += self.model.get_row_height(sheet, r)?;
                        r += 1;
                    }
                    if height >= window_height {
                        self.set_top_left_visible_cell(top_row + 1, left_column)?;
                    }
                    self.set_selected_range(row_start, column_start, new_row, column_end)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Sets the value of the first visible cell
    pub fn set_top_left_visible_cell(
        &mut self,
        top_row: i32,
        left_column: i32,
    ) -> Result<(), String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            0
        };

        if !is_valid_column_number(left_column) {
            return Err(format!("Invalid column: '{left_column}'"));
        }
        if !is_valid_row(top_row) {
            return Err(format!("Invalid row: '{top_row}'"));
        }
        if self.model.workbook.worksheet(sheet).is_err() {
            return Err(format!("Invalid worksheet index {}", sheet));
        }
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&0) {
                view.top_row = top_row;
                view.left_column = left_column;
            }
        }
        Ok(())
    }

    /// Sets the width of the window
    pub fn set_window_width(&mut self, window_width: f64) {
        if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
            view.window_width = window_width as i64;
        };
    }

    /// Gets the width of the window
    pub fn get_window_width(&mut self) -> Result<i64, String> {
        if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
            return Ok(view.window_width);
        };
        Err("View not found".to_string())
    }

    /// Sets the height of the window
    pub fn set_window_height(&mut self, window_height: f64) {
        if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
            view.window_height = window_height as i64;
        };
    }

    /// Gets the height of the window
    pub fn get_window_height(&mut self) -> Result<i64, String> {
        if let Some(view) = self.model.workbook.views.get_mut(&self.model.view_id) {
            return Ok(view.window_height);
        };
        Err("View not found".to_string())
    }

    /// User presses right arrow
    pub fn on_arrow_right(&mut self) -> Result<(), String> {
        let (sheet, window_width) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (view.sheet, view.window_width)
            } else {
                return Err("View not found".to_string());
            };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let new_column = view.column + 1;
        if !is_valid_column_number(new_column) {
            return Ok(());
        }
        // if the column is not fully visible we 'scroll' right until it is
        let mut width = 0.0;
        let mut column = view.left_column;
        while column <= new_column {
            width += self.model.get_column_width(sheet, column)?;
            column += 1;
        }
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.column = new_column;
                view.range = [view.row, new_column, view.row, new_column];
                if width > window_width as f64 {
                    view.left_column += 1;
                }
            }
        }
        Ok(())
    }

    /// User presses left arrow
    pub fn on_arrow_left(&mut self) -> Result<(), String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Err("View not found".to_string());
        };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let new_column = view.column - 1;
        if !is_valid_column_number(new_column) {
            return Ok(());
        }
        // if the column is not fully visible we 'scroll' right until it is
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.column = new_column;
                view.range = [view.row, new_column, view.row, new_column];
                if new_column < view.left_column {
                    view.left_column = new_column;
                }
            }
        }
        Ok(())
    }

    /// User presses up arrow key
    pub fn on_arrow_up(&mut self) -> Result<(), String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Err("View not found".to_string());
        };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let new_row = view.row - 1;
        if !is_valid_row(new_row) {
            return Ok(());
        }
        // if the column is not fully visible we 'scroll' right until it is
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.row = new_row;
                view.range = [new_row, view.column, new_row, view.column];
                if new_row < view.top_row {
                    view.top_row = new_row;
                }
            }
        }
        Ok(())
    }

    /// User presses down arrow key
    pub fn on_arrow_down(&mut self) -> Result<(), String> {
        let (sheet, window_height) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (view.sheet, view.window_height)
            } else {
                return Err("View not found".to_string());
            };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let new_row = view.row + 1;
        if !is_valid_row(new_row) {
            return Ok(());
        }
        // if the row is not fully visible we 'scroll' down until it is
        let mut height = 0.0;
        let mut row = view.top_row;
        while row <= new_row + 1 {
            height += self.model.get_row_height(sheet, row)?;
            row += 1;
        }
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.row = new_row;
                view.range = [new_row, view.column, new_row, view.column];
                if height > window_height as f64 {
                    view.top_row += 1;
                }
            }
        }
        Ok(())
    }

    // TODO: This function should be memoized
    /// Returns the x-coordinate of the cell in the top left corner
    pub fn get_scroll_x(&self) -> Result<f64, String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Err("View not found".to_string());
        };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let mut scroll_x = 0.0;
        for column in 1..view.left_column {
            scroll_x += self.model.get_column_width(sheet, column)?;
        }
        Ok(scroll_x)
    }

    // TODO: This function should be memoized
    /// Returns the y-coordinate of the cell in the top left corner
    pub fn get_scroll_y(&self) -> Result<f64, String> {
        let sheet = if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
            view.sheet
        } else {
            return Err("View not found".to_string());
        };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let mut scroll_y = 0.0;
        for row in 1..view.top_row {
            scroll_y += self.model.get_row_height(sheet, row)?;
        }
        Ok(scroll_y)
    }

    /// User presses page down.
    /// The `top_row` is now the first row that is not fully visible
    pub fn on_page_down(&mut self) -> Result<(), String> {
        let (sheet, window_height) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (view.sheet, view.window_height)
            } else {
                return Err("View not found".to_string());
            };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };
        let mut last_row = view.top_row;
        let mut height = self.model.get_row_height(sheet, last_row)?;
        while height <= window_height as f64 {
            last_row += 1;
            height += self.model.get_row_height(sheet, last_row)?;
        }
        if !is_valid_row(last_row) {
            return Ok(());
        }
        let row_delta = view.row - view.top_row;
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.top_row = last_row;
                view.row = view.top_row + row_delta;
                view.range = [view.row, view.column, view.row, view.column];
            }
        }
        Ok(())
    }

    /// On page up. tis needs to be the inverse of page down
    pub fn on_page_up(&mut self) -> Result<(), String> {
        let (sheet, window_height) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (view.sheet, view.window_height as f64)
            } else {
                return Err("View not found".to_string());
            };
        let worksheet = match self.model.workbook.worksheet(sheet) {
            Ok(s) => s,
            Err(_) => return Err("Worksheet not found".to_string()),
        };
        let view = match worksheet.views.get(&self.model.view_id) {
            Some(s) => s,
            None => return Err("View not found".to_string()),
        };

        let mut first_row = view.top_row;
        let mut height = self.model.get_row_height(sheet, first_row)?;
        while height <= window_height && first_row > 1 {
            first_row -= 1;
            height += self.model.get_row_height(sheet, first_row)?;
        }

        let row_delta = view.row - view.top_row;
        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.top_row = first_row;
                view.row = view.top_row + row_delta;
                view.range = [view.row, view.column, view.row, view.column];
            }
        }
        Ok(())
    }

    /// We extend the selection to cell (target_row, target_column)
    pub fn on_area_selecting(&mut self, target_row: i32, target_column: i32) -> Result<(), String> {
        let (sheet, window_width, window_height) =
            if let Some(view) = self.model.workbook.views.get(&self.model.view_id) {
                (
                    view.sheet,
                    view.window_width as f64,
                    view.window_height as f64,
                )
            } else {
                return Ok(());
            };
        let (selected_row, selected_column, range, top_row, left_column) =
            if let Ok(worksheet) = self.model.workbook.worksheet(sheet) {
                if let Some(view) = worksheet.views.get(&self.model.view_id) {
                    (
                        view.row,
                        view.column,
                        view.range,
                        view.top_row,
                        view.left_column,
                    )
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            };
        let [row_start, column_start, _row_end, _column_end] = range;

        let mut new_left_column = left_column;
        if target_column >= selected_column {
            let mut width = 0.0;
            let mut column = left_column;
            while column <= target_column {
                width += self.model.get_column_width(sheet, column)?;
                column += 1;
            }

            while width > window_width {
                width -= self.model.get_column_width(sheet, new_left_column)?;
                new_left_column += 1;
            }
        } else if target_column < new_left_column {
            new_left_column = target_column;
        }
        let mut new_top_row = top_row;
        if target_row >= selected_row {
            let mut height = 0.0;
            let mut row = top_row;
            while row <= target_row {
                height += self.model.get_row_height(sheet, row)?;
                row += 1;
            }
            while height > window_height {
                height -= self.model.get_row_height(sheet, new_top_row)?;
                new_top_row += 1;
            }
        } else if target_row < new_top_row {
            new_top_row = target_row;
        }

        if let Ok(worksheet) = self.model.workbook.worksheet_mut(sheet) {
            if let Some(view) = worksheet.views.get_mut(&self.model.view_id) {
                view.range = [row_start, column_start, target_row, target_column];
                if new_top_row != top_row {
                    view.top_row = new_top_row;
                }
                if new_left_column != left_column {
                    view.left_column = new_left_column;
                }
            }
        }

        Ok(())
    }
}
