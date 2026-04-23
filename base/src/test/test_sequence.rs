#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_sequence_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("A3"), "3");
}

#[test]
fn test_sequence_rows_cols() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(2,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("C1"), "3");
    assert_eq!(model._get_text("A2"), "4");
    assert_eq!(model._get_text("B2"), "5");
    assert_eq!(model._get_text("C2"), "6");
}

#[test]
fn test_sequence_start_step() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(4,1,10,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "10");
    assert_eq!(model._get_text("A2"), "15");
    assert_eq!(model._get_text("A3"), "20");
    assert_eq!(model._get_text("A4"), "25");
}

#[test]
fn test_sequence_negative_step() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(3,1,10,-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "10");
    assert_eq!(model._get_text("A2"), "9");
    assert_eq!(model._get_text("A3"), "8");
}

#[test]
fn test_sequence_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");

    model._set("B1", "=SEQUENCE(1,1,1,1,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#ERROR!");
}

#[test]
fn test_sequence_invalid_rows() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CALC!");
}
