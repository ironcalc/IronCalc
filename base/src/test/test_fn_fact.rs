#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn large_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=FACT(170)");
    model._set("A2", "=FACTDOUBLE(36)");

    model._set("B1", "=FACT(6)");
    model._set("B2", "=FACTDOUBLE(6)");

    model._set("C3", "=FACTDOUBLE(15)");

    model._set("F3", "=FACT(-0.1)");
    model._set("F4", "=FACT(0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"7.25742E+306");
    assert_eq!(model._get_text("A2"), *"1.67834E+21");
    assert_eq!(model._get_text("B1"), *"720");
    assert_eq!(model._get_text("B2"), *"48");
    assert_eq!(model._get_text("C3"), *"2027025");

    assert_eq!(model._get_text("F3"), *"#NUM!");
    assert_eq!(model._get_text("F4"), *"1");
}
