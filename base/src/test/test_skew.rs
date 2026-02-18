#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_skew_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SKEW()");
    model._set("A2", "=SKEW.P()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_skew_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'2");
    // B5 is empty
    model._set("B6", "true");
    model._set("A1", "=SKEW(B1:B6)");
    model._set("A2", "=SKEW.P(B1:B6)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
}

// Boundary condition tests
#[test]
fn test_skew_boundary_conditions() {
    let mut model = new_empty_model();

    // SKEW requires at least 3 numeric values
    model._set("A1", "=SKEW(1)");
    model._set("A2", "=SKEW(1, 2)");
    model._set("A3", "=SKEW(1, 2, 3)"); // Should work

    // SKEW.P requires at least 1 numeric value
    model._set("B1", "=SKEW.P(1)"); // Should work
    model._set("B2", "=SKEW.P()"); // Should error

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"0"); // Perfect symmetry = 0 skew
    assert_eq!(model._get_text("B1"), *"#DIV/0!"); // Single value has undefined skew
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

// Edge cases with identical values
#[test]
fn test_skew_identical_values() {
    let mut model = new_empty_model();

    // All identical values should cause division by zero (std = 0)
    model._set("A1", "=SKEW(5, 5, 5)");
    model._set("A2", "=SKEW.P(5, 5, 5, 5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

// Test with negative values and mixed signs
#[test]
fn test_skew_negative_values() {
    let mut model = new_empty_model();

    // Negative values
    model._set("A1", "=SKEW(-3, -2, -1)");
    model._set("A2", "=SKEW.P(-3, -2, -1)");

    // Mixed positive/negative (right-skewed)
    model._set("B1", "=SKEW(-1, 0, 10)");
    model._set("B2", "=SKEW.P(-1, 0, 10)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0"); // Symmetric
    assert_eq!(model._get_text("A2"), *"0"); // Symmetric

    // Should be positive (right-skewed due to outlier 10)
    let b1_val: f64 = model._get_text("B1").parse().unwrap();
    let b2_val: f64 = model._get_text("B2").parse().unwrap();
    assert!(b1_val > 0.0);
    assert!(b2_val > 0.0);
}

// Test mixed data types handling
#[test]
fn test_skew_mixed_data_types() {
    let mut model = new_empty_model();

    // Mix of numbers, text, booleans, empty cells
    model._set("A1", "1");
    model._set("A2", "true"); // Boolean in reference -> ignored
    model._set("A3", "'text"); // Text in reference -> ignored
    model._set("A4", "2");
    // A5 is empty -> ignored
    model._set("A6", "3");

    // Direct boolean and text arguments (coerced to numbers)
    model._set("B1", "=SKEW(1, 2, 3, TRUE, \"4\")"); // TRUE=1, "4"=4 → (1,2,3,1,4)
    model._set("B2", "=SKEW.P(A1:A6)"); // Range refs: only 1,2,3 used (booleans/text ignored)

    model.evaluate();

    // Direct args: SKEW(1,2,3,1,4) should work (not an error)
    assert_ne!(model._get_text("B1"), *"#ERROR!");
    // Range refs: SKEW.P(1,2,3) should be 0 (symmetric)
    assert_eq!(model._get_text("B2"), *"0");
}

// Test error propagation
#[test]
fn test_skew_error_propagation() {
    let mut model = new_empty_model();

    model._set("A1", "=1/0"); // DIV error
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("B1", "=SKEW(A1:A3)");
    model._set("B2", "=SKEW.P(A1, A2, A3)");

    model.evaluate();

    // Errors should propagate
    assert_eq!(model._get_text("B1"), *"#DIV/0!");
    assert_eq!(model._get_text("B2"), *"#DIV/0!");
}

// Test with known mathematical results
#[test]
fn test_skew_known_values() {
    let mut model = new_empty_model();

    // Right-skewed distribution: 1, 2, 2, 3, 8 (outlier pulls right)
    model._set("A1", "=SKEW(1, 2, 2, 3, 8)");
    model._set("A2", "=SKEW.P(1, 2, 2, 3, 8)");

    // Left-skewed distribution: 1, 6, 7, 7, 8 (outlier pulls left)
    model._set("B1", "=SKEW(1, 6, 7, 7, 8)");
    model._set("B2", "=SKEW.P(1, 6, 7, 7, 8)");

    // Perfectly symmetric distribution
    model._set("C1", "=SKEW(1, 2, 3, 4, 5)");
    model._set("C2", "=SKEW.P(1, 2, 3, 4, 5)");

    model.evaluate();

    // Right-skewed should be positive (> 0)
    let a1_val: f64 = model._get_text("A1").parse().unwrap();
    let a2_val: f64 = model._get_text("A2").parse().unwrap();
    assert!(a1_val > 0.0);
    assert!(a2_val > 0.0);

    // Left-skewed should be negative (< 0)
    let b1_val: f64 = model._get_text("B1").parse().unwrap();
    let b2_val: f64 = model._get_text("B2").parse().unwrap();
    assert!(b1_val < 0.0);
    assert!(b2_val < 0.0);

    // Symmetric should be exactly 0
    assert_eq!(model._get_text("C1"), *"0");
    assert_eq!(model._get_text("C2"), *"0");
}

// Test large dataset handling
#[test]
fn test_skew_large_dataset() {
    let mut model = new_empty_model();

    // Set up a larger dataset (normal distribution should have skew ≈ 0)
    for i in 1..=20 {
        model._set(&format!("A{i}"), &i.to_string());
    }

    model._set("B1", "=SKEW(A1:A20)");
    model._set("B2", "=SKEW.P(A1:A20)");

    model.evaluate();

    // Large symmetric dataset should have skew close to 0
    let b1_val: f64 = model._get_text("B1").parse().unwrap();
    let b2_val: f64 = model._get_text("B2").parse().unwrap();
    assert!(b1_val.abs() < 0.5); // Should be close to 0
    assert!(b2_val.abs() < 0.5); // Should be close to 0
}

// Test precision with small differences
#[test]
fn test_skew_precision() {
    let mut model = new_empty_model();

    // Test with very small numbers
    model._set("A1", "=SKEW(0.001, 0.002, 0.003)");
    model._set("A2", "=SKEW.P(0.001, 0.002, 0.003)");

    // Test with very large numbers
    model._set("B1", "=SKEW(1000000, 2000000, 3000000)");
    model._set("B2", "=SKEW.P(1000000, 2000000, 3000000)");

    model.evaluate();

    // Both should be 0 (perfect symmetry)
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("B1"), *"0");
    assert_eq!(model._get_text("B2"), *"0");
}

// Test ranges with no numeric values
#[test]
fn test_skew_empty_and_text_only() {
    let mut model = new_empty_model();

    // Range with only empty cells
    model._set("A1", "=SKEW(B1:B5)"); // Empty range
    model._set("A2", "=SKEW.P(B1:B5)"); // Empty range

    // Range with only text
    model._set("C1", "'text");
    model._set("C2", "'more");
    model._set("C3", "'words");
    model._set("A3", "=SKEW(C1:C3)");
    model._set("A4", "=SKEW.P(C1:C3)");

    model.evaluate();

    // All should error due to no numeric values
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#DIV/0!");
}

// Test SKEW vs SKEW.P differences
#[test]
fn test_skew_vs_skew_p_differences() {
    let mut model = new_empty_model();

    // Same dataset, different formulas
    model._set("A1", "=SKEW(1, 2, 3, 4, 10)"); // Sample skewness
    model._set("A2", "=SKEW.P(1, 2, 3, 4, 10)"); // Population skewness

    model.evaluate();

    // Both should be positive (right-skewed), but different values
    let skew_sample: f64 = model._get_text("A1").parse().unwrap();
    let skew_pop: f64 = model._get_text("A2").parse().unwrap();

    assert!(skew_sample > 0.0);
    assert!(skew_pop > 0.0);
    assert_ne!(skew_sample, skew_pop); // Should be different values
}
