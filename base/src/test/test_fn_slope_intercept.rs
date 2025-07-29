#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::test::util::new_empty_model;

// =============================================================================
// TEST CONSTANTS
// =============================================================================

const EXACT_TOLERANCE: f64 = 1e-10;
const STANDARD_TOLERANCE: f64 = 1e-9;
const HIGH_PRECISION_TOLERANCE: f64 = 1e-15;
const STABILITY_TOLERANCE: f64 = 1e-6;

// =============================================================================
// TEST HELPER FUNCTIONS
// =============================================================================

fn assert_approx_eq(actual: &str, expected: f64, tolerance: f64) {
    let actual_val: f64 = actual
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse result as number: {actual}"));
    assert!(
        (actual_val - expected).abs() < tolerance,
        "Expected ~{expected}, got {actual}"
    );
}

fn assert_slope_intercept_eq(
    model: &crate::Model,
    slope_cell: &str,
    intercept_cell: &str,
    expected_slope: f64,
    expected_intercept: f64,
    tolerance: f64,
) {
    assert_approx_eq(&model._get_text(slope_cell), expected_slope, tolerance);
    assert_approx_eq(
        &model._get_text(intercept_cell),
        expected_intercept,
        tolerance,
    );
}

fn assert_slope_intercept_error(
    model: &crate::Model,
    slope_cell: &str,
    intercept_cell: &str,
    expected_error: &str,
) {
    assert_eq!(model._get_text(slope_cell), *expected_error);
    assert_eq!(model._get_text(intercept_cell), *expected_error);
}

fn set_linear_data(model: &mut crate::Model, slope: f64, intercept: f64, x_values: &[f64]) {
    for (i, &x) in x_values.iter().enumerate() {
        let y = slope * x + intercept;
        model._set(&format!("B{}", i + 1), &y.to_string());
        model._set(&format!("C{}", i + 1), &x.to_string());
    }
}

// =============================================================================
// ARGUMENT VALIDATION TESTS
// =============================================================================

#[test]
fn test_slope_intercept_invalid_args() {
    let mut model = new_empty_model();

    // Wrong argument counts
    model._set("A1", "=SLOPE()");
    model._set("A2", "=SLOPE(B1:B3)");
    model._set("A3", "=INTERCEPT()");
    model._set("A4", "=INTERCEPT(B1:B3)");
    model._set("A5", "=SLOPE(B1:B3, C1:C3, D1:D3)");
    model._set("A6", "=INTERCEPT(B1:B3, C1:C3, D1:D3)");

    // Mismatched range sizes
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "10");
    model._set("C2", "20");
    model._set("A7", "=SLOPE(B1:B3, C1:C2)");
    model._set("A8", "=INTERCEPT(B1:B3, C1:C2)");

    // Direct invalid types
    model._set("A9", "=SLOPE(1;TRUE;3, 2;FALSE;6)");
    model._set("A10", "=INTERCEPT(\"1\";\"2\";\"3\", \"2\";\"4\";\"6\")");

    model.evaluate();

    // All should error appropriately
    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A9", "A10"] {
        assert_eq!(model._get_text(cell), "#ERROR!");
    }
    assert_slope_intercept_error(&model, "A7", "A8", "#N/A");
}

// =============================================================================
// CORE MATHEMATICAL FUNCTIONALITY TESTS
// =============================================================================

#[test]
fn test_slope_intercept_perfect_lines() {
    let mut model = new_empty_model();

    // Test 1: Positive slope through origin (y = 3x)
    set_linear_data(&mut model, 3.0, 0.0, &[1.0, 2.0, 3.0, 4.0]);
    model._set("A1", "=SLOPE(B1:B4, C1:C4)");
    model._set("A2", "=INTERCEPT(B1:B4, C1:C4)");

    // Test 2: Negative slope with intercept (y = -2x + 10)
    model._set("B5", "8");
    model._set("B6", "6");
    model._set("B7", "4");
    model._set("B8", "2");
    model._set("C5", "1");
    model._set("C6", "2");
    model._set("C7", "3");
    model._set("C8", "4");
    model._set("A3", "=SLOPE(B5:B8, C5:C8)");
    model._set("A4", "=INTERCEPT(B5:B8, C5:C8)");

    // Test 3: Zero slope (y = 7)
    model._set("B9", "7");
    model._set("B10", "7");
    model._set("B11", "7");
    model._set("C9", "10");
    model._set("C10", "20");
    model._set("C11", "30");
    model._set("A5", "=SLOPE(B9:B11, C9:C11)");
    model._set("A6", "=INTERCEPT(B9:B11, C9:C11)");

    model.evaluate();

    assert_slope_intercept_eq(&model, "A1", "A2", 3.0, 0.0, EXACT_TOLERANCE);
    assert_slope_intercept_eq(&model, "A3", "A4", -2.0, 10.0, EXACT_TOLERANCE);
    assert_slope_intercept_eq(&model, "A5", "A6", 0.0, 7.0, EXACT_TOLERANCE);
}

