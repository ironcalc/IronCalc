use serde::Serialize;

use napi::{self, bindgen_prelude::Uint8Array, Env, Result, Unknown};

use ironcalc::{
  base::{
    cf_types::CfRuleInput,
    types::{Link, Style, StyleIncludes, Theme},
    BorderArea, ClipboardData, UserModel as BaseModel,
  },
  export::{save_to_icalc, save_to_xlsx},
  import::{load_from_icalc, load_from_xlsx},
};

use crate::{area, js_to_color, leak_str, to_js_error, CellType, DefinedName, FmtSettings};

#[derive(Serialize)]
struct NamedStyleEntry {
  name: String,
  style: Style,
}

/// A workbook model implementing the "user" API: the same high level API used
/// by the IronCalc web application. Every action evaluates the model, keeps
/// undo/redo history and produces diffs for collaboration.
#[napi]
pub struct UserModel {
  model: BaseModel<'static>,
}

#[napi]
impl UserModel {
  /// Creates an empty workbook. `locale`, `timezone` and `languageId` default
  /// to "en", "UTC" and "en".
  #[napi(constructor)]
  pub fn new(
    name: String,
    locale: Option<String>,
    timezone: Option<String>,
    language_id: Option<String>,
  ) -> Result<Self> {
    let name = leak_str(&name);
    let locale = leak_str(&locale.unwrap_or_else(|| "en".to_string()));
    let timezone = leak_str(&timezone.unwrap_or_else(|| "UTC".to_string()));
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = BaseModel::new_empty(name, locale, timezone, language_id).map_err(to_js_error)?;
    Ok(Self { model })
  }

  /// Creates a user model from bytes in the internal binary ic format
  #[napi(factory)]
  pub fn from_bytes(bytes: &[u8], language_id: Option<String>) -> Result<UserModel> {
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = BaseModel::from_bytes(bytes, language_id).map_err(to_js_error)?;
    Ok(UserModel { model })
  }

  /// Creates a user model from an xlsx file
  #[napi(factory)]
  pub fn from_xlsx(
    file_path: String,
    locale: Option<String>,
    timezone: Option<String>,
    language_id: Option<String>,
  ) -> Result<UserModel> {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let timezone = timezone.unwrap_or_else(|| "UTC".to_string());
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = load_from_xlsx(&file_path, &locale, &timezone, language_id).map_err(to_js_error)?;
    Ok(UserModel {
      model: BaseModel::from_model(model),
    })
  }

  /// Creates a user model from an icalc file
  #[napi(factory)]
  pub fn from_icalc(file_name: String, language_id: Option<String>) -> Result<UserModel> {
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = load_from_icalc(&file_name, language_id).map_err(to_js_error)?;
    Ok(UserModel {
      model: BaseModel::from_model(model),
    })
  }

  // Persistence

  /// Saves the workbook to an xlsx file
  #[napi]
  pub fn save_to_xlsx(&self, file: String) -> Result<()> {
    let model = self.model.get_model();
    save_to_xlsx(model, &file).map_err(to_js_error)
  }

  /// Saves the workbook to a file in the internal binary ic format
  #[napi]
  pub fn save_to_icalc(&self, file: String) -> Result<()> {
    let model = self.model.get_model();
    save_to_icalc(model, &file).map_err(to_js_error)
  }

  /// Returns the workbook as bytes in the internal binary ic format
  #[napi]
  pub fn to_bytes(&self) -> Uint8Array {
    Uint8Array::new(self.model.to_bytes())
  }

  // Collaboration

  /// Applies a list of diffs produced by another model's `flushSendQueue`
  #[napi]
  pub fn apply_external_diffs(&mut self, external_diffs: &[u8]) -> Result<()> {
    self
      .model
      .apply_external_diffs(external_diffs)
      .map_err(to_js_error)
  }

  /// Returns (and clears) the queue of diffs produced by local edits
  #[napi]
  pub fn flush_send_queue(&mut self) -> Uint8Array {
    Uint8Array::new(self.model.flush_send_queue())
  }

  // Undo / redo and evaluation

  #[napi]
  pub fn undo(&mut self) -> Result<()> {
    self.model.undo().map_err(to_js_error)
  }

  #[napi]
  pub fn redo(&mut self) -> Result<()> {
    self.model.redo().map_err(to_js_error)
  }

