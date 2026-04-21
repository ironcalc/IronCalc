#![allow(clippy::unwrap_used)]

use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn fn_accrintm() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,7,1)");
    model._set("A3", "10%");
    model._set("A4", "$1,000");

    model._set("B1", "=ACCRINTM(A1,A2,A3,A4)");
    model._set("C1", "=ACCRINTM(A1)");

    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!B1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 50.0).abs() < 1e-9);
        }
        other => unreachable!("Expected number for B1, got {:?}", other),
    }
    assert_eq!(model._get_text("C1"), *"#ERROR!");
}

#[test]
fn fn_accrintm_parameters() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,7,1)");
    model._set("A3", "8%");
    model._set("A4", "1000");

    model._set("B1", "=ACCRINTM(A1,A2,A3,A4,0)");
    model._set("B2", "=ACCRINTM(A1,A2,A3,A4,1)");
    model._set("B3", "=ACCRINTM(A1,A2,A3,A4,4)");
    model._set("C1", "=ACCRINTM(A1,A2,A3,A4)");

    model.evaluate();

    match (
        model.get_cell_value_by_ref("Sheet1!B1"),
        model.get_cell_value_by_ref("Sheet1!B2"),
    ) {
        (Ok(CellValue::Number(v1)), Ok(CellValue::Number(v2))) => {
            assert!(v1 > 0.0 && v2 > 0.0);
        }
        other => unreachable!("Expected numbers for basis test, got {:?}", other),
    }

    match (
        model.get_cell_value_by_ref("Sheet1!B1"),
        model.get_cell_value_by_ref("Sheet1!C1"),
    ) {
        (Ok(CellValue::Number(v1)), Ok(CellValue::Number(v2))) => {
            assert!((v1 - v2).abs() < 1e-12);
        }
        other => unreachable!(
            "Expected matching numbers for default test, got {:?}",
            other
        ),
    }
}

#[test]
fn fn_accrintm_errors() {
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2020,1,1)");
    model._set("A2", "=DATE(2020,7,1)");
    model._set("A3", "8%");
    model._set("A4", "1000");

    model._set("B1", "=ACCRINTM()");
    model._set("B2", "=ACCRINTM(A1,A2,A3)");
    model._set("B3", "=ACCRINTM(A1,A2,A3,A4,0,1)");
    model._set("C1", "=ACCRINTM(A1,A2,A3,A4,-1)");
    model._set("C2", "=ACCRINTM(A1,A2,A3,A4,5)");
    model._set("D1", "=ACCRINTM(A2,A1,A3,A4)");
    model._set("E1", "=ACCRINTM(A1,A2,A3,0)");
    model._set("E2", "=ACCRINTM(A1,A2,A3,-1000)");
    model._set("E3", "=ACCRINTM(A1,A2,-8%,A4)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
    assert_eq!(model._get_text("D1"), *"#NUM!");
    assert_eq!(model._get_text("E2"), *"#NUM!");
    assert_eq!(model._get_text("E3"), *"#NUM!");

    match model.get_cell_value_by_ref("Sheet1!E1") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 0.0).abs() < 1e-9);
        }
        other => unreachable!("Expected 0 for E1, got {:?}", other),
    }
}

#[test]
fn fn_accrintm_combined() {
    let mut model = new_empty_model();
    model._set("C1", "=DATE(2016,4,5)");
    model._set("C2", "=DATE(2019,2,1)");
    model._set("A3", "5%");
    model._set("A4", "1000");
    model._set("B2", "=ACCRINTM(C1,C2,A3,A4)");

    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!B2") {
        Ok(CellValue::Number(v)) => {
            assert!((v - 141.11111111111111).abs() < 1e-9);
        }
        other => unreachable!("Expected number for B2, got {:?}", other),
    }
}

/// Test the exact Excel documentation example for ACCRINTM
/// https://support.microsoft.com/en-us/office/accrintm-function-f62f01f9-5754-4cc4-805b-0e70199328a7
/// Issue: 39539 (April 1, 2008)
/// Settlement: 39614 (June 15, 2008)
/// Rate: 0.1 (10%)
/// Par: 1000
/// Basis: 3 (Actual/365)
/// Expected result: 20.54794521
#[test]
fn fn_accrintm_excel_docs_example() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539,39614,0.1,1000,3)");
    model.evaluate();

    match model.get_cell_value_by_ref("Sheet1!A1") {
        Ok(CellValue::Number(v)) => {
            println!("ACCRINTM Excel example result: {}", v);
            assert!(
                (v - 20.54794520547945).abs() < 1e-6,
                "Expected ~20.54794521, got {}",
                v
            );
        }
        other => unreachable!("Expected number for A1, got {:?}", other),
    }
}
