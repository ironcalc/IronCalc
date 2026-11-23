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

const XLSX_ERRORS: [&str; 7] = [
    "#NULL!", "#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#NUM!", "#N/A",
];

/// Saves the model: the bytes of the xlsx file, the names of its parts and its first sheet
fn save(model: &Model) -> (Vec<u8>, Vec<String>, String) {
    let writer = save_xlsx_to_writer(model, Cursor::new(Vec::new())).unwrap();
    let bytes = writer.into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.clone())).unwrap();
    let parts = archive
        .file_names()
        .map(|name| name.unwrap().to_string())
        .collect();
    let mut sheet = String::new();
    archive
        .by_name("xl/worksheets/sheet1.xml")
        .unwrap()
        .read_to_string(&mut sheet)
        .unwrap();
    (bytes, parts, sheet)
}

/// The value of every `t="e"` cell of a sheet
fn error_values(sheet: &str) -> Vec<&str> {
    sheet
        .split("<c ")
        .skip(1)
        .filter(|cell| cell.contains(r#" t="e""#))
        .map(|cell| {
            let (_, value) = cell.split_once("<v>").unwrap();
            value.split_once("</v>").unwrap().0
        })
        .collect()
}

fn has_rich_data(parts: &[String]) -> bool {
    parts.iter().any(|name| name.starts_with("xl/richData/"))
}

// A `t="e"` cell only takes the errors of the xlsx standard. With any other calamine
// does not read the sheet, and Excel offers to repair the file.
#[test]
fn error_values_are_xlsx_errors() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    // (the error, how it is saved, what IronCalc reads back)
    // Excel saves #SPILL! and #CALC! as #VALUE!, with the actual error in the value
    // metadata. The errors only IronCalc has are saved as #VALUE! too.
    let cases = [
        ("#NULL!", r#" t="e"><v>#NULL!</v>"#, "#NULL!"),
        ("#DIV/0!", r#" t="e"><v>#DIV/0!</v>"#, "#DIV/0!"),
        ("#VALUE!", r#" t="e"><v>#VALUE!</v>"#, "#VALUE!"),
        ("#REF!", r#" t="e"><v>#REF!</v>"#, "#REF!"),
        ("#NAME?", r#" t="e"><v>#NAME?</v>"#, "#NAME?"),
        ("#NUM!", r#" t="e"><v>#NUM!</v>"#, "#NUM!"),
        ("#N/A", r#" t="e"><v>#N/A</v>"#, "#N/A"),
        ("#SPILL!", r#" t="e" vm="1"><v>#VALUE!</v>"#, "#SPILL!"),
        ("#CALC!", r#" t="e" vm="2"><v>#VALUE!</v>"#, "#CALC!"),
        ("#ERROR!", r#" t="e"><v>#VALUE!</v>"#, "#VALUE!"),
        ("#N/IMPL!", r#" t="e"><v>#VALUE!</v>"#, "#VALUE!"),
        ("#CIRC!", r#" t="e"><v>#VALUE!</v>"#, "#VALUE!"),
    ];
    for (row, (input, _, _)) in (1..).zip(cases) {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.evaluate();
    for (row, (input, _, _)) in (1..).zip(cases) {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), input);
    }

    let (bytes, parts, sheet) = save(&model);
    for (row, (_, saved, _)) in (1..).zip(cases) {
        assert!(sheet.contains(&format!(r#"<c r="A{row}"{saved}</c>"#)));
    }
    assert_eq!(error_values(&sheet).len(), cases.len());
    assert!(has_rich_data(&parts));

    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    for (row, (_, _, read)) in (1..).zip(cases) {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), read);
    }
    model.evaluate();
    for (row, (_, _, read)) in (1..).zip(cases) {
        assert_eq!(model.get_formatted_cell_value(0, row, 1).unwrap(), read);
    }
}

// Excel caches #SPILL! and #CALC! as #VALUE!, with the actual error in the value metadata
#[test]
fn spill_and_calc_are_cached_as_value() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let inputs = [
        (1, 1, "=LAMBDA(x,x)"),
        (2, 1, "=SEQUENCE(2)"),
        (3, 1, "in the way"),
        (4, 1, "=#SPILL!"),
        // each row gives two values, so both C1 and C2 are #CALC!
        (1, 3, "=BYROW(E1:F2,LAMBDA(r,r))"),
    ];
    for (row, column, input) in inputs {
        model
            .set_user_input(0, row, column, input.to_string())
            .unwrap();
    }
    model.evaluate();
    let expected = [
        (1, 1, "#CALC!"),
        (2, 1, "#SPILL!"),
        (4, 1, "#SPILL!"),
        (1, 3, "#CALC!"),
        (2, 3, "#CALC!"),
    ];
    for (row, column, value) in expected {
        assert_eq!(
            model.get_formatted_cell_value(0, row, column).unwrap(),
            value
        );
    }

    let (bytes, parts, sheet) = save(&model);
    assert!(sheet.contains(
        r#"<c r="A1" t="e" vm="2" cm="1"><f t="array" ref="A1:A1">_xlfn.LAMBDA(_xlpm.x,_xlpm.x)</f><v>#VALUE!</v></c>"#
    ));
    assert!(sheet.contains(
        r#"<c r="A2" t="e" vm="1" cm="1"><f t="array" ref="A2:A2">_xlfn.SEQUENCE(2)</f><v>#VALUE!</v></c>"#
    ));
    assert!(sheet.contains(r#"<c r="A4" t="e" vm="1"><f>#SPILL!</f><v>#VALUE!</v></c>"#));
    assert!(sheet.contains(r#"<c r="C2" t="e" vm="2"><v>#VALUE!</v></c>"#));
    let values = error_values(&sheet);
    assert_eq!(values.len(), expected.len());
    for value in values {
        assert!(XLSX_ERRORS.contains(&value), "{value} is not an xlsx error");
    }
    for part in [
        "xl/richData/rdrichvalue.xml",
        "xl/richData/rdrichvaluestructure.xml",
        "xl/richData/rdRichValueTypes.xml",
    ] {
        assert!(parts.iter().any(|name| name == part), "{part} is missing");
    }

    // IronCalc reads the actual errors back, also before calculating them again
    let workbook = load_from_xlsx_bytes(&bytes, "model", "en", "UTC").unwrap();
    let mut model = Model::from_workbook(workbook, "en").unwrap();
    for (row, column, value) in expected {
        assert_eq!(
            model.get_formatted_cell_value(0, row, column).unwrap(),
            value
        );
    }
    model.evaluate();
    for (row, column, value) in expected {
        assert_eq!(
            model.get_formatted_cell_value(0, row, column).unwrap(),
            value
        );
    }
}

// The rich values are only written for the errors that need them
#[test]
fn no_rich_data_without_spill_or_calc() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let inputs = ["#N/A", "#ERROR!", "=1/0", "=A3+1", "=UnknownName"];
    for (row, input) in (1..).zip(inputs) {
        model.set_user_input(0, row, 1, input.to_string()).unwrap();
    }
    model.evaluate();

    let (_, parts, sheet) = save(&model);
    assert!(!has_rich_data(&parts));
    assert!(!sheet.contains("vm="));
    for value in error_values(&sheet) {
        assert!(XLSX_ERRORS.contains(&value), "{value} is not an xlsx error");
    }
}
