//! Review-oriented YAML export: cells (formula or raw value) + optional named ranges.
//! Stable key order for PR diffs. No styling / formatting / evaluation required.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

use ironcalc_base::expressions::utils::number_to_column;
use ironcalc_base::types::Cell;
use ironcalc_base::Model;
use serde::Serialize;

use crate::error::XlsxError;

/// Options for [`save_to_yml`].
#[derive(Debug, Clone, Copy)]
pub struct YmlExportOptions {
    /// When `false`, style-only / empty cells are omitted (typical for PR review).
    pub include_empty_cells: bool,
    /// When `true`, emit a `named_ranges` map (global key = name; scoped = `Sheet!name`).
    pub include_named_ranges: bool,
}

impl Default for YmlExportOptions {
    fn default() -> Self {
        Self {
            include_empty_cells: false,
            include_named_ranges: true,
        }
    }
}

#[derive(Serialize)]
struct YmlDocument {
    cells: BTreeMap<String, serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    named_ranges: Option<BTreeMap<String, String>>,
}

fn number_yaml(n: f64) -> serde_yaml::Value {
    if n.is_finite() {
        serde_yaml::Value::Number(serde_yaml::Number::from(n))
    } else {
        serde_yaml::Value::String(n.to_string())
    }
}

/// Raw stored value for a non-formula cell. `None` means empty.
fn raw_cell_yaml(cell: &Cell, shared_strings: &[String]) -> Option<serde_yaml::Value> {
    match cell {
        Cell::EmptyCell { .. } => None,
        Cell::BooleanCell { v, .. } => Some(serde_yaml::Value::Bool(*v)),
        Cell::NumberCell { v, .. } => Some(number_yaml(*v)),
        Cell::ErrorCell { ei, .. } => Some(serde_yaml::Value::String(ei.to_string())),
        Cell::SharedString { si, .. } => {
            let s = shared_strings
                .get(*si as usize)
                .cloned()
                .unwrap_or_default();
            Some(serde_yaml::Value::String(s))
        }
        // Formula / array / spill handled by callers.
        Cell::CellFormula { .. } | Cell::ArrayFormula { .. } | Cell::SpillCell { .. } => None,
    }
}

fn a1_key(sheet_name: &str, row: i32, column: i32) -> Result<String, XlsxError> {
    let col = number_to_column(column).ok_or_else(|| {
        XlsxError::Workbook(format!("invalid column index {column}"))
    })?;
    Ok(format!("{sheet_name}!{col}{row}"))
}

fn is_empty_cell(cell: &Cell) -> bool {
    matches!(cell, Cell::EmptyCell { .. })
}

/// Writes a YAML snapshot of raw cell formulas/values (and optionally named ranges).
///
/// Overwrites `file_name` if it already exists (unlike [`super::save_to_icalc`]).
pub fn save_to_yml(
    model: &Model,
    file_name: &str,
    options: &YmlExportOptions,
) -> Result<(), XlsxError> {
    let mut cells = BTreeMap::new();

    for cell_index in model.get_all_cells() {
        let sheet = cell_index.index;
        let row = cell_index.row;
        let column = cell_index.column;
        let worksheet = model
            .workbook
            .worksheet(sheet)
            .map_err(XlsxError::Workbook)?;
        let Some(cell) = worksheet.cell(row, column) else {
            continue;
        };

        // Spill cells mirror an array/dynamic formula anchor — skip to avoid noise.
        if matches!(cell, Cell::SpillCell { .. }) {
            continue;
        }

        if !options.include_empty_cells && is_empty_cell(cell) {
            continue;
        }

        let key = a1_key(&worksheet.get_name(), row, column)?;

        let yaml_value = if let Some(formula) = model
            .get_cell_formula(sheet, row, column)
            .map_err(XlsxError::Workbook)?
        {
            serde_yaml::Value::String(formula)
        } else {
            match raw_cell_yaml(cell, &model.workbook.shared_strings) {
                Some(v) => v,
                None if options.include_empty_cells => {
                    serde_yaml::Value::String(String::new())
                }
                None => continue,
            }
        };

        cells.insert(key, yaml_value);
    }

    let named_ranges = if options.include_named_ranges {
        let mut map = BTreeMap::new();
        for dn in &model.workbook.defined_names {
            let key = match dn.sheet_id {
                Some(sheet_id) => {
                    let sheet_name = model
                        .workbook
                        .worksheet(sheet_id)
                        .map_err(XlsxError::Workbook)?
                        .get_name();
                    format!("{sheet_name}!{}", dn.name)
                }
                None => dn.name.clone(),
            };
            map.insert(key, dn.formula.clone());
        }
        Some(map)
    } else {
        None
    };

    let doc = YmlDocument {
        cells,
        named_ranges,
    };

    let yaml = serde_yaml::to_string(&doc).map_err(|e| XlsxError::IO(e.to_string()))?;
    let mut file = fs::File::create(file_name)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}
