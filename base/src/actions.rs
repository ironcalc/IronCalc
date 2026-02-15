use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::stringify::{
    to_localized_string, to_string_displaced, DisplaceData,
};
use crate::expressions::types::CellReferenceRC;
use crate::model::{CellStructure, Model};
use crate::types::{ArrayKind, Cell};

// NOTE: There is a difference with Excel behaviour when deleting cells/rows/columns
// In Excel if the whole range is deleted then it will substitute for #REF!
// In IronCalc, if one of the edges of the range is deleted will replace the edge with #REF!
// I feel this is unimportant for now.

impl<'a> Model<'a> {
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
            let node = &self.parsed_formulas[sheet as usize][f as usize].0.clone();
            let cell_reference = CellReferenceRC {
                sheet: self.workbook.worksheets[sheet as usize].get_name(),
                row,
                column,
            };
            // FIXME: This is not a very performant way if the formula has changed :S.
            let formula = to_localized_string(node, &cell_reference, self.locale, self.language);
            let formula_displaced = to_string_displaced(node, &cell_reference, displace_data);
            if formula != formula_displaced {
                self.update_cell_with_formula(sheet, row, column, format!("={formula_displaced}"))?;
            };
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
    /// It assumes that the caller has already checked that the move is valid
    /// (e.g. it does not split an array formula). And that dynamic array spills have been reset.
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
        let source_cell = match self
            .workbook
            .worksheet(sheet)?
            .cell(source_row, source_column)
        {
            Some(c) => c,
            None => return Ok(()),
        };
        let style = source_cell.get_style();

        let mut array = None;

        match source_cell {
            Cell::EmptyCell { .. }
            | Cell::BooleanCell { .. }
            | Cell::NumberCell { .. }
            | Cell::ErrorCell { .. }
            | Cell::SharedString { .. }
            | Cell::CellFormula { .. } => {
                // This is a regular cell, we can just move it.
            }
            Cell::SpillCell { .. } => {
                // This the spill of an array formula. Because dynamic arrays spills have been deleted
                // We delete the spill
                let worksheet = self.workbook.worksheet_mut(sheet)?;
                let sheet_data = &mut worksheet.sheet_data;
                if let Some(row_data) = sheet_data.get_mut(&source_row) {
                    row_data.remove(&source_column);
                };
                return Ok(());
            }
            Cell::ArrayFormula {
                r,
                kind: ArrayKind::Dynamic,
                ..
            } => {
                // We are moving the anchor of a dynamic formula.
                // We assume the spill has been taken care of by the caller
                debug_assert_eq!(*r, (1, 1));
            }
            Cell::ArrayFormula {
                r,
                kind: ArrayKind::Cse,
                ..
            } => {
                // This is an array formula, we need to move the whole range
                // We rely on the calling function to check that the move is valid and does not split the array formula
                array = Some(*r);
            }
        }
        let formula_or_value = self
            .get_cell_formula(sheet, source_row, source_column)?
            .unwrap_or_else(|| {
                source_cell.get_localized_text(
                    &self.workbook.shared_strings,
                    self.locale,
                    self.language,
                )
            });

        if let Some((width, height)) = array {
            // We are moving an array formula, we need to move the whole range
            self.set_user_array_formula(
                sheet,
                target_row,
                target_column,
                width,
                height,
                &formula_or_value,
            )?;
        } else {
            self.set_user_input(sheet, target_row, target_column, formula_or_value)?;
        }

        let worksheet = self.workbook.worksheet_mut(sheet)?;
        // copy style
        worksheet.set_cell_style(target_row, target_column, style)?;

