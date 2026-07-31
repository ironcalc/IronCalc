use napi::{self, bindgen_prelude::*, Env, Result, Unknown};

use ironcalc::{
  base::{
    cell::CellValue,
    types::{SheetState, Style, Theme},
    Model as BaseModel,
  },
  export::{save_to_icalc, save_to_xlsx},
  import::{load_from_icalc, load_from_xlsx},
};

use crate::{area, js_to_color, leak_str, to_js_error, CellType, DefinedName, FmtSettings};

/// A workbook model implementing the "raw" low level API. Nothing is
/// evaluated automatically: you need to call `evaluate` yourself. There is no
/// undo/redo history and no diffs are produced.
#[napi]
pub struct Model {
  pub(crate) model: BaseModel<'static>,
}

#[napi]
impl Model {
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

  /// Creates a model from an xlsx file
  #[napi(factory)]
  pub fn from_xlsx(
    file_path: String,
    locale: Option<String>,
    timezone: Option<String>,
    language_id: Option<String>,
  ) -> Result<Model> {
    let locale = locale.unwrap_or_else(|| "en".to_string());
    let timezone = timezone.unwrap_or_else(|| "UTC".to_string());
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = load_from_xlsx(&file_path, &locale, &timezone, language_id).map_err(to_js_error)?;
    Ok(Self { model })
  }

  /// Creates a model from an icalc file
  #[napi(factory)]
  pub fn from_icalc(file_name: String, language_id: Option<String>) -> Result<Model> {
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = load_from_icalc(&file_name, language_id).map_err(to_js_error)?;
    Ok(Self { model })
  }

  /// Creates a model from bytes in the internal binary ic format.
  /// This is the same format produced by `saveToIcalc` and `toBytes`.
  #[napi(factory)]
  pub fn from_bytes(bytes: &[u8], language_id: Option<String>) -> Result<Model> {
    let language_id = leak_str(&language_id.unwrap_or_else(|| "en".to_string()));
    let model = BaseModel::from_bytes(bytes, language_id).map_err(to_js_error)?;
    Ok(Self { model })
  }

  // Persistence

  /// Saves the workbook to an xlsx file
  #[napi]
  pub fn save_to_xlsx(&self, file: String) -> Result<()> {
    save_to_xlsx(&self.model, &file).map_err(to_js_error)
  }

  /// Saves the workbook to a file in the internal binary ic format
  #[napi]
  pub fn save_to_icalc(&self, file: String) -> Result<()> {
    save_to_icalc(&self.model, &file).map_err(to_js_error)
  }

  /// Returns the workbook as bytes in the internal binary ic format
  #[napi]
  pub fn to_bytes(&self) -> Uint8Array {
    Uint8Array::new(self.model.to_bytes())
  }

  /// Evaluates the workbook
  #[napi]
  pub fn evaluate(&mut self) {
    self.model.evaluate();
  }

  // Set values

