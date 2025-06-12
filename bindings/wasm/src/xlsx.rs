use ironcalc::{
    base::Model as BaseWorkbookModel, export::save_xlsx_to_writer, import::load_from_xlsx_bytes,
};
use std::io::{BufWriter, Cursor, Write};
use wasm_bindgen::prelude::{wasm_bindgen, JsError};

fn to_js_error(error: String) -> JsError {
    JsError::new(&error.to_string())
}

#[wasm_bindgen(js_name = fromXLSXBytes)]
pub fn from_xlsx_bytes(
    bytes: &[u8],
    name: &str,
    locale: &str,
    timezone: &str,
) -> Result<Vec<u8>, JsError> {
    let workbook = load_from_xlsx_bytes(bytes, name, locale, timezone)
        .map_err(|e| to_js_error(e.to_string()))?;
    let base_model =
        BaseWorkbookModel::from_workbook(workbook).map_err(|e| to_js_error(e.to_string()))?;
    Ok(base_model.to_bytes())
}

#[wasm_bindgen(js_name = toXLSXBytes)]
pub fn to_xlsx_bytes(bytes: &[u8]) -> Result<Vec<u8>, JsError> {
    let workbook = BaseWorkbookModel::from_bytes(bytes).map_err(to_js_error)?;
    let mut writer = BufWriter::new(Cursor::new(Vec::new()));
    save_xlsx_to_writer(&workbook, &mut writer).map_err(|e| to_js_error(e.to_string()))?;
    writer.flush().map_err(|e| to_js_error(e.to_string()))?;
    Ok(writer
        .into_inner()
        .map_err(|e| to_js_error(e.to_string()))?
        .into_inner())
}

fn main() {
    // This is required for cargo to compile this as a binary
}
