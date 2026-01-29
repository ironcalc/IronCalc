#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMSQ()");
    model._set("A2", "=SUMSQ(2)");
    model._set("A3", "=SUMSQ(1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"4");
    assert_eq!(model._get_text("A3"), *"5");
}
