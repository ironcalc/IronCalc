#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
use crate::test::util::new_empty_model;

// Helper function for approximate floating point comparison
fn assert_approx_eq(actual: &str, expected: f64, tolerance: f64) {
    let actual_val: f64 = actual
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse result as number: {actual}"));
    assert!(
        (actual_val - expected).abs() < tolerance,
        "Expected ~{expected}, got {actual}"
    );
}

// =============================================================================
// BASIC FUNCTIONALITY TESTS
// =============================================================================

#[test]
fn test_fn_var_no_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=VAR.S()");
    model._set("A2", "=VAR.P()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_var_basic_calculation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("B5", "5");
    model._set("A1", "=VAR.S(B1:B5)");
    model._set("A2", "=VAR.P(B1:B5)");
    model.evaluate();
    // Data: [1,2,3,4,5], mean=3, sample_var=2.5, pop_var=2.0
    assert_approx_eq(&model._get_text("A1"), 2.5, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 2.0, 1e-10);
}

// =============================================================================
// EDGE CASES - DATA SIZE
// =============================================================================

#[test]
fn test_fn_var_single_value() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("A1", "=VAR.S(B1)");
    model._set("A2", "=VAR.P(B1)");
    model.evaluate();
    // VAR.S needs ≥2 values (n-1 denominator), VAR.P works with 1 value
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_approx_eq(&model._get_text("A2"), 0.0, 1e-10);
}

#[test]
fn test_fn_var_empty_range() {
    let mut model = new_empty_model();
    model._set("A1", "=VAR.S(B1:B5)");
    model._set("A2", "=VAR.P(B1:B5)");
    model.evaluate();
    // Both should error with no data
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

#[test]
fn test_fn_var_zero_variance() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "5");
    model._set("B3", "5");
    model._set("A1", "=VAR.S(B1:B3)");
    model._set("A2", "=VAR.P(B1:B3)");
    model.evaluate();
    // All identical values should give zero variance
    assert_approx_eq(&model._get_text("A1"), 0.0, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 0.0, 1e-10);
}

// =============================================================================
// DATA TYPE HANDLING
// =============================================================================

#[test]
fn test_fn_var_mixed_data_types_direct_args() {
    let mut model = new_empty_model();
    // Direct arguments: booleans and string numbers should be converted
    model._set("A1", "=VAR.S(1, TRUE, 3, FALSE, 5)");
    model._set("A2", "=VAR.P(1, TRUE, 3, FALSE, 5)");
    model.evaluate();
    // Values: [1, 1, 3, 0, 5], mean=2, but current implementation gives different results
    assert_approx_eq(&model._get_text("A1"), 4.0, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 3.2, 1e-10);
}

#[test]
fn test_fn_var_string_numbers_direct_args() {
    let mut model = new_empty_model();
    model._set("A1", "=VAR.S(\"1\", \"2\", \"3\", \"4\")");
    model._set("A2", "=VAR.P(\"1\", \"2\", \"3\", \"4\")");
    model.evaluate();
    // String numbers as direct args should be parsed: [1,2,3,4], mean=2.5
    assert_approx_eq(&model._get_text("A1"), 1.667, 1e-3); // (5/3)
    assert_approx_eq(&model._get_text("A2"), 1.25, 1e-10);
}

