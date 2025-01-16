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
fn return_of_array_is_n_impl() {
    let mut model = new_empty_model();
    // We populate cells A1 to A3
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("C2", "=A1:A3");
    model._set("D2", "=SUM(SIN(A:A)");

    model.evaluate();

    assert_eq!(model._get_text("C2"), "#N/IMPL!".to_string());
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
