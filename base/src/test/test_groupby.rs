#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn groupby_sums_by_field_with_a_total() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("A3", "x");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", "=GROUPBY(A1:A3,B1:B3,SUM)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "x");
    assert_eq!(model._get_text("E1"), "4");
    assert_eq!(model._get_text("D2"), "y");
    assert_eq!(model._get_text("E2"), "2");
    assert_eq!(model._get_text("D3"), "Total");
    assert_eq!(model._get_text("E3"), "6");
}

#[test]
fn pivotby_crosses_two_fields() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=PIVOTBY({\"a\";\"a\";\"b\"},{\"p\";\"q\";\"p\"},{1;2;3},SUM)",
    );
    model.evaluate();
    assert_eq!(model._get_text("B1"), "p");
    assert_eq!(model._get_text("D1"), "Total");
    assert_eq!(model._get_text("A2"), "a");
    assert_eq!(model._get_text("C3"), "");
    assert_eq!(model._get_text("D4"), "6");
}

#[test]
fn a_function_name_alone_is_a_lambda() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "=BYROW(A1:B2,SUM)");
    model._set("D3", "=MAP({1,4},SQRT)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "3");
    assert_eq!(model._get_text("D2"), "7");
    assert_eq!(model._get_text("D3"), "1");
    assert_eq!(model._get_text("E3"), "2");
}
