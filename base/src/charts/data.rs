//! The values a chart plots, computed from the workbook on demand.
//!
//! A [`DataSource`] only holds references. [`Model::get_chart_data`] resolves
//! them into the categories and the points of every series, the same way for
//! the renderer and for the xlsx exporter (which writes them as the caches
//! Excel keeps next to the references). Nothing here is stored: the values
//! are always those of the evaluated cells.

use serde::{Deserialize, Serialize};

use crate::charts::common::{DataRange, DataSource, Orientation, Series};
use crate::expressions::utils::{number_to_column, parse_reference_a1};
use crate::types::CellType;
use crate::Model;

/// The points of one series, in the order of the categories.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SeriesData {
    /// The resolved name; the empty string is an empty name.
    pub name: String,
    /// One entry per cell of the values range. `None` is a cell with no
    /// point: empty or an error. Text is plotted as zero, as Excel does.
    pub values: Vec<Option<f64>>,
    /// The number format of the first cell of the values range.
    pub number_format: String,
}

/// What a chart plots.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ChartData {
    /// The formatted text of the category cells.
    pub categories: Vec<String>,
    pub series: Vec<SeriesData>,
}

/// A cell of a range reference, with `$` on both parts as Excel writes them.
fn absolute_cell(column: i32, row: i32) -> String {
    format!(
        "${}${row}",
        number_to_column(column).unwrap_or_else(|| "A".to_string())
    )
}

fn absolute_range(sheet: &str, column1: i32, row1: i32, column2: i32, row2: i32) -> String {
    if column1 == column2 && row1 == row2 {
        format!("{sheet}!{}", absolute_cell(column1, row1))
    } else {
        format!(
            "{sheet}!{}:{}",
            absolute_cell(column1, row1),
            absolute_cell(column2, row2)
        )
    }
}

/// Splits `Sheet1!$B$1:$E$6` into the sheet part, as written, and the corners
/// (column1, row1, column2, row2), rows and columns 1-based.
pub fn parse_range(range: &str) -> Option<(&str, i32, i32, i32, i32)> {
    let (sheet, cells) = range.rsplit_once('!')?;
    let (first, last) = match cells.split_once(':') {
        Some((first, last)) => (first, last),
        None => (cells, cells),
    };
    let first = parse_reference_a1(first)?;
    let last = parse_reference_a1(last)?;
    Some((
        sheet,
        first.column.min(last.column),
        first.row.min(last.row),
        first.column.max(last.column),
        first.row.max(last.row),
    ))
}

/// The sheet name without the quotes a reference may carry: `'My Sheet'` is `My Sheet`.
fn unquote_sheet_name(sheet: &str) -> String {
    match sheet
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        Some(inner) => inner.replace("''", "'"),
        None => sheet.to_string(),
    }
}

/// The categories and the series of a chart, expanding a single-range data
/// source into one series per column (or row) as Excel stores them.
pub fn expand_data_range(range: &DataRange) -> (String, Vec<Series>) {
    match range {
        DataRange::Manual { categories, series } => (categories.clone(), series.clone()),
        DataRange::Range { range, orientation } => {
            let Some((sheet, column1, row1, column2, row2)) = parse_range(range) else {
                return ("".to_string(), Vec::new());
            };
            let mut series = Vec::new();
            match orientation {
                Orientation::Columns => {
                    let categories = absolute_range(sheet, column1, row1 + 1, column1, row2);
                    for column in column1 + 1..=column2 {
                        series.push(Series {
                            name: absolute_range(sheet, column, row1, column, row1),
                            values: absolute_range(sheet, column, row1 + 1, column, row2),
                        });
                    }
                    (categories, series)
                }
                Orientation::Rows => {
                    let categories = absolute_range(sheet, column1 + 1, row1, column2, row1);
                    for row in row1 + 1..=row2 {
                        series.push(Series {
                            name: absolute_range(sheet, column1, row, column1, row),
                            values: absolute_range(sheet, column1 + 1, row, column2, row),
                        });
                    }
                    (categories, series)
                }
            }
        }
    }
}

/// A range reference resolved to a sheet index and 1-based corners.
struct CellRange {
    sheet: u32,
    column1: i32,
    row1: i32,
    column2: i32,
    row2: i32,
}

impl CellRange {
    /// The cells, row by row, as Excel flattens a two dimensional range.
    fn cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (self.row1..=self.row2)
            .flat_map(move |row| (self.column1..=self.column2).map(move |column| (row, column)))
    }
}

