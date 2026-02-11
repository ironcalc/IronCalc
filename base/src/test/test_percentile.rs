#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_percentile() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model._set("A1", "=PERCENTILE.INC(B1:B5,0.4)");
    model._set("A2", "=PERCENTILE.EXC(B1:B5,0.4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2.6");
    assert_eq!(model._get_text("A2"), *"2.4");
}

#[test]
fn test_fn_percentrank() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model._set("A1", "=PERCENTRANK.INC(B1:B5,3.5)");
    model._set("A2", "=PERCENTRANK.EXC(B1:B5,3.5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.625");
    assert_eq!(model._get_text("A2"), *"0.583");
}

#[test]
fn test_fn_percentrank_inc_single_element() {
    let mut model = new_empty_model();
    // Test single element array - should not cause division by zero
    model._set("B1", "5.0");
    model._set("A1", "=PERCENTRANK.INC(B1:B1,5.0)");
    model._set("A2", "=PERCENTRANK.INC(B1:B1,3.0)");
    model.evaluate();

    // For single element array with exact match, should return 0.5
    assert_eq!(model._get_text("A1"), *"0.5");
    // For single element array with no match, should return #N/A error
    assert!(model._get_text("A2").contains("#N/A"));
}

#[test]
fn test_fn_percentrank_inc_boundary_values() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    // Test values outside the range
    model._set("A1", "=PERCENTRANK.INC(B1:B5,0.5)"); // Below minimum
    model._set("A2", "=PERCENTRANK.INC(B1:B5,6.0)"); // Above maximum

    // Test exact matches at boundaries
    model._set("A3", "=PERCENTRANK.INC(B1:B5,1.0)"); // Exact minimum
    model._set("A4", "=PERCENTRANK.INC(B1:B5,5.0)"); // Exact maximum

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0"); // Below min should return 0
    assert_eq!(model._get_text("A2"), *"1"); // Above max should return 1
    assert_eq!(model._get_text("A3"), *"0"); // Exact min should return 0
    assert_eq!(model._get_text("A4"), *"1"); // Exact max should return 1
}
