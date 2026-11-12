#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Read};

use ironcalc::{export::save_xlsx_to_writer, import::load_from_xlsx_bytes};
use ironcalc_base::Model;

// Excel offers to repair a file with a cached #CIRC!, #ERROR! or #N/IMPL (issue #1631)
#[test]
fn ironcalc_only_errors_have_no_cached_value() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let inputs = ["=A1+1", "=(", "=INFO(\"osversion\")", "=1/0"];
    for (row, input) in (1..).zip(inputs) {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.evaluate();

    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let bytes = writer.into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut sheet)
        .unwrap();

    assert!(sheet.contains(r#"<c r="A1"><f>A1+1</f></c>"#));
    assert!(sheet.contains(r#"<c r="A2"><f>(</f></c>"#));
    assert!(sheet.contains(
        r#"<c r="A3" cm="1"><f t="array" ref="A3:A3">INFO(&quot;osversion&quot;)</f></c>"#
    ));
    assert!(sheet.contains(r#"<c r="A4" t="e"><f>1/0</f><v>#DIV/0!</v></c>"#));

    // IronCalc reads them back and calculates them again
    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let expected = ["#CIRC!", "#ERROR!", "#N/IMPL!", "#DIV/0!"];
    for (row, value) in (1..).zip(expected) {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), value);
    }
}

#[test]
fn spilled_ironcalc_only_errors_have_no_cached_value() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let inputs = ["1", "=INFO(\"osversion\")", "=("];
    for (row, input) in (1..).zip(inputs) {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.set_user_input(0, 1, 3, "=A1:A3".to_string()).unwrap();
    model.evaluate();

    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let bytes = writer.into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut sheet)
        .unwrap();

    assert!(sheet.contains(r#"<c r="C2"></c>"#));
    assert!(sheet.contains(r#"<c r="C3"></c>"#));

    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    let expected = ["1", "#N/IMPL!", "#ERROR!"];
    for (row, value) in (1..).zip(expected) {
        assert_eq!(model.get_formatted_cell_value(0, row, 3).unwrap(), value);
    }
}
