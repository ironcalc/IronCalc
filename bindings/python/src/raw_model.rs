use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;

use xlsx::base::cell::CellValue;
use xlsx::base::types::{SheetState, Style, Theme};
use xlsx::base::Model;
use xlsx::export::{save_to_icalc, save_to_xlsx};

use crate::types::PyCellType;
use crate::user_model::DefinedNameEntry;
use crate::{area, from_python, py_to_color, to_py_err, to_python};

fn cell_value_to_py(py: Python<'_>, value: CellValue) -> PyResult<Py<PyAny>> {
    match value {
        CellValue::None => Ok(py.None()),
        CellValue::String(s) => s.into_py_any(py),
        CellValue::Number(f) => f.into_py_any(py),
        CellValue::Boolean(b) => b.into_py_any(py),
    }
}

/// A workbook model implementing the "raw" low level API. Nothing is
/// evaluated automatically: you need to call `evaluate` yourself. There is no
/// undo/redo history and no diffs are produced.
#[pyclass(name = "Model")]
pub struct PyModel {
    pub(crate) model: Model<'static>,
}

#[pymethods]
impl PyModel {
    // Persistence

    /// Saves the workbook to an xlsx file
    pub fn save_to_xlsx(&self, file: &str) -> PyResult<()> {
        save_to_xlsx(&self.model, file).map_err(to_py_err)
    }

    /// Saves the workbook to a file in the internal binary ic format
    pub fn save_to_icalc(&self, file: &str) -> PyResult<()> {
        save_to_icalc(&self.model, file).map_err(to_py_err)
    }

    /// Returns the workbook as bytes in the internal binary ic format
    pub fn to_bytes(&self) -> Vec<u8> {
        self.model.to_bytes()
    }

    /// Evaluates the workbook
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    // Set values

