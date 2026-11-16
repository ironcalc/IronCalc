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
fn range_gives_an_array() {
    let mut model = new_empty_model();
    model._set("C1", "=1+1");
    model._set("C2", "5");
    model._set("D1", "=SUM(C1:C2)");
    model._set("A1", "=FORMULATEXT(C1:C2)");
    model._set("A4", "=FORMULATEXT(C1:D2)");
    model._set("A7", "=COUNTA(FORMULATEXT(C1:D2))");
    model._set("A8", "=SUM(--ISNA(FORMULATEXT(C1:D2)))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"=1+1");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A4"), *"=1+1");
    assert_eq!(model._get_text("B4"), *"=SUM(C1:C2)");
    assert_eq!(model._get_text("A5"), *"#N/A");
    assert_eq!(model._get_text("B5"), *"#N/A");
    assert_eq!(model._get_text("A7"), *"4");
    assert_eq!(model._get_text("A8"), *"2");
}

#[test]
fn range_spills_into_blocked_cell() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(C1:C2)");
    model._set("A2", "x");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#SPILL!");
}

#[test]
fn implicit_intersection_operator() {
    // @A:A in B1 intersects to A1, so FORMULATEXT acts on that single cell.
    let mut model = new_empty_model();
    model._set("A1", "=1 +  2");
    model._set("B1", "=FORMULATEXT(@A:A)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"=1+2");
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
fn test_sign_prefixed() {
    let mut model = new_empty_model();

    model._set("A2", "-B1-B2");
    assert_eq!(model._get_formula("A2"), *"=-B1-B2");

    model._set("A4", "-4-2");
    assert_eq!(model._get_formula("A4"), *"=-4-2");

    model._set("A5", "-42");
    assert_eq!(model._get_formula("A5"), *"");

    model._set("A6", "+42");
    assert_eq!(model._get_formula("A6"), *"");

    model._set("A7", "-B");
    assert_eq!(model._get_formula("A7"), *"=-B");

    model._set("A9", "-B1");
    assert_eq!(model._get_formula("A9"), *"=-B1");

    model.evaluate();

    assert_eq!(model._get_text("A4"), *"-6");
    assert_eq!(model._get_text("A5"), *"-42");
    assert_eq!(model._get_text("A6"), *"42");
    assert_eq!(model._get_text("A7"), *"#NAME?");
}
