#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn smoke_test() {
    let mut model = new_empty_model();
    model._set("A1", "=STDEV.P(10, 12, 23, 23, 16, 23, 21)");
    model._set("A2", "=STDEV.S(10, 12, 23, 23, 16, 23, 21)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"5.174505793");

    assert_eq!(model._get_text("A2"), *"5.589105048");
}

#[test]
fn numbers() {
    let mut model = new_empty_model();

    model._set("A2", "24");
    model._set("A3", "25");
    model._set("A4", "27");
    model._set("A5", "23");
    model._set("A6", "45");
    model._set("A7", "23.5");
    model._set("A8", "34");
    model._set("A9", "23");
    model._set("A10", "23");
    model._set("A11", "TRUE");
    model._set("A12", "'23");
    model._set("A13", "Text");
    model._set("A14", "FALSE");
    model._set("A15", "45");

    model._set("B1", "=VAR.P(A2:A15)");
    model._set("B2", "=VAR.S(A2:A15)");
    model._set("B3", "=VARA(A2:A15)");
    model._set("B4", "=VARPA(A2:A15)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"71.9625");
    assert_eq!(model._get_text("B2"), *"79.958333333");
    assert_eq!(model._get_text("B3"), *"240.248626374");
    assert_eq!(model._get_text("B4"), *"223.088010204");
}
