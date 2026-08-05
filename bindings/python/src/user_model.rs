use pyo3::prelude::*;

use xlsx::base::cf_types::CfRuleInput;
use xlsx::base::types::{Link, Style, StyleIncludes, Theme};
use xlsx::base::{BorderArea, ClipboardData, UserModel};
use xlsx::export::{save_to_icalc, save_to_xlsx};
use xlsx::import;

use crate::types::PyCellType;
use crate::{area, from_python, leak_str, py_to_color, to_py_err, to_python};

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct DefinedNameEntry {
    pub name: String,
    pub scope: Option<u32>,
    pub formula: String,
}

#[derive(Serialize)]
struct NamedStyleEntry {
    name: String,
    style: Style,
}

/// A workbook model implementing the "user" API: the same high level API used
/// by the IronCalc web application. Every action evaluates the model, keeps
/// undo/redo history and produces diffs for collaboration.
#[pyclass(name = "UserModel")]
pub struct PyUserModel {
    pub model: UserModel<'static>,
}

#[pymethods]
impl PyUserModel {
    #[new]
    #[pyo3(signature = (name, locale="en", tz="UTC", language_id="en"))]
    pub fn new(name: &str, locale: &str, tz: &str, language_id: &str) -> PyResult<Self> {
        let name = leak_str(name);
        let locale = leak_str(locale);
        let tz = leak_str(tz);
        let language_id = leak_str(language_id);
        let model = UserModel::new_empty(name, locale, tz, language_id).map_err(to_py_err)?;
        Ok(PyUserModel { model })
    }

    /// Creates a user model from bytes in the internal binary ic format
    #[staticmethod]
    #[pyo3(signature = (bytes, language_id="en"))]
    pub fn from_bytes(bytes: &[u8], language_id: &str) -> PyResult<Self> {
        let language_id = leak_str(language_id);
        let model = UserModel::from_bytes(bytes, language_id).map_err(to_py_err)?;
        Ok(PyUserModel { model })
    }

    /// Creates a user model from an xlsx file
    #[staticmethod]
    #[pyo3(signature = (file_path, locale="en", tz="UTC", language_id="en"))]
    pub fn from_xlsx(file_path: &str, locale: &str, tz: &str, language_id: &str) -> PyResult<Self> {
        let language_id = leak_str(language_id);
        let model =
            import::load_from_xlsx(file_path, locale, tz, language_id).map_err(to_py_err)?;
        Ok(PyUserModel {
            model: UserModel::from_model(model),
        })
    }

    /// Creates a user model from an icalc file
    #[staticmethod]
    #[pyo3(signature = (file_name, language_id="en"))]
    pub fn from_icalc(file_name: &str, language_id: &str) -> PyResult<Self> {
        let language_id = leak_str(language_id);
        let model = import::load_from_icalc(file_name, language_id).map_err(to_py_err)?;
        Ok(PyUserModel {
            model: UserModel::from_model(model),
        })
    }

    // Persistence

    /// Saves the workbook to an xlsx file
    pub fn save_to_xlsx(&self, file: &str) -> PyResult<()> {
        let model = self.model.get_model();
        save_to_xlsx(model, file).map_err(to_py_err)
    }

    /// Saves the workbook to a file in the internal binary ic format
    pub fn save_to_icalc(&self, file: &str) -> PyResult<()> {
        let model = self.model.get_model();
        save_to_icalc(model, file).map_err(to_py_err)
    }

    /// Returns the workbook as bytes in the internal binary ic format
    pub fn to_bytes(&self) -> Vec<u8> {
        self.model.to_bytes()
    }

    // Collaboration

    /// Applies a list of diffs produced by another model's `flush_send_queue`
    pub fn apply_external_diffs(&mut self, external_diffs: &[u8]) -> PyResult<()> {
        self.model
            .apply_external_diffs(external_diffs)
            .map_err(to_py_err)
    }

