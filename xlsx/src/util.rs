use ironcalc_base::Model;

pub fn get_metadata_sheet_index(model: &Model) -> Option<u32> {
    for (index, ws) in model.workbook.worksheets.iter().enumerate() {
        if ws.name.eq_ignore_ascii_case("METADATA") {
            return Some(index as u32);
        }
    }
    None
}

// Cheesy way to get the locale from the workbook metadata sheet
pub fn get_workbook_metadata(model: &Model) -> String {
    let metadata_sheet_index = get_metadata_sheet_index(model);
    let default_locale = "en".to_string();
    if let Some(sheet_index) = metadata_sheet_index {
        if let Ok(a1) = model.get_formatted_cell_value(sheet_index, 1, 1) {
            if a1 == "Locale" {
                match model.get_formatted_cell_value(sheet_index, 1, 2) {
                    Ok(v) if v == "en-GB" => {
                        return "en-GB".to_string();
                    }
                    _ => return default_locale,
                }
            }
        }
    }
    default_locale
}