#[test]
fn test_slope_intercept_regression() {
    let mut model = new_empty_model();

    // Non-perfect data: (1,1), (2,2), (5,3)
    // Manual calculation: slope = 6/13, intercept = 10/13
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "5");
    model._set("A1", "=SLOPE(B1:B3, C1:C3)");
    model._set("A2", "=INTERCEPT(B1:B3, C1:C3)");

    model.evaluate();

    assert_slope_intercept_eq(
        &model,
        "A1",
        "A2",
        0.461538462,
        0.769230769,
        STANDARD_TOLERANCE,
    );
}

// =============================================================================
// DEGENERATE CASES
// =============================================================================

#[test]
fn test_slope_intercept_insufficient_data() {
    let mut model = new_empty_model();

    // Single data point
    model._set("B1", "5");
    model._set("C1", "10");
    model._set("A1", "=SLOPE(B1, C1)");
    model._set("A2", "=INTERCEPT(B1, C1)");

    // Empty ranges
    model._set("A3", "=SLOPE(B5:B7, C5:C7)");
    model._set("A4", "=INTERCEPT(B5:B7, C5:C7)");

    // Identical x values (vertical line)
    model._set("B8", "1");
    model._set("B9", "2");
    model._set("B10", "3");
    model._set("C8", "5");
    model._set("C9", "5");
    model._set("C10", "5");
    model._set("A5", "=SLOPE(B8:B10, C8:C10)");
    model._set("A6", "=INTERCEPT(B8:B10, C8:C10)");

    model.evaluate();

    assert_slope_intercept_error(&model, "A1", "A2", "#DIV/0!");
    assert_slope_intercept_error(&model, "A3", "A4", "#DIV/0!");
    assert_slope_intercept_error(&model, "A5", "A6", "#DIV/0!");
}

// =============================================================================
// DATA FILTERING AND ERROR PROPAGATION
// =============================================================================

#[test]
fn test_slope_intercept_data_filtering() {
    let mut model = new_empty_model();

    // Mixed data types - only numeric pairs used: (1,1), (5,2), (9,3) -> y = 4x - 3
    model._set("B1", "1"); // Valid
    model._set("B2", ""); // Empty - ignored
    model._set("B3", "text"); // Text - ignored
    model._set("B4", "5"); // Valid
    model._set("B5", "TRUE"); // Boolean - ignored
    model._set("B6", "9"); // Valid
    model._set("C1", "1"); // Valid
    model._set("C2", ""); // Empty - ignored
    model._set("C3", "text"); // Text - ignored
    model._set("C4", "2"); // Valid
    model._set("C5", "FALSE"); // Boolean - ignored
    model._set("C6", "3"); // Valid

    model._set("A1", "=SLOPE(B1:B6, C1:C6)");
    model._set("A2", "=INTERCEPT(B1:B6, C1:C6)");

    model.evaluate();

    assert_slope_intercept_eq(&model, "A1", "A2", 4.0, -3.0, EXACT_TOLERANCE);
}