    /// Returns (and clears) the queue of diffs produced by local edits
    pub fn flush_send_queue(&mut self) -> Vec<u8> {
        self.model.flush_send_queue()
    }

    // Undo / redo and evaluation

    pub fn undo(&mut self) -> PyResult<()> {
        self.model.undo().map_err(to_py_err)
    }

    pub fn redo(&mut self) -> PyResult<()> {
        self.model.redo().map_err(to_py_err)
    }

    pub fn can_undo(&self) -> bool {
        self.model.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.model.can_redo()
    }

    /// Pauses automatic evaluation after each change
    pub fn pause_evaluation(&mut self) {
        self.model.pause_evaluation()
    }

    /// Resumes automatic evaluation after each change
    pub fn resume_evaluation(&mut self) {
        self.model.resume_evaluation()
    }

    /// Forces an evaluation of the workbook (only needed while paused)
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    // Workbook properties

    /// Returns the name of the workbook
    pub fn get_name(&self) -> String {
        self.model.get_name()
    }

    /// Sets the name of the workbook
    pub fn set_name(&mut self, name: &str) {
        self.model.set_name(name)
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

    /// Resolves a color (None, "#RRGGBB" or [theme, tint]) to a CSS hex string
    /// using the current workbook theme. Returns "" for no color.
    pub fn resolve_color(&self, color: &Bound<'_, PyAny>) -> PyResult<String> {
        let color = py_to_color(color)?;
        Ok(self.model.resolve_color(&color))
    }

    // Cell values

    /// Sets the user input in a cell: a value like "3.5", "Hello" or a formula like "=A1*2"
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .set_user_input(sheet, row, column, value)
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

    /// Returns the content of a cell as the user would see it in the editor:
    /// the formula if there is one or the raw value otherwise
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_cell_content(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns the formatted value of a cell (i.e. "$ 5.75")
    pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns the type of the content of a cell
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyCellType> {
        self.model
            .get_cell_type(sheet, row, column)
            .map(|cell_type| cell_type.into())
            .map_err(to_py_err)
    }

    /// Returns information about the array (spill) structure of a cell
    pub fn get_cell_array_structure<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let cell_structure = self
            .model
            .get_cell_array_structure(sheet, row, column)
            .map_err(to_py_err)?;
        to_python(py, &cell_structure)
    }

    /// Returns the bounds of all non-empty cells as (min_row, max_row, min_column, max_column).
    /// For an empty sheet, returns (1, 1, 1, 1).
    pub fn get_sheet_dimensions(&self, sheet: u32) -> PyResult<(i32, i32, i32, i32)> {
        let model = self.model.get_model();
        let worksheet = model.workbook.worksheet(sheet).map_err(to_py_err)?;
        let dimension = worksheet.dimension();
        Ok((
            dimension.min_row,
            dimension.max_row,
            dimension.min_column,
            dimension.max_column,
        ))
    }

    // Ranges

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

    /// Clears the formatting of all cells in the range, keeping the contents
    pub fn range_clear_formatting(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyResult<()> {
        self.model
            .range_clear_formatting(&area(sheet, start_row, start_column, end_row, end_column))
            .map_err(to_py_err)
    }

    /// Extends the content of the source area downwards/upwards until `to_row`
    pub fn auto_fill_rows(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
        to_row: i32,
    ) -> PyResult<()> {
        self.model
            .auto_fill_rows(
                &area(sheet, start_row, start_column, end_row, end_column),
                to_row,
            )
            .map_err(to_py_err)
    }

    /// Extends the content of the source area right/left until `to_column`
    pub fn auto_fill_columns(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
        to_column: i32,
    ) -> PyResult<()> {
        self.model
            .auto_fill_columns(
                &area(sheet, start_row, start_column, end_row, end_column),
                to_column,
            )
            .map_err(to_py_err)
    }

    // Sheets

    /// Adds a new sheet with an automatically generated name
    pub fn new_sheet(&mut self) -> PyResult<()> {
        self.model.new_sheet().map_err(to_py_err)
    }

    pub fn delete_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.delete_sheet(sheet).map_err(to_py_err)
    }

    pub fn duplicate_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.duplicate_sheet(sheet).map_err(to_py_err)
    }