  #[napi]
  pub fn can_undo(&self) -> bool {
    self.model.can_undo()
  }

  #[napi]
  pub fn can_redo(&self) -> bool {
    self.model.can_redo()
  }

  /// Pauses automatic evaluation after each change
  #[napi]
  pub fn pause_evaluation(&mut self) {
    self.model.pause_evaluation()
  }

  /// Resumes automatic evaluation after each change
  #[napi]
  pub fn resume_evaluation(&mut self) {
    self.model.resume_evaluation()
  }

  /// Forces an evaluation of the workbook (only needed while paused)
  #[napi]
  pub fn evaluate(&mut self) {
    self.model.evaluate();
  }

  // Workbook properties

  /// Returns the name of the workbook
  #[napi]
  pub fn get_name(&self) -> String {
    self.model.get_name()
  }

  /// Sets the name of the workbook
  #[napi]
  pub fn set_name(&mut self, name: String) {
    self.model.set_name(&name);
  }

  #[napi]
  pub fn get_timezone(&self) -> String {
    self.model.get_timezone()
  }

  #[napi]
  pub fn set_timezone(&mut self, timezone: String) -> Result<()> {
    self.model.set_timezone(&timezone).map_err(to_js_error)
  }

  #[napi]
  pub fn get_locale(&self) -> String {
    self.model.get_locale()
  }

  #[napi]
  pub fn set_locale(&mut self, locale: String) -> Result<()> {
    self.model.set_locale(&locale).map_err(to_js_error)
  }

  #[napi]
  pub fn get_language(&self) -> String {
    self.model.get_language()
  }

  #[napi]
  pub fn set_language(&mut self, language: String) -> Result<()> {
    self.model.set_language(&language).map_err(to_js_error)
  }

