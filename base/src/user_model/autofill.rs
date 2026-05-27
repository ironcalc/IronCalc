#![deny(missing_docs)]

use std::collections::HashMap;

use crate::{
    expressions::{
        types::Area,
        utils::{is_valid_column_number, is_valid_row},
    },
    model::CellStructure,
    user_model::sequence_detector::detect_progression,
    UserModel,
};

use crate::user_model::history::Diff;

impl<'a> UserModel<'a> {
    /// Scans the fill target rectangle (`row_start..=row_end`, `col_start..=col_end`) for
    /// CSE array formulas and prepares them for overwriting:
    ///
    /// * A CSE formula **completely** inside the rectangle is cleared; its original cell
    ///   values are returned keyed by `(row, col)` so the caller can emit correct undo diffs.
    /// * A CSE formula only **partially** overlapping the rectangle causes an immediate error.
    fn collect_and_clear_cse_in_fill_target(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        col_start: i32,
        col_end: i32,
    ) -> Result<HashMap<(i32, i32), Option<crate::types::Cell>>, String> {
        let mut saved: HashMap<(i32, i32), Option<crate::types::Cell>> = HashMap::new();
        let mut handled: Vec<(i32, i32)> = Vec::new();
        for row in row_start..=row_end {
            for col in col_start..=col_end {
                let (ar, ac, w, h) = match self.model.get_cell_structure(sheet, row, col)? {
                    CellStructure::ArrayFormula { range: (w, h) } if w > 1 || h > 1 => {
                        (row, col, w, h)
                    }
                    CellStructure::SpillArray {
                        anchor: (ar, ac),
                        range: (w, h),
                    } => (ar, ac, w, h),
                    _ => continue,
                };
                if handled.contains(&(ar, ac)) {
                    continue;
                }
                handled.push((ar, ac));
                let completely_covered = ar >= row_start
                    && ar + h - 1 <= row_end
                    && ac >= col_start
                    && ac + w - 1 <= col_end;
                if !completely_covered {
                    return Err(
                        "Cannot autofill: selection partially overlaps an array formula"
                            .to_string(),
                    );
                }
                // Save the old cell values before clearing so that undo diffs are correct.
                for r in ar..ar + h {
                    for c in ac..ac + w {
                        let cell = self.model.workbook.worksheet(sheet)?.cell(r, c).cloned();
                        saved.insert((r, c), cell);
                    }
                }
                let ws = self.model.workbook.worksheet_mut(sheet)?;
                for r in ar..ar + h {
                    for c in ac..ac + w {
                        let _ = ws.cell_clear_contents(r, c);
                    }
                }
            }
        }
        Ok(saved)
    }

