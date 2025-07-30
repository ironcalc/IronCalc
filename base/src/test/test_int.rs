#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn int_wrong_argument_count() {
    let mut model = new_empty_model();
    model._set("A1", "=INT()");
    model._set("A2", "=INT(5.7, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn int_basic_floor_behavior() {
    let mut model = new_empty_model();
    // INT returns the largest integer less than or equal to the number (floor function)
    model._set("A1", "=INT(5.7)"); // 5
    model._set("A2", "=INT(3.9)"); // 3
    model._set("A3", "=INT(5)"); // whole numbers unchanged
    model._set("A4", "=INT(0)"); // zero

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"5");
    assert_eq!(model._get_text("A2"), *"3");
    assert_eq!(model._get_text("A3"), *"5");
    assert_eq!(model._get_text("A4"), *"0");
}

#[test]
fn int_negative_floor_behavior() {
    let mut model = new_empty_model();
    // Critical: INT floors towards negative infinity, not towards zero
    // This is different from truncation behavior
    model._set("A1", "=INT(-5.7)"); // -6 (not -5)
    model._set("A2", "=INT(-3.1)"); // -4 (not -3)
    model._set("A3", "=INT(-0.9)"); // -1 (not 0)

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-6");
    assert_eq!(model._get_text("A2"), *"-4");
    assert_eq!(model._get_text("A3"), *"-1");
}
