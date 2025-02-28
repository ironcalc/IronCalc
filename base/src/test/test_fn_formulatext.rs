#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

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

    assert_eq!(model._get_text("A1"), *"#N/IMPL!");
    assert_eq!(model._get_text("A2"), *"#N/IMPL!");
}

#[test]
fn implicit_intersection_operator() {
    let mut model = new_empty_model();
    model._set("A1", "=1 +  2");
    model._set("B1", "=FORMULATEXT(@A:A)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#N/IMPL!");
}

#[test]
fn non_reference() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(42)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
