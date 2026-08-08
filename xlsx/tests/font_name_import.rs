#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::export::save_to_xlsx;
use ironcalc::import::load_from_xlsx;
use std::fs;

#[test]
// ECMA-376 §18.8.29: a <font>'s <name val="..."/> child carries the typeface. The importer
// must preserve it rather than replacing every font with the default name.
//
// example.xlsx defines fonts using two distinct typefaces — "Calibri" and "Tahoma" — so a
// correct import yields both names in the fonts table (and never silently collapses them to
// the default "Inter").
fn test_font_name_is_preserved_on_import() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    let names: Vec<&str> = model
        .workbook
        .styles
        .fonts
        .iter()
        .map(|f| f.name.as_str())
        .collect();

    assert!(
        names.contains(&"Calibri"),
        "expected a Calibri font, got {names:?}"
    );
    assert!(
        names.contains(&"Tahoma"),
        "expected the Tahoma font to survive import, got {names:?}"
    );

    // Round-trip: the preserved names must survive export → re-import.
    let temp_file_name = "temp_file_test_font_name.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    let reloaded_names: Vec<&str> = reloaded
        .workbook
        .styles
        .fonts
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert!(
        reloaded_names.contains(&"Tahoma"),
        "Tahoma must survive a save/reload round-trip, got {reloaded_names:?}"
    );
}
