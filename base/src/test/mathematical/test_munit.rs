#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_munit_2x2() {
    let mut model = new_empty_model();
    model._set("A1", "=MUNIT(2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "0");
    assert_eq!(model._get_text("A2"), "0");
    assert_eq!(model._get_text("B2"), "1");
}

#[test]
fn test_munit_3x3() {
    let mut model = new_empty_model();
    model._set("A1", "=MUNIT(3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "0");
    assert_eq!(model._get_text("C1"), "0");
    assert_eq!(model._get_text("A2"), "0");
    assert_eq!(model._get_text("B2"), "1");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("A3"), "0");
    assert_eq!(model._get_text("B3"), "0");
    assert_eq!(model._get_text("C3"), "1");
}

#[test]
fn test_munit_zero_error() {
    let mut model = new_empty_model();
    model._set("A1", "=MUNIT(0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn test_munit_negative_error() {
    let mut model = new_empty_model();
    model._set("A1", "=MUNIT(-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

#[test]
fn test_munit_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MUNIT()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}
