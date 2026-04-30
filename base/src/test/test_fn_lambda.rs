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

#[test]
fn sheet_local_name_takes_precedence_over_global() {
    // Global MyFn = LAMBDA(x, x * 2)
    // Sheet2-local MyFn = LAMBDA(x, x * 3)
    // Sheet1!A1 calls MyFn(5) => uses global => 10
    // Sheet2!A1 calls MyFn(5) => uses local  => 15
    let mut model = new_empty_model();
    model.new_sheet(); // adds Sheet2 at index 1

    model
        .new_defined_name("MyFn", None, "=LAMBDA(x, x * 2)")
        .unwrap();
    model
        .new_defined_name("MyFn", Some(1), "=LAMBDA(x, x * 3)")
        .unwrap();

    model._set("A1", "=MyFn(5)");
    model._set("Sheet2!A1", "=MyFn(5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"10");
    assert_eq!(model._get_text("Sheet2!A1"), *"15");
}

#[test]
fn let_bound_variable_captured_in_lambda_body() {
    let mut model = new_empty_model();
    // `a` is bound by LET; the LAMBDA body references `a` as a closure variable.
    // f(2) should return 2 + 1 = 3.
    model._set("A1", "=LET(a, 1, f, LAMBDA(x, x + a), f(2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"3");
}
