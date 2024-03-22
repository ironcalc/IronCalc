#![allow(clippy::unwrap_used)]

use crate::model::Model; // Add the missing error module

impl Model {
    /// The garbage collector cleans up any lefover elements in the workbook that are no longer used.
    /// These include:
    /// - Shared strings that are no longer referenced by any cell
    pub fn garbage_collector(&mut self) -> Result<(), String> {
        let cell_values = self.get_all_cell_values()?;

        self.shared_strings.retain(|index, _| {
            // A cell is referencing the string, so we keep it
            cell_values.contains(index)
        });

        Ok(())
    }

    /// Returns a vector of all formatted cell values in the workbook.
    fn get_all_cell_values(&self) -> Result<Vec<String>, String> {
        let cells = self.get_all_cells();
        let mut cell_values = Vec::new();

        for cell in cells {
            let cell_value = self.formatted_cell_value(cell.index, cell.row, cell.column);
            cell_values.push(cell_value?);
        }

        Ok(cell_values)
    }
}
