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
    model._set("A1", "=MROUND(10, 3)"); // 9
    model._set("A2", "=MROUND(11, 3)"); // 12
    model._set("A3", "=MROUND(1.3, 0.2)"); // 1.4

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9");
    assert_eq!(model._get_text("A2"), *"12");
    assert_eq!(model._get_text("A3"), *"1.4");
}

#[test]
fn mround_sign_validation() {
    let mut model = new_empty_model();
    // Number and significance must have same sign
    model._set("A1", "=MROUND(10, -3)"); // NUM error
    model._set("A2", "=MROUND(-10, 3)"); // NUM error
    model._set("A3", "=MROUND(-10, -3)"); // -9

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"-9");
}

#[test]
fn mround_special_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=MROUND(10, 0)"); // 0
    model._set("A2", "=MROUND(0, 5)"); // 0
    model._set("A3", "=MROUND(2.5, 5)"); // 5
                                         // Zero number with any significance sign
    model._set("A4", "=MROUND(0, -1)"); // 0
    model._set("A5", "=MROUND(0, -5)"); // 0

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"5");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
}

#[test]
fn mround_precision_edge_cases() {
    let mut model = new_empty_model();
    // Floating-point precision at midpoints
    model._set("A1", "=MROUND(1.5, 1)"); // 2
    model._set("A2", "=MROUND(-1.5, -1)"); // -2
    model._set("A3", "=MROUND(2.5, 1)"); // 3
    model._set("A4", "=MROUND(-2.5, -1)"); // -3
    model._set("A5", "=MROUND(0.15, 0.1)"); // 0.2
    model._set("A6", "=MROUND(-0.15, -0.1)"); // -0.2

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"-2");
    assert_eq!(model._get_text("A3"), *"3");
    assert_eq!(model._get_text("A4"), *"-3");
    assert_eq!(model._get_text("A5"), *"0.2");
    assert_eq!(model._get_text("A6"), *"-0.2");
}
