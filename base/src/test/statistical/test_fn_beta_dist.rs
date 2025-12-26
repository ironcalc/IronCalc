#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn empty_argument_cumulative() {
    let mut model = new_empty_model();

    model._set("A1", "=BETA.DIST(0.234, 2, 2.5, , 0.15, 1.2)");
    model._set("A2", "=BETA.DIST(0.234, 2, 2.5, C15 , 0.15, 1.2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.588288667");
    assert_eq!(model._get_text("A2"), *"0.588288667");
}