impl Model<'_> {
    fn resolve_range(&self, reference: &str) -> Result<CellRange, String> {
        let Some((sheet, column1, row1, column2, row2)) = parse_range(reference) else {
            return Err(format!("Invalid range: '{reference}'"));
        };
        let sheet_name = unquote_sheet_name(sheet);
        let Some(sheet) = self.get_sheet_index_by_name(&sheet_name) else {
            return Err(format!("Unknown sheet: '{sheet_name}'"));
        };
        Ok(CellRange {
            sheet,
            column1,
            row1,
            column2,
            row2,
        })
    }

    /// The point a cell contributes to a series.
    fn series_point(&self, sheet: u32, row: i32, column: i32) -> Result<Option<f64>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let Some(cell) = worksheet.cell(row, column) else {
            return Ok(None);
        };
        if cell.get_type() == CellType::ErrorValue {
            return Ok(None);
        }
        Ok(
            match cell.value(&self.workbook.shared_strings, self.language) {
                crate::cell::CellValue::None => None,
                crate::cell::CellValue::Number(value) => Some(value),
                // Text and booleans count as zero, as in Excel
                crate::cell::CellValue::String(_) | crate::cell::CellValue::Boolean(_) => Some(0.0),
            },
        )
    }

    /// The name of a series: a literal in quotes, the empty string, or the
    /// formatted text of the cell it points to.
    fn series_name(&self, name: &str) -> Result<String, String> {
        if name.is_empty() {
            return Ok("".to_string());
        }
        if name.len() >= 2 && name.starts_with('"') && name.ends_with('"') {
            return Ok(name[1..name.len() - 1].replace("\"\"", "\""));
        }
        let range = self.resolve_range(name)?;
        self.get_formatted_cell_value(range.sheet, range.row1, range.column1)
    }

    /// Resolves the references of a data source into the values to plot.
    ///
    /// Hidden rows and columns, empty cells and errors are all reported as
    /// they are; the flags of the [`DataSource`] are for the consumer to apply.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::charts::{DataRange, DataSource, EmptyCells, Orientation};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 2, "Sales".to_string())?;
    /// model.set_user_input(0, 2, 1, "January".to_string())?;
    /// model.set_user_input(0, 2, 2, "100".to_string())?;
    /// model.set_user_input(0, 3, 1, "February".to_string())?;
    /// model.set_user_input(0, 3, 2, "=B2*2".to_string())?;
    /// model.evaluate();
    /// let data = model.get_chart_data(&DataSource {
    ///     range: DataRange::Range {
    ///         range: "Sheet1!$A$1:$B$3".to_string(),
    ///         orientation: Orientation::Columns,
    ///     },
    ///     empty_cells: EmptyCells::Gaps,
    ///     na_as_empty: false,
    ///     show_hidden_data: false,
    /// })?;
    /// assert_eq!(data.categories, vec!["January", "February"]);
    /// assert_eq!(data.series[0].name, "Sales");
    /// assert_eq!(data.series[0].values, vec![Some(100.0), Some(200.0)]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_chart_data(&self, source: &DataSource) -> Result<ChartData, String> {
        let (categories, series) = expand_data_range(&source.range);
        let category_range = self.resolve_range(&categories)?;
        let mut category_text = Vec::new();
        for (row, column) in category_range.cells() {
            category_text.push(self.get_formatted_cell_value(category_range.sheet, row, column)?);
        }
        let mut series_data = Vec::new();
        for series in &series {
            let range = self.resolve_range(&series.values)?;
            let mut values = Vec::new();
            for (row, column) in range.cells() {
                values.push(self.series_point(range.sheet, row, column)?);
            }
            series_data.push(SeriesData {
                name: self.series_name(&series.name)?,
                values,
                number_format: self
                    .get_style_for_cell(range.sheet, range.row1, range.column1)?
                    .num_fmt,
            });
        }
        Ok(ChartData {
            categories: category_text,
            series: series_data,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::charts::common::EmptyCells;

    fn source(range: DataRange) -> DataSource {
        DataSource {
            range,
            empty_cells: EmptyCells::Gaps,
            na_as_empty: false,
            show_hidden_data: false,
        }
    }

    fn model_with_table() -> Model<'static> {
        let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
        model.new_sheet();
        model.rename_sheet_by_index(1, "My Sheet").unwrap();
        let rows = [
            ["", "North", "South"],
            ["Q1", "10.5", "1"],
            ["Q2", "", "=1/0"],
            ["Q3", "text", "TRUE"],
            ["Q4", "=B2*2", "#N/A"],
        ];
        for (r, row) in rows.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                if !value.is_empty() {
                    model
                        .set_user_input(1, r as i32 + 1, c as i32 + 1, value.to_string())
                        .unwrap();
                }
            }
        }
        model.evaluate();
        model
    }

    #[test]
    fn expands_a_range_by_columns() {
        let (categories, series) = expand_data_range(&DataRange::Range {
            range: "Sheet1!$B$1:$E$6".to_string(),
            orientation: Orientation::Columns,
        });
        assert_eq!(categories, "Sheet1!$B$2:$B$6");
        assert_eq!(series.len(), 3);
        assert_eq!(series[0].name, "Sheet1!$C$1");
        assert_eq!(series[0].values, "Sheet1!$C$2:$C$6");
        assert_eq!(series[2].name, "Sheet1!$E$1");
        assert_eq!(series[2].values, "Sheet1!$E$2:$E$6");
    }

    #[test]
    fn expands_a_range_by_rows() {
        let (categories, series) = expand_data_range(&DataRange::Range {
            range: "'My Sheet'!B1:E6".to_string(),
            orientation: Orientation::Rows,
        });
        assert_eq!(categories, "'My Sheet'!$C$1:$E$1");
        assert_eq!(series.len(), 5);
        assert_eq!(series[0].name, "'My Sheet'!$B$2");
        assert_eq!(series[0].values, "'My Sheet'!$C$2:$E$2");
        assert_eq!(series[4].values, "'My Sheet'!$C$6:$E$6");
    }

    #[test]
    fn an_invalid_range_has_no_series() {
        let (categories, series) = expand_data_range(&DataRange::Range {
            range: "nonsense".to_string(),
            orientation: Orientation::Columns,
        });
        assert_eq!(categories, "");
        assert!(series.is_empty());
    }

    #[test]
    fn resolves_a_range_source() {
        let model = model_with_table();
        let data = model
            .get_chart_data(&source(DataRange::Range {
                range: "'My Sheet'!$A$1:$C$5".to_string(),
                orientation: Orientation::Columns,
            }))
            .unwrap();
        assert_eq!(data.categories, vec!["Q1", "Q2", "Q3", "Q4"]);
        assert_eq!(data.series.len(), 2);
        assert_eq!(data.series[0].name, "North");
        // empty cell: no point; text: zero; formula: its value
        assert_eq!(
            data.series[0].values,
            vec![Some(10.5), None, Some(0.0), Some(21.0)]
        );
        assert_eq!(data.series[1].name, "South");
        // errors have no point, booleans count as zero
        assert_eq!(
            data.series[1].values,
            vec![Some(1.0), None, Some(0.0), None]
        );
        assert_eq!(data.series[0].number_format, "general");
    }

    #[test]
    fn resolves_a_manual_source_with_literal_and_empty_names() {
        let model = model_with_table();
        let data = model
            .get_chart_data(&source(DataRange::Manual {
                categories: "'My Sheet'!$B$1:$C$1".to_string(),
                series: vec![
                    Series {
                        name: "\"Say \"\"hi\"\"\"".to_string(),
                        values: "'My Sheet'!$B$2:$C$2".to_string(),
                    },
                    Series {
                        name: "".to_string(),
                        values: "'My Sheet'!$B$4:$C$4".to_string(),
                    },
                ],
            }))
            .unwrap();
        assert_eq!(data.categories, vec!["North", "South"]);
        assert_eq!(data.series[0].name, "Say \"hi\"");
        assert_eq!(data.series[0].values, vec![Some(10.5), Some(1.0)]);
        assert_eq!(data.series[1].name, "");
        assert_eq!(data.series[1].values, vec![Some(0.0), Some(0.0)]);
    }

    #[test]
    fn unknown_sheets_and_bad_references_are_errors() {
        let model = model_with_table();
        assert!(model
            .get_chart_data(&source(DataRange::Range {
                range: "Nowhere!$A$1:$C$5".to_string(),
                orientation: Orientation::Columns,
            }))
            .is_err());
        assert!(model
            .get_chart_data(&source(DataRange::Manual {
                categories: "nonsense".to_string(),
                series: vec![],
            }))
            .is_err());
    }
}
