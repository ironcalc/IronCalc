use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::stringify::{to_string, to_string_displaced, DisplaceData};
use crate::expressions::types::CellReferenceRC;
use crate::model::Model;

// NOTE: There is a difference with Excel behaviour when deleting cells/rows/columns
// In Excel if the whole range is deleted then it will substitute for #REF!
// In IronCalc, if one of the edges of the range is deleted will replace the edge with #REF!
// I feel this is unimportant for now.

impl Model {
    fn shift_cell_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        displace_data: &DisplaceData,
    ) -> Result<(), String> {
        if let Some(f) = self
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .and_then(|c| c.get_formula())
        {
            let node = &self.parsed_formulas[sheet as usize][f as usize].clone();
            let cell_reference = CellReferenceRC {
                sheet: self.workbook.worksheets[sheet as usize].get_name(),
                row,
                column,
            };
            // FIXME: This is not a very performant way if the formula has changed :S.
            let formula = to_string(node, &cell_reference);
            let formula_displaced = to_string_displaced(node, &cell_reference, displace_data);
            if formula != formula_displaced {
                self.update_cell_with_formula(sheet, row, column, format!("={formula_displaced}"))?;
            }
        }
        Ok(())
    }
    /// This function iterates over all cells in the model and shifts their formulas according to the displacement data.
    ///
    /// # Arguments
    ///
    /// * `displace_data` - A reference to `DisplaceData` describing the displacement's direction and magnitude.
    fn displace_cells(&mut self, displace_data: &DisplaceData) -> Result<(), String> {
        let cells = self.get_all_cells();
        for cell in cells {
            self.shift_cell_formula(cell.index, cell.row, cell.column, displace_data)?;
        }
        Ok(())
    }

    /// Retrieves the column indices for a specific row in a given sheet, sorted in ascending or descending order.
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `row` - The row number to retrieve columns for.
    /// * `descending` - If true, the columns are returned in descending order; otherwise, in ascending order.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing either:
    /// - `Ok(Vec<i32>)`: A vector of column indices for the specified row, sorted according to the `descending` flag.
    /// - `Err(String)`: An error message if the sheet cannot be found.
    fn get_columns_for_row(
        &self,
        sheet: u32,
        row: i32,
        descending: bool,
    ) -> Result<Vec<i32>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        if let Some(row_data) = worksheet.sheet_data.get(&row) {
            let mut columns: Vec<i32> = row_data.keys().copied().collect();
            columns.sort_unstable();
            if descending {
                columns.reverse();
            }
            Ok(columns)
        } else {
            Ok(vec![])
        }
    }

    /// Moves the contents of cell (source_row, source_column) to (target_row, target_column).
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `source_row` - The row index of the cell's current location.
    /// * `source_column` - The column index of the cell's current location.
    /// * `target_row` - The row index of the cell's new location.
    /// * `target_column` - The column index of the cell's new location.
    fn move_cell(
        &mut self,
        sheet: u32,
        source_row: i32,
        source_column: i32,
        target_row: i32,
        target_column: i32,
    ) -> Result<(), String> {
        let source_cell = self
            .workbook
            .worksheet(sheet)?
            .cell(source_row, source_column)
            .ok_or("Expected Cell to exist")?;
        let style = source_cell.get_style();
        // FIXME: we need some user_input getter instead of get_text
        let formula_or_value = self
            .get_cell_formula(sheet, source_row, source_column)?
            .unwrap_or_else(|| source_cell.get_text(&self.workbook.shared_strings, &self.language));
        self.set_user_input(sheet, target_row, target_column, formula_or_value)?;
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_style(target_row, target_column, style)?;
        self.cell_clear_all(sheet, source_row, source_column)
    }

    /// Inserts one or more new columns into the model at the specified index.
    ///
    /// This method shifts existing columns to the right to make space for the new columns.
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `column` - The index at which the new columns should be inserted.
    /// * `column_count` - The number of columns to insert.
    pub fn insert_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        if column_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        // check if it is possible:
        let dimensions = self.workbook.worksheet(sheet)?.dimension();
        let last_column = dimensions.max_column + column_count;
        if last_column > LAST_COLUMN {
            return Err(
                "Cannot shift cells because that would delete cells at the end of a row"
                    .to_string(),
            );
        }
        let worksheet = self.workbook.worksheet(sheet)?;
        let all_rows: Vec<i32> = worksheet.sheet_data.keys().copied().collect();
        for row in all_rows {
            let sorted_columns = self.get_columns_for_row(sheet, row, true)?;
            for col in sorted_columns {
                if col >= column {
                    self.move_cell(sheet, row, col, row, col + column_count)?;
                } else {
                    // Break because columns are in descending order.
                    break;
                }
            }
        }

        // Update all formulas in the workbook
        self.displace_cells(
            &(DisplaceData::Column {
                sheet,
                column,
                delta: column_count,
            }),
        )?;

        // In the list of columns:
        // * Keep all the columns to the left
        // * Displace all the columns to the right

        let worksheet = &mut self.workbook.worksheet_mut(sheet)?;

        let mut new_columns = Vec::new();
        for col in worksheet.cols.iter_mut() {
            // range under study
            let min = col.min;
            let max = col.max;
            if column > max {
                // If the range under study is to our left, this is a noop
            } else if column <= min {
                // If the range under study is to our right, we displace it
                col.min = min + column_count;
                col.max = max + column_count;
            } else {
                // If the range under study is in the middle we augment it
                col.max = max + column_count;
            }
            new_columns.push(col.clone());
        }
        // TODO: If in a row the cell to the right and left have the same style we should copy it

        worksheet.cols = new_columns;

        Ok(())
    }

    /// Deletes one or more columns from the model starting at the specified index.
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `column` - The index of the first column to delete.
    /// * `count` - The number of columns to delete.
    pub fn delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        if column_count <= 0 {
            return Err("Please use insert columns instead".to_string());
        }

        // first column being deleted
        let column_start = column;
        // last column being deleted
        let column_end = column + column_count - 1;

        // Move cells
        let worksheet = &self.workbook.worksheet(sheet)?;
        let mut all_rows: Vec<i32> = worksheet.sheet_data.keys().copied().collect();
        // We do not need to do that, but it is safer to eliminate sources of randomness in the algorithm
        all_rows.sort_unstable();

        for r in all_rows {
            let columns: Vec<i32> = self.get_columns_for_row(sheet, r, false)?;
            for col in columns {
                if col >= column_start {
                    if col > column_end {
                        self.move_cell(sheet, r, col, r, col - column_count)?;
                    } else {
                        self.cell_clear_all(sheet, r, col)?;
                    }
                }
            }
        }
        // Update all formulas in the workbook

        self.displace_cells(
            &(DisplaceData::Column {
                sheet,
                column,
                delta: -column_count,
            }),
        )?;
        let worksheet = &mut self.workbook.worksheet_mut(sheet)?;

        // deletes all the column styles
        let mut new_columns = Vec::new();
        for col in worksheet.cols.iter_mut() {
            // range under study
            let min = col.min;
            let max = col.max;
            // In the diagram:
            // |xxxxx| range we are studying [min, max]
            // |*****| range we are deleting [column_start, column_end]
            // we are going to split it in three big cases:
            // ----------------|xxxxxxxx|-----------------
            // -----|*****|------------------------------- Case A
            // -------|**********|------------------------ Case B
            // -------------|**************|-------------- Case C
            // ------------------|****|------------------- Case D
            // ---------------------|**********|---------- Case E
            // -----------------------------|*****|------- Case F
            if column_start < min {
                if column_end < min {
                    // Case A
                    // We displace all columns
                    let mut new_column = col.clone();
                    new_column.min = min - column_count;
                    new_column.max = max - column_count;
                    new_columns.push(new_column);
                } else if column_end < max {
                    // Case B
                    // We displace the end
                    let mut new_column = col.clone();
                    new_column.min = column_start;
                    new_column.max = max - column_count;
                    new_columns.push(new_column);
                } else {
                    // Case C
                    // skip this, we are deleting the whole range
                }
            } else if column_start <= max {
                if column_end <= max {
                    // Case D
                    // We displace the end
                    let mut new_column = col.clone();
                    new_column.max = max - column_count;
                    new_columns.push(new_column);
                } else {
                    // Case E
                    let mut new_column = col.clone();
                    new_column.max = column_start - 1;
                    new_columns.push(new_column);
                }
            } else {
                // Case F
                // No action required
                new_columns.push(col.clone());
            }
        }
        worksheet.cols = new_columns;

        Ok(())
    }

    /// Inserts one or more new rows into the model at the specified index.
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `row` - The index at which the new rows should be inserted.
    /// * `row_count` - The number of rows to insert.
    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        if row_count <= 0 {
            return Err("Cannot add a negative number of cells :)".to_string());
        }
        // Check if it is possible:
        let dimensions = self.workbook.worksheet(sheet)?.dimension();
        let last_row = dimensions.max_row + row_count;
        if last_row > LAST_ROW {
            return Err(
                "Cannot shift cells because that would delete cells at the end of a column"
                    .to_string(),
            );
        }

        // Move cells
        let worksheet = &self.workbook.worksheet(sheet)?;
        let mut all_rows: Vec<i32> = worksheet.sheet_data.keys().copied().collect();
        all_rows.sort_unstable();
        all_rows.reverse();
        for r in all_rows {
            if r >= row {
                // We do not really need the columns in any order
                let columns: Vec<i32> = self.get_columns_for_row(sheet, r, false)?;
                for column in columns {
                    self.move_cell(sheet, r, column, r + row_count, column)?;
                }
            } else {
                // Rows are in descending order
                break;
            }
        }
        // In the list of rows styles:
        // * Add all rows above the rows we are inserting unchanged
        // * Shift the ones below
        let rows = &self.workbook.worksheets[sheet as usize].rows;
        let mut new_rows = vec![];
        for r in rows {
            if r.r < row {
                new_rows.push(r.clone());
            } else if r.r >= row {
                let mut new_row = r.clone();
                new_row.r = r.r + row_count;
                new_rows.push(new_row);
            }
        }
        self.workbook.worksheets[sheet as usize].rows = new_rows;

        // Update all formulas in the workbook
        self.displace_cells(
            &(DisplaceData::Row {
                sheet,
                row,
                delta: row_count,
            }),
        )?;

        Ok(())
    }

    /// Deletes one or more rows from the model starting at the specified index.
    ///
    /// # Arguments
    ///
    /// * `sheet` - The sheet number to retrieve columns from.
    /// * `row` - The index of the first row to delete.
    /// * `row_count` - The number of rows to delete.
    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<(), String> {
        if row_count <= 0 {
            return Err("Please use insert rows instead".to_string());
        }
        // Move cells
        let worksheet = &self.workbook.worksheet(sheet)?;
        let mut all_rows: Vec<i32> = worksheet.sheet_data.keys().copied().collect();
        all_rows.sort_unstable();

        for r in all_rows {
            if r >= row {
                // We do not need ordered, but it is safer to eliminate sources of randomness in the algorithm
                let columns: Vec<i32> = self.get_columns_for_row(sheet, r, false)?;
                if r >= row + row_count {
                    // displace all cells in column
                    for column in columns {
                        self.move_cell(sheet, r, column, r - row_count, column)?;
                    }
                } else {
                    // remove all cells in row
                    // FIXME: We could just remove the entire row in one go
                    for column in columns {
                        self.cell_clear_all(sheet, r, column)?;
                    }
                }
            }
        }
        // In the list of rows styles:
        // * Add all rows above the rows we are deleting unchanged
        // * Skip all those we are deleting
        // * Shift the ones below
        let rows = &self.workbook.worksheets[sheet as usize].rows;
        let mut new_rows = vec![];
        for r in rows {
            if r.r < row {
                new_rows.push(r.clone());
            } else if r.r >= row + row_count {
                let mut new_row = r.clone();
                new_row.r = r.r - row_count;
                new_rows.push(new_row);
            }
        }
        self.workbook.worksheets[sheet as usize].rows = new_rows;
        self.displace_cells(
            &(DisplaceData::Row {
                sheet,
                row,
                delta: -row_count,
            }),
        )?;
        Ok(())
    }

    /// Displaces cells due to a move column action
    /// from initial_column to target_column = initial_column + column_delta
    /// References will be updated following:
    /// Cell references:
    ///    * All cell references to initial_column will go to target_column
    ///    * All cell references to columns in between (initial_column, target_column] will be displaced one to the left
    ///    * All other cell references are left unchanged
    ///      Ranges. This is the tricky bit:
    ///    * Column is one of the extremes of the range. The new extreme would be target_column.
    ///      Range is then normalized
    ///    * Any other case, range is left unchanged.
    ///      NOTE: This does NOT move the data in the columns or move the colum styles
    pub fn move_column_action(
        &mut self,
        sheet: u32,
        column: i32,
        delta: i32,
    ) -> Result<(), String> {
        // Check boundaries
        let target_column = column + delta;
        if !(1..=LAST_COLUMN).contains(&target_column) {
            return Err("Target column out of boundaries".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err("Initial column out of boundaries".to_string());
        }

        // TODO: Add the actual displacement of data and styles

        // Update all formulas in the workbook
        self.displace_cells(
            &(DisplaceData::ColumnMove {
                sheet,
                column,
                delta,
            }),
        )?;

        Ok(())
    }
}
