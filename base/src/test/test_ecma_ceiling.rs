#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn ecma_ceiling_matches_ceiling() {
    let mut model = new_empty_model();
    model._set("A1", "=ECMA.CEILING(4.3,2)");
    model._set("A2", "=ECMA.CEILING(-4.3,-2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("A2"), "-6");
}
