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
        .map(|(_, _, _, hidden, _)| hidden)
        .unwrap()
}

fn comment(model: &ironcalc::base::Model, name: &str) -> String {
    model
        .get_defined_name_list()
        .into_iter()
        .find(|(n, ..)| n == name)
        .map(|(_, _, _, _, comment)| comment)
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

    // Every name in this workbook carries a comment
    assert_eq!(comment(&model, "HiddenName"), "Hidden name");
    assert_eq!(comment(&model, "LocalBonus"), "Sheet scope (Tests)");
    assert_eq!(comment(&model, "Scope_Test"), "Sheet scope (Data)");

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
    assert!(workbook_xml.contains(
        "<definedName name=\"HiddenName\" hidden=\"1\" comment=\"Hidden name\">Inputs!$B$4</definedName>"
    ));
    assert_eq!(workbook_xml.matches("hidden=\"1\"").count(), 1);
    assert!(workbook_xml.contains(
        "<definedName name=\"FXRate\" comment=\"Single cell\">Inputs!$B$4</definedName>"
    ));

    // And it survives a reload
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert!(hidden_flag(&model, "HiddenName"));
    assert!(!hidden_flag(&model, "FXRate"));
    assert_eq!(comment(&model, "HiddenName"), "Hidden name");
    assert_eq!(comment(&model, "LocalBonus"), "Sheet scope (Tests)");
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_defined_name_comment_is_escaped_and_round_trips() {
    let mut model = ironcalc::base::Model::new_empty("comments", "en", "UTC", "en").unwrap();
    let text = "Tom & Jerry's <rate> is \"high\"";
    model
        .new_defined_name("Rate", None, "Sheet1!$A$1", false, text)
        .unwrap();
    // No comment, no attribute
    model
        .new_defined_name("Plain", None, "Sheet1!$A$2", false, "")
        .unwrap();

    let temp_file_name = "temp_file_test_defined_name_comment.xlsx";
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
    assert!(workbook_xml.contains(
        "<definedName name=\"Rate\" comment=\"Tom &amp; Jerry&apos;s &lt;rate&gt; is &quot;high&quot;\">Sheet1!$A$1</definedName>"
    ));
    assert!(workbook_xml.contains("<definedName name=\"Plain\">Sheet1!$A$2</definedName>"));

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(comment(&model, "Rate"), text);
    assert_eq!(comment(&model, "Plain"), "");
    fs::remove_file(temp_file_name).unwrap();
}
