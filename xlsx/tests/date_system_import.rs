#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use ironcalc::{export::save_xlsx_to_writer, import::load_from_xlsx, import::load_from_xlsx_bytes};
use ironcalc_base::{types::DateSystem, Model};

#[test]
fn excel_files_use_the_1900_date_system() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    assert_eq!(model.workbook.settings.date_system, DateSystem::Excel1900);
}

#[test]
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
