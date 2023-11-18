#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_product_arguments() {
    let mut model = new_empty_model();

    // Incorrect number of arguments
    model._set("A1", "=PRODUCT()");

    model.evaluate();
    // Error (Incorrect number of arguments)
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