    /// Sets an input in a cell, parsing it as a user would type it:
    /// "3.5" is a number, "Hello" a string, "=A1*2" a formula
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .set_user_input(sheet, row, column, value.to_string())
            .map_err(to_py_err)
    }

    /// Sets an array (spill) formula in the range
    pub fn set_user_array_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
        formula: &str,
    ) -> PyResult<()> {
        self.model
            .set_user_array_formula(sheet, row, column, width, height, formula)
            .map_err(to_py_err)
    }

    /// Sets a string value in a cell without input parsing
    pub fn update_cell_with_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .update_cell_with_text(sheet, row, column, value)
            .map_err(to_py_err)
    }

    /// Sets a number in a cell without input parsing
    pub fn update_cell_with_number(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: f64,
    ) -> PyResult<()> {
        self.model
            .update_cell_with_number(sheet, row, column, value)
            .map_err(to_py_err)
    }

    /// Sets a boolean in a cell without input parsing
    pub fn update_cell_with_bool(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: bool,
    ) -> PyResult<()> {
        self.model
            .update_cell_with_bool(sheet, row, column, value)
            .map_err(to_py_err)
    }

    /// Sets a formula (i.e. "=A1*2") in a cell
    pub fn update_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
    ) -> PyResult<()> {
        self.model
            .update_cell_with_formula(sheet, row, column, formula.to_string())
            .map_err(to_py_err)
    }

    /// Clears the contents of a single cell, keeping the formatting
    pub fn clear_cell_contents(&mut self, sheet: u32, row: i32, column: i32) -> PyResult<()> {
        self.model
            .range_clear_contents(&area(sheet, row, column, row, column))
            .map_err(to_py_err)
    }

    /// Clears the contents of all cells in the range, keeping the formatting
    pub fn range_clear_contents(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyResult<()> {
        self.model
            .range_clear_contents(&area(sheet, start_row, start_column, end_row, end_column))
            .map_err(to_py_err)
    }

    /// Clears contents and formatting of all cells in the range
    pub fn range_clear_all(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyResult<()> {
        self.model
            .range_clear_all(&area(sheet, start_row, start_column, end_row, end_column))
            .map_err(to_py_err)
    }

    // Get values

    /// Returns the content of a cell as the user would see it in the editor:
    /// the formula if there is one or the raw value otherwise
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_localized_cell_content(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns the formula of a cell, if any
    pub fn get_cell_formula(&self, sheet: u32, row: i32, column: i32) -> PyResult<Option<String>> {
        self.model
            .get_cell_formula(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns the value of a cell as a native Python value
    /// (None, str, float or bool)
    pub fn get_cell_value(
        &self,
        py: Python<'_>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Py<PyAny>> {
        let value = self
            .model
            .get_cell_value_by_index(sheet, row, column)
            .map_err(to_py_err)?;
        cell_value_to_py(py, value)
    }

    /// Returns the value of a cell referenced like "Sheet1!C4" as a native
    /// Python value (None, str, float or bool)
    pub fn get_cell_value_by_ref(&self, py: Python<'_>, cell_ref: &str) -> PyResult<Py<PyAny>> {
        let value = self
            .model
            .get_cell_value_by_ref(cell_ref)
            .map_err(to_py_err)?;
        cell_value_to_py(py, value)
    }

    /// Returns the type of the content of a cell
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyCellType> {
        self.model
            .get_cell_type(sheet, row, column)
            .map(|cell_type| cell_type.into())
            .map_err(to_py_err)
    }

    /// Returns the formatted value of a cell (i.e. "$ 5.75")
    pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns True if the cell is empty
    pub fn is_empty_cell(&self, sheet: u32, row: i32, column: i32) -> PyResult<bool> {
        self.model
            .is_empty_cell(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns all non-empty cells as a list of (sheet, row, column) tuples
    pub fn get_all_cells(&self) -> Vec<(u32, i32, i32)> {
        self.model
            .get_all_cells()
            .into_iter()
            .map(|c| (c.index, c.row, c.column))
            .collect()
    }

    /// Returns a markdown-like representation of the sheet, useful for debugging
    pub fn get_sheet_markup(&self, sheet: u32) -> PyResult<String> {
        self.model.get_sheet_markup(sheet).map_err(to_py_err)
    }

    // Styles

    /// Sets the style of a cell from a style dictionary
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        style: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let style: Style = from_python(style)?;
        self.model
            .set_cell_style(sheet, row, column, &style)
            .map_err(to_py_err)
    }

    /// Returns the style of a cell as a dictionary
    pub fn get_cell_style<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let style = self
            .model
            .get_style_for_cell(sheet, row, column)
            .map_err(to_py_err)?;
        to_python(py, &style)
    }

    /// Sets the default style for a whole column
    pub fn set_column_style(
        &mut self,
        sheet: u32,
        column: i32,
        style: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let style: Style = from_python(style)?;
        self.model
            .set_column_style(sheet, column, &style)
            .map_err(to_py_err)
    }

    /// Returns the default style of a column, if any
    pub fn get_column_style<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        column: i32,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let style = self
            .model
            .get_column_style(sheet, column)
            .map_err(to_py_err)?;
        match style {
            Some(style) => Ok(Some(to_python(py, &style)?)),
            None => Ok(None),
        }
    }

    pub fn delete_column_style(&mut self, sheet: u32, column: i32) -> PyResult<()> {
        self.model
            .delete_column_style(sheet, column)
            .map_err(to_py_err)
    }

    /// Sets the default style for a whole row
    pub fn set_row_style(
        &mut self,
        sheet: u32,
        row: i32,
        style: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let style: Style = from_python(style)?;
        self.model
            .set_row_style(sheet, row, &style)
            .map_err(to_py_err)
    }

    /// Returns the default style of a row, if any
    pub fn get_row_style<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        row: i32,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let style = self.model.get_row_style(sheet, row).map_err(to_py_err)?;
        match style {
            Some(style) => Ok(Some(to_python(py, &style)?)),
            None => Ok(None),
        }
    }

    pub fn delete_row_style(&mut self, sheet: u32, row: i32) -> PyResult<()> {
        self.model.delete_row_style(sheet, row).map_err(to_py_err)
    }

    // Rows and columns

    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .insert_rows(sheet, row, row_count)
            .map_err(to_py_err)
    }

    pub fn insert_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .insert_columns(sheet, column, column_count)
            .map_err(to_py_err)
    }

    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .delete_rows(sheet, row, row_count)
            .map_err(to_py_err)
    }

    pub fn delete_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .delete_columns(sheet, column, column_count)
            .map_err(to_py_err)
    }

    pub fn get_column_width(&self, sheet: u32, column: i32) -> PyResult<f64> {
        self.model
            .get_column_width(sheet, column)
            .map_err(to_py_err)
    }

    pub fn get_row_height(&self, sheet: u32, row: i32) -> PyResult<f64> {
        self.model.get_row_height(sheet, row).map_err(to_py_err)
    }

    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> PyResult<()> {
        self.model
            .set_column_width(sheet, column, width)
            .map_err(to_py_err)
    }

    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> PyResult<()> {
        self.model
            .set_row_height(sheet, row, height)
            .map_err(to_py_err)
    }

    pub fn set_column_hidden(&mut self, sheet: u32, column: i32, hidden: bool) -> PyResult<()> {
        self.model
            .set_column_hidden(sheet, column, hidden)
            .map_err(to_py_err)
    }

    pub fn set_row_hidden(&mut self, sheet: u32, row: i32, hidden: bool) -> PyResult<()> {
        self.model
            .set_row_hidden(sheet, row, hidden)
            .map_err(to_py_err)
    }

    pub fn is_column_hidden(&self, sheet: u32, column: i32) -> PyResult<bool> {
        self.model
            .is_column_hidden(sheet, column)
            .map_err(to_py_err)
    }

    pub fn is_row_hidden(&self, sheet: u32, row: i32) -> PyResult<bool> {
        self.model.is_row_hidden(sheet, row).map_err(to_py_err)
    }

    // Frozen rows/columns

    pub fn get_frozen_columns_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_columns_count(sheet)
            .map_err(to_py_err)
    }

    pub fn get_frozen_rows_count(&self, sheet: u32) -> PyResult<i32> {
        self.model.get_frozen_rows_count(sheet).map_err(to_py_err)
    }

    pub fn set_frozen_columns_count(&mut self, sheet: u32, column_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_columns(sheet, column_count)
            .map_err(to_py_err)
    }

    pub fn set_frozen_rows_count(&mut self, sheet: u32, row_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_rows(sheet, row_count)
            .map_err(to_py_err)
    }

    // Sheets

    /// Returns the list of sheets with their properties (name, state, color, ...)
    pub fn get_worksheets_properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        to_python(py, &self.model.get_worksheets_properties())
    }

    /// Sets the sheet tab color. Accepts None, "#RRGGBB" or [theme, tint]
    pub fn set_sheet_color(&mut self, sheet: u32, color: &Bound<'_, PyAny>) -> PyResult<()> {
        let color = py_to_color(color)?;
        self.model.set_sheet_color(sheet, &color).map_err(to_py_err)
    }

    /// Sets the sheet visibility state: "visible", "hidden" or "veryHidden"
    pub fn set_sheet_state(&mut self, sheet: u32, state: &str) -> PyResult<()> {
        let state = match state {
            "visible" => SheetState::Visible,
            "hidden" => SheetState::Hidden,
            "veryHidden" => SheetState::VeryHidden,
            _ => return Err(to_py_err(format!("Invalid sheet state: '{state}'"))),
        };
        self.model.set_sheet_state(sheet, state).map_err(to_py_err)
    }

    /// Adds a new sheet with the given name
    pub fn add_sheet(&mut self, sheet_name: &str) -> PyResult<()> {
        self.model.add_sheet(sheet_name).map_err(to_py_err)
    }

    /// Adds a new sheet with an automatically generated name
    pub fn new_sheet(&mut self) {
        self.model.new_sheet();
    }

    pub fn delete_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.delete_sheet(sheet).map_err(to_py_err)
    }

    pub fn rename_sheet(&mut self, sheet: u32, new_name: &str) -> PyResult<()> {
        self.model
            .rename_sheet_by_index(sheet, new_name)
            .map_err(to_py_err)
    }

    /// Returns the bounds of all non-empty cells as (min_row, max_row, min_column, max_column).
    /// For an empty sheet, returns (1, 1, 1, 1).
    pub fn get_sheet_dimensions(&self, sheet: u32) -> PyResult<(i32, i32, i32, i32)> {
        let worksheet = self.model.workbook.worksheet(sheet).map_err(to_py_err)?;
        let dimension = worksheet.dimension();
        Ok((
            dimension.min_row,
            dimension.max_row,
            dimension.min_column,
            dimension.max_column,
        ))
    }

    pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> PyResult<()> {
        self.model
            .set_show_grid_lines(sheet, show_grid_lines)
            .map_err(to_py_err)
    }

    // Defined names

    /// Returns the list of defined names as [{"name", "scope", "formula"}]
    pub fn get_defined_name_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let data: Vec<DefinedNameEntry> = self
            .model
            .get_defined_name_list()
            .into_iter()
            .map(|(name, scope, formula)| DefinedNameEntry {
                name,
                scope,
                formula,
            })
            .collect();
        to_python(py, &data)
    }

    /// Creates a new defined name. `scope` is a sheet index or None for global scope.
    #[pyo3(signature = (name, scope, formula))]
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> PyResult<()> {
        self.model
            .new_defined_name(name, scope, formula)
            .map_err(to_py_err)
    }

    #[pyo3(signature = (name, scope, new_name, new_scope, new_formula))]
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> PyResult<()> {
        self.model
            .update_defined_name(name, scope, new_name, new_scope, new_formula)
            .map_err(to_py_err)
    }

    #[pyo3(signature = (name, scope))]
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> PyResult<()> {
        self.model
            .delete_defined_name(name, scope)
            .map_err(to_py_err)
    }

    // Workbook properties

    /// Returns the workbook theme
    pub fn get_theme<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        to_python(py, &self.model.get_theme())
    }

    /// Sets the workbook theme
    pub fn set_theme(&mut self, theme: &Bound<'_, PyAny>) -> PyResult<()> {
        let theme: Theme = from_python(theme)?;
        self.model.set_theme(theme);
        Ok(())
    }

    pub fn get_timezone(&self) -> String {
        self.model.get_timezone()
    }

    pub fn set_timezone(&mut self, timezone: &str) -> PyResult<()> {
        self.model.set_timezone(timezone).map_err(to_py_err)
    }

    pub fn get_locale(&self) -> String {
        self.model.get_locale()
    }

    pub fn set_locale(&mut self, locale: &str) -> PyResult<()> {
        self.model.set_locale(locale).map_err(to_py_err)
    }

    pub fn get_language(&self) -> String {
        self.model.get_language()
    }

    pub fn set_language(&mut self, language: &str) -> PyResult<()> {
        self.model.set_language(language).map_err(to_py_err)
    }

    /// Returns locale dependent formatting settings (currency, date formats, ...)
    pub fn get_fmt_settings<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let settings: crate::FmtSettings = self.model.get_fmt_settings().into();
        to_python(py, &settings)
    }

    #[allow(clippy::panic)]
    pub fn test_panic(&self) -> PyResult<()> {
        panic!("This function panics for testing panic handling");
    }
}
