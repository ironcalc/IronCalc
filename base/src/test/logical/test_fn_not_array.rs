#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// NOT on a single value keeps its scalar behaviour.

#[test]
fn not_scalar() {
    let mut model = new_empty_model();
    model._set("A1", "=NOT(TRUE)");
    model._set("A2", "=NOT(0)");
    model._set("A3", "=NOT(B3)");
    model._set("A4", r#"=NOT("false")"#);
    model._set("A5", r#"=NOT("x")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "TRUE");
    assert_eq!(model._get_text("A4"), "TRUE");
    assert_eq!(model._get_text("A5"), "#VALUE!");
}

// NOT on a list, a range or a computed array flips every value and spills one
// TRUE/FALSE per input value (#1528).

#[test]
fn not_literal_list_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=NOT({TRUE;FALSE})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("A2"), "TRUE");
}

#[test]
fn not_horizontal_list_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=NOT({1,0,2})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("C1"), "FALSE");
}

#[test]
fn not_comparison_on_range_spills() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "=NOT(A1:A2>1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("B2"), "FALSE");
}

#[test]
fn not_range_spills() {
    let mut model = new_empty_model();
    model._set("A1", "TRUE");
    model._set("A2", "0");
    model._set("A4", "=1/0");
    model._set("B1", "=NOT(A1:A4)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "FALSE");
    assert_eq!(model._get_text("B2"), "TRUE");
    // An empty cell reads as FALSE, as it does for a scalar argument.
    assert_eq!(model._get_text("B3"), "TRUE");
    // An error in the range stays in its own slot.
    assert_eq!(model._get_text("B4"), "#DIV/0!");
}

#[test]
fn not_text_in_list() {
    let mut model = new_empty_model();
    model._set("A1", r#"=NOT({"TRUE","false","x"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("C1"), "#VALUE!");
}

#[test]
fn not_wrong_number_of_args() {
    let mut model = new_empty_model();
    model._set("A1", "=NOT(TRUE, FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}