#[test]
fn test_fn_var_invalid_string_direct_args() {
    let mut model = new_empty_model();
    model._set("A1", "=VAR.S(\"1\", \"invalid\", \"3\")");
    model.evaluate();
    // Invalid strings should cause VALUE error
    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn test_fn_var_range_data_filtering() {
    let mut model = new_empty_model();
    // Test that ranges properly filter out non-numeric data
    model._set("B1", "1"); // number - included
    model._set("B2", ""); // empty - ignored
    model._set("B3", "3"); // number - included
    model._set("B4", "text"); // text - ignored
    model._set("B5", "5"); // number - included
    model._set("B6", "TRUE"); // boolean in range - ignored
    model._set("A1", "=VAR.S(B1:B6)");
    model._set("A2", "=VAR.P(B1:B6)");
    model.evaluate();
    // Only numbers used: [1,3,5], mean=3, sample_var=4, pop_var=8/3
    assert_approx_eq(&model._get_text("A1"), 4.0, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 2.667, 1e-3);
}

// =============================================================================
// NUMERICAL EDGE CASES
// =============================================================================

#[test]
fn test_fn_var_negative_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "-10");
    model._set("B2", "-5");
    model._set("B3", "0");
    model._set("B4", "5");
    model._set("B5", "10");
    model._set("A1", "=VAR.S(B1:B5)");
    model._set("A2", "=VAR.P(B1:B5)");
    model.evaluate();
    // Values: [-10,-5,0,5,10], mean=0, sample_var=62.5, pop_var=50
    assert_approx_eq(&model._get_text("A1"), 62.5, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 50.0, 1e-10);
}

#[test]
fn test_fn_var_scientific_notation() {
    let mut model = new_empty_model();
    model._set("B1", "1E6");
    model._set("B2", "1.001E6");
    model._set("B3", "1.002E6");
    model._set("A1", "=VAR.S(B1:B3)");
    model._set("A2", "=VAR.P(B1:B3)");
    model.evaluate();
    // Should handle scientific notation properly
    assert_approx_eq(&model._get_text("A1"), 1e6, 1e3); // Large variance due to data values
    assert_approx_eq(&model._get_text("A2"), 666666.67, 1e3);
}

#[test]
fn test_fn_var_very_small_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "0.0000001");
    model._set("B2", "0.0000002");
    model._set("B3", "0.0000003");
    model._set("A1", "=VAR.S(B1:B3)");
    model._set("A2", "=VAR.P(B1:B3)");
    model.evaluate();
    // Test numerical precision with very small numbers
    assert_approx_eq(&model._get_text("A1"), 1e-14, 1e-15);
    assert_approx_eq(&model._get_text("A2"), 6.667e-15, 1e-16);
}

#[test]
fn test_fn_var_large_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "1000000");
    model._set("B2", "1000001");
    model._set("B3", "1000002");
    model._set("A1", "=VAR.S(B1:B3)");
    model._set("A2", "=VAR.P(B1:B3)");
    model.evaluate();
    // Test numerical stability with large numbers
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
    assert_approx_eq(&model._get_text("A2"), 0.667, 1e-3);
}

// =============================================================================
// ERROR HANDLING
// =============================================================================

#[test]
fn test_fn_var_error_propagation() {
    let mut model = new_empty_model();

    // Test that specific errors are propagated instead of generic "Error in range"
    model._set("A1", "1");
    model._set("A2", "=1/0"); // #DIV/0! error
    model._set("A3", "=VALUE(\"invalid\")"); // #VALUE! error
    model._set("A4", "3");

    model._set("B1", "=VAR.S(A1:A2,A4)"); // Contains #DIV/0!
    model._set("B2", "=VAR.P(A1,A3,A4)"); // Contains #VALUE!

    model.evaluate();

    // Should propagate specific errors, not generic "Error in range"
    assert_eq!(model._get_text("B1"), "#DIV/0!");
    assert_eq!(model._get_text("B2"), "#VALUE!");
}

#[test]
fn test_fn_var_multiple_ranges() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("C1", "3");
    model._set("C2", "4");
    model._set("A1", "=VAR.S(B1:B2, C1:C2)");
    model._set("A2", "=VAR.P(B1:B2, C1:C2)");
    model.evaluate();
    // Multiple ranges: [1,2,3,4], mean=2.5, sample_var=5/3, pop_var=1.25
    assert_approx_eq(&model._get_text("A1"), 1.667, 1e-3);
    assert_approx_eq(&model._get_text("A2"), 1.25, 1e-10);
}
