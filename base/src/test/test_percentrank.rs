#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ============================================================================
// PERCENTRANK.INC BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_fn_percentrank_inc_basic() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model._set("A1", "=PERCENTRANK.INC(B1:B5,3.5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.625");
}

#[test]
fn test_fn_percentrank_inc_boundary_values() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B5,0.5)"); // Below minimum
    model._set("A2", "=PERCENTRANK.INC(B1:B5,6.0)"); // Above maximum
    model._set("A3", "=PERCENTRANK.INC(B1:B5,1.0)"); // Exact minimum
    model._set("A4", "=PERCENTRANK.INC(B1:B5,5.0)"); // Exact maximum
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0"); // Below min should return 0
    assert_eq!(model._get_text("A2"), *"1"); // Above max should return 1
    assert_eq!(model._get_text("A3"), *"0"); // Exact min should return 0
    assert_eq!(model._get_text("A4"), *"1"); // Exact max should return 1
}

#[test]
fn test_fn_percentrank_inc_single_element() {
    let mut model = new_empty_model();
    model._set("B1", "5.0");
    model._set("A1", "=PERCENTRANK.INC(B1:B1,5.0)");
    model._set("A2", "=PERCENTRANK.INC(B1:B1,3.0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.5");
    assert!(model._get_text("A2").contains("#N/A"));
}

#[test]
fn test_fn_percentrank_inc_empty_array() {
    let mut model = new_empty_model();
    model._set("A1", "=PERCENTRANK.INC(B1:B1,5)");
    model.evaluate();

    assert!(model._get_text("A1").contains("#NUM!"));
}

#[test]
fn test_fn_percentrank_inc_with_duplicates() {
    let mut model = new_empty_model();
    // Array with duplicates: [1, 2, 2, 3, 3, 3]
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "2");
    model._set("B4", "3");
    model._set("B5", "3");
    model._set("B6", "3");

    model._set("A1", "=PERCENTRANK.INC(B1:B6,2)");
    model._set("A2", "=PERCENTRANK.INC(B1:B6,2.5)"); // Interpolation
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.2");
    assert_eq!(model._get_text("A2"), *"0.5");
}

#[test]
fn test_fn_percentrank_inc_with_negative_values() {
    let mut model = new_empty_model();
    // Array with negative values: [-5, -2, 0, 2, 5]
    model._set("B1", "-5");
    model._set("B2", "-2");
    model._set("B3", "0");
    model._set("B4", "2");
    model._set("B5", "5");

    model._set("A1", "=PERCENTRANK.INC(B1:B5,-2)");
    model._set("A2", "=PERCENTRANK.INC(B1:B5,0)");
    model._set("A3", "=PERCENTRANK.INC(B1:B5,-3.5)"); // Interpolation
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.25");
    assert_eq!(model._get_text("A2"), *"0.5");
    assert_eq!(model._get_text("A3"), *"0.125");
}

#[test]
fn test_fn_percentrank_inc_exact_vs_interpolated() {
    let mut model = new_empty_model();
    // Array [10, 20, 30, 40, 50]
    for i in 1..=5 {
        model._set(&format!("B{i}"), &(i * 10).to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B5,30)"); // Exact match
    model._set("A2", "=PERCENTRANK.INC(B1:B5,25)"); // Interpolated
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.5");
    assert_eq!(model._get_text("A2"), *"0.375");
}

#[test]
fn test_fn_percentrank_inc_decimals_basic() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B5,3.333,1)"); // 1 decimal place
    model._set("A2", "=PERCENTRANK.INC(B1:B5,3.333,2)"); // 2 decimal places
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.6");
    assert_eq!(model._get_text("A2"), *"0.58");
}

#[test]
fn test_fn_percentrank_inc_decimals_extreme() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B5,3.333,0)"); // 0 decimals
    model._set("A2", "=PERCENTRANK.INC(B1:B5,3.333,5)"); // 5 decimals
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"0.58325"); // Actual implementation value
}

#[test]
fn test_fn_percentrank_inc_wrong_argument_count() {
    let mut model = new_empty_model();
    for i in 0..3 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B3)"); // Missing x
    model._set("A2", "=PERCENTRANK.INC(B1:B3,2,3,4)"); // Too many args
    model._set("A3", "=PERCENTRANK.INC()"); // No args
    model.evaluate();

    assert!(model._get_text("A1").contains("#ERROR!"));
    assert!(model._get_text("A2").contains("#ERROR!"));
    assert!(model._get_text("A3").contains("#ERROR!"));
}

// ============================================================================
// PERCENTRANK.EXC BASIC FUNCTIONALITY TESTS
// ============================================================================