  /// Returns locale dependent formatting settings (currency, date formats, ...)
  #[napi(ts_return_type = "FmtSettings")]
  pub fn get_fmt_settings<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    let settings: FmtSettings = self.model.get_fmt_settings().into();
    env.to_js_value(&settings).map_err(to_js_error)
  }

  /// Returns the workbook theme
  #[napi(ts_return_type = "IronCalcTheme")]
  pub fn get_theme<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    env
      .to_js_value(&self.model.get_theme())
      .map_err(to_js_error)
  }

  /// Sets the workbook theme
  #[napi]
  pub fn set_theme(
    &mut self,
    env: Env,
    #[napi(ts_arg_type = "IronCalcTheme")] theme: Unknown,
  ) -> Result<()> {
    let theme: Theme = env.from_js_value(theme).map_err(to_js_error)?;
    self.model.set_theme(theme);
    Ok(())
  }

  /// Resolves a color (null, "#RRGGBB" or [theme, tint]) to a CSS hex string
  /// using the current workbook theme. Returns "" for no color.
  #[napi]
  pub fn resolve_color(
    &self,
    env: Env,
    #[napi(ts_arg_type = "Color | null")] color: Option<Unknown>,
  ) -> Result<String> {
    let color = js_to_color(&env, color)?;
    Ok(self.model.resolve_color(&color))
  }

  // Cell values

  /// Sets the user input in a cell: a value like "3.5", "Hello" or a formula like "=A1*2"
  #[napi]
  pub fn set_user_input(&mut self, sheet: u32, row: i32, column: i32, value: String) -> Result<()> {
    self
      .model
      .set_user_input(sheet, row, column, &value)
      .map_err(to_js_error)
  }

  /// Sets an array (spill) formula in the range
  #[napi]
  pub fn set_user_array_formula(
    &mut self,
    sheet: u32,
    row: i32,
    column: i32,
    width: i32,
    height: i32,
    formula: String,
  ) -> Result<()> {
    self
      .model
      .set_user_array_formula(sheet, row, column, width, height, &formula)
      .map_err(to_js_error)
  }

  /// Returns the content of a cell as the user would see it in the editor:
  /// the formula if there is one or the raw value otherwise
  #[napi]
  pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_cell_content(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns the formatted value of a cell (i.e. "$ 5.75")
  #[napi]
  pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_formatted_cell_value(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns the type of the content of a cell
  #[napi]
  pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<CellType> {
    self
      .model
      .get_cell_type(sheet, row, column)
      .map(|cell_type| cell_type.into())
      .map_err(to_js_error)
  }

  /// Returns information about the array (spill) structure of a cell
  #[napi(ts_return_type = "CellArrayStructure")]
  pub fn get_cell_array_structure<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Unknown<'e>> {
    let cell_structure = self
      .model
      .get_cell_array_structure(sheet, row, column)
      .map_err(to_js_error)?;
    env.to_js_value(&cell_structure).map_err(to_js_error)
  }

  /// Returns the bounds of all non-empty cells as
  /// [minRow, maxRow, minColumn, maxColumn]. For an empty sheet returns [1, 1, 1, 1].
  #[napi(ts_return_type = "[number, number, number, number]")]
  pub fn get_sheet_dimensions<'e>(&self, env: &'e Env, sheet: u32) -> Result<Unknown<'e>> {
    let model = self.model.get_model();
    let worksheet = model.workbook.worksheet(sheet).map_err(to_js_error)?;
    let dimension = worksheet.dimension();
    env
      .to_js_value(&(
        dimension.min_row,
        dimension.max_row,
        dimension.min_column,
        dimension.max_column,
      ))
      .map_err(to_js_error)
  }

  // Ranges

  /// Clears contents and formatting of all cells in the range
  #[napi]
  pub fn range_clear_all(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    self
      .model
      .range_clear_all(&area(sheet, start_row, start_column, end_row, end_column))
      .map_err(to_js_error)
  }

  /// Clears the contents of all cells in the range, keeping the formatting
  #[napi]
  pub fn range_clear_contents(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    self
      .model
      .range_clear_contents(&area(sheet, start_row, start_column, end_row, end_column))
      .map_err(to_js_error)
  }

  /// Clears the formatting of all cells in the range, keeping the contents
  #[napi]
  pub fn range_clear_formatting(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    self
      .model
      .range_clear_formatting(&area(sheet, start_row, start_column, end_row, end_column))
      .map_err(to_js_error)
  }

  /// Extends the content of the source area downwards/upwards until `toRow`
  #[napi]
  pub fn auto_fill_rows(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
    to_row: i32,
  ) -> Result<()> {
    self
      .model
      .auto_fill_rows(
        &area(sheet, start_row, start_column, end_row, end_column),
        to_row,
      )
      .map_err(to_js_error)
  }

  /// Extends the content of the source area right/left until `toColumn`
  #[napi]
  pub fn auto_fill_columns(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
    to_column: i32,
  ) -> Result<()> {
    self
      .model
      .auto_fill_columns(
        &area(sheet, start_row, start_column, end_row, end_column),
        to_column,
      )
      .map_err(to_js_error)
  }

  // Sheets

  /// Adds a new sheet with an automatically generated name
  #[napi]
  pub fn new_sheet(&mut self) -> Result<()> {
    self.model.new_sheet().map_err(to_js_error)
  }

  #[napi]
  pub fn delete_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.delete_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn duplicate_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.duplicate_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn hide_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.hide_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn unhide_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.unhide_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn rename_sheet(&mut self, sheet: u32, name: String) -> Result<()> {
    self.model.rename_sheet(sheet, &name).map_err(to_js_error)
  }

  /// Moves the sheet to a new position in the list of sheets
  #[napi]
  pub fn move_sheet(&mut self, sheet: u32, new_index: u32) -> Result<()> {
    self.model.move_sheet(sheet, new_index).map_err(to_js_error)
  }

  /// Sets the sheet tab color. Accepts null, "#RRGGBB" or [theme, tint]
  #[napi]
  pub fn set_sheet_color(
    &mut self,
    env: Env,
    sheet: u32,
    #[napi(ts_arg_type = "Color | null")] color: Option<Unknown>,
  ) -> Result<()> {
    let color = js_to_color(&env, color)?;
    self
      .model
      .set_sheet_color(sheet, &color)
      .map_err(to_js_error)
  }

  /// Returns the list of sheets with their properties (name, state, color, ...)
  #[napi(ts_return_type = "Array<WorksheetProperties>")]
  pub fn get_worksheets_properties<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    env
      .to_js_value(&self.model.get_worksheets_properties())
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> Result<()> {
    self
      .model
      .set_show_grid_lines(sheet, show_grid_lines)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_show_grid_lines(&self, sheet: u32) -> Result<bool> {
    self.model.get_show_grid_lines(sheet).map_err(to_js_error)
  }

  // Links

  /// Returns the link attached to the cell or null if there isn't one.
  #[napi(ts_return_type = "Link | null")]
  pub fn get_cell_link<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Unknown<'e>> {
    let link = self
      .model
      .get_cell_link(sheet, row, column)
      .map_err(to_js_error)?;
    env.to_js_value(&link).map_err(to_js_error)
  }

  /// Attaches a link to a cell, replacing the existing one if there was one.
  /// The link is only metadata: the text displayed in the cell is the cell content.
  #[napi]
  pub fn set_cell_link(
    &mut self,
    env: Env,
    sheet: u32,
    row: i32,
    column: i32,
    #[napi(ts_arg_type = "Link")] link: Unknown,
  ) -> Result<()> {
    let link: Link = env.from_js_value(link).map_err(to_js_error)?;
    self
      .model
      .set_cell_link(sheet, row, column, link)
      .map_err(to_js_error)
  }

  /// Removes the link attached to the cell. It is not an error if the cell has no link.
  #[napi]
  pub fn delete_cell_link(&mut self, sheet: u32, row: i32, column: i32) -> Result<()> {
    self
      .model
      .delete_cell_link(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns all the links in the worksheet sorted by (row, column).
  #[napi(ts_return_type = "Array<CellLink>")]
  pub fn get_links<'e>(&self, env: &'e Env, sheet: u32) -> Result<Unknown<'e>> {
    let links = self.model.get_links_list(sheet).map_err(to_js_error)?;
    env.to_js_value(&links).map_err(to_js_error)
  }

  // Rows and columns

  #[napi]
  pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<()> {
    self
      .model
      .insert_rows(sheet, row, row_count)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn insert_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> Result<()> {
    self
      .model
      .insert_columns(sheet, column, column_count)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<()> {
    self
      .model
      .delete_rows(sheet, row, row_count)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn delete_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> Result<()> {
    self
      .model
      .delete_columns(sheet, column, column_count)
      .map_err(to_js_error)
  }

  /// Moves `columnCount` columns starting at `column` by `delta` positions
  #[napi]
  pub fn move_columns(
    &mut self,
    sheet: u32,
    column: i32,
    column_count: i32,
    delta: i32,
  ) -> Result<()> {
    self
      .model
      .move_columns_action(sheet, column, column_count, delta)
      .map_err(to_js_error)
  }

  /// Moves `rowCount` rows starting at `row` by `delta` positions
  #[napi]
  pub fn move_rows(&mut self, sheet: u32, row: i32, row_count: i32, delta: i32) -> Result<()> {
    self
      .model
      .move_rows_action(sheet, row, row_count, delta)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64> {
    self.model.get_row_height(sheet, row).map_err(to_js_error)
  }

  #[napi]
  pub fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64> {
    self
      .model
      .get_column_width(sheet, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_rows_height(
    &mut self,
    sheet: u32,
    row_start: i32,
    row_end: i32,
    height: f64,
  ) -> Result<()> {
    self
      .model
      .set_rows_height(sheet, row_start, row_end, height)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_columns_width(
    &mut self,
    sheet: u32,
    column_start: i32,
    column_end: i32,
    width: f64,
  ) -> Result<()> {
    self
      .model
      .set_columns_width(sheet, column_start, column_end, width)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_rows_hidden(
    &mut self,
    sheet: u32,
    row_start: i32,
    row_end: i32,
    hidden: bool,
  ) -> Result<()> {
    self
      .model
      .set_rows_hidden(sheet, row_start, row_end, hidden)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_columns_hidden(
    &mut self,
    sheet: u32,
    column_start: i32,
    column_end: i32,
    hidden: bool,
  ) -> Result<()> {
    self
      .model
      .set_columns_hidden(sheet, column_start, column_end, hidden)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32> {
    self.model.get_frozen_rows_count(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32> {
    self
      .model
      .get_frozen_columns_count(sheet)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_frozen_rows_count(&mut self, sheet: u32, count: i32) -> Result<()> {
    self
      .model
      .set_frozen_rows_count(sheet, count)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_frozen_columns_count(&mut self, sheet: u32, count: i32) -> Result<()> {
    self
      .model
      .set_frozen_columns_count(sheet, count)
      .map_err(to_js_error)
  }

  /// Returns the last non-empty column in the row before `column`, if any
  #[napi]
  pub fn get_last_non_empty_in_row_before_column(
    &self,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Option<i32>> {
    self
      .model
      .get_last_non_empty_in_row_before_column(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns the first non-empty column in the row after `column`, if any
  #[napi]
  pub fn get_first_non_empty_in_row_after_column(
    &self,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Option<i32>> {
    self
      .model
      .get_first_non_empty_in_row_after_column(sheet, row, column)
      .map_err(to_js_error)
  }

  // Styles

  /// Updates a single style property in all cells of the range.
  /// `stylePath` examples: "font.b", "font.color", "fill.color",
  /// "alignment.horizontal", "num_fmt". The value is always a string,
  /// i.e. "true", "#FF5566", "center", "#,##0.00".
  #[napi]
  #[allow(clippy::too_many_arguments)]
  pub fn update_range_style(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
    style_path: String,
    value: String,
  ) -> Result<()> {
    self
      .model
      .update_range_style(
        &area(sheet, start_row, start_column, end_row, end_column),
        &style_path,
        &value,
      )
      .map_err(to_js_error)
  }

  /// Returns the style of a cell
  #[napi(ts_return_type = "CellStyle")]
  pub fn get_cell_style<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Unknown<'e>> {
    let style = self
      .model
      .get_cell_style(sheet, row, column)
      .map_err(to_js_error)?;
    env.to_js_value(&style).map_err(to_js_error)
  }

  /// Returns the style of a cell together with any conditional formatting
  /// decorations (icon, data bar, rating)
  #[napi(ts_return_type = "ExtendedCellStyle")]
  pub fn get_extended_cell_style<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Unknown<'e>> {
    let style = self
      .model
      .get_extended_cell_style(sheet, row, column)
      .map_err(to_js_error)?;
    env.to_js_value(&style).map_err(to_js_error)
  }

  /// Pastes a matrix of styles (list of rows, each a list of style objects)
  /// starting at the selected cell
  #[napi]
  pub fn on_paste_styles(
    &mut self,
    env: Env,
    #[napi(ts_arg_type = "Array<Array<CellStyle>>")] styles: Unknown,
  ) -> Result<()> {
    let styles: Vec<Vec<Style>> = env.from_js_value(styles).map_err(to_js_error)?;
    self.model.on_paste_styles(&styles).map_err(to_js_error)
  }

  /// Applies a border to an area. `borderArea` is an object like
  /// {item: {style: "thin", color: "#000000"}, type: "All"}
  #[napi]
  #[allow(clippy::too_many_arguments)]
  pub fn set_area_with_border(
    &mut self,
    env: Env,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
    #[napi(ts_arg_type = "BorderArea")] border_area: Unknown,
  ) -> Result<()> {
    let border: BorderArea = env.from_js_value(border_area).map_err(to_js_error)?;
    self
      .model
      .set_area_with_border(
        &area(sheet, start_row, start_column, end_row, end_column),
        &border,
      )
      .map_err(to_js_error)
  }

  // Named styles

  /// Returns the names of all named styles in the workbook
  #[napi]
  pub fn get_named_style_list(&self) -> Vec<String> {
    self.model.get_named_style_list()
  }

  /// Returns the style associated with the named style
  #[napi(ts_return_type = "CellStyle")]
  pub fn get_named_style<'e>(&self, env: &'e Env, name: String) -> Result<Unknown<'e>> {
    let style = self.model.get_named_style(&name).map_err(to_js_error)?;
    env.to_js_value(&style).map_err(to_js_error)
  }

  /// Returns which formatting categories the named style includes
  #[napi(ts_return_type = "StyleIncludes")]
  pub fn get_named_style_includes<'e>(&self, env: &'e Env, name: String) -> Result<Unknown<'e>> {
    let includes = self
      .model
      .get_named_style_includes(&name)
      .map_err(to_js_error)?;
    env.to_js_value(&includes).map_err(to_js_error)
  }

  /// Creates a new named style from a style object. `includes` selects
  /// which formatting categories the style carries; null means all of them.
  #[napi]
  pub fn create_named_style(
    &mut self,
    env: Env,
    name: String,
    #[napi(ts_arg_type = "CellStyle")] style: Unknown,
    #[napi(ts_arg_type = "StyleIncludes | null")] includes: Option<Unknown>,
  ) -> Result<()> {
    let style: Style = env.from_js_value(style).map_err(to_js_error)?;
    let includes: StyleIncludes = match includes {
      Some(obj) => env.from_js_value(obj).map_err(to_js_error)?,
      None => StyleIncludes::default(),
    };
    self
      .model
      .create_named_style(&name, &style, includes)
      .map_err(to_js_error)
  }

  /// Updates (and possibly renames) an existing named style
  #[napi]
  pub fn update_named_style(
    &mut self,
    env: Env,
    name: String,
    new_name: String,
    #[napi(ts_arg_type = "CellStyle")] style: Unknown,
    #[napi(ts_arg_type = "StyleIncludes | null")] includes: Option<Unknown>,
  ) -> Result<()> {
    let style: Style = env.from_js_value(style).map_err(to_js_error)?;
    let includes: StyleIncludes = match includes {
      Some(obj) => env.from_js_value(obj).map_err(to_js_error)?,
      None => StyleIncludes::default(),
    };
    self
      .model
      .update_named_style(&name, &new_name, &style, includes)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn delete_named_style(&mut self, name: String) -> Result<()> {
    self.model.delete_named_style(&name).map_err(to_js_error)
  }

  /// Returns all Excel built-in named styles as a list of {name, style}
  #[napi(ts_return_type = "Array<NamedStyle>")]
  pub fn get_builtin_named_styles<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    let entries: Vec<NamedStyleEntry> = self
      .model
      .get_builtin_named_styles()
      .into_iter()
      .map(|(name, style)| NamedStyleEntry { name, style })
      .collect();
    env.to_js_value(&entries).map_err(to_js_error)
  }

  /// Applies a named style to the current selection.
  /// If the style is a built-in not yet in the workbook, it is added first.
  #[napi]
  pub fn on_apply_named_style(&mut self, name: String) -> Result<()> {
    self.model.on_apply_named_style(&name).map_err(to_js_error)
  }

  // Conditional formatting

  /// Returns the list of conditional formatting rules of the sheet
  #[napi(ts_return_type = "Array<ConditionalFormattingView>")]
  pub fn get_conditional_formatting_list<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
  ) -> Result<Unknown<'e>> {
    let list = self
      .model
      .get_conditional_formatting_list(sheet)
      .map_err(to_js_error)?;
    env.to_js_value(&list).map_err(to_js_error)
  }

  /// Adds a conditional formatting rule to a range like "A1:B10".
  /// `rule` is an object, i.e. {type: "CellIs", operator: "GreaterThan",
  /// formula: "5", format: {fill: {color: "#FF0000"}}}
  #[napi]
  pub fn add_conditional_formatting(
    &mut self,
    env: Env,
    sheet: u32,
    range: String,
    #[napi(ts_arg_type = "CfRuleInput")] rule: Unknown,
  ) -> Result<()> {
    let rule: CfRuleInput = env.from_js_value(rule).map_err(to_js_error)?;
    self
      .model
      .add_conditional_formatting(sheet, &range, rule)
      .map_err(to_js_error)
  }

  /// Updates the conditional formatting rule at `index`
  #[napi]
  pub fn update_conditional_formatting(
    &mut self,
    env: Env,
    sheet: u32,
    index: u32,
    new_range: String,
    #[napi(ts_arg_type = "CfRuleInput")] new_rule: Unknown,
  ) -> Result<()> {
    let new_rule: CfRuleInput = env.from_js_value(new_rule).map_err(to_js_error)?;
    self
      .model
      .update_conditional_formatting(sheet, index, &new_range, new_rule)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn delete_conditional_formatting(&mut self, sheet: u32, index: u32) -> Result<()> {
    self
      .model
      .delete_conditional_formatting(sheet, index)
      .map_err(to_js_error)
  }

  /// Returns the differential style (dxf) applied by the rule at `index`
  #[napi(ts_return_type = "Dxf")]
  pub fn get_dxf_for_conditional_formatting<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    index: u32,
  ) -> Result<Unknown<'e>> {
    let dxf = self
      .model
      .get_dxf_for_conditional_formatting(sheet, index)
      .map_err(to_js_error)?;
    env.to_js_value(&dxf).map_err(to_js_error)
  }

  #[napi]
  pub fn raise_conditional_formatting_priority(&mut self, sheet: u32, index: u32) -> Result<()> {
    self
      .model
      .raise_conditional_formatting_priority(sheet, index)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn lower_conditional_formatting_priority(&mut self, sheet: u32, index: u32) -> Result<()> {
    self
      .model
      .lower_conditional_formatting_priority(sheet, index)
      .map_err(to_js_error)
  }

  // Defined names

  /// Returns the list of defined names as [{name, scope, formula}].
  /// `scope` is omitted for globally scoped names.
  #[napi(ts_return_type = "Array<DefinedName>")]
  pub fn get_defined_name_list<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    let data: Vec<DefinedName> = self
      .model
      .get_defined_name_list()
      .into_iter()
      .map(|(name, scope, formula)| DefinedName {
        name,
        scope,
        formula,
      })
      .collect();
    env.to_js_value(&data).map_err(to_js_error)
  }

  /// Creates a new defined name. `scope` is a sheet index or null for global scope.
  #[napi]
  pub fn new_defined_name(
    &mut self,
    name: String,
    scope: Option<u32>,
    formula: String,
  ) -> Result<()> {
    self
      .model
      .new_defined_name(&name, scope, &formula)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn update_defined_name(
    &mut self,
    name: String,
    scope: Option<u32>,
    new_name: String,
    new_scope: Option<u32>,
    new_formula: String,
  ) -> Result<()> {
    self
      .model
      .update_defined_name(&name, scope, &new_name, new_scope, &new_formula)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn delete_defined_name(&mut self, name: String, scope: Option<u32>) -> Result<()> {
    self
      .model
      .delete_defined_name(&name, scope)
      .map_err(to_js_error)
  }

  /// Throws if the defined name is not valid
  #[napi]
  pub fn is_valid_defined_name(
    &mut self,
    name: String,
    scope: Option<u32>,
    formula: String,
  ) -> Result<()> {
    self
      .model
      .is_valid_defined_name(&name, scope, &formula)
      .map(|_| ())
      .map_err(to_js_error)
  }

  // Selection. Some operations (applying named styles, pasting styles,
  // copying to the clipboard) act on the current selection.

  #[napi]
  pub fn get_selected_sheet(&self) -> u32 {
    self.model.get_selected_sheet()
  }

  /// Returns the selected cell as [sheet, row, column]
  #[napi(ts_return_type = "[number, number, number]")]
  pub fn get_selected_cell(&self) -> Vec<i32> {
    let (sheet, row, column) = self.model.get_selected_cell();
    vec![sheet as i32, row, column]
  }

  /// Returns the selected view (sheet, selected range, top left visible cell, ...)
  #[napi(ts_return_type = "SelectedView")]
  pub fn get_selected_view<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    env
      .to_js_value(&self.model.get_selected_view())
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_selected_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.set_selected_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn set_selected_cell(&mut self, row: i32, column: i32) -> Result<()> {
    self
      .model
      .set_selected_cell(row, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_selected_range(
    &mut self,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    self
      .model
      .set_selected_range(start_row, start_column, end_row, end_column)
      .map_err(to_js_error)
  }

  // Clipboard

  /// Copies the selected area, returning an object with the csv text,
  /// the internal data and the copied range
  #[napi(ts_return_type = "Clipboard")]
  pub fn copy_to_clipboard<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    let data = self.model.copy_to_clipboard().map_err(to_js_error)?;
    env.to_js_value(&data).map_err(to_js_error)
  }

  /// Pastes data copied with `copyToClipboard` into the selected area
  #[napi]
  pub fn paste_from_clipboard(
    &mut self,
    env: Env,
    source_sheet: u32,
    #[napi(ts_arg_type = "[number, number, number, number]")] source_range: Unknown,
    #[napi(ts_arg_type = "ClipboardData")] clipboard: Unknown,
    is_cut: bool,
  ) -> Result<()> {
    let source_range: (i32, i32, i32, i32) =
      env.from_js_value(source_range).map_err(to_js_error)?;
    let clipboard: ClipboardData = env.from_js_value(clipboard).map_err(to_js_error)?;
    self
      .model
      .paste_from_clipboard(source_sheet, source_range, &clipboard, is_cut)
      .map_err(to_js_error)
  }

  /// Pastes a csv string starting at the top-left corner of the given area
  #[napi]
  pub fn paste_csv_string(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
    csv: String,
  ) -> Result<()> {
    self
      .model
      .paste_csv_string(
        &area(sheet, start_row, start_column, end_row, end_column),
        &csv,
      )
      .map_err(to_js_error)
  }
}
