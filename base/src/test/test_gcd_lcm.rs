#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM()");
    model._set("A2", "=LCM(2, 3)");

    model._set("A3", "=GCD()");
    model._set("A4", "=GCD(10, 25)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"5");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM({2, 3}, {4, 5, 6})");
    model._set("A2", "=GCD({10, 25}, {35, 40, 50})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"60");
    assert_eq!(model._get_text("A2"), *"5");
}
