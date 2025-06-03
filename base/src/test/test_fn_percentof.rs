#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_percentof_arguments() {
    let mut model = new_empty_model();
    // Incorrect number of arguments
    model._set("A1", "=PERCENTOF()");
    model._set("A2", "=PERCENTOF(10)");

    // Correct use of function
    model._set("A3", "=PERCENTOF(10,100)");
    model._set("A4", "=PERCENTOF(500,1000");

    model.evaluate();
    // Error (Incorrect number of arguments)
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");

    // Success
    assert_eq!(model._get_text("A3"), *"0.1");
    assert_eq!(model._get_text("A4"), *"0.5")
}
