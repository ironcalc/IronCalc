#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_sqrt_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SQRT(4)");
    model._set("A2", "=SQRT()");
    model._set("A3", "=SQRT(4, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_sqrtpi_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SQRTPI()");
    model._set("A2", "=SQRTPI(4, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_sign_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SIGN(-42)");
    model._set("A2", "=SIGN(13)");
    model._set("A3", "=SIGN(0)");
    model._set("A4", "=SIGN()");
    model._set("A5", "=SIGN('')");
    model._set("A6", "=SIGN('Hello')");
    model._set("A7", "=SIGN(1, 2)");
    model._set("A8", "=SIGN(B8)");
    model._set("A9", "=SIGN(4-4)");
    model._set("A10", "=SIGN(-0.000001)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"-1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"0");
    assert_eq!(model._get_text("A9"), *"0");
    assert_eq!(model._get_text("A10"), *"-1");
}
