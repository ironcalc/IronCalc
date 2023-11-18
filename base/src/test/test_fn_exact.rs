#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_exact_args_number() {
    let mut model = new_empty_model();

    model._set("A1", "=EXACT(1)");
    model._set("A2", "=EXACT(1, 1, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn fn_exact() {
    let mut model = new_empty_model();

    model._set("A1", "=EXACT(2.3, 2.3)");
    model._set("A2", r#"=EXACT(2.3, "2.3")"#);
    model._set("A3", r#"=EXACT("Hello", "hello")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"FALSE");
}