#[test]
fn test_slope_intercept_error_propagation() {
    let mut model = new_empty_model();

    // Error in y values
    model._set("B1", "1");
    model._set("B2", "=1/0"); // Division by zero
    model._set("B3", "3");
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("A1", "=SLOPE(B1:B3, C1:C3)");
    model._set("A2", "=INTERCEPT(B1:B3, C1:C3)");

    // Error in x values
    model._set("B4", "1");
    model._set("B5", "2");
    model._set("B6", "3");
    model._set("C4", "1");
    model._set("C5", "=SQRT(-1)"); // NaN error
    model._set("C6", "3");
    model._set("A3", "=SLOPE(B4:B6, C4:C6)");
    model._set("A4", "=INTERCEPT(B4:B6, C4:C6)");

    model.evaluate();

    assert_slope_intercept_error(&model, "A1", "A2", "#DIV/0!");
    assert_slope_intercept_error(&model, "A3", "A4", "#NUM!");
}

// =============================================================================
// NUMERICAL PRECISION AND EXTREMES
// =============================================================================

#[test]
fn test_slope_intercept_numeric_precision() {
    let mut model = new_empty_model();

    // Very small slope near machine epsilon
    model._set("B1", "5.0001");
    model._set("B2", "5.0002");
    model._set("B3", "5.0003");
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("A1", "=SLOPE(B1:B3, C1:C3)");
    model._set("A2", "=INTERCEPT(B1:B3, C1:C3)");

    // Large numbers with stability concerns
    model._set("B4", "1000000");
    model._set("B5", "3000000");
    model._set("B6", "5000000");
    model._set("C4", "1000");
    model._set("C5", "2000");
    model._set("C6", "3000");
    model._set("A3", "=SLOPE(B4:B6, C4:C6)");
    model._set("A4", "=INTERCEPT(B4:B6, C4:C6)");

    model.evaluate();

    assert_slope_intercept_eq(&model, "A1", "A2", 0.0001, 5.0, HIGH_PRECISION_TOLERANCE);
    assert_slope_intercept_eq(&model, "A3", "A4", 2000.0, -1000000.0, STABILITY_TOLERANCE);
}

// =============================================================================
// RANGE ORIENTATIONS AND PERFORMANCE
// =============================================================================

#[test]
fn test_slope_intercept_range_orientations() {
    let mut model = new_empty_model();

    // Row-wise ranges: y = 3x - 1
    model._set("B1", "2"); // (1,2)
    model._set("C1", "5"); // (2,5)
    model._set("D1", "8"); // (3,8)
    model._set("B2", "1");
    model._set("C2", "2");
    model._set("D2", "3");
    model._set("A1", "=SLOPE(B1:D1, B2:D2)");
    model._set("A2", "=INTERCEPT(B1:D1, B2:D2)");

    model.evaluate();

    assert_slope_intercept_eq(&model, "A1", "A2", 3.0, -1.0, EXACT_TOLERANCE);
}

#[test]
fn test_slope_intercept_large_dataset() {
    let mut model = new_empty_model();

    // Test with 20 points: y = 0.1x + 100
    for i in 1..=20 {
        let y = 0.1 * i as f64 + 100.0;
        model._set(&format!("B{i}"), &y.to_string());
        model._set(&format!("C{i}"), &i.to_string());
    }

    model._set("A1", "=SLOPE(B1:B20, C1:C20)");
    model._set("A2", "=INTERCEPT(B1:B20, C1:C20)");

    model.evaluate();

    assert_slope_intercept_eq(&model, "A1", "A2", 0.1, 100.0, EXACT_TOLERANCE);
}

// =============================================================================
// REAL-WORLD EDGE CASES
// =============================================================================

#[test]
fn test_slope_intercept_statistical_outliers() {
    let mut model = new_empty_model();

    // Most points follow y = 2x + 1, with one outlier: (1,3), (2,5), (3,7), (4,9), (5,100)
    model._set("B1", "3");
    model._set("B2", "5");
    model._set("B3", "7");
    model._set("B4", "9");
    model._set("B5", "100"); // Statistical outlier
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("C4", "4");
    model._set("C5", "5");

    model._set("A1", "=SLOPE(B1:B5, C1:C5)");
    model._set("A2", "=INTERCEPT(B1:B5, C1:C5)");

    model.evaluate();

    // With outlier: mathematically correct results
    assert_approx_eq(&model._get_text("A1"), 19.8, STANDARD_TOLERANCE);
    assert_approx_eq(&model._get_text("A2"), -34.6, STANDARD_TOLERANCE);
}
