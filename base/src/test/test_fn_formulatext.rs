#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT()");
    model._set("A2", "=FORMULATEXT(\"B\",\"A\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn multi_sheet_ref() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "=FORMULATEXT(Sheet1!A1:Sheet2!A1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn implicit_intersection() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(C1:C2)");
    model._set("A2", "=FORMULATEXT(D1:E1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn non_reference() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(42)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
