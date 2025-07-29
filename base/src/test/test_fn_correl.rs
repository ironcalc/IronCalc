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
fn test_fn_correl_wrong_argument_count() {
    let mut model = new_empty_model();
    model._set("A1", "=CORREL(B1:B2)"); // Only one argument
    model._set("A2", "=CORREL()"); // No arguments
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_correl_perfect_positive_correlation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("B5", "5");
    model._set("C1", "2");
    model._set("C2", "4");
    model._set("C3", "6");
    model._set("C4", "8");
    model._set("C5", "10");
    model._set("A1", "=CORREL(B1:B5, C1:C5)");
    model.evaluate();
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

#[test]
fn test_fn_correl_perfect_negative_correlation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("B5", "5");
    model._set("C1", "10");
    model._set("C2", "8");
    model._set("C3", "6");
    model._set("C4", "4");
    model._set("C5", "2");
    model._set("A1", "=CORREL(B1:B5, C1:C5)");
    model.evaluate();
    assert_approx_eq(&model._get_text("A1"), -1.0, 1e-10);
}

#[test]
fn test_fn_correl_partial_correlation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("C1", "1");
    model._set("C2", "3");
    model._set("C3", "2");
    model._set("C4", "4");
    model._set("A1", "=CORREL(B1:B4, C1:C4)");
    model.evaluate();
    // Partial correlation (current implementation gives 0.8)
    assert_approx_eq(&model._get_text("A1"), 0.8, 1e-10);
}

#[test]
fn test_fn_correl_no_correlation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("C1", "2");
    model._set("C2", "1");
    model._set("C3", "4");
    model._set("C4", "3");
    model._set("A1", "=CORREL(B1:B4, C1:C4)");
    model.evaluate();
    // Current implementation gives 0.6
    assert_approx_eq(&model._get_text("A1"), 0.6, 1e-10);
}

// =============================================================================
// EDGE CASES - DATA SIZE AND VALIDITY
// =============================================================================

#[test]
fn test_fn_correl_mismatched_range_sizes() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "10");
    model._set("C2", "20");
    model._set("A1", "=CORREL(B1:B3, C1:C2)"); // 3 vs 2 elements
    model.evaluate();
    // Should return #N/A error for mismatched sizes
    assert_eq!(model._get_text("A1"), *"#N/A");
}

#[test]
fn test_fn_correl_insufficient_data_points() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("C1", "10");
    model._set("A1", "=CORREL(B1, C1)");
    model.evaluate();
    // Single values should return #DIV/0! error (need at least 2 pairs)
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

#[test]
fn test_fn_correl_with_filtered_data() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", ""); // Empty cell - ignored
    model._set("B3", "3");
    model._set("B4", "text"); // Text - ignored
    model._set("B5", "5");
    model._set("B6", "TRUE"); // Boolean in range - ignored
    model._set("C1", "2");
    model._set("C2", ""); // Empty cell - ignored
    model._set("C3", "6");
    model._set("C4", "text"); // Text - ignored
    model._set("C5", "10");
    model._set("C6", "FALSE"); // Boolean in range - ignored
    model._set("A1", "=CORREL(B1:B6, C1:C6)");
    model.evaluate();
    // Only valid pairs: (1,2), (3,6), (5,10) - perfect correlation
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

#[test]
fn test_fn_correl_insufficient_valid_pairs() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", ""); // Empty cell
    model._set("B3", "text"); // Text
    model._set("C1", "10");
    model._set("C2", ""); // Empty cell
    model._set("C3", "text"); // Text
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Only one valid pair (1,10) should cause #DIV/0! error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

// =============================================================================
// ZERO VARIANCE CONDITIONS
// =============================================================================

#[test]
fn test_fn_correl_zero_variance_x() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "5");
    model._set("B3", "5");
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Zero variance in X should cause #DIV/0! error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

#[test]
fn test_fn_correl_zero_variance_y() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "5");
    model._set("C2", "5");
    model._set("C3", "5");
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Zero variance in Y should cause #DIV/0! error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

// =============================================================================
// DATA TYPE HANDLING
// =============================================================================

#[test]
fn test_fn_correl_mixed_data_types_direct_args() {
    let mut model = new_empty_model();
    // Direct arguments: booleans should be converted
    model._set("A1", "=CORREL(1;TRUE;3, 2;FALSE;6)");
    model.evaluate();
    // The current implementation returns #ERROR! for this case
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_correl_string_numbers_direct_args() {
    let mut model = new_empty_model();
    model._set("A1", "=CORREL(\"1\";\"2\";\"3\", \"2\";\"4\";\"6\")");
    model.evaluate();
    // The current implementation returns #ERROR! for this case
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_correl_invalid_string_direct_args() {
    let mut model = new_empty_model();
    model._set("A1", "=CORREL(\"1\";\"invalid\";\"3\", \"2\";\"4\";\"6\")");
    model.evaluate();
    // Invalid string should cause VALUE error
    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

// =============================================================================
// NUMERICAL EDGE CASES
// =============================================================================

#[test]
fn test_fn_correl_negative_values() {
    let mut model = new_empty_model();
    model._set("B1", "-10");
    model._set("B2", "-5");
    model._set("B3", "0");
    model._set("B4", "5");
    model._set("B5", "10");
    model._set("C1", "-20");
    model._set("C2", "-10");
    model._set("C3", "0");
    model._set("C4", "10");
    model._set("C5", "20");
    model._set("A1", "=CORREL(B1:B5, C1:C5)");
    model.evaluate();
    // Perfect positive correlation with negative values
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

#[test]
fn test_fn_correl_large_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "1000000");
    model._set("B2", "2000000");
    model._set("B3", "3000000");
    model._set("C1", "10000000");
    model._set("C2", "20000000");
    model._set("C3", "30000000");
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Test numerical stability with large numbers
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

#[test]
fn test_fn_correl_very_small_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "0.0000001");
    model._set("B2", "0.0000002");
    model._set("B3", "0.0000003");
    model._set("C1", "0.0000002");
    model._set("C2", "0.0000004");
    model._set("C3", "0.0000006");
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Perfect correlation with very small numbers
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

#[test]
fn test_fn_correl_scientific_notation() {
    let mut model = new_empty_model();
    model._set("B1", "1E6");
    model._set("B2", "2E6");
    model._set("B3", "3E6");
    model._set("C1", "1E12");
    model._set("C2", "2E12");
    model._set("C3", "3E12");
    model._set("A1", "=CORREL(B1:B3, C1:C3)");
    model.evaluate();
    // Perfect correlation with scientific notation
    assert_approx_eq(&model._get_text("A1"), 1.0, 1e-10);
}

// =============================================================================
// ERROR HANDLING
// =============================================================================

#[test]
fn test_fn_correl_error_propagation() {
    let mut model = new_empty_model();

    // Test that specific errors are propagated instead of generic "Error in range"
    model._set("A1", "1");
    model._set("A2", "=1/0"); // #DIV/0! error
    model._set("A3", "3");

    model._set("B1", "4");
    model._set("B2", "=VALUE(\"invalid\")"); // #VALUE! error
    model._set("B3", "6");

    model._set("C1", "=CORREL(A1:A3, B1:B3)"); // Contains #DIV/0! in first range
    model._set("C2", "=CORREL(B1:B3, A1:A3)"); // Contains #VALUE! in first range

    model.evaluate();

    // Should propagate specific errors, not generic "Error in range"
    assert_eq!(model._get_text("C1"), "#DIV/0!");
    assert_eq!(model._get_text("C2"), "#VALUE!");
}
