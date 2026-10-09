#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── ROWS ──────────────────────────────────────────────────────────────────────

#[test]
fn test_rows_reference() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("C1", "=ROWS(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_rows_dynamic_array_function() {
    let mut model = new_empty_model();
    model._set("C1", "=ROWS(SEQUENCE(3))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_rows_array_literal() {
    let mut model = new_empty_model();
    model._set("C1", "=ROWS({1;2;3})");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_rows_range_arithmetic() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("C1", "=ROWS(A1:A3*2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "3");
}

#[test]
fn test_rows_filtered_array_keeps_its_own_size() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    // FILTER drops the 1, so the spilled array has two rows
    model._set("C1", "=ROWS(FILTER(A1:A3,A1:A3>1))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "2");
}

#[test]
fn test_rows_scalar_is_one() {
    let mut model = new_empty_model();
    model._set("C1", "=ROWS(SEQUENCE(1,4,1,1)*0+7)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
}

// ── COLUMNS ───────────────────────────────────────────────────────────────────

#[test]
fn test_columns_reference() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("E1", "=COLUMNS(A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
}

#[test]
fn test_columns_dynamic_array_function() {
    let mut model = new_empty_model();
    model._set("E1", "=COLUMNS(SEQUENCE(1,4))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "4");
}

#[test]
fn test_columns_array_literal() {
    let mut model = new_empty_model();
    model._set("E1", "=COLUMNS({1,2,3,4})");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "4");
}

#[test]
fn test_columns_transposed_array() {
    let mut model = new_empty_model();
    model._set("E1", "=COLUMNS(TRANSPOSE({1;2;3;4}))");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "4");
}

#[test]
fn test_columns_scalar_is_one() {
    let mut model = new_empty_model();
    model._set("E1", "=COLUMNS(1+1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "1");
}

// ── shared behaviour ──────────────────────────────────────────────────────────

#[test]
fn test_rows_and_columns_propagate_errors() {
    let mut model = new_empty_model();
    model._set("C1", "=ROWS(1/0)");
    model._set("C2", "=COLUMNS(1/0)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#DIV/0!");
    assert_eq!(model._get_text("C2"), "#DIV/0!");
}

#[test]
fn test_rows_and_columns_intersection_operator_still_narrows() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    // @A1:C1 in B3 is column-aligned -> B1, a single cell.
    model._set("B3", "=COLUMNS(@A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("B3"), "1");
}
