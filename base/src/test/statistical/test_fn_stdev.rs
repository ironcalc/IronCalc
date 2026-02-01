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

    model._set("B1", "=STDEV.P(A2:A15)");
    model._set("B2", "=STDEV.S(A2:A15)");
    model._set("B3", "=STDEVA(A2:A15)");
    model._set("B4", "=STDEVPA(A2:A15)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"8.483071378");
    assert_eq!(model._get_text("B2"), *"8.941942369");
    assert_eq!(model._get_text("B3"), *"15.499955689");
    assert_eq!(model._get_text("B4"), *"14.936131032");
}
