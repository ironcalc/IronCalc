#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A1:A5 = x,y,x,z,y   B1:B5 = 10,20,30,40,50   C1:C5 = 1,1,2,2,2
fn model() -> crate::model::Model<'static> {
    let mut model = new_empty_model();
    for (i, ((a, b), c)) in ["x", "y", "x", "z", "y"]
        .iter()
        .zip([10, 20, 30, 40, 50])
        .zip([1, 1, 2, 2, 2])
        .enumerate()
    {
        let row = i + 1;
        model._set(&format!("A{row}"), a);
        model._set(&format!("B{row}"), &b.to_string());
        model._set(&format!("C{row}"), &c.to_string());
    }
    model._set("E1", "x");
    model._set("F1", "z");
    model
}

#[test]
fn countifs_inline_array_criteria_spills() {
    let mut model = model();
    model._set("H1", r#"=COUNTIFS(A1:A5,{"x","y"})"#);
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2");
    assert_eq!(model._get_text("I1"), "2");
}

#[test]
fn sum_of_countifs_inline_array_criteria() {
    // Issue #1454
    let mut model = model();
    model._set("H1", r#"=SUM(COUNTIFS(A1:A5,{"x","y"}))"#);
    model._set("H2", r#"=SUM(COUNTIF(A1:A5,{"x","y","z"}))"#);
    model.evaluate();
    assert_eq!(model._get_text("H1"), "4");
    assert_eq!(model._get_text("H2"), "5");
}

#[test]
fn countifs_array_criteria_with_other_scalar_criteria() {
    let mut model = model();
    model._set("H1", r#"=SUM(COUNTIFS(A1:A5,{"x","y"},C1:C5,2))"#);
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2"); // x@3, y@5
}

#[test]
fn countifs_vertical_array_and_range_criteria() {
    let mut model = model();
    model._set("H1", r#"=COUNTIFS(A1:A5,{"x";"z"})"#);
    model._set("J1", "=COUNTIFS(A1:A5,E1:F1)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2");
    assert_eq!(model._get_text("H2"), "1");
    assert_eq!(model._get_text("J1"), "2");
    assert_eq!(model._get_text("K1"), "1");
}

#[test]
fn countifs_scalar_criteria_unchanged() {
    let mut model = model();
    model._set("H1", r#"=COUNTIFS(A1:A5,"x")"#);
    model._set("H2", "=COUNTIFS(A1:A5,E1,C1:C5,2)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2");
    assert_eq!(model._get_text("H2"), "1");
}

#[test]
fn sumifs_averageifs_minifs_maxifs_array_criteria() {
    let mut model = model();
    model._set("H1", r#"=SUMIFS(B1:B5,A1:A5,{"x","y"})"#);
    model._set("H2", r#"=AVERAGEIFS(B1:B5,A1:A5,{"x","y"})"#);
    model._set("H3", r#"=MINIFS(B1:B5,A1:A5,{"x","y"})"#);
    model._set("H4", r#"=MAXIFS(B1:B5,A1:A5,{"x","y"})"#);
    model._set("H5", r#"=SUM(SUMIFS(B1:B5,A1:A5,{"x","y"},C1:C5,{1,2}))"#);
    model.evaluate();
    assert_eq!(model._get_text("H1"), "40"); // x: 10+30
    assert_eq!(model._get_text("I1"), "70"); // y: 20+50
    assert_eq!(model._get_text("H2"), "20");
    assert_eq!(model._get_text("I2"), "35");
    assert_eq!(model._get_text("H3"), "10");
    assert_eq!(model._get_text("I3"), "20");
    assert_eq!(model._get_text("H4"), "30");
    assert_eq!(model._get_text("I4"), "50");
    assert_eq!(model._get_text("H5"), "60"); // pairs (x,1)=10 and (y,2)=50
}

#[test]
fn mismatched_criteria_arrays_are_value_error() {
    let mut model = model();
    model._set("H1", r#"=COUNTIFS(A1:A5,{"x","y"},C1:C5,{1,2,3})"#);
    model.evaluate();
    assert_eq!(model._get_text("H1"), "#VALUE!");
}