        // delete source cell content and style
        let sheet_data = &mut worksheet.sheet_data;
        if let Some(row_data) = sheet_data.get_mut(&source_row) {
            row_data.remove(&source_column);
        };
        Ok(())
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
        if !self.can_insert_columns(sheet, column, column_count)? {
            return Err(
                "Cannot insert columns because that would break an array formula".to_string(),
            );
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
        self.reset_dynamic_array_spills(sheet)?;
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
        if !(1..=LAST_COLUMN).contains(&column) {
            return Err(format!("Column number '{column}' is not valid."));
        }
        if column + column_count - 1 > LAST_COLUMN {
            return Err("Cannot delete columns beyond the last column of the sheet".to_string());
        }
        if !self.can_delete_columns(sheet, column, column_count)? {
            return Err(
                "Cannot delete columns because that would break an array formula".to_string(),
            );
        }

        self.reset_dynamic_array_spills(sheet)?;
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
                        let ws = self.workbook.worksheet_mut(sheet)?;
                        let sheet_data = &mut ws.sheet_data;

                        if let Some(row_data) = sheet_data.get_mut(&r) {
                            row_data.remove(&col);
                        }
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

    // Returns true if inserting rows at `row` would not split any array formula.
    // Inserting at `row` shifts every row >= `row` down. A formula whose anchor
    // row is strictly above `row` but whose spill extends to `row` or below would
    // be split, so we must reject that.
    fn can_insert_rows(&self, sheet: u32, row: i32, _row_count: i32) -> Result<bool, String> {
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (_, height) } =
                self.get_cell_structure(sheet, r, c)?
            {
                // The formula spans rows [r, r + height - 1].
                // Inserting at `row` splits it when the anchor is above `row`
                // but the spill reaches `row` or beyond.
                if r < row && row < r + height {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    // Returns true if inserting columns at `column` would not split any array formula.
    fn can_insert_columns(
        &self,
        sheet: u32,
        column: i32,
        _column_count: i32,
    ) -> Result<bool, String> {
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (width, _) } =
                self.get_cell_structure(sheet, r, c)?
            {
                if c < column && column < c + width {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    // Returns true if deleting rows [row, row + row_count - 1] would not break any
    // array formula. An array formula must be either fully inside the deleted range
    // or fully outside it; any partial overlap is rejected.
    fn can_delete_rows(&self, sheet: u32, row: i32, row_count: i32) -> Result<bool, String> {
        let row_end = row + row_count; // exclusive upper bound
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (_, height) } =
                self.get_cell_structure(sheet, r, c)?
            {
                // Formula row span: [r, r + height - 1]
                let overlaps = r < row_end && r + height > row;
                let contained = r >= row && r + height <= row_end;
                if overlaps && !contained {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    // Returns true if deleting columns [column, column + column_count - 1] would not
    // break any array formula.
    fn can_delete_columns(
        &self,
        sheet: u32,
        column: i32,
        column_count: i32,
    ) -> Result<bool, String> {
        let col_end = column + column_count; // exclusive upper bound
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };
        for (r, c) in cell_coords {
            if let CellStructure::ArrayFormula { range: (width, _) } =
                self.get_cell_structure(sheet, r, c)?
            {
                // Formula column span: [c, c + width - 1]
                let overlaps = c < col_end && c + width > column;
                let contained = c >= column && c + width <= col_end;
                if overlaps && !contained {
                    return Ok(false);
                }
            }
        }
        Ok(true)
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
        if !self.can_insert_rows(sheet, row, row_count)? {
            return Err("Cannot insert rows because that would break an array formula".to_string());
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

        self.reset_dynamic_array_spills(sheet)?;
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
        if !(1..=LAST_ROW).contains(&row) {
            return Err(format!("Row number '{row}' is not valid."));
        }
        if row + row_count - 1 > LAST_ROW {
            return Err("Cannot delete rows beyond the last row of the sheet".to_string());
        }
        if !self.can_delete_rows(sheet, row, row_count)? {
            return Err("Cannot delete rows because that would break an array formula".to_string());
        }

        self.reset_dynamic_array_spills(sheet)?;
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
                    let ws = self.workbook.worksheet_mut(sheet)?;
                    let sheet_data = &mut ws.sheet_data;

                    if let Some(row_data) = sheet_data.get_mut(&r) {
                        for column in columns {
                            row_data.remove(&column);
                        }
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

    // Inner column move: no boundary/can check, no spill reset.
    // Caller must have validated and reset spills before calling this.
    fn move_column_unchecked(&mut self, sheet: u32, column: i32, delta: i32) -> Result<(), String> {
        let target_column = column + delta;
        let original_refs = self
            .workbook
            .worksheet(sheet)?
            .column_cell_references(column)?;
        let mut original_cells = Vec::new();
        for r in &original_refs {
            let cell = self
                .workbook
                .worksheet(sheet)?
                .cell(r.row, column)
                .ok_or("Expected Cell to exist")?;
            let style_idx = cell.get_style();
            let formula_or_value =
                self.get_cell_formula(sheet, r.row, column)?
                    .unwrap_or_else(|| {
                        cell.get_localized_text(
                            &self.workbook.shared_strings,
                            self.locale,
                            self.language,
                        )
                    });

            let mut array = None;

            match cell {
                Cell::EmptyCell { .. }
                | Cell::BooleanCell { .. }
                | Cell::NumberCell { .. }
                | Cell::ErrorCell { .. }
                | Cell::SharedString { .. }
                | Cell::CellFormula { .. } => {
                    // This is a regular cell, we can just move it.
                }
                Cell::SpillCell { .. } => {
                    // This the spill of an array formula. Because dynamic arrays spills have been deleted
                    // We delete the spill
                    let worksheet = self.workbook.worksheet_mut(sheet)?;
                    let sheet_data = &mut worksheet.sheet_data;
                    if let Some(row_data) = sheet_data.get_mut(&r.row) {
                        row_data.remove(&column);
                    };
                    continue;
                }
                Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Dynamic,
                    ..
                } => {
                    // We are moving the anchor of a dynamic formula.
                    // We assume the spill has been taken care of by the caller
                    debug_assert_eq!(*r, (1, 1));
                }
                Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Cse,
                    ..
                } => {
                    // This is an array formula, we need to move the whole range
                    // We rely on the calling function to check that the move is valid and does not split the array formula
                    array = Some(*r);
                }
            }

            original_cells.push((r.row, formula_or_value, style_idx, array));
            let ws = self.workbook.worksheet_mut(sheet)?;
            let sheet_data = &mut ws.sheet_data;
            if let Some(row_data) = sheet_data.get_mut(&r.row) {
                row_data.remove(&column);
            }
        }
        let width = self
            .workbook
            .worksheet(sheet)?
            .get_actual_column_width(column)?;
        let style = self.workbook.worksheet(sheet)?.get_column_style(column)?;
        let hidden = self.workbook.worksheet(sheet)?.is_column_hidden(column)?;
        if delta > 0 {
            for c in column + 1..=target_column {
                let refs = self.workbook.worksheet(sheet)?.column_cell_references(c)?;
                for r in refs {
                    self.move_cell(sheet, r.row, c, r.row, c - 1)?;
                }
                let w = self.workbook.worksheet(sheet)?.get_actual_column_width(c)?;
                let s = self.workbook.worksheet(sheet)?.get_column_style(c)?;
                let h = self.workbook.worksheet(sheet)?.is_column_hidden(c)?;
                self.workbook
                    .worksheet_mut(sheet)?
                    .set_column_width_and_style(c - 1, w, h, s)?;
            }
        } else {
            for c in (target_column..=column - 1).rev() {
                let refs = self.workbook.worksheet(sheet)?.column_cell_references(c)?;
                for r in refs {
                    self.move_cell(sheet, r.row, c, r.row, c + 1)?;
                }
                let w = self.workbook.worksheet(sheet)?.get_actual_column_width(c)?;
                let s = self.workbook.worksheet(sheet)?.get_column_style(c)?;
                let h = self.workbook.worksheet(sheet)?.is_column_hidden(c)?;
                self.workbook
                    .worksheet_mut(sheet)?
                    .set_column_width_and_style(c + 1, w, h, s)?;
            }
        }
        for (r, value, style_idx, array) in original_cells {
            if let Some(a) = array {
                self.set_user_array_formula(sheet, r, target_column, a.0, a.1, &value)?;
            } else {
                self.set_user_input(sheet, r, target_column, value)?;
            }
            self.workbook
                .worksheet_mut(sheet)?
                .set_cell_style(r, target_column, style_idx)?;
        }
        self.workbook
            .worksheet_mut(sheet)?
            .set_column_width_and_style(target_column, width, hidden, style)?;
        self.displace_cells(
            &(DisplaceData::ColumnMove {
                sheet,
                column,
                delta,
            }),
        )?;
        Ok(())
    }

    // Inner row move: no boundary/can check, no spill reset.
    fn move_row_unchecked(&mut self, sheet: u32, row: i32, delta: i32) -> Result<(), String> {
        let target_row = row + delta;
        let original_cols = self.get_columns_for_row(sheet, row, false)?;
        let mut original_cells = Vec::new();
        for c in &original_cols {
            let cell = self
                .workbook
                .worksheet(sheet)?
                .cell(row, *c)
                .ok_or("Expected Cell to exist")?;
            let style_idx = cell.get_style();
            let formula_or_value = self.get_cell_formula(sheet, row, *c)?.unwrap_or_else(|| {
                cell.get_localized_text(&self.workbook.shared_strings, self.locale, self.language)
            });
            let mut array = None;

            match cell {
                Cell::EmptyCell { .. }
                | Cell::BooleanCell { .. }
                | Cell::NumberCell { .. }
                | Cell::ErrorCell { .. }
                | Cell::SharedString { .. }
                | Cell::CellFormula { .. } => {
                    // This is a regular cell, we can just move it.
                }
                Cell::SpillCell { .. } => {
                    // This the spill of an array formula. Because dynamic arrays spills have been deleted
                    // We delete the spill
                    let worksheet = self.workbook.worksheet_mut(sheet)?;
                    let sheet_data = &mut worksheet.sheet_data;
                    if let Some(row_data) = sheet_data.get_mut(&row) {
                        row_data.remove(c);
                    };
                    continue;
                }
                Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Dynamic,
                    ..
                } => {
                    // We are moving the anchor of a dynamic formula.
                    // We assume the spill has been taken care of by the caller
                    debug_assert_eq!(*r, (1, 1));
                }
                Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Cse,
                    ..
                } => {
                    // This is an array formula, we need to move the whole range
                    // We rely on the calling function to check that the move is valid and does not split the array formula
                    array = Some(*r);
                }
            }
            original_cells.push((*c, formula_or_value, style_idx, array));
            let ws = self.workbook.worksheet_mut(sheet)?;
            let sheet_data = &mut ws.sheet_data;
            if let Some(row_data) = sheet_data.get_mut(&row) {
                row_data.remove(c);
            }
        }
        if delta > 0 {
            for r in row + 1..=target_row {
                let cols = self.get_columns_for_row(sheet, r, false)?;
                for c in cols {
                    self.move_cell(sheet, r, c, r - 1, c)?;
                }
            }
        } else {
            for r in (target_row..=row - 1).rev() {
                let cols = self.get_columns_for_row(sheet, r, false)?;
                for c in cols {
                    self.move_cell(sheet, r, c, r + 1, c)?;
                }
            }
        }
        for (c, value, style_idx, array) in original_cells {
            if let Some(array_range) = array {
                self.set_user_array_formula(
                    sheet,
                    target_row,
                    c,
                    array_range.0,
                    array_range.1,
                    &value,
                )?;
            } else {
                self.set_user_input(sheet, target_row, c, value)?;
            }
            self.workbook
                .worksheet_mut(sheet)?
                .set_cell_style(target_row, c, style_idx)?;
        }
        let worksheet = &mut self.workbook.worksheet_mut(sheet)?;
        let mut new_rows = Vec::new();
        for r in worksheet.rows.iter() {
            if r.r == row {
                let mut nr = r.clone();
                nr.r = target_row;
                new_rows.push(nr);
            } else if delta > 0 && r.r > row && r.r <= target_row {
                let mut nr = r.clone();
                nr.r -= 1;
                new_rows.push(nr);
            } else if delta < 0 && r.r < row && r.r >= target_row {
                let mut nr = r.clone();
                nr.r += 1;
                new_rows.push(nr);
            } else {
                new_rows.push(r.clone());
            }
        }
        worksheet.rows = new_rows;
        self.displace_cells(&(DisplaceData::RowMove { sheet, row, delta }))?;
        Ok(())
    }

    // Returns true if moving columns [column, column+column_count-1] by delta would not
    // split any CSE array formula. A formula is OK if its column span is fully within
    // the moved group, fully within the displaced zone, or fully outside both.
    fn can_move_columns_action(
        &self,
        sheet: u32,
        column: i32,
        column_count: i32,
        delta: i32,
    ) -> Result<bool, String> {
        if delta == 0 {
            return Ok(true);
        }

        let group_start = column;
        let group_end = column + column_count - 1;

        let (displace_start, displace_end) = if delta > 0 {
            (group_end + 1, group_end + delta)
        } else {
            (group_start + delta, group_start - 1)
        };

        let overlaps = |a_start: i32, a_end: i32, b_start: i32, b_end: i32| {
            a_start <= b_end && b_start <= a_end
        };

        let contains = |a_start: i32, a_end: i32, b_start: i32, b_end: i32| {
            a_start <= b_start && b_end <= a_end
        };

        let interval_is_safe = |array_start: i32, array_end: i32| {
            let safe_for = |start: i32, end: i32| {
                !overlaps(start, end, array_start, array_end)
                    || contains(start, end, array_start, array_end)
            };
            safe_for(group_start, group_end) && safe_for(displace_start, displace_end)
        };

        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };

        for (r, c) in cell_coords {
            match self.get_cell_structure(sheet, r, c)? {
                CellStructure::ArrayFormula { range } => {
                    let (width, _) = range;
                    let array_start_col = c;
                    let array_end_col = c + width - 1;

                    if !interval_is_safe(array_start_col, array_end_col) {
                        return Ok(false);
                    }
                }
                CellStructure::SpillArray { anchor, range } => {
                    let (width, _) = range;
                    let (_, array_start_col) = anchor;
                    let array_end_col = array_start_col + width - 1;

                    if !interval_is_safe(array_start_col, array_end_col) {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }

    // Returns true if moving rows [row, row+row_count-1] by delta would not
    // split any CSE array formula.
    // That could happen because:
    // * rows are moved in the middle of an array formula
    // * we move part of an array
    fn can_move_rows_action(
        &self,
        sheet: u32,
        row: i32,
        row_count: i32,
        delta: i32,
    ) -> Result<bool, String> {
        if delta == 0 {
            return Ok(true);
        }

        let group_start = row;
        let group_end = row + row_count - 1;

        let (displace_start, displace_end) = if delta > 0 {
            (group_end + 1, group_end + delta)
        } else {
            (group_start + delta, group_start - 1)
        };

        let overlaps = |a_start: i32, a_end: i32, b_start: i32, b_end: i32| {
            a_start <= b_end && b_start <= a_end
        };

        let contains = |a_start: i32, a_end: i32, b_start: i32, b_end: i32| {
            a_start <= b_start && b_end <= a_end
        };

        let interval_is_safe = |array_start: i32, array_end: i32| {
            let safe_for = |start: i32, end: i32| {
                !overlaps(start, end, array_start, array_end)
                    || contains(start, end, array_start, array_end)
            };

            safe_for(group_start, group_end) && safe_for(displace_start, displace_end)
        };

        // list of all the cells in the sheet
        let cell_coords: Vec<(i32, i32)> = {
            let worksheet = self.workbook.worksheet(sheet)?;
            worksheet
                .sheet_data
                .iter()
                .flat_map(|(r, row_data)| row_data.keys().map(move |c| (*r, *c)))
                .collect()
        };

        for (r, c) in cell_coords {
            match self.get_cell_structure(sheet, r, c)? {
                CellStructure::ArrayFormula { range } => {
                    let (_, height) = range;
                    let array_start_row = r;
                    let array_end_row = r + height - 1;

                    if !interval_is_safe(array_start_row, array_end_row) {
                        return Ok(false);
                    }
                }
                CellStructure::SpillArray { anchor, range } => {
                    let (_, height) = range;
                    let (array_start_row, _) = anchor;
                    let array_end_row = array_start_row + height - 1;

                    if !interval_is_safe(array_start_row, array_end_row) {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }

        Ok(true)
    }

    /// Moves a group of columns [column, column+column_count-1] by delta positions.
    /// CSE array formulas fully within the moved group are preserved as arrays.
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
    ///      NOTE: This moves the data and column styles along with the formulas
    pub fn move_columns_action(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if column_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = column + delta;
        let target_last = column + column_count - 1 + delta;
        if !(1..=LAST_COLUMN).contains(&target_first) || !(1..=LAST_COLUMN).contains(&target_last) {
            return Err("Target column out of boundaries".to_string());
        }
        if !(1..=LAST_COLUMN).contains(&column)
            || !(1..=LAST_COLUMN).contains(&(column + column_count - 1))
        {
            return Err("Initial column out of boundaries".to_string());
        }
        if !self.can_move_columns_action(sheet, column, column_count, delta)? {
            return Err(
                "Cannot move columns because that would split an array formula".to_string(),
            );
        }
        self.reset_dynamic_array_spills(sheet)?;

        // Move columns in the correct order
        if delta > 0 {
            for col in (column..column + column_count).rev() {
                self.move_column_unchecked(sheet, col, delta)?;
            }
        } else {
            for col in column..column + column_count {
                self.move_column_unchecked(sheet, col, delta)?;
            }
        }

        Ok(())
    }

    /// Displaces cells due to a move row action
    /// from initial_row to target_row = initial_row + row_delta
    /// References will be updated following the same rules as move_column_action
    /// NOTE: This moves the data and row styles along with the formulas
    /// Moves a group of rows [row, row+row_count-1] by delta positions.
    /// CSE array formulas fully within the moved group are preserved as arrays.
    pub fn move_rows_action(
        &mut self,
        sheet: u32,
        row: i32,
        row_count: i32,
        delta: i32,
    ) -> Result<(), String> {
        if row_count <= 0 || delta == 0 {
            return Ok(());
        }
        let target_first = row + delta;
        let target_last = row + row_count - 1 + delta;
        if !(1..=LAST_ROW).contains(&target_first) || !(1..=LAST_ROW).contains(&target_last) {
            return Err("Target row out of boundaries".to_string());
        }
        if !(1..=LAST_ROW).contains(&row) || !(1..=LAST_ROW).contains(&(row + row_count - 1)) {
            return Err("Initial row out of boundaries".to_string());
        }
        if !self.can_move_rows_action(sheet, row, row_count, delta)? {
            return Err("Cannot move rows because that would split an array formula".to_string());
        }
        self.reset_dynamic_array_spills(sheet)?;

        // Move rows in the correct order
        if delta > 0 {
            for r in (row..row + row_count).rev() {
                self.move_row_unchecked(sheet, r, delta)?;
            }
        } else {
            for r in row..row + row_count {
                self.move_row_unchecked(sheet, r, delta)?;
            }
        }
        Ok(())
    }
}
