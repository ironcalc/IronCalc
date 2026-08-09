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

#[test]
fn test_fn_product_text_converted_to_number() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT("1")"#);
    model._set("A2", r#"=PRODUCT("1e2")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"100");
}

#[test]
fn test_fn_product_invalid_text() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT("a")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn test_fn_product_text_in_range_not_converted() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(B1:D1)"#);
    model._set("B1", r#"="100""#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
}

#[test]
fn test_fn_product_text_in_reference_not_converted() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(B1)"#);
    model._set("B1", r#"="100""#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
}

#[test]
fn test_fn_product_text_in_indirect_reference_not_converted() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(INDIRECT("B1"))"#);
    model._set("B1", r#"="100""#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
}

#[test]
fn test_fn_product_text_in_indirect_reference() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(INDIRECT("B1"))"#);
    model._set("B1", r#"100"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"100");
}

#[test]
fn test_fn_product_invalid_text_in_range() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(B1:D1)"#);
    model._set("B1", "a");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
}

#[test]
fn test_fn_product_invalid_text_in_reference() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(B1)"#);
    model._set("B1", r#"a"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
}

#[test]
fn test_fn_product_boolean_values_converted() {
    let mut model = new_empty_model();

    model._set("A1", r#"=PRODUCT(TRUE)"#);
    model._set("A2", r#"=PRODUCT(FALSE)"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"0");
}
