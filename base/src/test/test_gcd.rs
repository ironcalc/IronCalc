#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

#[test]
fn test_fn_gcd_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=GCD()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_gcd_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=GCD(12)");
    model._set("A2", "=GCD(60,36)");
    model._set("A3", "=GCD(15,25,35)");
    model._set("A4", "=GCD(12.7,8.3)"); // Decimal truncation
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"12");
    assert_eq!(model._get_text("A2"), *"12");
    assert_eq!(model._get_text("A3"), *"5");
    assert_eq!(model._get_text("A4"), *"4");
}

#[test]
fn test_fn_gcd_zeros_and_edge_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=GCD(0)");
    model._set("A2", "=GCD(0,12)");
    model._set("A3", "=GCD(12,0)");
    model._set("A4", "=GCD(1,2,3,4,5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"12");
    assert_eq!(model._get_text("A3"), *"12");
    assert_eq!(model._get_text("A4"), *"1");
}

#[test]
fn test_fn_gcd_error_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=GCD(-5)");
    model._set("A2", "=GCD(12,-8)");
    model._set("B1", "=1/0"); // Infinity
    model._set("A3", "=GCD(B1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
}

#[test]
fn test_fn_gcd_ranges() {
    let mut model = new_empty_model();
    // Range with numbers
    model._set("B1", "12");
    model._set("B2", "18");
    model._set("B3", "24");
    model._set("A1", "=GCD(B1:B3)");

    // Range with mixed data (text ignored)
    model._set("C1", "12");
    model._set("C2", "text");
    model._set("C3", "6");
    model._set("A2", "=GCD(C1:C3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"6");
    assert_eq!(model._get_text("A2"), *"6");
}
