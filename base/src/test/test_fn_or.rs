#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_or() {
    let mut model = new_empty_model();
    model._set("A1", "=OR(1, 0)");
    model._set("A2", "=OR(0, 0)");
    model._set("A3", "=OR(true, false)");
    model._set("A4", "=OR(false, false)");

    model.evaluate();
    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"FALSE");
}

#[test]
fn fn_or_no_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=OR()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn fn_or_missing_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=OR(,)");
    model._set("A2", "=OR(,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
    assert_eq!(model._get_text("A2"), *"TRUE");
}
