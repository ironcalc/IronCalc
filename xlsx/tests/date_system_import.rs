#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Read};

use ironcalc::{export::save_xlsx_to_writer, import::load_from_xlsx, import::load_from_xlsx_bytes};
use ironcalc_base::{cell::CellValue, types::DateSystem, Model};

#[test]
fn excel_files_use_the_1900_date_system() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    assert_eq!(model.workbook.settings.date_system, DateSystem::Excel1900);
}

#[test]
#[ignore]
fn date1904_selects_the_1904_date_system() {
    let model = load_from_xlsx(
        "tests/calc_tests/DATE_AND_TIME/DATE_excel_1904.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    assert_eq!(model.workbook.settings.date_system, DateSystem::Excel1904);
}

#[test]
fn libreoffice_files_use_the_ironcalc_date_system() {
    let model = load_from_xlsx("tests/libreoffice_888_example.xlsx", "en", "UTC", "en").unwrap();
    assert!(model
        .workbook
        .metadata
        .application
        .starts_with("LibreOffice"));
    assert_eq!(model.workbook.settings.date_system, DateSystem::IronCalc);
}

#[test]
fn files_without_metadata_use_the_1900_date_system() {
    // Google Sheets exports have no docProps/app.xml
    let model = load_from_xlsx("tests/freeze_google_sheets.xlsx", "en", "UTC", "en").unwrap();
    assert_eq!(model.workbook.settings.date_system, DateSystem::Excel1900);
}

#[test]
fn files_written_by_ironcalc_use_the_ironcalc_date_system() {
    let model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let workbook = load_from_xlsx_bytes(&writer.into_inner(), "model", "en", "UTC").unwrap();
    assert!(workbook.metadata.application.starts_with("IronCalc"));
    assert_eq!(workbook.settings.date_system, DateSystem::IronCalc);
}

fn round_trip(
    date_system: DateSystem,
) -> (
    ironcalc_base::types::Workbook,
    zip::ZipArchive<Cursor<Vec<u8>>>,
) {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.workbook.settings.date_system = date_system;
    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let bytes = writer.into_inner();
    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    (workbook, zip::ZipArchive::new(Cursor::new(bytes)).unwrap())
}

fn read(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> String {
    let mut s = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    s
}

#[test]
fn every_date_system_survives_an_export_and_import() {
    for date_system in [
        DateSystem::IronCalc,
        DateSystem::Excel1900,
        DateSystem::Excel1904,
    ] {
        let (workbook, _) = round_trip(date_system);
        assert_eq!(workbook.settings.date_system, date_system);
    }
}

#[test]
fn export_writes_the_date_system_property_and_date1904() {
    let (_, mut archive) = round_trip(DateSystem::Excel1900);
    let custom = read(&mut archive, "docProps/custom.xml");
    assert!(custom.contains(r#"name="IronCalcDateSystem""#));
    assert!(custom.contains("<vt:lpwstr>1900-01-01</vt:lpwstr>"));
    assert!(!read(&mut archive, "xl/workbook.xml").contains("date1904"));
    assert!(read(&mut archive, "[Content_Types].xml").contains("/docProps/custom.xml"));
    assert!(read(&mut archive, "_rels/.rels").contains("docProps/custom.xml"));

    let (_, mut archive) = round_trip(DateSystem::IronCalc);
    assert!(read(&mut archive, "docProps/custom.xml").contains("<vt:lpwstr>1899-12-30</vt:lpwstr>"));

    let (_, mut archive) = round_trip(DateSystem::Excel1904);
    assert!(read(&mut archive, "xl/workbook.xml").contains(r#"<workbookPr date1904="1"/>"#));
    assert!(read(&mut archive, "docProps/custom.xml").contains("<vt:lpwstr>1904-01-01</vt:lpwstr>"));
}

#[test]
fn excel_files_keep_the_1900_system_after_a_round_trip() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let workbook = load_from_xlsx_bytes(&writer.into_inner(), "model", "en", "UTC").unwrap();
    assert_eq!(workbook.settings.date_system, DateSystem::Excel1900);
}

// The LibreOffice DATE workbook is the acceptance test of the IronCalc date
// system: it must agree cell by cell, with no #N/IMPL! involved.
#[test]
fn libreoffice_date_workbook_evaluates_without_nimpl() {
    let mut model = load_from_xlsx(
        "tests/calc_tests/DATE_AND_TIME/DATE_LibreOffice.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    assert_eq!(model.workbook.settings.date_system, DateSystem::IronCalc);
    model.evaluate();
    // The raw values: the cells of the workbook carry date formats
    let text = |cell: &str| {
        let reference = ironcalc_base::expressions::utils::parse_reference_a1(cell).unwrap();
        match model
            .get_cell_value_by_index(0, reference.row, reference.column)
            .unwrap()
        {
            CellValue::Number(n) => format!("{n}"),
            CellValue::String(s) => s,
            CellValue::Boolean(b) => format!("{b}"),
            CellValue::None => "".to_string(),
        }
    };
    assert_eq!(text("E2"), "-100"); // DATE(1899,9,21)
    assert_eq!(text("J3"), "-103615"); // DATE(1616,4,22)
    assert_eq!(text("L3"), "Friday"); // TEXT(J3,"dddd")
    assert_eq!(text("C22"), "-839"); // YEAR(-1000000)
    assert_eq!(text("E22"), "#VALUE!"); // DATE(-839,2,2)
    assert_eq!(text("E25"), "0"); // DATE of an empty cell's parts
    for row in 2..=27 {
        for col in ["B", "C", "D", "E"] {
            let value = text(&format!("{col}{row}"));
            assert_ne!(value, "#N/IMPL!", "{col}{row}");
        }
    }
}
