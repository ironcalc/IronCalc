#![allow(clippy::unwrap_used)]

use std::path::Path;

use ironcalc_base::cell::CellValue;
use ironcalc_base::types::*;
use ironcalc_base::{expressions::utils::number_to_column, Model};

use crate::export::save_to_xlsx;
use crate::import::load_from_xlsx;

pub struct CompareError {
    message: String,
}

type CompareResult<T> = std::result::Result<T, CompareError>;

pub struct Diff {
    pub sheet_name: String,
    pub row: i32,
    pub column: i32,
    pub value1: Cell,
    pub value2: Cell,
    pub reason: String,
}

// TODO use f64::EPSILON
const EPS: f64 = 5e-8;
// const EPS: f64 = f64::EPSILON;

fn numbers_are_close(x: f64, y: f64, eps: f64) -> bool {
    let norm = (x * x + y * y).sqrt();
    if norm == 0.0 {
        return true;
    }
    let d = f64::abs(x - y);
    if d < eps {
        return true;
    }
    d / norm < eps
}
/// Compares two Models in the internal representation and returns a list of differences
pub fn compare(model1: &Model, model2: &Model) -> CompareResult<Vec<Diff>> {
    let ws1 = model1.workbook.get_worksheet_names();
    let ws2 = model2.workbook.get_worksheet_names();
    if ws1.len() != ws2.len() {
        return Err(CompareError {
            message: "Different number of sheets".to_string(),
        });
    }
    let eps = if let Ok(CellValue::Number(v)) = model1.get_cell_value_by_ref("METADATA!A1") {
        v
    } else {
        EPS
    };
    let mut diffs = Vec::new();
    let cells = model1.get_all_cells();
    for cell in cells {
        let sheet = cell.index;
        let row = cell.row;
        let column = cell.column;
        let cell1 = &model1
            .workbook
            .worksheet(sheet)
            .unwrap()
            .cell(row, column)
            .cloned()
            .unwrap_or_default();
        let cell2 = &model2
            .workbook
            .worksheet(sheet)
            .unwrap()
            .cell(row, column)
            .cloned()
            .unwrap_or_default();
        match (cell1, cell2) {
            (Cell::EmptyCell { .. }, Cell::EmptyCell { .. }) => {}
            (Cell::NumberCell { .. }, Cell::NumberCell { .. }) => {}
            (Cell::BooleanCell { .. }, Cell::BooleanCell { .. }) => {}
            (Cell::ErrorCell { .. }, Cell::ErrorCell { .. }) => {}
            (Cell::SharedString { .. }, Cell::SharedString { .. }) => {}
            (
                Cell::CellFormulaNumber { v: value1, .. },
                Cell::CellFormulaNumber { v: value2, .. },
            ) => {
                if !numbers_are_close(*value1, *value2, eps) {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Numbers are different".to_string(),
                    });
                }
            }
            (
                Cell::CellFormulaString { v: value1, .. },
                Cell::CellFormulaString { v: value2, .. },
            ) => {
                // FIXME: We should compare the actual value, not just the index
                if value1 != value2 {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Strings are different".to_string(),
                    });
                }
            }
            (
                Cell::CellFormulaBoolean { v: value1, .. },
                Cell::CellFormulaBoolean { v: value2, .. },
            ) => {
                // FIXME: We should compare the actual value, not just the index
                if value1 != value2 {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Booleans are different".to_string(),
                    });
                }
            }
            (
                Cell::CellFormulaError { ei: index1, .. },
                Cell::CellFormulaError { ei: index2, .. },
            ) => {
                // FIXME: We should compare the actual value, not just the index
                if index1 != index2 {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Errors are different".to_string(),
                    });
                }
            }
            (_, _) => {
                diffs.push(Diff {
                    sheet_name: ws1[cell.index as usize].clone(),
                    row,
                    column,
                    value1: cell1.clone(),
                    value2: cell2.clone(),
                    reason: "Types are different".to_string(),
                });
            }
        }
    }
    Ok(diffs)
}

pub(crate) fn compare_models(m1: &Model, m2: &Model) -> Result<(), String> {
    match compare(m1, m2) {
        Ok(diffs) => {
            if diffs.is_empty() {
                Ok(())
            } else {
                let mut message = "".to_string();
                for diff in diffs {
                    message = format!(
                        "{}\n.Diff: {}!{}{}, value1: {:?}, value2 {:?}\n {}",
                        message,
                        diff.sheet_name,
                        number_to_column(diff.column).unwrap(),
                        diff.row,
                        &diff.value1,
                        &diff.value2,
                        diff.reason
                    );
                }
                Err(format!("Models are different: {}", message))
            }
        }
        Err(r) => Err(format!("Models are different: {}", r.message)),
    }
}

/// Tests that file in file_path produces the same results in Excel and in IronCalc.
pub fn test_file(file_path: &str) -> Result<(), String> {
    let model1 = load_from_xlsx(file_path, "en", "UTC").unwrap();
    let mut model2 = load_from_xlsx(file_path, "en", "UTC").unwrap();
    model2.evaluate();
    compare_models(&model1, &model2)
}

/// Tests that file in file_path can be converted to xlsx and read again
pub fn test_load_and_saving(file_path: &str, temp_dir_name: &Path) -> Result<(), String> {
    let model1 = load_from_xlsx(file_path, "en", "UTC").unwrap();

    let base_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    let temp_path_buff = temp_dir_name.join(base_name);
    let temp_file_path = &format!("{}.xlsx", temp_path_buff.to_str().unwrap());
    // test can save
    save_to_xlsx(&model1, temp_file_path).unwrap();
    // test can open
    let mut model2 = load_from_xlsx(temp_file_path, "en", "UTC").unwrap();
    model2.evaluate();
    compare_models(&model1, &model2)
}

#[cfg(test)]
mod tests {
    use crate::compare::compare;
    use ironcalc_base::Model;

    #[test]
    fn compare_different_sheets() {
        let mut model1 = Model::new_empty("model", "en", "UTC").unwrap();
        model1.new_sheet();
        let model2 = Model::new_empty("model", "en", "UTC").unwrap();

        assert!(compare(&model1, &model2).is_err());
    }
}
