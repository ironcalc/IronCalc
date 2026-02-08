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

#[test]
fn test_language_independence() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(1, 2)");
    model._set("B1", "=FORMULATEXT(A1)");

    model.evaluate();
    model.set_language("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=SOMME(1,2)");
    assert_eq!(model._get_text("B1"), *"=SUM(1,2)");
}

#[test]
fn test_locale() {
    let mut model = new_empty_model();

    model._set("A1", "=SUM(1.123, 2)");
    model._set("B1", "=FORMULATEXT(A1)");
    model.evaluate();
    model.set_language("fr").unwrap();
    model.set_locale("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=SOMME(1,123;2)");
    assert_eq!(model._get_text("B1"), *"=SUM(1,123;2)");
}

#[test]
fn test_prefixes() {
    let mut model = new_empty_model();

    // model._set("A1", "+A1+A2");
    // assert_eq!(model._get_formula("A1"), *"=+A1+A2");

    model._set("A2", "-A1-A2");
    assert_eq!(model._get_formula("A2"), *"=-A1-A2");

    // model._set("A3", "+4+2");
    // assert_eq!(model._get_formula("A3"), *"=4+2");

    model._set("A4", "-4-2");
    assert_eq!(model._get_formula("A4"), *"=-4-2");

    model._set("A5", "-42");
    assert_eq!(model._get_formula("A5"), *"");

    model._set("A6", "+42");
    assert_eq!(model._get_formula("A6"), *"");

    model._set("A7", "-A");
    assert_eq!(model._get_formula("A7"), *"=-A");

    model._set("A8", "+A");
    assert_eq!(model._get_formula("A8"), *"=+A");

    model._set("A9", "-A1");
    assert_eq!(model._get_formula("A9"), *"=-A1");

    // model._set("A10", "+A1");
    // assert_eq!(model._get_formula("A10"), *"=+A1");

    model._set("A11", "+=2");
    assert_eq!(model._get_formula("A11"), *"");

    model.evaluate();

    assert_eq!(model._get_text("A3"), *"6");
    assert_eq!(model._get_text("A4"), *"-6");
    assert_eq!(model._get_text("A5"), *"-42");
    assert_eq!(model._get_text("A6"), *"42");
    assert_eq!(model._get_text("A7"), *"#NAME?");
    assert_eq!(model._get_text("A8"), *"#NAME?");
    assert_eq!(model._get_text("A11"), *"+=2");
}
