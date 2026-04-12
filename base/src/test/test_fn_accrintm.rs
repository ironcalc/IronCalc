#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_accrintm_placeholder() {
    let mut model = new_empty_model();
    model._set("A1", "=ACCRINTM(39539, 39614, 0.1, 1000, 3)");
    model.evaluate();
    // Expected: 20.547945205 (basis 3, Actual/365, 75 days)
    // This test will panic with todo!() until P3 implementation
    assert_eq!(model._get_text("A1"), *"20.547945205");
}
