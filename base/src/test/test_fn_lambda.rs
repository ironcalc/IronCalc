#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_evaluation() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)(1, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
}

#[test]
fn evaluation_in_let() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x, 1, y, LAMBDA(a, b, a + b), y(x, 22))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"23");
}

#[test]
fn evaluation_with_defined_name() {
    let mut model = new_empty_model();
    model
        .new_defined_name("MySum", None, "=LAMBDA(x, y, x + y)")
        .unwrap();
    model._set("A1", "=MySum(1, 2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn returns_calculation_error() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x, y, x + y)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#CALC!");
}
