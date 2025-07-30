#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

#[test]
fn test_fn_lcm_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_lcm_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM(12)");
    model._set("A2", "=LCM(25,40)");
    model._set("A3", "=LCM(4,6,8)");
    model._set("A4", "=LCM(4.7,6.3)"); // Decimal truncation
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"12");
    assert_eq!(model._get_text("A2"), *"200");
    assert_eq!(model._get_text("A3"), *"24");
    assert_eq!(model._get_text("A4"), *"12");
}

#[test]
fn test_fn_lcm_zeros_and_edge_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM(0)");
    model._set("A2", "=LCM(0,12)");
    model._set("A3", "=LCM(12,0)");
    model._set("A4", "=LCM(1,2,3,4,5)");
    model.evaluate();

    // LCM with any zero = 0
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"60");
}

#[test]
fn test_fn_lcm_error_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=LCM(-5)");
    model._set("A2", "=LCM(12,-8)");
    model._set("B1", "=1/0"); // Infinity
    model._set("A3", "=LCM(B1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
}

#[test]
fn test_fn_lcm_ranges() {
    let mut model = new_empty_model();
    // Range with numbers
    model._set("B1", "4");
    model._set("B2", "6");
    model._set("B3", "8");
    model._set("A1", "=LCM(B1:B3)");

    // Range with mixed data (text ignored)
    model._set("C1", "4");
    model._set("C2", "text");
    model._set("C3", "6");
    model._set("A2", "=LCM(C1:C3)");

    // Zero in range
    model._set("D1", "4");
    model._set("D2", "0");
    model._set("A3", "=LCM(D1:D2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"24");
    assert_eq!(model._get_text("A2"), *"12");
    assert_eq!(model._get_text("A3"), *"0");
}
