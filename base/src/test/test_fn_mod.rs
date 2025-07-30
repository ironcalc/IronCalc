#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn mod_function() {
    let mut model = new_empty_model();
    model._set("A1", "=MOD(9,4)");
    model._set("A2", "=MOD(-3,2)");
    model._set("A3", "=MOD(3,-2)");
    model._set("A4", "=MOD(3,0)");
    model._set("A5", "=MOD(1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"-1");
    assert_eq!(model._get_text("A4"), *"#DIV/0!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
}

#[test]
fn quotient_function() {
    let mut model = new_empty_model();
    model._set("A1", "=QUOTIENT(5,2)");
    model._set("A2", "=QUOTIENT(5,-2)");
    model._set("A3", "=QUOTIENT(5,0)");
    model._set("A4", "=QUOTIENT(5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"-2"); // Fixed: should truncate toward zero, not floor
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn quotient_function_truncate_toward_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=QUOTIENT(5,-2)"); // positive, negative truncation (original bug case)
    model._set("A2", "=QUOTIENT(7,3)"); // positive, positive truncation
    model._set("A3", "=QUOTIENT(-7,3)"); // negative, positive truncation
    model._set("A4", "=QUOTIENT(-7,-3)"); // negative, negative truncation
    model._set("A5", "=QUOTIENT(6,3)"); // exact division

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-2"); // The key fix
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"-2");
    assert_eq!(model._get_text("A4"), *"2");
    assert_eq!(model._get_text("A5"), *"2");
}
