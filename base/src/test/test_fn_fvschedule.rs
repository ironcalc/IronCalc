#![allow(clippy::unwrap_used)]

use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn computation() {
    let mut model = new_empty_model();
    model._set("B1", "0.1");
    model._set("B2", "0.2");
    model._set("A1", "=FVSCHEDULE(100,B1:B2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "132");
}

#[test]
fn fvschedule_basic_with_precise_assertion() {
    let mut model = new_empty_model();
    model._set("A1", "1000");
    model._set("B1", "0.09");
    model._set("B2", "0.11");
    model._set("B3", "0.1");

    model._set("C1", "=FVSCHEDULE(A1,B1:B3)");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!C1"),
        Ok(CellValue::Number(1330.89))
    );
}

#[test]
fn fvschedule_compound_rates() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "0.1");
    model._set("A3", "0.2");
    model._set("A4", "0.3");

    model._set("B1", "=FVSCHEDULE(A1, A2:A4)");

    model.evaluate();

    // 1 * (1+0.1) * (1+0.2) * (1+0.3) = 1 * 1.1 * 1.2 * 1.3 = 1.716
    assert_eq!(model._get_text("B1"), "1.716");
}

#[test]
fn fvschedule_ignore_non_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "0.1");
    model._set("A3", "foo"); // non-numeric value should be ignored
    model._set("A4", "0.2");

    model._set("B1", "=FVSCHEDULE(A1, A2:A4)");

    model.evaluate();

    // 1 * (1+0.1) * (1+0.2) = 1 * 1.1 * 1.2 = 1.32
    assert_eq!(model._get_text("B1"), "1.32");
}

#[test]
fn fvschedule_argument_count() {
    let mut model = new_empty_model();
    model._set("A1", "=FVSCHEDULE()");
    model._set("A2", "=FVSCHEDULE(1)");
    model._set("A3", "=FVSCHEDULE(1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn fvschedule_edge_cases() {
    let mut model = new_empty_model();

    // Test with zero principal
    model._set("A1", "0");
    model._set("A2", "0.1");
    model._set("A3", "0.2");
    model._set("B1", "=FVSCHEDULE(A1, A2:A3)");

    // Test with negative principal
    model._set("C1", "-100");
    model._set("D1", "=FVSCHEDULE(C1, A2:A3)");

    // Test with zero rates
    model._set("E1", "100");
    model._set("E2", "0");
    model._set("E3", "0");
    model._set("F1", "=FVSCHEDULE(E1, E2:E3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0"); // 0 * anything = 0
    assert_eq!(model._get_text("D1"), "-132"); // -100 * 1.1 * 1.2 = -132
    assert_eq!(model._get_text("F1"), "100"); // 100 * 1 * 1 = 100
}

#[test]
fn fvschedule_rate_validation() {
    let mut model = new_empty_model();

    // Test with rate exactly -1 (should cause error due to validation in patch 1)
    model._set("A1", "100");
    model._set("A2", "-1");
    model._set("A3", "0.1");
    model._set("B1", "=FVSCHEDULE(A1, A2:A3)");

    // Test with rate less than -1 (should cause error)
    model._set("C1", "100");
    model._set("C2", "-1.5");
    model._set("C3", "0.1");
    model._set("D1", "=FVSCHEDULE(C1, C2:C3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "#NUM!");
    assert_eq!(model._get_text("D1"), "#NUM!");
}
