#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

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
