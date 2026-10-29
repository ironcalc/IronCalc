#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn empty_text_returns_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=DECIMAL(\"\", 2)");
    model._set("A2", "=DECIMAL(A3, 2)");
    model._set("A4", "=DECIMAL(\"\", 99)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn text_longer_than_255_characters_is_value_error() {
    let mut model = new_empty_model();
    // 255 characters are still converted
    model._set("A1", "=DECIMAL(REPT(\"0\", 254)&\"1\", 2)");
    model._set("A2", "=DECIMAL(REPT(\"0\", 255)&\"1\", 2)");
    model._set("A3", "=DECIMAL(REPT(\"0\", 255)&\"1\", 36)");
    model._set("A4", "=DECIMAL(REPT(\"0\", 256)&\"1\", 99)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn base_min_length_max_255() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN(BASE(5, 2, 255))");
    model._set("A2", "=BASE(5, 2, 255.9)");
    model._set("A3", "=BASE(5, 2, 256)");
    model._set("A4", "=BASE(5, 2, 1E6)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"255");
    assert_eq!(model._get_text("A2"), "0".repeat(252) + "101");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

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
