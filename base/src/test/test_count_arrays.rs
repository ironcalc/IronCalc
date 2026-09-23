#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn count_counts_numbers_in_an_array() {
    let mut model = new_empty_model();
    model._set("A1", "=COUNT({1,2,\"a\"})");
    model._set("A2", "=COUNT({1,2},{3,\"a\"})");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "3");
}
