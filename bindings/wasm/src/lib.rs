use wasm_bindgen::{
    prelude::{wasm_bindgen, JsError},
    JsValue,
};

use ironcalc_base::{
    expressions::{lexer::util::get_tokens as tokenizer, types::Area, utils::number_to_column},
    types::{CellType, Style},
    BorderArea, ClipboardData, UserModel as BaseModel,
};

fn to_js_error(error: String) -> JsError {
    JsError::new(&error.to_string())
}

/// Return an array with a list of all the tokens from a formula
/// This is used by the UI to color them according to a theme.
#[wasm_bindgen(js_name = "getTokens")]
pub fn get_tokens(formula: &str) -> Result<JsValue, JsError> {
    let tokens = tokenizer(formula);
    serde_wasm_bindgen::to_value(&tokens).map_err(JsError::from)
}

#[wasm_bindgen(js_name = "columnNameFromNumber")]
pub fn column_name_from_number(column: i32) -> Result<String, JsError> {
    match number_to_column(column) {
        Some(c) => Ok(c),
        None => Err(JsError::new("Invalid column number")),
    }
}

#[wasm_bindgen]
pub struct Model {
    model: BaseModel,
}

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, locale: &str, timezone: &str) -> Result<Model, JsError> {
        let model = BaseModel::new_empty(name, locale, timezone).map_err(to_js_error)?;
        Ok(Model { model })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Model, JsError> {
        let model = BaseModel::from_bytes(bytes).map_err(to_js_error)?;
        Ok(Model { model })
    }

    pub fn undo(&mut self) -> Result<(), JsError> {
        self.model.undo().map_err(to_js_error)
    }

    pub fn redo(&mut self) -> Result<(), JsError> {
        self.model.redo().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "canUndo")]
    pub fn can_undo(&self) -> bool {
        self.model.can_undo()
    }

    #[wasm_bindgen(js_name = "canRedo")]
    pub fn can_redo(&self) -> bool {
        self.model.can_redo()
    }

    #[wasm_bindgen(js_name = "pauseEvaluation")]
    pub fn pause_evaluation(&mut self) {
        self.model.pause_evaluation()
    }

    #[wasm_bindgen(js_name = "resumeEvaluation")]
    pub fn resume_evaluation(&mut self) {
        self.model.resume_evaluation()
    }

    pub fn evaluate(&mut self) {
        self.model.evaluate();
    }

    #[wasm_bindgen(js_name = "flushSendQueue")]
    pub fn flush_send_queue(&mut self) -> Vec<u8> {
        self.model.flush_send_queue()
    }

    #[wasm_bindgen(js_name = "applyExternalDiffs")]
    pub fn apply_external_diffs(&mut self, diffs: &[u8]) -> Result<(), JsError> {
        self.model.apply_external_diffs(diffs).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getCellContent")]
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String, JsError> {
        self.model
            .get_cell_content(sheet, row, column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "newSheet")]
    pub fn new_sheet(&mut self) {
        self.model.new_sheet()
    }

    #[wasm_bindgen(js_name = "deleteSheet")]
    pub fn delete_sheet(&mut self, sheet: u32) -> Result<(), JsError> {
        self.model.delete_sheet(sheet).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "renameSheet")]
    pub fn rename_sheet(&mut self, sheet: u32, name: &str) -> Result<(), JsError> {
        self.model.rename_sheet(sheet, name).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setSheetColor")]
    pub fn set_sheet_color(&mut self, sheet: u32, color: &str) -> Result<(), JsError> {
        self.model
            .set_sheet_color(sheet, color)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "rangeClearAll")]
    pub fn range_clear_all(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Result<(), JsError> {
        let range = Area {
            sheet,
            row: start_row,
            column: start_column,
            width: end_column - start_column + 1,
            height: end_row - start_row + 1,
        };
        self.model.range_clear_all(&range).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "rangeClearContents")]
    pub fn range_clear_contents(
        &mut self,
        sheet: u32,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Result<(), JsError> {
        let range = Area {
            sheet,
            row: start_row,
            column: start_column,
            width: end_column - start_column + 1,
            height: end_row - start_row + 1,
        };
        self.model.range_clear_contents(&range).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "insertRow")]
    pub fn insert_row(&mut self, sheet: u32, row: i32) -> Result<(), JsError> {
        self.model.insert_row(sheet, row).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "insertColumn")]
    pub fn insert_column(&mut self, sheet: u32, column: i32) -> Result<(), JsError> {
        self.model.insert_column(sheet, column).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "deleteRow")]
    pub fn delete_row(&mut self, sheet: u32, row: i32) -> Result<(), JsError> {
        self.model.delete_row(sheet, row).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "deleteColumn")]
    pub fn delete_column(&mut self, sheet: u32, column: i32) -> Result<(), JsError> {
        self.model.delete_column(sheet, column).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setRowHeight")]
    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> Result<(), JsError> {
        self.model
            .set_row_height(sheet, row, height)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setColumnWidth")]
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), JsError> {
        self.model
            .set_column_width(sheet, column, width)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getRowHeight")]
    pub fn get_row_height(&mut self, sheet: u32, row: i32) -> Result<f64, JsError> {
        self.model.get_row_height(sheet, row).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getColumnWidth")]
    pub fn get_column_width(&mut self, sheet: u32, column: i32) -> Result<f64, JsError> {
        self.model
            .get_column_width(sheet, column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setUserInput")]
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        input: &str,
    ) -> Result<(), JsError> {
        self.model
            .set_user_input(sheet, row, column, input)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getFormattedCellValue")]
    pub fn get_formatted_cell_value(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<String, JsError> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getFrozenRowsCount")]
    pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32, JsError> {
        self.model.get_frozen_rows_count(sheet).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getFrozenColumnsCount")]
    pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32, JsError> {
        self.model
            .get_frozen_columns_count(sheet)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setFrozenRowsCount")]
    pub fn set_frozen_rows_count(&mut self, sheet: u32, count: i32) -> Result<(), JsError> {
        self.model
            .set_frozen_rows_count(sheet, count)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setFrozenColumnsCount")]
    pub fn set_frozen_columns_count(&mut self, sheet: u32, count: i32) -> Result<(), JsError> {
        self.model
            .set_frozen_columns_count(sheet, count)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "updateRangeStyle")]
    pub fn update_range_style(
        &mut self,
        range: JsValue,
        style_path: &str,
        value: &str,
    ) -> Result<(), JsError> {
        let range: Area =
            serde_wasm_bindgen::from_value(range).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .update_range_style(&range, style_path, value)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getCellStyle")]
    pub fn get_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<JsValue, JsError> {
        self.model
            .get_cell_style(sheet, row, column)
            .map_err(to_js_error)
            .map(|x| serde_wasm_bindgen::to_value(&x).unwrap())
    }

    #[wasm_bindgen(js_name = "onPasteStyles")]
    pub fn on_paste_styles(&mut self, styles: JsValue) -> Result<(), JsError> {
        let styles: &Vec<Vec<Style>> = &serde_wasm_bindgen::from_value(styles).unwrap();
        self.model.on_paste_styles(styles).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getCellType")]
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<i32, JsError> {
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

    #[wasm_bindgen(js_name = "getWorksheetsProperties")]
    pub fn get_worksheets_properties(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.model.get_worksheets_properties()).unwrap()
    }

    #[wasm_bindgen(js_name = "getSelectedSheet")]
    pub fn get_selected_sheet(&self) -> u32 {
        self.model.get_selected_sheet()
    }

    #[wasm_bindgen(js_name = "getSelectedCell")]
    pub fn get_selected_cell(&self) -> Vec<i32> {
        let (sheet, row, column) = self.model.get_selected_cell();
        vec![sheet as i32, row, column]
    }

    #[wasm_bindgen(js_name = "getSelectedView")]
    pub fn get_selected_view(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.model.get_selected_view()).unwrap()
    }

    #[wasm_bindgen(js_name = "setSelectedSheet")]
    pub fn set_selected_sheet(&mut self, sheet: u32) -> Result<(), JsError> {
        self.model.set_selected_sheet(sheet).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setSelectedCell")]
    pub fn set_selected_cell(&mut self, row: i32, column: i32) -> Result<(), JsError> {
        self.model
            .set_selected_cell(row, column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setSelectedRange")]
    pub fn set_selected_range(
        &mut self,
        start_row: i32,
        start_column: i32,
        end_row: i32,
        end_column: i32,
    ) -> Result<(), JsError> {
        self.model
            .set_selected_range(start_row, start_column, end_row, end_column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setTopLeftVisibleCell")]
    pub fn set_top_left_visible_cell(
        &mut self,
        top_row: i32,
        top_column: i32,
    ) -> Result<(), JsError> {
        self.model
            .set_top_left_visible_cell(top_row, top_column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setShowGridLines")]
    pub fn set_show_grid_lines(
        &mut self,
        sheet: u32,
        show_grid_lines: bool,
    ) -> Result<(), JsError> {
        self.model
            .set_show_grid_lines(sheet, show_grid_lines)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getShowGridLines")]
    pub fn get_show_grid_lines(&mut self, sheet: u32) -> Result<bool, JsError> {
        self.model.get_show_grid_lines(sheet).map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "autoFillRows")]
    pub fn auto_fill_rows(&mut self, source_area: JsValue, to_row: i32) -> Result<(), JsError> {
        let area: Area =
            serde_wasm_bindgen::from_value(source_area).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .auto_fill_rows(&area, to_row)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "autoFillColumns")]
    pub fn auto_fill_columns(
        &mut self,
        source_area: JsValue,
        to_column: i32,
    ) -> Result<(), JsError> {
        let area: Area =
            serde_wasm_bindgen::from_value(source_area).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .auto_fill_columns(&area, to_column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onArrowRight")]
    pub fn on_arrow_right(&mut self) -> Result<(), JsError> {
        self.model.on_arrow_right().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onArrowLeft")]
    pub fn on_arrow_left(&mut self) -> Result<(), JsError> {
        self.model.on_arrow_left().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onArrowUp")]
    pub fn on_arrow_up(&mut self) -> Result<(), JsError> {
        self.model.on_arrow_up().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onArrowDown")]
    pub fn on_arrow_down(&mut self) -> Result<(), JsError> {
        self.model.on_arrow_down().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onPageDown")]
    pub fn on_page_down(&mut self) -> Result<(), JsError> {
        self.model.on_page_down().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onPageUp")]
    pub fn on_page_up(&mut self) -> Result<(), JsError> {
        self.model.on_page_up().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setWindowWidth")]
    pub fn set_window_width(&mut self, window_width: f64) {
        self.model.set_window_width(window_width);
    }

    #[wasm_bindgen(js_name = "setWindowHeight")]
    pub fn set_window_height(&mut self, window_height: f64) {
        self.model.set_window_height(window_height);
    }

    #[wasm_bindgen(js_name = "getScrollX")]
    pub fn get_scroll_x(&self) -> Result<f64, JsError> {
        self.model.get_scroll_x().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "getScrollY")]
    pub fn get_scroll_y(&self) -> Result<f64, JsError> {
        self.model.get_scroll_y().map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onExpandSelectedRange")]
    pub fn on_expand_selected_range(&mut self, key: &str) -> Result<(), JsError> {
        self.model
            .on_expand_selected_range(key)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "onAreaSelecting")]
    pub fn on_area_selecting(
        &mut self,
        target_row: i32,
        target_column: i32,
    ) -> Result<(), JsError> {
        self.model
            .on_area_selecting(target_row, target_column)
            .map_err(to_js_error)
    }

    #[wasm_bindgen(js_name = "setAreaWithBorder")]
    pub fn set_area_with_border(
        &mut self,
        area: JsValue,
        border_area: JsValue,
    ) -> Result<(), JsError> {
        let range: Area =
            serde_wasm_bindgen::from_value(area).map_err(|e| to_js_error(e.to_string()))?;
        let border: BorderArea =
            serde_wasm_bindgen::from_value(border_area).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .set_area_with_border(&range, &border)
            .map_err(|e| to_js_error(e.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.model.to_bytes()
    }

    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self) -> String {
        self.model.get_name()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&mut self, name: &str) {
        self.model.set_name(name);
    }

    #[wasm_bindgen(js_name = "copyToClipboard")]
    pub fn copy_to_clipboard(&self) -> Result<JsValue, JsError> {
        let data = self
            .model
            .copy_to_clipboard()
            .map_err(|e| to_js_error(e.to_string()));
        data.map(|x| serde_wasm_bindgen::to_value(&x).unwrap())
    }

    #[wasm_bindgen(js_name = "pasteFromClipboard")]
    pub fn paste_from_clipboard(
        &mut self,
        source_range: JsValue,
        clipboard: JsValue,
    ) -> Result<(), JsError> {
        let source_range: (i32, i32, i32, i32) =
            serde_wasm_bindgen::from_value(source_range).map_err(|e| to_js_error(e.to_string()))?;
        let clipboard: ClipboardData =
            serde_wasm_bindgen::from_value(clipboard).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .paste_from_clipboard(source_range, &clipboard)
            .map_err(|e| to_js_error(e.to_string()))
    }

    #[wasm_bindgen(js_name = "pasteCsvText")]
    pub fn paste_csv_string(&mut self, area: JsValue, csv: &str) -> Result<(), JsError> {
        let range: Area =
            serde_wasm_bindgen::from_value(area).map_err(|e| to_js_error(e.to_string()))?;
        self.model
            .paste_csv_string(&range, csv)
            .map_err(|e| to_js_error(e.to_string()))
    }
}