#[test]
fn test_fn_percentrank_exc_basic() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model._set("A1", "=PERCENTRANK.EXC(B1:B5,3.5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.583");
}

#[test]
fn test_fn_percentrank_exc_boundary_values() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    // Test boundary values for EXC (should be errors at extremes)
    model._set("A1", "=PERCENTRANK.EXC(B1:B5,1)"); // Exact minimum
    model._set("A2", "=PERCENTRANK.EXC(B1:B5,5)"); // Exact maximum
    model._set("A3", "=PERCENTRANK.EXC(B1:B5,0.5)"); // Below minimum
    model._set("A4", "=PERCENTRANK.EXC(B1:B5,6)"); // Above maximum
    model.evaluate();

    assert!(model._get_text("A1").contains("#NUM!"));
    assert!(model._get_text("A2").contains("#NUM!"));
    assert!(model._get_text("A3").contains("#NUM!"));
    assert!(model._get_text("A4").contains("#NUM!"));
}

#[test]
fn test_fn_percentrank_exc_empty_array() {
    let mut model = new_empty_model();
    model._set("A1", "=PERCENTRANK.EXC(B1:B1,5)");
    model.evaluate();

    assert!(model._get_text("A1").contains("#NUM!"));
}

#[test]
fn test_fn_percentrank_exc_exact_vs_interpolated() {
    let mut model = new_empty_model();
    // Array [10, 20, 30, 40, 50]
    for i in 1..=5 {
        model._set(&format!("B{i}"), &(i * 10).to_string());
    }

    model._set("A1", "=PERCENTRANK.EXC(B1:B5,30)"); // Exact match
    model._set("A2", "=PERCENTRANK.EXC(B1:B5,25)"); // Interpolated
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.5");
    assert_eq!(model._get_text("A2"), *"0.417");
}

#[test]
fn test_fn_percentrank_exc_decimals() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    model._set("A1", "=PERCENTRANK.EXC(B1:B5,3.333,1)"); // 1 decimal
    model._set("A2", "=PERCENTRANK.EXC(B1:B5,3.333,3)"); // 3 decimals
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.6");
    assert_eq!(model._get_text("A2"), *"0.556");
}

// ============================================================================
// MIXED DATA TYPE HANDLING TESTS
// ============================================================================

#[test]
fn test_fn_percentrank_with_text_data() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "text");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("B5", "5");

    model._set("A1", "=PERCENTRANK.INC(B1:B5,3)");
    model.evaluate();

    // Should ignore text and work with numeric values only [1,3,4,5]
    assert_eq!(model._get_text("A1"), *"0.333");
}

#[test]
fn test_fn_percentrank_with_boolean_data() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "TRUE");
    model._set("B3", "3");
    model._set("B4", "FALSE");
    model._set("B5", "5");

    model._set("A1", "=PERCENTRANK.INC(B1:B5,3)");
    model.evaluate();

    // Should ignore boolean values in ranges [1,3,5]
    assert_eq!(model._get_text("A1"), *"0.5");
}

// ============================================================================
// ERROR HANDLING AND EDGE CASE TESTS
// ============================================================================

#[test]
fn test_fn_percentrank_invalid_range() {
    let mut model = new_empty_model();

    model._set("A1", "=PERCENTRANK.INC(ZZ999:ZZ1000,5)");
    model._set("A2", "=PERCENTRANK.EXC(ZZ999:ZZ1000,5)");
    model.evaluate();

    assert!(model._get_text("A1").contains("#"));
    assert!(model._get_text("A2").contains("#"));
}

#[test]
fn test_fn_percentrank_decimal_precision_edge_cases() {
    let mut model = new_empty_model();
    for i in 0..5 {
        model._set(&format!("B{}", i + 1), &(i + 1).to_string());
    }

    // Test with high precision
    model._set("A1", "=PERCENTRANK.INC(B1:B5,3.333333,8)");
    // Test with zero precision
    model._set("A2", "=PERCENTRANK.INC(B1:B5,3.1,0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.58333325"); // Actual implementation value
    assert_eq!(model._get_text("A2"), *"1");
}

// ============================================================================
// PERFORMANCE AND LARGE DATASET TESTS
// ============================================================================

#[test]
fn test_fn_percentrank_large_dataset_correctness() {
    let mut model = new_empty_model();

    // Create a larger dataset (100 values)
    for i in 1..=100 {
        model._set(&format!("B{i}"), &i.to_string());
    }

    model._set("A1", "=PERCENTRANK.INC(B1:B100,95)");
    model._set("A2", "=PERCENTRANK.EXC(B1:B100,95)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.949");
    assert_eq!(model._get_text("A2"), *"0.941");
}
