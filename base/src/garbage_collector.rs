#![allow(clippy::unwrap_used)]

use crate::model::Model; // Add the missing error module

impl Model {
    /// The garbage collector cleans up any lefover elements in the workbook that are no longer used.
    /// These include:
    /// - Shared strings that are no longer referenced by any cell
    pub fn garbage_collector(&mut self) -> Result<(), String> {
        let cell_values = self.get_all_cell_values().unwrap();
        let mut new_shared_strings = self.shared_strings.clone();

        for (index, _) in self.shared_strings.iter() {
            // A cell is referencing the string, so we continue
            if cell_values.contains(index) {
                continue;
            }

            new_shared_strings.remove(index);
        }

        self.shared_strings = new_shared_strings;

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
