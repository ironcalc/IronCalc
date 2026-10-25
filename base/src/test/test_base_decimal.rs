#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=BASE()");
    model._set("A2", "=BASE(10)");
    model._set("A3", "=BASE(10, 2)");
    model._set("A4", "=BASE(10, 2, 8)");
    model._set("A5", "=BASE(10, 2, 8, 1)");

    model._set("B1", "=DECIMAL()");
    model._set("B2", "=DECIMAL(1010)");
    model._set("B3", "=DECIMAL(1010, 2)");
    model._set("B4", "=DECIMAL(1010, 2, 8)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"1010");
    assert_eq!(model._get_text("A4"), *"00001010");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"10");
    assert_eq!(model._get_text("B4"), *"#ERROR!");
}