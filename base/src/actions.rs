use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::stringify::DisplaceData;
use crate::model::Model;

// NOTE: There is a difference with Excel behaviour when deleting cells/rows/columns
// In Excel if the whole range is deleted then it will substitute for #REF!
// In IronCalc, if one of the edges of the range is deleted will replace the edge with #REF!
// I feel this is unimportant for now.

impl Model {
    fn displace_cells(&mut self, displace_data: &DisplaceData) {
        let cells = self.get_all_cells();
        for cell in cells {
            self.shift_cell_formula(cell.index, cell.row, cell.column, displace_data);
        }
    }
    /// Returns the list of columns in row
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

    /// Moves the contents of cell (source_row, source_column) tp (target_row, target_column)
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
            .cell_formula(sheet, source_row, source_column)?
            .unwrap_or_else(|| source_cell.get_text(&self.workbook.shared_strings, &self.language));
        self.set_user_input(sheet, target_row, target_column, formula_or_value);
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_style(target_row, target_column, style);
        self.delete_cell(sheet, source_row, source_column)?;
        Ok(())
    }

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
        self.displace_cells(&DisplaceData::Column {
            sheet,
            column,
            delta: column_count,
        });

        Ok(())
    }

    pub fn delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<(), String> {
        if column_count <= 0 {
            return Err("Please use insert columns instead".to_string());
        }

        // Move cells
        let worksheet = &self.workbook.worksheet(sheet)?;
        let mut all_rows: Vec<i32> = worksheet.sheet_data.keys().copied().collect();
        // We do not need to do that, but it is safer to eliminate sources of randomness in the algorithm
        all_rows.sort_unstable();

        for r in all_rows {
            let columns: Vec<i32> = self.get_columns_for_row(sheet, r, false)?;
            for col in columns {
                if col >= column {
                    if col >= column + column_count {
                        self.move_cell(sheet, r, col, r, col - column_count)?;
                    } else {
                        self.delete_cell(sheet, r, col)?;
                    }
                }
            }
        }
        // Update all formulas in the workbook

        self.displace_cells(&DisplaceData::Column {
            sheet,
            column,
            delta: -column_count,
        });

        Ok(())
    }

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
        self.displace_cells(&DisplaceData::Row {
            sheet,
            row,
            delta: row_count,
        });

        Ok(())
    }

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
                        self.delete_cell(sheet, r, column)?;
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
        self.displace_cells(&DisplaceData::Row {
            sheet,
            row,
            delta: -row_count,
        });
        Ok(())
    }

    /// Displaces cells due to a move column action
    /// from initial_column to target_column = initial_column + column_delta
    /// References will be updated following:
    /// Cell references:
    ///    * All cell references to initial_column will go to target_column
    ///    * All cell references to columns in between (initial_column, target_column] will be displaced one to the left
    ///    * All other cell references are left unchanged
    /// Ranges. This is the tricky bit:
    ///    * Column is one of the extremes of the range. The new extreme would be target_column.
    ///      Range is then normalized
    ///    * Any other case, range is left unchanged.
    /// NOTE: This does NOT move the data in the columns or move the colum styles
    pub fn move_column_action(
        &mut self,
        sheet: u32,
        column: i32,
        delta: i32,
    ) -> Result<(), &'static str> {
        // Check boundaries
        let target_column = column + delta;
        if !(1..=LAST_COLUMN).contains(&target_column) {
            return Err("Target column out of boundaries");
        }
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err("Initial column out of boundaries");
        }

        // TODO: Add the actual displacement of data and styles

        // Update all formulas in the workbook
        self.displace_cells(&DisplaceData::ColumnMove {
            sheet,
            column,
            delta,
        });

        Ok(())
    }
}