    /// Fills the cells from `source_area` until `to_row`.
    /// This simulates the user clicking on the cell outline handle and dragging it downwards (or upwards)
    pub fn auto_fill_rows(&mut self, source_area: &Area, to_row: i32) -> Result<(), String> {
        let mut diff_list = Vec::new();
        let sheet = source_area.sheet;
        let row1 = source_area.row;
        let column1 = source_area.column;
        let width = source_area.width;
        let height = source_area.height;

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
        if width <= 0 || height <= 0 {
            return Err(format!("Invalid width='{}' or height='{}'", width, height));
        }

        let last_column = column1 + width - 1;
        let last_row = row1 + height - 1;

        if !is_valid_column_number(last_column) {
            return Err(format!("Invalid column: '{last_column}'"));
        }
        if !is_valid_row(last_row) {
            return Err(format!("Invalid row: '{last_row}'"));
        }

        if !is_valid_row(to_row) {
            return Err(format!("Invalid row: '{to_row}'"));
        }

        // anchor_row is the first row that repeats in each case.
        let anchor_row;
        let sign;
        // this is the range of rows we are going to fill
        let row_range: Vec<i32>;

        if to_row > last_row {
            // we go downwards, we start from `row1 + height1` to `to_row`,
            anchor_row = row1;
            sign = 1;
            row_range = (last_row + 1..=to_row).collect();
        } else if to_row < row1 {
            // we go upwards, starting from `row1 - 1` all the way to `to_row`
            anchor_row = last_row;
            sign = -1;
            row_range = (to_row..row1).rev().collect();
        } else {
            return Err("Invalid parameters for autofill".to_string());
        }

        // Fill target: rows in row_range, all source columns.
        let fill_row_start = if sign < 0 { to_row } else { last_row + 1 };
        let fill_row_end = if sign < 0 { row1 - 1 } else { to_row };
        let saved_cse = self.collect_and_clear_cse_in_fill_target(
            sheet,
            fill_row_start,
            fill_row_end,
            column1,
            last_column,
        )?;

        for column in column1..=last_column {
            let mut index = 0;
            let locale = &self.model.locale;
            let values = if sign < 0 {
                (row1..=last_row)
                    .rev()
                    .map(|row| self.get_cell_content(sheet, row, column))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                (row1..=last_row)
                    .map(|row| self.get_cell_content(sheet, row, column))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let case_seed = self.get_cell_content(sheet, row1, column)?;
            let possible_progression = detect_progression(&values, locale, &case_seed);
            for (range_idx, row_ref) in row_range.iter().enumerate() {
                let row = *row_ref;

                let old_value = saved_cse.get(&(row, column)).cloned().unwrap_or_else(|| {
                    self.model
                        .workbook
                        .worksheet(sheet)
                        .ok()
                        .and_then(|ws| ws.cell(row, column).cloned())
                });
                let old_style = self.model.get_cell_style_or_none(sheet, row, column)?;

                let source_row = anchor_row + index;
                let target_value;

                // compute the new value and set it
                if let Some(ref detected_progression) = possible_progression {
                    target_value = detected_progression.next(range_idx);
                } else {
                    target_value = self
                        .model
                        .extend_to(sheet, source_row, column, row, column)?;
                }

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

                index = (index + sign) % source_area.height;
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
        let width = source_area.width;
        let height = source_area.height;

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
        if width <= 0 || height <= 0 {
            return Err(format!("Invalid width='{}' or height='{}'", width, height));
        }

        let last_column = column1 + width - 1;
        let last_row = row1 + height - 1;

        if !is_valid_column_number(last_column) {
            return Err(format!("Invalid column: '{last_column}'"));
        }
        if !is_valid_row(last_row) {
            return Err(format!("Invalid row: '{last_row}'"));
        }

        if !is_valid_column_number(to_column) {
            return Err(format!("Invalid column: '{to_column}'"));
        }

        // anchor_column is the first column that repeats in each case.
        let anchor_column;
        let sign;
        // this is the range of columns we are going to fill
        let column_range: Vec<i32>;

        if to_column > last_column {
            // we go right, we start from `last_column + 1` to `to_column`,
            anchor_column = column1;
            sign = 1;
            column_range = (last_column + 1..to_column + 1).collect();
        } else if to_column < column1 {
            // we go left, starting from `column1 - 1` all the way to `to_column`
            anchor_column = last_column;
            sign = -1;
            column_range = (to_column..column1).rev().collect();
        } else {
            return Err("Invalid parameters for autofill".to_string());
        }

        // Fill target: all source rows, columns in column_range.
        let fill_col_start = if sign < 0 { to_column } else { last_column + 1 };
        let fill_col_end = if sign < 0 { column1 - 1 } else { to_column };
        let saved_cse = self.collect_and_clear_cse_in_fill_target(
            sheet,
            row1,
            last_row,
            fill_col_start,
            fill_col_end,
        )?;

        for row in row1..=last_row {
            let mut index = 0;
            let locale = &self.model.locale;
            let values = if sign < 0 {
                (column1..=last_column)
                    .rev()
                    .map(|column| self.get_cell_content(sheet, row, column))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                (column1..=last_column)
                    .map(|column| self.get_cell_content(sheet, row, column))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let case_seed = self.get_cell_content(sheet, row, column1)?;
            let possible_progression = detect_progression(&values, locale, &case_seed);
            for (range_idx, column_ref) in column_range.iter().enumerate() {
                let column = *column_ref;

                // Save value and style first
                let old_value = saved_cse.get(&(row, column)).cloned().unwrap_or_else(|| {
                    self.model
                        .workbook
                        .worksheet(sheet)
                        .ok()
                        .and_then(|ws| ws.cell(row, column).cloned())
                });
                let old_style = self.model.get_cell_style_or_none(sheet, row, column)?;

                let source_column = anchor_column + index;
                let target_value;

                // compute the new value and set it
                if let Some(ref detected_progression) = possible_progression {
                    target_value = detected_progression.next(range_idx);
                } else {
                    target_value = self
                        .model
                        .extend_to(sheet, row, source_column, row, column)?;
                }

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
}
