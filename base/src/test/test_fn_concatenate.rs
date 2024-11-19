#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_concatenate_args_number() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCATENATE()");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn fn_concatenate() {
    let mut model = new_empty_model();
    model._set("A1", "Hello");
    model._set("A2", " my ");
    model._set("A3", "World");

    model._set("B1", r#"=CONCATENATE(A1, A2, A3, "!")"#);
    // This will break once we implement the implicit intersection operator
    // It should be:
    model._set("C2", r#"=CONCATENATE(@A1:A3, "!")"#);
    model._set("B2", r#"=CONCATENATE(A1:A3, "!")"#);
    model._set("B3", r#"=CONCAT(A1:A3, "!")"#);

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Hello my World!");
    assert_eq!(model._get_text("B2"), *"#N/IMPL!");
    assert_eq!(model._get_text("B3"), *"Hello my World!");
    assert_eq!(model._get_text("C2"), *" my !");
}
