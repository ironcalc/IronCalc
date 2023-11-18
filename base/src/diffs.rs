use crate::{
    expressions::{
        parser::{
            move_formula::ref_is_in_area,
            stringify::{to_string, to_string_displaced, DisplaceData},
            walk::forward_references,
        },
        types::{Area, CellReferenceIndex, CellReferenceRC},
    },
    model::Model,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum CellValue {
    Value(String),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetCellValue {
    cell: CellReferenceIndex,
    new_value: CellValue,
    old_value: CellValue,
}

impl Model {
    pub(crate) fn shift_cell_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        displace_data: &DisplaceData,
    ) {
        if let Some(f) = self
            .workbook
            .worksheet(sheet)
            .expect("Worksheet must exist")
            .cell(row, column)
            .expect("Cell must exist")
            .get_formula()
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
                self.update_cell_with_formula(sheet, row, column, format!("={formula_displaced}"))
                    .expect("Failed to shift cell formula");
            }
        }
    }

    pub fn forward_references(
        &mut self,
        source_area: &Area,
        target: &CellReferenceIndex,
    ) -> Result<Vec<SetCellValue>, String> {
        let mut diff_list: Vec<SetCellValue> = Vec::new();
        let target_area = &Area {
            sheet: target.sheet,
            row: target.row,
            column: target.column,
            width: source_area.width,
            height: source_area.height,
        };
        // Walk over every formula
        let cells = self.get_all_cells();
        for cell in cells {
            if let Some(f) = self
                .workbook
                .worksheet(cell.index)
                .expect("Worksheet must exist")
                .cell(cell.row, cell.column)
                .expect("Cell must exist")
                .get_formula()
            {
                let sheet = cell.index;
                let row = cell.row;
                let column = cell.column;

                // If cell is in the source or target area, skip
                if ref_is_in_area(sheet, row, column, source_area)
                    || ref_is_in_area(sheet, row, column, target_area)
                {
                    continue;
                }

                // Get the formula
                // Get a copy of the AST
                let node = &mut self.parsed_formulas[sheet as usize][f as usize].clone();
                let cell_reference = CellReferenceRC {
                    sheet: self.workbook.worksheets[sheet as usize].get_name(),
                    column: cell.column,
                    row: cell.row,
                };
                let context = CellReferenceIndex { sheet, column, row };
                let formula = to_string(node, &cell_reference);
                let target_sheet_name = &self.workbook.worksheets[target.sheet as usize].name;
                forward_references(
                    node,
                    &context,
                    source_area,
                    target.sheet,
                    target_sheet_name,
                    target.row,
                    target.column,
                );

                // If the string representation of the formula has changed update the cell
                let updated_formula = to_string(node, &cell_reference);
                if formula != updated_formula {
                    self.update_cell_with_formula(
                        sheet,
                        row,
                        column,
                        format!("={updated_formula}"),
                    )?;
                    // Update the diff list
                    diff_list.push(SetCellValue {
                        cell: CellReferenceIndex { sheet, column, row },
                        new_value: CellValue::Value(format!("={}", updated_formula)),
                        old_value: CellValue::Value(format!("={}", formula)),
                    });
                }
            }
        }
        Ok(diff_list)
    }
}
