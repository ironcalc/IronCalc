#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_colum() {
    let mut model = new_empty_model();
    // We populate cells A1 to A3
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("C2", "=@A1:A3");

    model.evaluate();

    assert_eq!(model._get_text("C2"), "2".to_string());
}

#[test]
fn return_of_array_spills() {
    let mut model = new_empty_model();
    // We populate cells A1 to A3
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    // With dynamic arrays, =A1:A3 spills downward from C2
    model._set("C2", "=A1:A3");
    model._set("D2", "=SUM(SIN(A:A)");

    model.evaluate();

    assert_eq!(model._get_text("C2"), "1".to_string());
    assert_eq!(model._get_text("C3"), "2".to_string());
    assert_eq!(model._get_text("C4"), "3".to_string());
    assert_eq!(model._get_text("D2"), "1.89188842".to_string());
}

#[test]
fn concat() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT(@B1:B3)");
    model._set("A2", "=CONCAT(B1:B3)");
    model._set("B1", "Hello");
    model._set("B2", " ");
    model._set("B3", "world!");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"Hello");
    assert_eq!(model._get_text("A2"), *"Hello world!");
}

#[test]
fn offset_returning_1x1_array_in_scalar_context() {
    // OFFSET(SingleCellRef, r, c) returns a 1x1 array internally.
    // When such an expression appears inside another scalar formula
    // (e.g. multiplied by a number, or wrapped in IF), it should 
    // transparently unwrap the 1x1 array to its single value.
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");

    model._set("A1", "=2 * IF(TRUE, OFFSET(B1, 2, 0), 0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "60".to_string());
}
