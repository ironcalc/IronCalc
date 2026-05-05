#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── BYCOL ─────────────────────────────────────────────────────────────────────

#[test]
fn bycol_sum_each_column() {
    let mut model = new_empty_model();
    // 2×3 grid
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    // SUM of each column: 5, 7, 9
    model._set("A4", "=BYCOL(A1:C2, LAMBDA(col, SUM(col)))");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "5");
    assert_eq!(model._get_text("B4"), "7");
    assert_eq!(model._get_text("C4"), "9");
    assert_eq!(model._get_text("D4"), "");
}

#[test]
fn bycol_max_each_column() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("B1", "1");
    model._set("A2", "7");
    model._set("B2", "4");
    model._set("A4", "=BYCOL(A1:B2, LAMBDA(col, MAX(col)))");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "7");
    assert_eq!(model._get_text("B4"), "4");
    assert_eq!(model._get_text("C4"), "");
}

#[test]
fn bycol_single_column() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "=BYCOL(A1:A3, LAMBDA(col, SUM(col)))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "60");
    assert_eq!(model._get_text("C1"), "");
}

#[test]
fn bycol_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=BYCOL(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── BYROW ─────────────────────────────────────────────────────────────────────

#[test]
fn byrow_sum_each_row() {
    let mut model = new_empty_model();
    // 3×2 grid
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("A3", "5");
    model._set("B3", "6");
    // SUM of each row: 3, 7, 11
    model._set("D1", "=BYROW(A1:B3, LAMBDA(row, SUM(row)))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("D2"), "7");
    assert_eq!(model._get_text("D3"), "11");
    assert_eq!(model._get_text("D4"), "");
}

#[test]
fn byrow_min_each_row() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("B1", "2");
    model._set("A2", "8");
    model._set("B2", "3");
    model._set("D1", "=BYROW(A1:B2, LAMBDA(row, MIN(row)))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("D2"), "3");
    assert_eq!(model._get_text("D3"), "");
}

#[test]
fn byrow_single_row() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A3", "=BYROW(A1:C1, LAMBDA(row, SUM(row)))");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "60");
    assert_eq!(model._get_text("A4"), "");
}

#[test]
fn byrow_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=BYROW(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}
