use ironcalc_base::{cell::CellValue, Model};

use crate::util::get_metadata_sheet_index;

// Excel serial date of the Unix epoch (1970-01-01)
const UNIX_EPOCH_SERIAL: f64 = 25569.0;
const MILLISECONDS_PER_DAY: f64 = 86_400_000.0;

/// Looks in the METADATA sheet for a row with "NOW" in column A and an Excel
/// serial date in column B (typically `=NOW()` with the value Excel cached on
/// save) and returns it as milliseconds since the Unix epoch.
pub fn get_metadata_timestamp(model: &Model) -> Option<i64> {
    let sheet_index = get_metadata_sheet_index(model)?;
    for row in 1..=32 {
        if let Ok(CellValue::String(label)) = model.get_cell_value_by_index(sheet_index, row, 1) {
            if label == "NOW" {
                if let Ok(CellValue::Number(serial)) =
                    model.get_cell_value_by_index(sheet_index, row, 2)
                {
                    let milliseconds = (serial - UNIX_EPOCH_SERIAL) * MILLISECONDS_PER_DAY;
                    return Some(milliseconds.round() as i64);
                }
                return None;
            }
        }
    }
    None
}

/// Mocks the engine clock to the timestamp stored in the METADATA sheet so
/// that volatile functions (NOW, TODAY, ...) evaluate to the same values
/// Excel saved. Workbooks without a timestamp use the actual system time.
/// This module only exists behind the `mock_time` feature, which this
/// crate's own tests enable through the self dev-dependency in Cargo.toml.
pub fn set_mock_time_from_metadata(model: &Model) {
    #[allow(clippy::unwrap_used)]
    let milliseconds = get_metadata_timestamp(model).unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    });
    ironcalc_base::mock_time::set_mock_time(milliseconds);
}
