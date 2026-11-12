#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Read};

use ironcalc::export::save_xlsx_to_writer;
use ironcalc_base::{expressions::types::Area, types::SheetState, Model};

fn export(model: &Model) -> zip::ZipArchive<Cursor<Vec<u8>>> {
    let writer = save_xlsx_to_writer(model, Cursor::new(Vec::new())).unwrap();
    zip::ZipArchive::new(Cursor::new(writer.into_inner())).unwrap()
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

fn new_model() -> Model<'static> {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.new_sheet();
    model.new_sheet();
    model
}

#[test]
fn default_is_a1_of_the_first_sheet() {
    let model = new_model();
    let mut archive = export(&model);
    assert!(read(&mut archive, "xl/workbook.xml").contains(r#"<workbookView activeTab="0"/>"#));
    let sheet1 = read(&mut archive, "xl/worksheets/sheet1.xml");
    assert!(sheet1.contains(r#"<sheetView workbookViewId="0" tabSelected="1">"#));
    assert!(sheet1.contains(r#"<selection activeCell="A1" sqref="A1"/>"#));
}

#[test]
fn focus_on_open() {
    let mut model = new_model();
    model.set_focus_on_open(Some((1, 5, 3))).unwrap();
    let mut archive = export(&model);
    assert!(read(&mut archive, "xl/workbook.xml").contains(r#"<workbookView activeTab="1"/>"#));

    let sheet1 = read(&mut archive, "xl/worksheets/sheet1.xml");
    assert!(!sheet1.contains("tabSelected"));
    assert!(sheet1.contains(r#"<selection activeCell="A1" sqref="A1"/>"#));

    let sheet2 = read(&mut archive, "xl/worksheets/sheet2.xml");
    assert!(sheet2.contains(r#"<sheetView workbookViewId="0" tabSelected="1">"#));
    assert!(sheet2.contains(r#"<selection activeCell="C5" sqref="C5"/>"#));
}

#[test]
fn far_away_cell_is_scrolled_into_view() {
    let mut model = new_model();
    model.set_focus_on_open(Some((0, 500, 30))).unwrap();
    let mut archive = export(&model);
    let sheet1 = read(&mut archive, "xl/worksheets/sheet1.xml");
    assert!(
        sheet1.contains(r#"<sheetView workbookViewId="0" tabSelected="1" topLeftCell="AD500">"#)
    );
    assert!(sheet1.contains(r#"<selection activeCell="AD500" sqref="AD500"/>"#));
}

#[test]
fn far_away_cell_with_frozen_rows() {
    let mut model = new_model();
    model.set_frozen_rows(0, 2).unwrap();
    model.set_focus_on_open(Some((0, 500, 30))).unwrap();
    let mut archive = export(&model);
    let sheet1 = read(&mut archive, "xl/worksheets/sheet1.xml");
    assert!(sheet1.contains(r#"tabSelected="1" topLeftCell="AD1">"#));
    assert!(sheet1.contains(
        r#"<pane ySplit="2" topLeftCell="AD500" activePane="bottomLeft" state="frozen"/>"#
    ));
    assert!(sheet1.contains(r#"<selection pane="bottomLeft" activeCell="AD500" sqref="AD500"/>"#));
}

#[test]
fn merged_cell_is_selected_whole() {
    let mut model = new_model();
    model
        .merge_cells(&Area {
            sheet: 0,
            row: 2,
            column: 2,
            width: 2,
            height: 3,
        })
        .unwrap();
    model.set_focus_on_open(Some((0, 3, 3))).unwrap();
    let mut archive = export(&model);
    let sheet1 = read(&mut archive, "xl/worksheets/sheet1.xml");
    assert!(sheet1.contains(r#"<selection activeCell="B2" sqref="B2:C4"/>"#));
}

#[test]
fn hidden_sheet_is_ignored() {
    let mut model = new_model();
    model.set_focus_on_open(Some((1, 5, 3))).unwrap();
    model.set_sheet_state(1, SheetState::Hidden).unwrap();
    let mut archive = export(&model);
    assert!(read(&mut archive, "xl/workbook.xml").contains(r#"<workbookView activeTab="0"/>"#));
    let sheet2 = read(&mut archive, "xl/worksheets/sheet2.xml");
    assert!(!sheet2.contains("tabSelected"));
    assert!(sheet2.contains(r#"<selection activeCell="A1" sqref="A1"/>"#));
}
