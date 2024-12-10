#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_geomean_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=GEOMEAN()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_geomean_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'2");
    // B5 is empty
    model._set("B6", "true");
    model._set("A1", "=GEOMEAN(B1:B6)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.817120593");
}