  /// Sets an input in a cell, parsing it as a user would type it:
  /// "3.5" is a number, "Hello" a string, "=A1*2" a formula
  #[napi]
  pub fn set_user_input(&mut self, sheet: u32, row: i32, column: i32, value: String) -> Result<()> {
    self
      .model
      .set_user_input(sheet, row, column, value)
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

  /// Sets a string value in a cell without input parsing
  #[napi]
  pub fn update_cell_with_text(
    &mut self,
    sheet: u32,
    row: i32,
    column: i32,
    value: String,
  ) -> Result<()> {
    self
      .model
      .update_cell_with_text(sheet, row, column, &value)
      .map_err(to_js_error)
  }

  /// Sets a number in a cell without input parsing
  #[napi]
  pub fn update_cell_with_number(
    &mut self,
    sheet: u32,
    row: i32,
    column: i32,
    value: f64,
  ) -> Result<()> {
    self
      .model
      .update_cell_with_number(sheet, row, column, value)
      .map_err(to_js_error)
  }

  /// Sets a boolean in a cell without input parsing
  #[napi]
  pub fn update_cell_with_bool(
    &mut self,
    sheet: u32,
    row: i32,
    column: i32,
    value: bool,
  ) -> Result<()> {
    self
      .model
      .update_cell_with_bool(sheet, row, column, value)
      .map_err(to_js_error)
  }

  /// Sets a formula (i.e. "=A1*2") in a cell
  #[napi]
  pub fn update_cell_with_formula(
    &mut self,
    sheet: u32,
    row: i32,
    column: i32,
    formula: String,
  ) -> Result<()> {
    self
      .model
      .update_cell_with_formula(sheet, row, column, formula)
      .map_err(to_js_error)
  }

  /// Clears the contents of a single cell, keeping the formatting
  #[napi]
  pub fn clear_cell_contents(&mut self, sheet: u32, row: i32, column: i32) -> Result<()> {
    self
      .model
      .range_clear_contents(&area(sheet, row, column, row, column))
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

  // Get values

  /// Returns the content of a cell as the user would see it in the editor:
  /// the formula if there is one or the raw value otherwise
  #[napi]
  pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_localized_cell_content(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns the formula of a cell, if any
  #[napi]
  pub fn get_cell_formula(&self, sheet: u32, row: i32, column: i32) -> Result<Option<String>> {
    self
      .model
      .get_cell_formula(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns the value of a cell as a native JS value
  /// (null, string, number or boolean)
  #[napi]
  pub fn get_cell_value(
    &self,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<Option<Either3<f64, String, bool>>> {
    let value = self
      .model
      .get_cell_value_by_index(sheet, row, column)
      .map_err(to_js_error)?;
    Ok(match value {
      CellValue::None => None,
      CellValue::String(s) => Some(Either3::B(s)),
      CellValue::Number(f) => Some(Either3::A(f)),
      CellValue::Boolean(b) => Some(Either3::C(b)),
    })
  }

  /// Returns the value of a cell referenced like "Sheet1!C4" as a native JS
  /// value (null, string, number or boolean)
  #[napi]
  pub fn get_cell_value_by_ref(
    &self,
    cell_ref: String,
  ) -> Result<Option<Either3<f64, String, bool>>> {
    let value = self
      .model
      .get_cell_value_by_ref(&cell_ref)
      .map_err(to_js_error)?;
    Ok(match value {
      CellValue::None => None,
      CellValue::String(s) => Some(Either3::B(s)),
      CellValue::Number(f) => Some(Either3::A(f)),
      CellValue::Boolean(b) => Some(Either3::C(b)),
    })
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

  /// Returns the formatted value of a cell (i.e. "$ 5.75")
  #[napi]
  pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_formatted_cell_value(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns true if the cell is empty
  #[napi]
  pub fn is_empty_cell(&self, sheet: u32, row: i32, column: i32) -> Result<bool> {
    self
      .model
      .is_empty_cell(sheet, row, column)
      .map_err(to_js_error)
  }

  /// Returns all non-empty cells as a list of [sheet, row, column] tuples
  #[napi(ts_return_type = "Array<[number, number, number]>")]
  pub fn get_all_cells<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    let cells: Vec<(u32, i32, i32)> = self
      .model
      .get_all_cells()
      .into_iter()
      .map(|c| (c.index, c.row, c.column))
      .collect();
    env.to_js_value(&cells).map_err(to_js_error)
  }

  /// Returns a markdown-like representation of the sheet, useful for debugging
  #[napi]
  pub fn get_sheet_markup(&self, sheet: u32) -> Result<String> {
    self.model.get_sheet_markup(sheet).map_err(to_js_error)
  }

  // Styles

  /// Sets the style of a cell from a style object
  #[napi]
  pub fn set_cell_style(
    &mut self,
    env: Env,
    sheet: u32,
    row: i32,
    column: i32,
    #[napi(ts_arg_type = "CellStyle")] style: Unknown,
  ) -> Result<()> {
    let style: Style = env.from_js_value(style).map_err(to_js_error)?;
    self
      .model
      .set_cell_style(sheet, row, column, &style)
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
      .get_style_for_cell(sheet, row, column)
      .map_err(to_js_error)?;
    env.to_js_value(&style).map_err(to_js_error)
  }

  /// Sets the default style for a whole column
  #[napi]
  pub fn set_column_style(
    &mut self,
    env: Env,
    sheet: u32,
    column: i32,
    #[napi(ts_arg_type = "CellStyle")] style: Unknown,
  ) -> Result<()> {
    let style: Style = env.from_js_value(style).map_err(to_js_error)?;
    self
      .model
      .set_column_style(sheet, column, &style)
      .map_err(to_js_error)
  }

  /// Returns the default style of a column, if any
  #[napi(ts_return_type = "CellStyle | null")]
  pub fn get_column_style<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    column: i32,
  ) -> Result<Option<Unknown<'e>>> {
    let style = self
      .model
      .get_column_style(sheet, column)
      .map_err(to_js_error)?;
    match style {
      Some(style) => Ok(Some(env.to_js_value(&style).map_err(to_js_error)?)),
      None => Ok(None),
    }
  }

  /// Deletes the default style of a column
  #[napi]
  pub fn delete_column_style(&mut self, sheet: u32, column: i32) -> Result<()> {
    self
      .model
      .delete_column_style(sheet, column)
      .map_err(to_js_error)
  }

  /// Sets the default style for a whole row
  #[napi]
  pub fn set_row_style(
    &mut self,
    env: Env,
    sheet: u32,
    row: i32,
    #[napi(ts_arg_type = "CellStyle")] style: Unknown,
  ) -> Result<()> {
    let style: Style = env.from_js_value(style).map_err(to_js_error)?;
    self
      .model
      .set_row_style(sheet, row, &style)
      .map_err(to_js_error)
  }

  /// Returns the default style of a row, if any
  #[napi(ts_return_type = "CellStyle | null")]
  pub fn get_row_style<'e>(
    &self,
    env: &'e Env,
    sheet: u32,
    row: i32,
  ) -> Result<Option<Unknown<'e>>> {
    let style = self.model.get_row_style(sheet, row).map_err(to_js_error)?;
    match style {
      Some(style) => Ok(Some(env.to_js_value(&style).map_err(to_js_error)?)),
      None => Ok(None),
    }
  }

  /// Deletes the default style of a row
  #[napi]
  pub fn delete_row_style(&mut self, sheet: u32, row: i32) -> Result<()> {
    self.model.delete_row_style(sheet, row).map_err(to_js_error)
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

  /// Moves a column by `delta` positions
  #[napi]
  pub fn move_column(&mut self, sheet: u32, column: i32, delta: i32) -> Result<()> {
    self
      .model
      .move_columns_action(sheet, column, 1, delta)
      .map_err(to_js_error)
  }

  /// Moves a row by `delta` positions
  #[napi]
  pub fn move_row(&mut self, sheet: u32, row: i32, delta: i32) -> Result<()> {
    self
      .model
      .move_rows_action(sheet, row, 1, delta)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64> {
    self
      .model
      .get_column_width(sheet, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64> {
    self.model.get_row_height(sheet, row).map_err(to_js_error)
  }

  #[napi]
  pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<()> {
    self
      .model
      .set_column_width(sheet, column, width)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<()> {
    self
      .model
      .set_row_height(sheet, row, height)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_column_hidden(&mut self, sheet: u32, column: i32, hidden: bool) -> Result<()> {
    self
      .model
      .set_column_hidden(sheet, column, hidden)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_row_hidden(&mut self, sheet: u32, row: i32, hidden: bool) -> Result<()> {
    self
      .model
      .set_row_hidden(sheet, row, hidden)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn is_column_hidden(&self, sheet: u32, column: i32) -> Result<bool> {
    self
      .model
      .is_column_hidden(sheet, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn is_row_hidden(&self, sheet: u32, row: i32) -> Result<bool> {
    self.model.is_row_hidden(sheet, row).map_err(to_js_error)
  }

  // Frozen rows/columns

  #[napi]
  pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32> {
    self
      .model
      .get_frozen_columns_count(sheet)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32> {
    self.model.get_frozen_rows_count(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn set_frozen_columns_count(&mut self, sheet: u32, column_count: i32) -> Result<()> {
    self
      .model
      .set_frozen_columns(sheet, column_count)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_frozen_rows_count(&mut self, sheet: u32, row_count: i32) -> Result<()> {
    self
      .model
      .set_frozen_rows(sheet, row_count)
      .map_err(to_js_error)
  }

  // Sheets

  /// Returns the list of sheets with their properties (name, state, color, ...)
  #[napi(ts_return_type = "Array<WorksheetProperties>")]
  pub fn get_worksheets_properties<'e>(&self, env: &'e Env) -> Result<Unknown<'e>> {
    env
      .to_js_value(&self.model.get_worksheets_properties())
      .map_err(to_js_error)
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

  /// Sets the sheet visibility state: "visible", "hidden" or "veryHidden"
  #[napi]
  pub fn set_sheet_state(&mut self, sheet: u32, state: String) -> Result<()> {
    let state = match state.as_str() {
      "visible" => SheetState::Visible,
      "hidden" => SheetState::Hidden,
      "veryHidden" => SheetState::VeryHidden,
      _ => return Err(to_js_error(format!("Invalid sheet state: '{state}'"))),
    };
    self
      .model
      .set_sheet_state(sheet, state)
      .map_err(to_js_error)
  }

  /// Adds a new sheet with the given name
  #[napi]
  pub fn add_sheet(&mut self, sheet_name: String) -> Result<()> {
    self.model.add_sheet(&sheet_name).map_err(to_js_error)
  }

  /// Adds a new sheet with an automatically generated name
  #[napi]
  pub fn new_sheet(&mut self) {
    self.model.new_sheet();
  }

  #[napi]
  pub fn delete_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.delete_sheet(sheet).map_err(to_js_error)
  }

  #[napi]
  pub fn rename_sheet(&mut self, sheet: u32, new_name: String) -> Result<()> {
    self
      .model
      .rename_sheet_by_index(sheet, &new_name)
      .map_err(to_js_error)
  }

  /// Returns the bounds of all non-empty cells as
  /// [minRow, maxRow, minColumn, maxColumn]. For an empty sheet returns [1, 1, 1, 1].
  #[napi(ts_return_type = "[number, number, number, number]")]
  pub fn get_sheet_dimensions<'e>(&self, env: &'e Env, sheet: u32) -> Result<Unknown<'e>> {
    let worksheet = self.model.workbook.worksheet(sheet).map_err(to_js_error)?;
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

  #[napi]
  pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> Result<()> {
    self
      .model
      .set_show_grid_lines(sheet, show_grid_lines)
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

  // Workbook properties

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
}