    pub fn hide_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.hide_sheet(sheet).map_err(to_py_err)
    }

    pub fn unhide_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.unhide_sheet(sheet).map_err(to_py_err)
    }

    pub fn rename_sheet(&mut self, sheet: u32, name: &str) -> PyResult<()> {
        self.model.rename_sheet(sheet, name).map_err(to_py_err)
    }

    /// Moves the sheet to a new position in the list of sheets
    pub fn move_sheet(&mut self, sheet: u32, new_index: u32) -> PyResult<()> {
        self.model.move_sheet(sheet, new_index).map_err(to_py_err)
    }

    /// Sets the sheet tab color. Accepts None, "#RRGGBB" or [theme, tint]
    pub fn set_sheet_color(&mut self, sheet: u32, color: &Bound<'_, PyAny>) -> PyResult<()> {
        let color = py_to_color(color)?;
        self.model.set_sheet_color(sheet, &color).map_err(to_py_err)
    }

    /// Returns the list of sheets with their properties (name, state, color, ...)
    pub fn get_worksheets_properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        to_python(py, &self.model.get_worksheets_properties())
    }

    pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> PyResult<()> {
        self.model
            .set_show_grid_lines(sheet, show_grid_lines)
            .map_err(to_py_err)
    }

    pub fn get_show_grid_lines(&self, sheet: u32) -> PyResult<bool> {
        self.model.get_show_grid_lines(sheet).map_err(to_py_err)
    }

    // Links

    /// Returns the link attached to the cell as a dict or None if there isn't one.
    /// External links: {"type": "External", "target": "...", "tooltip": ...}
    /// Internal links: {"type": "Internal", "location": "...", "tooltip": ...}
    pub fn get_cell_link<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let link = self
            .model
            .get_cell_link(sheet, row, column)
            .map_err(to_py_err)?;
        to_python(py, &link)
    }

    /// Attaches a link to a cell, replacing the existing one if there was one.
    /// If `label` is given it becomes the content of the cell (the displayed text).
    /// A new link also applies the link style (underline + theme hyperlink color)
    /// to the cell. The whole operation is a single undo step.
    #[pyo3(signature = (sheet, row, column, link, label=None))]
    pub fn set_cell_link(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        link: &Bound<'_, PyAny>,
        label: Option<&str>,
    ) -> PyResult<()> {
        let link: Link = from_python(link)?;
        self.model
            .set_cell_link(sheet, row, column, link, label)
            .map_err(to_py_err)
    }

    /// Removes the link attached to the cell. It is not an error if the cell has no link.
    pub fn delete_cell_link(&mut self, sheet: u32, row: i32, column: i32) -> PyResult<()> {
        self.model
            .delete_cell_link(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns all the links in the worksheet sorted by (row, column), each entry
    /// a dict with the cell and the link fields flattened:
    /// {"row": 2, "column": 2, "type": "External", "target": "...", "tooltip": None}
    pub fn get_links<'py>(&self, py: Python<'py>, sheet: u32) -> PyResult<Bound<'py, PyAny>> {
        let links = self.model.get_links_list(sheet).map_err(to_py_err)?;
        to_python(py, &links)
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

    /// Moves `column_count` columns starting at `column` by `delta` positions
    pub fn move_columns(
        &mut self,
        sheet: u32,
        column: i32,
        column_count: i32,
        delta: i32,
    ) -> PyResult<()> {
        self.model
            .move_columns_action(sheet, column, column_count, delta)
            .map_err(to_py_err)
    }

    /// Moves `row_count` rows starting at `row` by `delta` positions
    pub fn move_rows(&mut self, sheet: u32, row: i32, row_count: i32, delta: i32) -> PyResult<()> {
        self.model
            .move_rows_action(sheet, row, row_count, delta)
            .map_err(to_py_err)
    }

    pub fn get_row_height(&self, sheet: u32, row: i32) -> PyResult<f64> {
        self.model.get_row_height(sheet, row).map_err(to_py_err)
    }

    pub fn get_column_width(&self, sheet: u32, column: i32) -> PyResult<f64> {
        self.model
            .get_column_width(sheet, column)
            .map_err(to_py_err)
    }

    pub fn set_rows_height(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        height: f64,
    ) -> PyResult<()> {
        self.model
            .set_rows_height(sheet, row_start, row_end, height)
            .map_err(to_py_err)
    }

    pub fn set_columns_width(
        &mut self,
        sheet: u32,
        column_start: i32,
        column_end: i32,
        width: f64,
    ) -> PyResult<()> {
        self.model
            .set_columns_width(sheet, column_start, column_end, width)
            .map_err(to_py_err)
    }

    pub fn set_rows_hidden(
        &mut self,
        sheet: u32,
        row_start: i32,
        row_end: i32,
        hidden: bool,
    ) -> PyResult<()> {
        self.model
            .set_rows_hidden(sheet, row_start, row_end, hidden)
            .map_err(to_py_err)
    }

    pub fn set_columns_hidden(
        &mut self,
        sheet: u32,
        column_start: i32,
        column_end: i32,
        hidden: bool,
    ) -> PyResult<()> {
        self.model
            .set_columns_hidden(sheet, column_start, column_end, hidden)
            .map_err(to_py_err)
    }

    pub fn get_frozen_rows_count(&self, sheet: u32) -> PyResult<i32> {
        self.model.get_frozen_rows_count(sheet).map_err(to_py_err)
    }

    pub fn get_frozen_columns_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_columns_count(sheet)
            .map_err(to_py_err)
    }

    pub fn set_frozen_rows_count(&mut self, sheet: u32, count: i32) -> PyResult<()> {
        self.model
            .set_frozen_rows_count(sheet, count)
            .map_err(to_py_err)
    }

    pub fn set_frozen_columns_count(&mut self, sheet: u32, count: i32) -> PyResult<()> {
        self.model
            .set_frozen_columns_count(sheet, count)
            .map_err(to_py_err)
    }

    /// Returns the last non-empty column in the row before `column`, if any
    pub fn get_last_non_empty_in_row_before_column(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Option<i32>> {
        self.model
            .get_last_non_empty_in_row_before_column(sheet, row, column)
            .map_err(to_py_err)
    }

    /// Returns the first non-empty column in the row after `column`, if any
    pub fn get_first_non_empty_in_row_after_column(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Option<i32>> {
        self.model
            .get_first_non_empty_in_row_after_column(sheet, row, column)
            .map_err(to_py_err)
    }

    // Styles

    /// Updates a single style property in all cells of the range.
    /// `style_path` examples: "font.b", "font.color", "fill.color",
    /// "alignment.horizontal", "num_fmt". The value is always a string,
    /// i.e. "true", "#FF5566", "center", "#,##0.00".
    #[allow(clippy::too_many_arguments)]
    pub fn update_range_style(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
        style_path: &str,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .update_range_style(
                &area(sheet, start_row, start_column, end_row, end_column),
                style_path,
                value,
            )
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
            .get_cell_style(sheet, row, column)
            .map_err(to_py_err)?;
        to_python(py, &style)
    }

    /// Returns the style of a cell together with the name of the named style
    /// it is based on, if any
    pub fn get_extended_cell_style<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let style = self
            .model
            .get_extended_cell_style(sheet, row, column)
            .map_err(to_py_err)?;
        to_python(py, &style)
    }

    /// Pastes a matrix of styles (list of rows, each a list of style dictionaries)
    /// starting at the selected cell
    pub fn on_paste_styles(&mut self, styles: &Bound<'_, PyAny>) -> PyResult<()> {
        let styles: Vec<Vec<Style>> = from_python(styles)?;
        self.model.on_paste_styles(&styles).map_err(to_py_err)
    }

    /// Applies a border to an area. `border_area` is a dictionary like
    /// {"item": {"style": "thin", "color": "#000000"}, "type": "All"}
    pub fn set_area_with_border(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
        border_area: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let border: BorderArea = from_python(border_area)?;
        self.model
            .set_area_with_border(
                &area(sheet, start_row, start_column, end_row, end_column),
                &border,
            )
            .map_err(to_py_err)
    }

    // Named styles

    /// Returns the names of all named styles in the workbook
    pub fn get_named_style_list(&self) -> Vec<String> {
        self.model.get_named_style_list()
    }

    /// Returns the style associated with the named style
    pub fn get_named_style<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyAny>> {
        let style = self.model.get_named_style(name).map_err(to_py_err)?;
        to_python(py, &style)
    }

    /// Returns which formatting categories the named style includes
    pub fn get_named_style_includes<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let includes = self
            .model
            .get_named_style_includes(name)
            .map_err(to_py_err)?;
        to_python(py, &includes)
    }

    /// Creates a new named style from a style dictionary. `includes` selects
    /// which formatting categories the style carries; None means all of them.
    #[pyo3(signature = (name, style, includes=None))]
    pub fn create_named_style(
        &mut self,
        name: &str,
        style: &Bound<'_, PyAny>,
        includes: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let style: Style = from_python(style)?;
        let includes: StyleIncludes = match includes {
            Some(obj) => from_python(obj)?,
            None => StyleIncludes::default(),
        };
        self.model
            .create_named_style(name, &style, includes)
            .map_err(to_py_err)
    }

    /// Updates (and possibly renames) an existing named style
    #[pyo3(signature = (name, new_name, style, includes=None))]
    pub fn update_named_style(
        &mut self,
        name: &str,
        new_name: &str,
        style: &Bound<'_, PyAny>,
        includes: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let style: Style = from_python(style)?;
        let includes: StyleIncludes = match includes {
            Some(obj) => from_python(obj)?,
            None => StyleIncludes::default(),
        };
        self.model
            .update_named_style(name, new_name, &style, includes)
            .map_err(to_py_err)
    }

    pub fn delete_named_style(&mut self, name: &str) -> PyResult<()> {
        self.model.delete_named_style(name).map_err(to_py_err)
    }

    /// Returns all Excel built-in named styles as a list of {"name", "style"}
    pub fn get_builtin_named_styles<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let entries: Vec<NamedStyleEntry> = self
            .model
            .get_builtin_named_styles()
            .into_iter()
            .map(|(name, style)| NamedStyleEntry { name, style })
            .collect();
        to_python(py, &entries)
    }

    /// Applies a named style to the current selection.
    /// If the style is a built-in not yet in the workbook, it is added first.
    pub fn on_apply_named_style(&mut self, name: &str) -> PyResult<()> {
        self.model.on_apply_named_style(name).map_err(to_py_err)
    }

    // Conditional formatting

    /// Returns the list of conditional formatting rules of the sheet
    pub fn get_conditional_formatting_list<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let list = self
            .model
            .get_conditional_formatting_list(sheet)
            .map_err(to_py_err)?;
        to_python(py, &list)
    }

    /// Adds a conditional formatting rule to a range like "A1:B10".
    /// `rule` is a dictionary, i.e. {"type": "cellIs", "operator": "greaterThan",
    /// "formula": "5", "dxf": {"fill": {"color": "#FF0000"}}}
    pub fn add_conditional_formatting(
        &mut self,
        sheet: u32,
        range: &str,
        rule: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let rule: CfRuleInput = from_python(rule)?;
        self.model
            .add_conditional_formatting(sheet, range, rule)
            .map_err(to_py_err)
    }

    /// Updates the conditional formatting rule at `index`
    pub fn update_conditional_formatting(
        &mut self,
        sheet: u32,
        index: u32,
        new_range: &str,
        new_rule: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let new_rule: CfRuleInput = from_python(new_rule)?;
        self.model
            .update_conditional_formatting(sheet, index, new_range, new_rule)
            .map_err(to_py_err)
    }

    pub fn delete_conditional_formatting(&mut self, sheet: u32, index: u32) -> PyResult<()> {
        self.model
            .delete_conditional_formatting(sheet, index)
            .map_err(to_py_err)
    }

    /// Returns the differential style (dxf) applied by the rule at `index`
    pub fn get_dxf_for_conditional_formatting<'py>(
        &self,
        py: Python<'py>,
        sheet: u32,
        index: u32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let dxf = self
            .model
            .get_dxf_for_conditional_formatting(sheet, index)
            .map_err(to_py_err)?;
        to_python(py, &dxf)
    }

    pub fn raise_conditional_formatting_priority(
        &mut self,
        sheet: u32,
        index: u32,
    ) -> PyResult<()> {
        self.model
            .raise_conditional_formatting_priority(sheet, index)
            .map_err(to_py_err)
    }

    pub fn lower_conditional_formatting_priority(
        &mut self,
        sheet: u32,
        index: u32,
    ) -> PyResult<()> {
        self.model
            .lower_conditional_formatting_priority(sheet, index)
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

    #[pyo3(signature = (name, scope, formula))]
    pub fn is_valid_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> PyResult<()> {
        self.model
            .is_valid_defined_name(name, scope, formula)
            .map(|_| ())
            .map_err(to_py_err)
    }

    // Selection. Some operations (applying named styles, pasting styles,
    // copying to the clipboard) act on the current selection.

    pub fn get_selected_sheet(&self) -> u32 {
        self.model.get_selected_sheet()
    }

    /// Returns the selected cell as (sheet, row, column)
    pub fn get_selected_cell(&self) -> (u32, i32, i32) {
        self.model.get_selected_cell()
    }

    /// Returns the selected view (sheet, selected range, top left visible cell, ...)
    pub fn get_selected_view<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        to_python(py, &self.model.get_selected_view())
    }

    pub fn set_selected_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model.set_selected_sheet(sheet).map_err(to_py_err)
    }

    pub fn set_selected_cell(&mut self, row: i32, column: i32) -> PyResult<()> {
        self.model.set_selected_cell(row, column).map_err(to_py_err)
    }

    pub fn set_selected_range(
        &mut self,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> PyResult<()> {
        self.model
            .set_selected_range(start_row, start_column, end_row, end_column)
            .map_err(to_py_err)
    }

    // Clipboard

    /// Copies the selected area, returning a dictionary with the tsv text,
    /// the internal data and the copied range
    pub fn copy_to_clipboard<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let data = self.model.copy_to_clipboard().map_err(to_py_err)?;
        to_python(py, &data)
    }

    /// Pastes data copied with `copy_to_clipboard` into the selected area
    pub fn paste_from_clipboard(
        &mut self,
        source_sheet: u32,
        source_range: (i32, i32, i32, i32),
        clipboard: &Bound<'_, PyAny>,
        is_cut: bool,
    ) -> PyResult<()> {
        let clipboard: ClipboardData = from_python(clipboard)?;
        self.model
            .paste_from_clipboard(source_sheet, source_range, &clipboard, is_cut)
            .map_err(to_py_err)
    }

    /// Pastes a csv string starting at the top-left corner of the given area
    pub fn paste_csv_string(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
        csv: &str,
    ) -> PyResult<()> {
        self.model
            .paste_csv_string(
                &area(sheet, start_row, start_column, end_row, end_column),
                csv,
            )
            .map_err(to_py_err)
    }
}
