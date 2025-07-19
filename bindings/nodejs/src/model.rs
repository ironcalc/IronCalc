#![deny(clippy::all)]

use napi::{self, bindgen_prelude::*, JsUnknown, Result};
use serde::Serialize;

use ironcalc::{
  base::{
    types::{CellType, Style},
    Model as BaseModel,
  },
  error::XlsxError,
  export::{save_to_icalc, save_to_xlsx},
  import::{load_from_icalc, load_from_xlsx},
};

#[derive(Serialize)]
struct DefinedName {
  name: String,
  scope: Option<u32>,
  formula: String,
}

fn to_js_error(error: String) -> Error {
  Error::new(Status::Unknown, error)
}

fn to_node_error(error: XlsxError) -> Error {
  Error::new(Status::Unknown, error.to_string())
}

#[napi]
pub struct Model {
  model: BaseModel,
}

#[napi]
impl Model {
  #[napi(constructor)]
  pub fn new(name: String, locale: String, timezone: String) -> Result<Self> {
    let model = BaseModel::new_empty(&name, &locale, &timezone).map_err(to_js_error)?;
    Ok(Self { model })
  }

  #[napi(factory)]
  pub fn from_xlsx(file_path: String, locale: String, tz: String) -> Result<Model> {
    let model = load_from_xlsx(&file_path, &locale, &tz)
      .map_err(|error| Error::new(Status::Unknown, error.to_string()))?;
    Ok(Self { model })
  }

  #[napi(factory)]
  pub fn from_icalc(file_name: String) -> Result<Model> {
    let model = load_from_icalc(&file_name)
      .map_err(|error| Error::new(Status::Unknown, error.to_string()))?;
    Ok(Self { model })
  }

  #[napi]
  pub fn save_to_xlsx(&self, file: String) -> Result<()> {
    save_to_xlsx(&self.model, &file).map_err(to_node_error)
  }

  #[napi]
  pub fn save_to_icalc(&self, file: String) -> Result<()> {
    save_to_icalc(&self.model, &file).map_err(to_node_error)
  }

  #[napi]
  pub fn evaluate(&mut self) {
    self.model.evaluate();
  }

  #[napi]
  pub fn set_user_input(&mut self, sheet: u32, row: i32, column: i32, value: String) -> Result<()> {
    self
      .model
      .set_user_input(sheet, row, column, value)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn clear_cell_contents(&mut self, sheet: u32, row: i32, column: i32) -> Result<()> {
    self
      .model
      .cell_clear_contents(sheet, row, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_cell_content(sheet, row, column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "getCellType")]
  pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<i32> {
    Ok(
      match self
        .model
        .get_cell_type(sheet, row, column)
        .map_err(to_js_error)?
      {
        CellType::Number => 1,
        CellType::Text => 2,
        CellType::LogicalValue => 4,
        CellType::ErrorValue => 16,
        CellType::Array => 64,
        CellType::CompoundData => 128,
      },
    )
  }

  #[napi]
  pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_formatted_cell_value(sheet, row, column)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn set_cell_style(
    &mut self,
    env: Env,
    sheet: u32,
    row: i32,
    column: i32,
    style: JsUnknown,
  ) -> Result<()> {
    let style: Style = env
      .from_js_value(style)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .set_cell_style(sheet, row, column, &style)
      .map_err(to_js_error)
  }

  #[napi(js_name = "getCellStyle")]
  pub fn get_cell_style(
    &mut self,
    env: Env,
    sheet: u32,
    row: i32,
    column: i32,
  ) -> Result<JsUnknown> {
    let style = self
      .model
      .get_style_for_cell(sheet, row, column)
      .map_err(to_js_error)?;

    env
      .to_js_value(&style)
      .map_err(|e| to_js_error(e.to_string()))
  }

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

  // I don't _think_ serializing to JsUnknown can't fail
  // FIXME: Remove this clippy directive
  #[napi(js_name = "getWorksheetsProperties")]
  #[allow(clippy::unwrap_used)]
  pub fn get_worksheets_properties(&self, env: Env) -> JsUnknown {
    env
      .to_js_value(&self.model.get_worksheets_properties())
      .unwrap()
  }

  #[napi]
  pub fn set_sheet_color(&mut self, sheet: u32, color: String) -> Result<()> {
    self
      .model
      .set_sheet_color(sheet, &color)
      .map_err(to_js_error)
  }

  #[napi]
  pub fn add_sheet(&mut self, sheet_name: String) -> Result<()> {
    self.model.add_sheet(&sheet_name).map_err(to_js_error)
  }

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

  #[napi(js_name = "getDefinedNameList")]
  pub fn get_defined_name_list(&self, env: Env) -> Result<JsUnknown> {
    let data: Vec<DefinedName> = self
      .model
      .workbook
      .get_defined_names_with_scope()
      .iter()
      .map(|s| DefinedName {
        name: s.0.to_owned(),
        scope: s.1,
        formula: s.2.to_owned(),
      })
      .collect();
    env
      .to_js_value(&data)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "newDefinedName")]
  pub fn new_defined_name(
    &mut self,
    name: String,
    scope: Option<u32>,
    formula: String,
  ) -> Result<()> {
    self
      .model
      .new_defined_name(&name, scope, &formula)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "updateDefinedName")]
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
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "deleteDefinedName")]
  pub fn delete_definedname(&mut self, name: String, scope: Option<u32>) -> Result<()> {
    self
      .model
      .delete_defined_name(&name, scope)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "moveColumn")]
  pub fn move_column(&mut self, sheet: u32, column: i32, delta: i32) -> Result<()> {
    self
      .model
      .move_column_action(sheet, column, delta)
      .map_err(to_js_error)
  }

  #[napi(js_name = "moveRow")]
  pub fn move_row(&mut self, sheet: u32, row: i32, delta: i32) -> Result<()> {
    self
      .model
      .move_row_action(sheet, row, delta)
      .map_err(to_js_error)
  }
}
