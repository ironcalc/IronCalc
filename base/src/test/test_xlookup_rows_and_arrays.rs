#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn xlookup_returns_whole_rows_and_takes_lists() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("C1", "10");
    model._set("C2", "20");
    model._set("C3", "30");
    model._set("E1", "=XLOOKUP(2,A1:A3,B1:C3)");
    model._set("E2", "=XLOOKUP(\"b\",{\"a\",\"b\"},{1,2})");
    model._set("E3", "=SUM(XLOOKUP(3,A1:A3,A1:C3))");
    model._set("E4", "=XLOOKUP({1;3},A1:A3,C1:C3)");
    model._set("E6", "=XLOOKUP(9,A1:A3,B1:B3,\"none\")");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "b");
    assert_eq!(model._get_text("F1"), "20");
    assert_eq!(model._get_text("E2"), "2");
    // the text "c" in the matching row is left out of the sum
    assert_eq!(model._get_text("E3"), "33");
    assert_eq!(model._get_text("E4"), "10");
    assert_eq!(model._get_text("E5"), "30");
    assert_eq!(model._get_text("E6"), "none");
}
