#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_sum_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM()");
    model._set("A2", "=SUM(1, 2, 3)");
    model._set("A3", "=SUM(1, )");
    model._set("A4", "=SUM(1,   , 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"4");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM({1, 2, 3})");
    model._set("A2", "=SUM({1; 2; 3})");
    model._set("A3", "=SUM({1, 2; 3, 4})");
    model._set("A4", "=SUM({1, 2; 3, 4; 5, 6})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"6");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"10");
    assert_eq!(model._get_text("A4"), *"21");
}
