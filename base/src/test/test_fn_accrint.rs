#![allow(clippy::unwrap_used)]

use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn fn_accrint() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,1,1)");
    model._set("A3", "=DATE(2020,1,31)");
    model._set("A4", "10%");
    model._set("A5", "$1,000");
    model._set("A6", "2");

    model._set("B1", "=ACCRINT(A1,A2,A3,A4,A5,A6)");
    model._set("C1", "=ACCRINT(A1)");
    model._set("C2", "=ACCRINT(A1,A2,A3,A4,A5,3)");

    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!B1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 8.333333333333334).abs() < 1e-9);
        }
        other => unreachable!("Expected number for B1, got {:?}", other),
    }
    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
}

#[test]
fn fn_accrint_parameters() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,1,1)");
    model._set("A3", "=DATE(2020,7,1)");
    model._set("A4", "8%");
    model._set("A5", "1000");

    model._set("B1", "=ACCRINT(A1,A2,A3,A4,A5,2,0,TRUE)");
    model._set("B2", "=ACCRINT(A1,A2,A3,A4,A5,2,1,TRUE)");
    model._set("B3", "=ACCRINT(A1,A2,A3,A4,A5,2,4,TRUE)");
    model._set("B4", "=ACCRINT(A1,A2,A3,A4,A5,1)");
    model._set("B5", "=ACCRINT(A1,A2,A3,A4,A5,4)");
    model._set("B6", "=ACCRINT(A1,A2,A3,A4,A5,2)");
    model._set("B7", "=ACCRINT(A1,A2,A3,A4,A5,2,0)");

    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!B1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 40.0).abs() < 1e-9);
        }
        other => unreachable!("Expected number for B1, got {:?}", other),
    }

    match (
        model.get_cell_value_by_ref("Sheet1!B1"),
        model.get_cell_value_by_ref("Sheet1!B6"),
    ) {
        (Ok(CellValue::Number(v1)), Ok(CellValue::Number(v2))) => {
            assert!((v1 - v2).abs() < 1e-12);
        }
        other => unreachable!("Expected matching numbers, got {:?}", other),
    }
}

#[test]
fn fn_accrint_errors() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,1,1)");
    model._set("A3", "=DATE(2020,7,1)");
    model._set("A4", "8%");
    model._set("A5", "1000");

    model._set("B1", "=ACCRINT()");
    model._set("B2", "=ACCRINT(A1,A2,A3,A4,A5)");
    model._set("B3", "=ACCRINT(A1,A2,A3,A4,A5,2,0,TRUE,1)");
    model._set("C1", "=ACCRINT(A1,A2,A3,A4,A5,0)");
    model._set("C2", "=ACCRINT(A1,A2,A3,A4,A5,3)");
    model._set("C3", "=ACCRINT(A1,A2,A3,A4,A5,-1)");
    model._set("D1", "=ACCRINT(A1,A2,A3,A4,A5,2,-1)");
    model._set("D2", "=ACCRINT(A1,A2,A3,A4,A5,2,5)");
    model._set("E1", "=ACCRINT(A3,A2,A1,A4,A5,2)");
    model._set("E2", "=ACCRINT(A1,A3,A1,A4,A5,2)");
    model._set("F1", "=ACCRINT(A1,A2,A3,A4,0,2)");
    model._set("F2", "=ACCRINT(A1,A2,A3,A4,-1000,2)");
    model._set("F3", "=ACCRINT(A1,A2,A3,-8%,A5,2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
    assert_eq!(model._get_text("C3"), *"#NUM!");
    assert_eq!(model._get_text("D1"), *"#NUM!");
    assert_eq!(model._get_text("D2"), *"#NUM!");
    assert_eq!(model._get_text("E1"), *"#NUM!");
    assert_eq!(model._get_text("E2"), *"#NUM!");
    assert_eq!(model._get_text("F2"), *"#NUM!");
    assert_eq!(model._get_text("F3"), *"#NUM!");

    match model.get_cell_value_by_ref("Sheet1!F1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 0.0).abs() < 1e-9);
        }
        other => unreachable!("Expected 0 for F1, got {:?}", other),
    }
}

#[test]
fn fn_accrint_combined() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2018,10,15)");
    model._set("A2", "=DATE(2019,2,1)");
    model._set("A3", "5%");
    model._set("A4", "1000");

    model._set("B1", "=ACCRINT(A1,A1,A2,A3,A4,2)");

    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!B1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 14.722222222222221).abs() < 1e-9);
        }
        other => unreachable!("Expected number for B1, got {:?}", other),
    }
}
