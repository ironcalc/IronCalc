#![deny(clippy::all)]

use serde::Serialize;

use napi::{self, bindgen_prelude::*, JsUnknown, Result};

use ironcalc::base::{
  expressions::types::Area,
  types::{CellType, Style},
  BorderArea, ClipboardData, UserModel as BaseModel,
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

#[napi]
pub struct UserModel {
  model: BaseModel,
}

#[napi]
impl UserModel {
  #[napi(constructor)]
  pub fn new(name: String, locale: String, timezone: String) -> Result<Self> {
    let model = BaseModel::new_empty(&name, &locale, &timezone).map_err(to_js_error)?;
    Ok(Self { model })
  }

  #[napi(factory)]
  pub fn from_bytes(bytes: &[u8]) -> Result<UserModel> {
    let model = BaseModel::from_bytes(bytes).map_err(to_js_error)?;
    Ok(UserModel { model })
  }

  pub fn undo(&mut self) -> Result<()> {
    self.model.undo().map_err(to_js_error)
  }

  pub fn redo(&mut self) -> Result<()> {
    self.model.redo().map_err(to_js_error)
  }

  #[napi(js_name = "canUndo")]
  pub fn can_undo(&self) -> bool {
    self.model.can_undo()
  }

  #[napi(js_name = "canRedo")]
  pub fn can_redo(&self) -> bool {
    self.model.can_redo()
  }

  #[napi(js_name = "pauseEvaluation")]
  pub fn pause_evaluation(&mut self) {
    self.model.pause_evaluation()
  }

  #[napi(js_name = "resumeEvaluation")]
  pub fn resume_evaluation(&mut self) {
    self.model.resume_evaluation()
  }

  pub fn evaluate(&mut self) {
    self.model.evaluate();
  }

  #[napi(js_name = "flushSendQueue")]
  pub fn flush_send_queue(&mut self) -> Vec<u8> {
    self.model.flush_send_queue()
  }

  #[napi(js_name = "applyExternalDiffs")]
  pub fn apply_external_diffs(&mut self, diffs: &[u8]) -> Result<()> {
    self.model.apply_external_diffs(diffs).map_err(to_js_error)
  }

  #[napi(js_name = "getCellContent")]
  pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_cell_content(sheet, row, column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "newSheet")]
  pub fn new_sheet(&mut self) -> Result<()> {
    self.model.new_sheet().map_err(to_js_error)
  }

  #[napi(js_name = "deleteSheet")]
  pub fn delete_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.delete_sheet(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "hideSheet")]
  pub fn hide_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.hide_sheet(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "unhideSheet")]
  pub fn unhide_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.unhide_sheet(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "renameSheet")]
  pub fn rename_sheet(&mut self, sheet: u32, name: String) -> Result<()> {
    self.model.rename_sheet(sheet, &name).map_err(to_js_error)
  }

  #[napi(js_name = "setSheetColor")]
  pub fn set_sheet_color(&mut self, sheet: u32, color: String) -> Result<()> {
    self
      .model
      .set_sheet_color(sheet, &color)
      .map_err(to_js_error)
  }

  #[napi(js_name = "rangeClearAll")]
  pub fn range_clear_all(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    let range = Area {
      sheet,
      row: start_row,
      column: start_column,
      width: end_column - start_column + 1,
      height: end_row - start_row + 1,
    };
    self.model.range_clear_all(&range).map_err(to_js_error)
  }

  #[napi(js_name = "rangeClearContents")]
  pub fn range_clear_contents(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    let range = Area {
      sheet,
      row: start_row,
      column: start_column,
      width: end_column - start_column + 1,
      height: end_row - start_row + 1,
    };
    self.model.range_clear_contents(&range).map_err(to_js_error)
  }

  #[napi(js_name = "rangeClearFormatting")]
  pub fn range_clear_formatting(
    &mut self,
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
  ) -> Result<()> {
    let range = Area {
      sheet,
      row: start_row,
      column: start_column,
      width: end_column - start_column + 1,
      height: end_row - start_row + 1,
    };
    self
      .model
      .range_clear_formatting(&range)
      .map_err(to_js_error)
  }

  #[napi(js_name = "insertRows")]
  pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<()> {
    self
      .model
      .insert_rows(sheet, row, row_count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "insertColumns")]
  pub fn insert_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> Result<()> {
    self
      .model
      .insert_columns(sheet, column, column_count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "deleteRows")]
  pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> Result<()> {
    self
      .model
      .delete_rows(sheet, row, row_count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "deleteColumns")]
  pub fn delete_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> Result<()> {
    self
      .model
      .delete_columns(sheet, column, column_count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setRowsHeight")]
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

  #[napi(js_name = "setColumnsWidth")]
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

  #[napi(js_name = "getRowHeight")]
  pub fn get_row_height(&mut self, sheet: u32, row: i32) -> Result<f64> {
    self.model.get_row_height(sheet, row).map_err(to_js_error)
  }

  #[napi(js_name = "getColumnWidth")]
  pub fn get_column_width(&mut self, sheet: u32, column: i32) -> Result<f64> {
    self
      .model
      .get_column_width(sheet, column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setUserInput")]
  pub fn set_user_input(&mut self, sheet: u32, row: i32, column: i32, input: String) -> Result<()> {
    self
      .model
      .set_user_input(sheet, row, column, &input)
      .map_err(to_js_error)
  }

  #[napi(js_name = "getFormattedCellValue")]
  pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String> {
    self
      .model
      .get_formatted_cell_value(sheet, row, column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "getFrozenRowsCount")]
  pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32> {
    self.model.get_frozen_rows_count(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "getFrozenColumnsCount")]
  pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32> {
    self
      .model
      .get_frozen_columns_count(sheet)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setFrozenRowsCount")]
  pub fn set_frozen_rows_count(&mut self, sheet: u32, count: i32) -> Result<()> {
    self
      .model
      .set_frozen_rows_count(sheet, count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setFrozenColumnsCount")]
  pub fn set_frozen_columns_count(&mut self, sheet: u32, count: i32) -> Result<()> {
    self
      .model
      .set_frozen_columns_count(sheet, count)
      .map_err(to_js_error)
  }

  #[napi(js_name = "updateRangeStyle")]
  pub fn update_range_style(
    &mut self,
    env: Env,
    range: JsUnknown,
    style_path: String,
    value: String,
  ) -> Result<()> {
    let range: Area = env
      .from_js_value(range)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .update_range_style(&range, &style_path, &value)
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
      .get_cell_style(sheet, row, column)
      .map_err(to_js_error)?;

    env
      .to_js_value(&style)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "onPasteStyles")]
  pub fn on_paste_styles(&mut self, env: Env, styles: JsUnknown) -> Result<()> {
    let styles: &Vec<Vec<Style>> = &env
      .from_js_value(styles)
      .map_err(|e| to_js_error(e.to_string()))?;
    self.model.on_paste_styles(styles).map_err(to_js_error)
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

  // I don't _think_ serializing to JsUnknown can't fail
  // FIXME: Remove this clippy directive
  #[napi(js_name = "getWorksheetsProperties")]
  #[allow(clippy::unwrap_used)]
  pub fn get_worksheets_properties(&self, env: Env) -> JsUnknown {
    env
      .to_js_value(&self.model.get_worksheets_properties())
      .unwrap()
  }

  #[napi(js_name = "getSelectedSheet")]
  pub fn get_selected_sheet(&self) -> u32 {
    self.model.get_selected_sheet()
  }

  #[napi(js_name = "getSelectedCell")]
  pub fn get_selected_cell(&self) -> Vec<i32> {
    let (sheet, row, column) = self.model.get_selected_cell();
    vec![sheet as i32, row, column]
  }

  // I don't _think_ serializing to JsUnknown can't fail
  // FIXME: Remove this clippy directive
  #[napi(js_name = "getSelectedView")]
  #[allow(clippy::unwrap_used)]
  pub fn get_selected_view(&self, env: Env) -> JsUnknown {
    env.to_js_value(&self.model.get_selected_view()).unwrap()
  }

  #[napi(js_name = "setSelectedSheet")]
  pub fn set_selected_sheet(&mut self, sheet: u32) -> Result<()> {
    self.model.set_selected_sheet(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "setSelectedCell")]
  pub fn set_selected_cell(&mut self, row: i32, column: i32) -> Result<()> {
    self
      .model
      .set_selected_cell(row, column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setSelectedRange")]
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

  #[napi(js_name = "setTopLeftVisibleCell")]
  pub fn set_top_left_visible_cell(&mut self, top_row: i32, top_column: i32) -> Result<()> {
    self
      .model
      .set_top_left_visible_cell(top_row, top_column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setShowGridLines")]
  pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> Result<()> {
    self
      .model
      .set_show_grid_lines(sheet, show_grid_lines)
      .map_err(to_js_error)
  }

  #[napi(js_name = "getShowGridLines")]
  pub fn get_show_grid_lines(&mut self, sheet: u32) -> Result<bool> {
    self.model.get_show_grid_lines(sheet).map_err(to_js_error)
  }

  #[napi(js_name = "autoFillRows")]
  pub fn auto_fill_rows(&mut self, env: Env, source_area: JsUnknown, to_row: i32) -> Result<()> {
    let area: Area = env
      .from_js_value(source_area)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .auto_fill_rows(&area, to_row)
      .map_err(to_js_error)
  }

  #[napi(js_name = "autoFillColumns")]
  pub fn auto_fill_columns(
    &mut self,
    env: Env,
    source_area: JsUnknown,
    to_column: i32,
  ) -> Result<()> {
    let area: Area = env
      .from_js_value(source_area)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .auto_fill_columns(&area, to_column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "onArrowRight")]
  pub fn on_arrow_right(&mut self) -> Result<()> {
    self.model.on_arrow_right().map_err(to_js_error)
  }

  #[napi(js_name = "onArrowLeft")]
  pub fn on_arrow_left(&mut self) -> Result<()> {
    self.model.on_arrow_left().map_err(to_js_error)
  }

  #[napi(js_name = "onArrowUp")]
  pub fn on_arrow_up(&mut self) -> Result<()> {
    self.model.on_arrow_up().map_err(to_js_error)
  }

  #[napi(js_name = "onArrowDown")]
  pub fn on_arrow_down(&mut self) -> Result<()> {
    self.model.on_arrow_down().map_err(to_js_error)
  }

  #[napi(js_name = "onPageDown")]
  pub fn on_page_down(&mut self) -> Result<()> {
    self.model.on_page_down().map_err(to_js_error)
  }

  #[napi(js_name = "onPageUp")]
  pub fn on_page_up(&mut self) -> Result<()> {
    self.model.on_page_up().map_err(to_js_error)
  }

  #[napi(js_name = "setWindowWidth")]
  pub fn set_window_width(&mut self, window_width: f64) {
    self.model.set_window_width(window_width);
  }

  #[napi(js_name = "setWindowHeight")]
  pub fn set_window_height(&mut self, window_height: f64) {
    self.model.set_window_height(window_height);
  }

  #[napi(js_name = "getScrollX")]
  pub fn get_scroll_x(&self) -> Result<f64> {
    self.model.get_scroll_x().map_err(to_js_error)
  }

  #[napi(js_name = "getScrollY")]
  pub fn get_scroll_y(&self) -> Result<f64> {
    self.model.get_scroll_y().map_err(to_js_error)
  }

  #[napi(js_name = "onExpandSelectedRange")]
  pub fn on_expand_selected_range(&mut self, key: String) -> Result<()> {
    self
      .model
      .on_expand_selected_range(&key)
      .map_err(to_js_error)
  }

  #[napi(js_name = "onAreaSelecting")]
  pub fn on_area_selecting(&mut self, target_row: i32, target_column: i32) -> Result<()> {
    self
      .model
      .on_area_selecting(target_row, target_column)
      .map_err(to_js_error)
  }

  #[napi(js_name = "setAreaWithBorder")]
  pub fn set_area_with_border(
    &mut self,
    env: Env,
    area: JsUnknown,
    border_area: JsUnknown,
  ) -> Result<()> {
    let range: Area = env
      .from_js_value(area)
      .map_err(|e| to_js_error(e.to_string()))?;
    let border: BorderArea = env
      .from_js_value(border_area)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .set_area_with_border(&range, &border)
      .map_err(|e| to_js_error(e.to_string()))?;
    Ok(())
  }

  #[napi(js_name = "toBytes")]
  pub fn to_bytes(&self) -> Vec<u8> {
    self.model.to_bytes()
  }

  #[napi(js_name = "getName")]
  pub fn get_name(&self) -> String {
    self.model.get_name()
  }

  #[napi(js_name = "setName")]
  pub fn set_name(&mut self, name: String) {
    self.model.set_name(&name);
  }

  #[napi(js_name = "copyToClipboard")]
  pub fn copy_to_clipboard(&self, env: Env) -> Result<JsUnknown> {
    let data = self
      .model
      .copy_to_clipboard()
      .map_err(|e| to_js_error(e.to_string()))?;

    env
      .to_js_value(&data)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "pasteFromClipboard")]
  pub fn paste_from_clipboard(
    &mut self,
    env: Env,
    source_sheet: u32,
    source_range: JsUnknown,
    clipboard: JsUnknown,
    is_cut: bool,
  ) -> Result<()> {
    let source_range: (i32, i32, i32, i32) = env
      .from_js_value(source_range)
      .map_err(|e| to_js_error(e.to_string()))?;
    let clipboard: ClipboardData = env
      .from_js_value(clipboard)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .paste_from_clipboard(source_sheet, source_range, &clipboard, is_cut)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "pasteCsvText")]
  pub fn paste_csv_string(&mut self, env: Env, area: JsUnknown, csv: String) -> Result<()> {
    let range: Area = env
      .from_js_value(area)
      .map_err(|e| to_js_error(e.to_string()))?;
    self
      .model
      .paste_csv_string(&range, &csv)
      .map_err(|e| to_js_error(e.to_string()))
  }

  #[napi(js_name = "getDefinedNameList")]
  pub fn get_defined_name_list(&self, env: Env) -> Result<JsUnknown> {
    let data: Vec<DefinedName> = self
      .model
      .get_defined_name_list()
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
