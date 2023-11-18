#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_choose_args_number() {
    let mut model = new_empty_model();
    model._set("A1", "=CHOOSE()");
    model._set("A2", "=CHOOSE(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_choose_incorrect_index() {
    let mut model = new_empty_model();
    model._set("A1", "=CHOOSE(-1, 42)");
    model._set("A2", "=CHOOSE(0, 42)");
    model._set("A3", "=CHOOSE(1, 42)");
    model._set("A4", "=CHOOSE(2, 42)");
    model._set("B1", "TEST");
    model._set("A5", "=CHOOSE(B1, 42)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"42");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#VALUE!");
}

#[test]
fn test_fn_choose_basic_tests() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("A1", "=CHOOSE(3.1, B1, B2, B3)");
    model._set("A2", "=SUM(B1:CHOOSE(1, B1, B2, B3))");
    model._set("A3", "=SUM(CHOOSE(3, B1:B1, B1:B2, B1:B3))");
    model._set("A4", "=CHOOSE(3,\"Wide\",115,\"world\",8)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"6");
    assert_eq!(model._get_text("A4"), *"world");
}
