#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(4.3,2)");
    model._set("A2", "=CEILING(-4.3,-2)");
    model._set("A3", "=CEILING(-4.3,2)");
    model._set("B1", "=FLOOR(4.3,2)");
    model._set("B2", "=FLOOR(-4.3,-2)");
    model._set("B3", "=FLOOR(4.3,-2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"6");
    assert_eq!(model._get_text("A2"), *"-4");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("B1"), *"4");
    assert_eq!(model._get_text("B2"), *"-6");
    assert_eq!(model._get_text("B3"), *"#NUM!");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(1)");
    model._set("A2", "=CEILING(1,2,3)");
    model._set("B1", "=FLOOR(1)");
    model._set("B2", "=FLOOR(1,2,3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn zero_significance() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(4.3,0)");
    model._set("B1", "=FLOOR(4.3,0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("B1"), *"#DIV/0!");
}

#[test]
fn already_multiple() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(6,3)");
    model._set("B1", "=FLOOR(6,3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"6");
    assert_eq!(model._get_text("B1"), *"6");
}

#[test]
fn smaller_than_significance() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(1.3,2)");
    model._set("B1", "=FLOOR(1.3,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("B1"), *"0");
}

#[test]
fn fractional_significance() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(4.3,2.5)");
    model._set("B1", "=FLOOR(4.3,2.5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"5");
    assert_eq!(model._get_text("B1"), *"2.5");
}

#[test]
fn opposite_sign_error() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(4.3,-2)");
    model._set("B1", "=FLOOR(-4.3,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("B1"), *"#NUM!");
}

#[test]
fn zero_value() {
    let mut model = new_empty_model();
    model._set("A1", "=CEILING(0,2)");
    model._set("B1", "=FLOOR(0,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("B1"), *"0");
}

#[test]
fn coercion_cases() {
    let mut model = new_empty_model();
    model._set("B1", "'4.3"); // text that can be coerced
    model._set("B2", "TRUE"); // boolean
                              // B3 left blank

    model._set("C1", "=CEILING(B1,2)");
    model._set("C2", "=FLOOR(B2,1)");
    model._set("C3", "=CEILING(B3,2)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"6");
    assert_eq!(model._get_text("C2"), *"1");
    assert_eq!(model._get_text("C3"), *"0");
}

#[test]
fn error_propagation() {
    let mut model = new_empty_model();
    model._set("A1", "=1/0"); // #DIV/0! in value
    model._set("A2", "#REF!"); // #REF! error literal as significance

    model._set("B1", "=CEILING(A1,2)");
    model._set("B2", "=FLOOR(4.3,A2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#DIV/0!");
    assert_eq!(model._get_text("B2"), *"#REF!");
}
