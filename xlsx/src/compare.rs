#![allow(clippy::unwrap_used, clippy::panic)]

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

            (Cell::NumberCell { v: value1, .. }, Cell::NumberCell { v: value2, .. }) => {
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

            (Cell::BooleanCell { v: value1, .. }, Cell::BooleanCell { v: value2, .. }) => {
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

            (Cell::ErrorCell { ei: value1, .. }, Cell::ErrorCell { ei: value2, .. }) => {
                if value1 != value2 {
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

            (Cell::SharedString { si: value1, .. }, Cell::SharedString { si: value2, .. }) => {
                // FIXME: compare resolved shared-string contents, not indices,
                // if the two workbooks can have different shared-string tables.
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
                Cell::CellFormula { v: v1, .. } | Cell::ArrayFormula { v: v1, .. },
                Cell::CellFormula { v: v2, .. } | Cell::ArrayFormula { v: v2, .. },
            ) => {
                let mismatch = match (v1, v2) {
                    (FormulaValue::Unevaluated, FormulaValue::Unevaluated) => false,
                    (FormulaValue::Boolean(a), FormulaValue::Boolean(b)) => a != b,
                    (FormulaValue::Number(a), FormulaValue::Number(b)) => {
                        !numbers_are_close(*a, *b, eps)
                    }
                    (FormulaValue::Text(a), FormulaValue::Text(b)) => a != b,
                    (FormulaValue::Error { ei: e1, .. }, FormulaValue::Error { ei: e2, .. }) => {
                        e1 != e2
                    }
                    _ => true,
                };
                if mismatch {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Formula values are different".to_string(),
                    });
                }
            }

            (Cell::SpillCell { v: v1, .. }, Cell::SpillCell { v: v2, .. }) => {
                let mismatch = match (v1, v2) {
                    (SpillValue::Boolean(a), SpillValue::Boolean(b)) => a != b,
                    (SpillValue::Number(a), SpillValue::Number(b)) => {
                        !numbers_are_close(*a, *b, eps)
                    }
                    (SpillValue::Text(a), SpillValue::Text(b)) => a != b,
                    (SpillValue::Error(a), SpillValue::Error(b)) => a != b,
                    _ => true,
                };
                if mismatch {
                    diffs.push(Diff {
                        sheet_name: ws1[cell.index as usize].clone(),
                        row,
                        column,
                        value1: cell1.clone(),
                        value2: cell2.clone(),
                        reason: "Spill values are different".to_string(),
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

fn cell_display(cell: &Cell) -> String {
    match cell {
        Cell::EmptyCell { .. } => "(empty)".to_string(),
        Cell::NumberCell { v, .. } => format!("{v}"),
        Cell::BooleanCell { v, .. } => format!("{v}"),
        Cell::ErrorCell { ei, .. } => format!("{ei} (error)"),
        Cell::SharedString { si, .. } => format!("shared_string[{si}]"),
        Cell::CellFormula {
            v: FormulaValue::Unevaluated,
            ..
        } => "(unevaluated formula)".to_string(),
        Cell::CellFormula {
            v: FormulaValue::Boolean(v),
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Boolean(v),
            ..
        } => format!("{v} (bool)"),
        Cell::CellFormula {
            v: FormulaValue::Number(v),
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Number(v),
            ..
        } => format!("{v} (number)"),
        Cell::CellFormula {
            v: FormulaValue::Text(v),
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Text(v),
            ..
        } => format!("\"{v}\" (string)"),
        Cell::CellFormula {
            v: FormulaValue::Error { ei, .. },
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Error { ei, .. },
            ..
        } => format!("{ei} (error)"),
        Cell::ArrayFormula {
            v: FormulaValue::Unevaluated,
            s,
            r,
            kind,
            ..
        } => {
            format!("(unevaluated {kind:?} formula, size={s}, range={r:?})")
        }
        Cell::SpillCell {
            v: SpillValue::Number(v),
            s,
            a,
        } => {
            format!("{v} (spill, size={s}, area={a:?})")
        }
        Cell::SpillCell {
            v: SpillValue::Boolean(v),
            s,
            a,
        } => {
            format!("{v} (spill, size={s}, area={a:?})")
        }
        Cell::SpillCell {
            v: SpillValue::Error(ei),
            s,
            a,
        } => {
            format!("{ei} (spill, size={s}, area={a:?})")
        }
        Cell::SpillCell {
            v: SpillValue::Text(v),
            s,
            a,
        } => {
            format!("\"{v}\" (spill, size={s}, area={a:?})")
        }
    }
}

pub(crate) fn compare_models(m1: &Model, m2: &Model) -> Result<(), String> {
    match compare(m1, m2) {
        Ok(diffs) => {
            if diffs.is_empty() {
                Ok(())
            } else {
                let count = diffs.len();
                let mut lines = format!(
                    "Models are different ({count} diff{}):\n",
                    if count == 1 { "" } else { "s" }
                );
                for diff in diffs {
                    let col = number_to_column(diff.column).unwrap();
                    let cell_ref = format!("{}!{}{}", diff.sheet_name, col, diff.row);
                    let excel = cell_display(&diff.value1);
                    let ironcalc = cell_display(&diff.value2);
                    lines.push_str(&format!(
                        "\n  {cell_ref:<16}  Excel: {excel:<30}  IronCalc: {ironcalc:<30}  [{}]",
                        diff.reason
                    ));
                }
                Err(lines)
            }
        }
        Err(r) => Err(format!("Models are different: {}", r.message)),
    }
}

// Cheesy way to get the locale from the workbook metadata sheet
fn get_workbook_metadata(model: &Model) -> String {
    // let mut index = 0;
    let mut metadata_sheet_index = None;
    for (index, ws) in model.workbook.worksheets.iter().enumerate() {
        if ws.name.eq_ignore_ascii_case("METADATA") {
            metadata_sheet_index = Some(index as u32);
            break;
        }
    }
    let default_locale = "en".to_string();
    if let Some(sheet_index) = metadata_sheet_index {
        if let Ok(a1) = model.get_formatted_cell_value(sheet_index, 1, 1) {
            if a1 == "Locale" {
                match model.get_formatted_cell_value(sheet_index, 1, 2) {
                    Ok(v) if v == "en-GB" => {
                        return "en-GB".to_string();
                    }
                    _ => return default_locale,
                }
            }
        }
    }
    default_locale
}

/// Tests that file in file_path produces the same results in Excel and in IronCalc.
pub fn test_file(file_path: &str) -> Result<(), String> {
    // FIXME: we need to load the model twice :S
    let model1 = load_from_xlsx(file_path, "en", "UTC", "en").unwrap();
    let locale = get_workbook_metadata(&model1);
    let model1 = load_from_xlsx(file_path, &locale, "UTC", "en").unwrap();
    let mut model2 = load_from_xlsx(file_path, &locale, "UTC", "en").unwrap();
    model2.evaluate();
    compare_models(&model1, &model2)
}

/// Tests that file in file_path can be converted to xlsx and read again
pub fn test_load_and_saving(file_path: &str, temp_dir_name: &Path) -> Result<(), String> {
    // FIXME: we need to evaluate the model twice :S
    let model1 = load_from_xlsx(file_path, "en", "UTC", "en").unwrap();
    let locale = get_workbook_metadata(&model1);

    let model1 = load_from_xlsx(file_path, &locale, "UTC", "en").unwrap();

    let base_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();

    let temp_path_buff = temp_dir_name.join(base_name);
    let temp_file_path = &format!("{}.xlsx", temp_path_buff.to_str().unwrap());
    // test can save
    save_to_xlsx(&model1, temp_file_path).unwrap();
    // test can open
    let mut model2 = load_from_xlsx(temp_file_path, &locale, "UTC", "en").unwrap();
    model2.evaluate();
    compare_models(&model1, &model2)
}

#[cfg(test)]
mod tests {
    use crate::compare::compare;
    use ironcalc_base::Model;

    #[test]
    fn compare_different_sheets() {
        let mut model1 = Model::new_empty("model", "en", "UTC", "en").unwrap();
        model1.new_sheet();
        let model2 = Model::new_empty("model", "en", "UTC", "en").unwrap();

        assert!(compare(&model1, &model2).is_err());
    }
}
