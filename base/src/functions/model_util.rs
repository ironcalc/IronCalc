use crate::{
    constants::{LAST_COLUMN, LAST_ROW},
    model::Model,
    worksheet::WorksheetDimension,
};

impl<'a> Model<'a> {
    /// Resolves LAST_ROW / LAST_COLUMN sentinels to the actual sheet extents.
    ///
    /// Returns `Err(())` when the sheet index is invalid. The caller is
    /// responsible for mapping that to a `CalcResult` error — it owns the
    /// `cell` context needed for a useful message.
    ///
    /// # Usage
    ///
    /// ```rust
    /// // In a function returning Result<_, CalcResult>:
    /// let dim = self.get_max_rc(sheet, row1, col1, row2, col2)
    ///     .map_err(|_| CalcResult::new_error(Error::ERROR, *cell, ...))?;
    ///
    /// // In a function returning CalcResult directly:
    /// let dim = match self.get_max_rc(sheet, row1, col1, row2, col2) {
    ///     Ok(d) => d,
    ///     Err(_) => return CalcResult::new_error(Error::ERROR, cell, ...),
    /// };
    /// ```
    pub fn get_max_rc(
        &self,
        sheet: u32,
        row1: i32,
        col1: i32,
        row2: i32,
        col2: i32,
    ) -> Result<WorksheetDimension, ()> {
        let needs_row = row1 == 1 && row2 == LAST_ROW;
        let needs_col = col1 == 1 && col2 == LAST_COLUMN;

        // Single worksheet() call covers both row and column resolution.
        let (r_max, c_max) = if needs_row || needs_col {
            let dim = self.workbook.worksheet(sheet).map_err(|_| ())?.dimension();
            (
                if needs_row { dim.max_row } else { row2 },
                if needs_col { dim.max_column } else { col2 },
            )
        } else {
            (row2, col2)
        };

        Ok(WorksheetDimension {
            min_row: row1,
            max_row: r_max,
            min_column: col1,
            max_column: c_max,
        })
    }
}
