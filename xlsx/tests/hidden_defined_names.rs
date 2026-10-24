#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use std::fs;

use ironcalc::export::save_to_xlsx;
use ironcalc::import::load_from_xlsx;

fn hidden_flag(model: &ironcalc::base::Model, name: &str) -> bool {
    model
        .get_defined_name_list()
        .into_iter()
        .find(|(n, ..)| n == name)
        .map(|(_, _, _, hidden)| hidden)
        .unwrap()
}

#[test]
fn test_hidden_defined_names_round_trip() {
    let model = load_from_xlsx("tests/calc_tests/named_ranges.xlsx", "en", "UTC", "en").unwrap();

    // HiddenName is marked hidden="1" in the workbook, the others are not
    assert!(hidden_flag(&model, "HiddenName"));
    assert!(!hidden_flag(&model, "FXRate"));
    assert!(!hidden_flag(&model, "LocalBonus"));
    // It still resolves as a regular name (Inputs!$B$4, same as FXRate)
    let hidden = model
        .workbook
        .defined_names
        .iter()
        .find(|dn| dn.name == "HiddenName")
        .unwrap();
    assert_eq!(hidden.formula, "Inputs!$B$4");
    assert_eq!(hidden.sheet_id, None);

    // Export keeps the attribute and nothing else gets it
    let temp_file_name = "temp_file_test_hidden_defined_names.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let file = fs::File::open(temp_file_name).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut workbook_xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("xl/workbook.xml").unwrap(),
        &mut workbook_xml,
    )
    .unwrap();
    drop(archive);
    assert!(workbook_xml
        .contains("<definedName name=\"HiddenName\" hidden=\"1\">Inputs!$B$4</definedName>"));
    assert_eq!(workbook_xml.matches("hidden=\"1\"").count(), 1);
    assert!(workbook_xml.contains("<definedName name=\"FXRate\">Inputs!$B$4</definedName>"));

    // And it survives a reload
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert!(hidden_flag(&model, "HiddenName"));
    assert!(!hidden_flag(&model, "FXRate"));
    fs::remove_file(temp_file_name).unwrap();
}
