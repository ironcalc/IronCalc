#![allow(clippy::unwrap_used)]

use crate::expressions::types::Area;
use crate::test::util::new_empty_model;

fn area(sheet: u32, row: i32, column: i32, width: i32, height: i32) -> Area {
    Area {
        sheet,
        row,
        column,
        width,
        height,
    }
}

#[test]
fn references_and_ranges() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 2, 2, "=A1+SUM(C3:D5)*$A$1+A1".to_string())
        .unwrap();
    assert_eq!(
        model.get_cell_precedents(0, 2, 2).unwrap(),
        vec![area(0, 1, 1, 1, 1), area(0, 3, 3, 2, 3)]
    );
}

#[test]
fn no_formula() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "42".to_string()).unwrap();
    assert!(model.get_cell_precedents(0, 1, 1).unwrap().is_empty());
    assert!(model.get_cell_precedents(0, 9, 9).unwrap().is_empty());
    model.set_user_input(0, 1, 2, "=1+2".to_string()).unwrap();
    assert!(model.get_cell_precedents(0, 1, 2).unwrap().is_empty());
    assert!(model.get_cell_precedents(7, 1, 1).is_err());
}

#[test]
fn other_sheets_and_full_columns() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model
        .set_user_input(0, 1, 1, "=Data!B2+SUM(C:C)+SUM(2:3)".to_string())
        .unwrap();
    assert_eq!(
        model.get_cell_precedents(0, 1, 1).unwrap(),
        vec![
            area(1, 2, 2, 1, 1),
            area(0, 1, 3, 1, 1_048_576),
            area(0, 2, 1, 16_384, 2),
        ]
    );
}

#[test]
fn defined_names() {
    let mut model = new_empty_model();
    model.new_defined_name("one", None, "Sheet1!$D$4").unwrap();
    model
        .new_defined_name("block", None, "Sheet1!$E$1:$F$2")
        .unwrap();
    model
        .set_user_input(0, 1, 1, "=one+SUM(block)".to_string())
        .unwrap();
    assert_eq!(
        model.get_cell_precedents(0, 1, 1).unwrap(),
        vec![area(0, 4, 4, 1, 1), area(0, 1, 5, 2, 2)]
    );
}

#[test]
fn range_operator_and_spill() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "=SUM(B1:C2:D4)+SUM(E1#)".to_string())
        .unwrap();
    assert_eq!(
        model.get_cell_precedents(0, 1, 1).unwrap(),
        vec![
            area(0, 1, 2, 2, 2),
            area(0, 4, 4, 1, 1),
            area(0, 1, 5, 1, 1)
        ]
    );
}
