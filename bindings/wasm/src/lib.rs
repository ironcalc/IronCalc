use wasm_bindgen::{
    prelude::{wasm_bindgen, JsError},
    JsValue,
};

use ironcalc_base::{
    expressions::{lexer::util::get_tokens as tokenizer, types::Area},
    types::CellType,
    UserModel as BaseModel,
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
#[wasm_bindgen]
pub struct Model {
    model: BaseModel,
}

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    pub fn new(locale: &str, timezone: &str) -> Result<Model, JsError> {
        let model = BaseModel::new_empty("workbook", locale, timezone).map_err(to_js_error)?;
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
}
