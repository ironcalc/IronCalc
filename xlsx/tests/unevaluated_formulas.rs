#![allow(clippy::unwrap_used)]

use std::io::{Cursor, Read};

use ironcalc::{export::save_xlsx_to_writer, import::load_from_xlsx_bytes};
use ironcalc_base::Model;

// A formula that was never evaluated (typed while evaluation was paused) has no
// value to cache, so it is written without one (issue #1629)
#[test]
fn unevaluated_formulas_are_written_without_a_cached_value() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "=1+1".to_string()).unwrap();
    model
        .set_user_array_formula(0, 1, 2, 1, 2, "=A1*{1;2}")
        .unwrap();

    let writer = save_xlsx_to_writer(&model, Cursor::new(Vec::new())).unwrap();
    let bytes = writer.into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut sheet)
        .unwrap();

    assert!(sheet.contains(r#"<c r="A1"><f>1+1</f></c>"#), "{sheet}");
    assert!(
        sheet.contains(r#"<c r="B1"><f t="array" ref="B1:B2">A1*{1;2}</f></c>"#),
        "{sheet}"
    );

    // IronCalc calculates them when it reads the file again
    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "4");
}
