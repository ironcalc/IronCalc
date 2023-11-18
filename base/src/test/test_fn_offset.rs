#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_offset_reference() {
    let mut model = new_empty_model();
    model._set("B1", "12");
    model._set("B2", "13");
    model._set("B3", "15");

    model._set("A1", "=SUM(B1:OFFSET($B$1,3,0))");
    model._set("A2", "=SUM(OFFSET(A1, 1, 1):B3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"40");
    assert_eq!(model._get_text("A2"), *"28");
}
