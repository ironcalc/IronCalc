#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn they_spill() {
    let mut model = new_empty_model();
    model._set("A1", "42");
    model._set("A2", "5");
    model._set("A3", "7");

    model._set("B1", "=A1:A3");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"42");
    assert_eq!(model._get_text("B2"), *"5");
    assert_eq!(model._get_text("B3"), *"7");
}

#[test]
fn spill_error() {
    let mut model = new_empty_model();
    model._set("A1", "42");
    model._set("A2", "5");
    model._set("A3", "7");

    model._set("B1", "=A1:A3");
    model._set("B2", "4");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#SPILL!");
    assert_eq!(model._get_text("B2"), *"4");
    assert_eq!(model._get_text("B3"), *"");
}
