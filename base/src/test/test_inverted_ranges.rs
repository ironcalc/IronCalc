#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn sum_inverted_range_vertical() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("B1", "=SUM(A1:A3)");
    model._set("B2", "=SUM(A3:A1)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), model._get_text("B2"));
}

#[test]
fn sum_inverted_range_horizontal() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");

    model._set("D1", "=SUM(A1:C1)");
    model._set("D2", "=SUM(C1:A1)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), model._get_text("D2"));
}

#[test]
fn sum_inverted_range_rectangular() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");

    model._set("D1", "=SUM(A1:C2)"); // Default: top left to bottom right
    model._set("D2", "=SUM(C2:A1)"); // Bottom right to top left
    model._set("D3", "=SUM(C1:A2)"); // Top right to bottom left
    model._set("D4", "=SUM(A2:C1)"); // Bottom left to top right
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"21");
    assert_eq!(model._get_text("D1"), model._get_text("D2"));
    assert_eq!(model._get_text("D1"), model._get_text("D3"));
    assert_eq!(model._get_text("D1"), model._get_text("D4"));
}

#[test]
fn sum_inverted_range_with_absolute_references() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("B1", "=SUM(A$1:A3)");
    model._set("B2", "=SUM(A3:A$1)");
    model._set("B3", "=SUM(A$1:A$3)");
    model._set("B4", "=SUM(A$3:A$1)");
    model._set("B5", "=SUM($A1:$A3)");
    model._set("B6", "=SUM($A3:$A1)");
    model._set("B7", "=SUM($A$1:$A$3)");
    model._set("B8", "=SUM($A$3:$A$1)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"6");
    assert_eq!(model._get_text("B1"), model._get_text("B2"));
    assert_eq!(model._get_text("B1"), model._get_text("B3"));
    assert_eq!(model._get_text("B1"), model._get_text("B4"));
    assert_eq!(model._get_text("B1"), model._get_text("B5"));
    assert_eq!(model._get_text("B1"), model._get_text("B6"));
    assert_eq!(model._get_text("B1"), model._get_text("B7"));
    assert_eq!(model._get_text("B1"), model._get_text("B8"));
}

#[test]
fn inverted_range_with_blanks() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "");

    model._set("B1", "=SUM(A1:A4)");
    model._set("B2", "=SUM(A4:A1)");

    model._set("C1", "=COUNTA(A1:A4)"); // Counts non-blank cells
    model._set("C2", "=COUNTA(A4:A1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), model._get_text("B2"));
    assert_eq!(model._get_text("C1"), model._get_text("C2"));
}

#[test]
fn sum_inverted_range_with_errors() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("A4", "=1/0");

    model._set("B1", "=SUM(A1:A4)");
    model._set("B2", "=SUM(A4:A1)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), model._get_text("B2"));
}

#[test]
fn other_functions() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("B1", "=AVERAGE(A1:A3)");
    model._set("B2", "=AVERAGE(A3:A1)");

    model._set("C1", "=COUNT(A1:A3)");
    model._set("C2", "=COUNT(A3:A1)");

    model._set("D1", "=INDEX(A1:A3, 2)");
    model._set("D2", "=INDEX(A3:A1, 2)");

    model._set("E1", "=MATCH(2, A1:A3)");
    model._set("E2", "=MATCH(2, A3:A1)");

    model._set("F1", "=SUMIF(A1:A3, \">1\")");
    model._set("F2", "=SUMIF(A3:A1, \">1\")");

    model._set("G1", "=CONCATENATE(A1:A3)");
    model._set("G2", "=CONCATENATE(A3:A1)");

    model._set("H1", "=ROWS(A1:A3)");
    model._set("H2", "=ROWS(A3:A1)");

    model._set("I1", "=COLUMNS(A1:A3)");
    model._set("I2", "=COLUMNS(A3:A1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), model._get_text("B2"));
    assert_eq!(model._get_text("C1"), model._get_text("C2"));
    assert_eq!(model._get_text("D1"), model._get_text("D2"));
    assert_eq!(model._get_text("E1"), model._get_text("E2"));
    assert_eq!(model._get_text("F1"), model._get_text("F2"));
    assert_eq!(model._get_text("G1"), model._get_text("G2"));
    assert_eq!(model._get_text("H1"), model._get_text("H2"));
    assert_eq!(model._get_text("I1"), model._get_text("I2"));
}

#[test]
fn mixed_data_types() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "abc");
    model._set("A3", "TRUE");
    model._set("A4", "2");

    model._set("B1", "=SUM(A1:A4)");
    model._set("B2", "=SUM(A4:A1)");

    model._set("C1", "=COUNT(A1:A4)");
    model._set("C2", "=COUNT(A4:A1)");

    model._set("D1", "=COUNTIF(A1:A4, \"TRUE\")");
    model._set("D2", "=COUNTIF(A4:A1, \"TRUE\")");

    model.evaluate();

    assert_eq!(model._get_text("B1"), model._get_text("B2"));
    assert_eq!(model._get_text("C1"), model._get_text("C2"));
    assert_eq!(model._get_text("D1"), model._get_text("D2"));
}
