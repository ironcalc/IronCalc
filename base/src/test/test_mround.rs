#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn mround_wrong_argument_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MROUND()");
    model._set("A2", "=MROUND(10)");
    model._set("A3", "=MROUND(10, 3, 5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn mround_basic_rounding() {
    let mut model = new_empty_model();
    // MROUND rounds to nearest multiple of significance
    model._set("A1", "=MROUND(10, 3)"); // 9 (closest multiple of 3)
    model._set("A2", "=MROUND(11, 3)"); // 12 (rounds up at midpoint)
    model._set("A3", "=MROUND(1.3, 0.2)"); // 1.4 (decimal significance)

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9");
    assert_eq!(model._get_text("A2"), *"12");
    assert_eq!(model._get_text("A3"), *"1.4");
}

#[test]
fn mround_sign_validation() {
    let mut model = new_empty_model();
    // Critical: number and significance must have same sign
    model._set("A1", "=MROUND(10, -3)"); // positive number, negative significance
    model._set("A2", "=MROUND(-10, 3)"); // negative number, positive significance
    model._set("A3", "=MROUND(-10, -3)"); // both negative - valid

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"-9");
}

#[test]
fn mround_special_cases() {
    let mut model = new_empty_model();
    // Zero significance always returns 0
    model._set("A1", "=MROUND(10, 0)");
    model._set("A2", "=MROUND(0, 5)"); // zero rounds to zero
    model._set("A3", "=MROUND(2.5, 5)"); // midpoint rounding (rounds up)

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"5");
}
