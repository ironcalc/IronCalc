#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn set(model: &mut crate::Model, cell: &str, value: &str) -> (u32, i32, i32) {
    model._set(cell, value);
    let r = model._parse_reference(cell);
    (r.sheet, r.row, r.column)
}

#[test]
fn only_dependents_are_recalculated() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "=A1*2");
    model._set("A3", "=A2+1");
    model._set("B1", "=SUM(A1:A3)");
    model._set("C1", "=10");
    model.evaluate();
    let edited = set(&mut model, "A1", "5");
    let recalculated = model.evaluate_incremental(&[edited]).unwrap();
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "11");
    assert_eq!(model._get_text("B1"), "26");
    assert_eq!(recalculated.len(), 3, "A2, A3 and B1; not C1");
}

#[test]
fn edited_formulas_and_volatile_ones() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "=A1");
    model._set("D1", "=INDIRECT(\"B1\")*10");
    model.evaluate();
    // a formula changes what it refers to
    let e1 = set(&mut model, "C1", "=B1+100");
    model.evaluate_incremental(&[e1]).unwrap();
    assert_eq!(model._get_text("C1"), "102");
    let e2 = set(&mut model, "B1", "3");
    model.evaluate_incremental(&[e2]).unwrap();
    assert_eq!(model._get_text("C1"), "103");
    assert_eq!(
        model._get_text("D1"),
        "30",
        "INDIRECT is always recalculated"
    );
    // A1 no longer feeds C1
    let e3 = set(&mut model, "A1", "50");
    model.evaluate_incremental(&[e3]).unwrap();
    assert_eq!(model._get_text("C1"), "103");
}

#[test]
fn circular_reference_made_by_an_edit() {
    let mut model = new_empty_model();
    model._set("A1", "=B1+1");
    model._set("B1", "1");
    model.evaluate();
    let e = set(&mut model, "B1", "=A1");
    // the order a cycle is visited in matters, so it gets a full evaluation
    assert!(model.evaluate_incremental(&[e]).is_none());
    assert_eq!(model._get_text("A1"), "#CIRC!");
    let e = set(&mut model, "B1", "4");
    model.evaluate_incremental(&[e]).unwrap();
    assert_eq!(model._get_text("A1"), "5");
}

#[test]
fn spills_fall_back_to_a_full_evaluation() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("B1", "=SEQUENCE(A1)");
    model._set("C1", "=SUM(B1:B5)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "6");
    let e = set(&mut model, "A1", "4");
    assert!(model.evaluate_incremental(&[e]).is_none());
    assert_eq!(model._get_text("C1"), "10");
    // typing into the spill blocks it
    let e = set(&mut model, "B3", "x");
    assert!(model.evaluate_incremental(&[e]).is_none());
    assert_eq!(model._get_text("B1"), "#SPILL!");
    let e = set(&mut model, "D1", "1");
    assert!(
        model.evaluate_incremental(&[e]).is_none(),
        "a blocked spill may unblock"
    );
}
