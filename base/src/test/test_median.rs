#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_median_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=MEDIAN()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_median_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'2");
    // B5 empty
    model._set("B6", "true");
    model._set("A1", "=MEDIAN(B1:B6)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
}

#[test]
fn test_fn_median_empty_values_error() {
    let mut model = new_empty_model();
    // Test with only non-numeric values (should return #DIV/0! error, not 0)
    model._set("B1", "\"text\"");
    model._set("B2", "\"more text\"");
    model._set("B3", ""); // empty cell
    model._set("A1", "=MEDIAN(B1:B3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
}

#[test]
fn test_fn_median_with_error_values() {
    let mut model = new_empty_model();
    // Test that error values are properly handled and don't break sorting
    model._set("B1", "1");
    model._set("B2", "=SQRT(-1)"); // This produces #NUM! error
    model._set("B3", "3");
    model._set("B4", "5");
    model._set("A1", "=MEDIAN(B1:B4)");
    model.evaluate();

    // Should propagate the error from B2
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn test_fn_median_mixed_values() {
    let mut model = new_empty_model();
    // Test median calculation with mixed numeric and text values
    model._set("B1", "1");
    model._set("B2", "\"text\""); // String, should be ignored
    model._set("B3", "3");
    model._set("B4", "5");
    model._set("B5", ""); // Empty cell
    model._set("A1", "=MEDIAN(B1:B5)");
    model.evaluate();

    // Should return median of [1, 3, 5] = 3, ignoring text and empty cells
    assert_eq!(model._get_text("A1"), *"3");
}

#[test]
fn test_fn_median_single_value() {
    let mut model = new_empty_model();
    // Test median of a single literal value
    model._set("A1", "=MEDIAN(42)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"42");

    // Test median of a single value in a range
    model._set("B1", "7.5");
    model._set("A2", "=MEDIAN(B1:B1)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"7.5");
}

#[test]
fn test_fn_median_two_values() {
    let mut model = new_empty_model();
    // Test with 2 values - should return average
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("A1", "=MEDIAN(B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
}

#[test]
fn test_fn_median_four_values() {
    let mut model = new_empty_model();
    // Test with 4 values - should return average of middle two
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "3");
    model._set("C4", "4");
    model._set("A1", "=MEDIAN(C1:C4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2.5");
}

#[test]
fn test_fn_median_unsorted_data() {
    let mut model = new_empty_model();
    // Test with 6 values in non-sorted order
    model._set("D1", "10");
    model._set("D2", "1");
    model._set("D3", "5");
    model._set("D4", "8");
    model._set("D5", "3");
    model._set("D6", "7");
    model._set("A1", "=MEDIAN(D1:D6)");
    model.evaluate();
    // Sorted: [1, 3, 5, 7, 8, 10] -> median = (5+7)/2 = 6
    assert_eq!(model._get_text("A1"), *"6");
}

#[test]
fn test_fn_median_odd_length_datasets() {
    let mut model = new_empty_model();

    // Test with 5 values in random order
    model._set("C1", "20");
    model._set("C2", "5");
    model._set("C3", "15");
    model._set("C4", "10");
    model._set("C5", "25");
    model._set("A1", "=MEDIAN(C1:C5)");
    model.evaluate();
    // Sorted: [5, 10, 15, 20, 25] -> median = 15
    assert_eq!(model._get_text("A1"), *"15");

    // Test with 7 values including decimals
    model._set("D1", "1.1");
    model._set("D2", "2.2");
    model._set("D3", "3.3");
    model._set("D4", "4.4");
    model._set("D5", "5.5");
    model._set("D6", "6.6");
    model._set("D7", "7.7");
    model._set("A2", "=MEDIAN(D1:D7)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"4.4");
}

#[test]
fn test_fn_median_identical_values() {
    let mut model = new_empty_model();

    // Test with all same integers
    model._set("B1", "5");
    model._set("B2", "5");
    model._set("B3", "5");
    model._set("B4", "5");
    model._set("A1", "=MEDIAN(B1:B4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"5");

    // Test with all same decimals
    model._set("C1", "3.14");
    model._set("C2", "3.14");
    model._set("C3", "3.14");
    model._set("A2", "=MEDIAN(C1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"3.14");
}

#[test]
fn test_fn_median_negative_numbers() {
    let mut model = new_empty_model();

    // Test with all negative numbers
    model._set("B1", "-5");
    model._set("B2", "-3");
    model._set("B3", "-1");
    model._set("A1", "=MEDIAN(B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"-3");

    // Test with mix of positive and negative numbers
    model._set("C1", "-10");
    model._set("C2", "-5");
    model._set("C3", "0");
    model._set("C4", "5");
    model._set("C5", "10");
    model._set("A2", "=MEDIAN(C1:C5)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"0");

    // Test with negative decimals
    model._set("D1", "-2.5");
    model._set("D2", "-1.5");
    model._set("D3", "-0.5");
    model._set("D4", "0.5");
    model._set("A3", "=MEDIAN(D1:D4)");
    model.evaluate();
    // Sorted: [-2.5, -1.5, -0.5, 0.5] -> median = (-1.5 + -0.5)/2 = -1
    assert_eq!(model._get_text("A3"), *"-1");
}

#[test]
fn test_fn_median_mixed_argument_types() {
    let mut model = new_empty_model();

    // Test with combination of individual values and ranges
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("C1", "7");
    model._set("C2", "9");

    // MEDIAN(range, individual value, range)
    model._set("A1", "=MEDIAN(B1:B2, 4, B3, C1:C2)");
    model.evaluate();
    // Values: [1, 3, 4, 5, 7, 9] -> median = (4+5)/2 = 4.5
    assert_eq!(model._get_text("A1"), *"4.5");

    // Test with multiple individual arguments
    model._set("A2", "=MEDIAN(10, 20, 30, 40, 50)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), *"30");
}

#[test]
fn test_fn_median_large_dataset() {
    let mut model = new_empty_model();

    // Test with larger dataset (20 values)
    for i in 1..=20 {
        model._set(&format!("A{i}"), &(i * 2).to_string());
    }
    model._set("B1", "=MEDIAN(A1:A20)");
    model.evaluate();
    // Values: [2, 4, 6, ..., 40] (20 values) -> median = (20+22)/2 = 21
    assert_eq!(model._get_text("B1"), *"21");

    // Test with larger odd dataset (21 values)
    model._set("A21", "42");
    model._set("B2", "=MEDIAN(A1:A21)");
    model.evaluate();
    // Values: [2, 4, 6, ..., 40, 42] (21 values) -> median = 22 (11th value)
    assert_eq!(model._get_text("B2"), *"22");
}

#[test]
fn test_fn_median_high_precision() {
    let mut model = new_empty_model();

    // Test with high precision decimals
    model._set("A1", "1.123456789");
    model._set("A2", "2.987654321");
    model._set("A3", "3.555555555");
    model._set("B1", "=MEDIAN(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"2.987654321");

    // Test with very small numbers
    model._set("C1", "0.0000001");
    model._set("C2", "0.0000002");
    model._set("C3", "0.0000003");
    model._set("B2", "=MEDIAN(C1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("B2"), *"0.0000002");
}

#[test]
fn test_fn_median_large_numbers() {
    let mut model = new_empty_model();

    // Test with very large numbers
    model._set("C1", "1000000");
    model._set("C2", "2000000");
    model._set("C3", "3000000");
    model._set("C4", "4000000");
    model._set("A1", "=MEDIAN(C1:C4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2500000");
}

#[test]
fn test_fn_median_scientific_notation() {
    let mut model = new_empty_model();

    // Test with scientific notation
    model._set("D1", "1E6");
    model._set("D2", "2E6");
    model._set("D3", "3E6");
    model._set("A1", "=MEDIAN(D1:D3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2000000");
}

#[test]
fn test_fn_median_multiple_ranges() {
    let mut model = new_empty_model();

    // Test with multiple non-contiguous ranges
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");

    model._set("C1", "7");
    model._set("C2", "8");
    model._set("C3", "9");

    model._set("E1", "4");
    model._set("E2", "5");
    model._set("E3", "6");

    model._set("B1", "=MEDIAN(A1:A3, C1:C3, E1:E3)");
    model.evaluate();
    // Values: [1, 2, 3, 7, 8, 9, 4, 5, 6] sorted: [1, 2, 3, 4, 5, 6, 7, 8, 9] -> median = 5
    assert_eq!(model._get_text("B1"), *"5");
}

#[test]
fn test_fn_median_zeros_and_small_numbers() {
    let mut model = new_empty_model();

    // Test with zeros and small numbers
    model._set("A1", "0");
    model._set("A2", "0.001");
    model._set("A3", "0.002");
    model._set("A4", "0.003");
    model._set("B1", "=MEDIAN(A1:A4)");
    model.evaluate();
    // Sorted: [0, 0.001, 0.002, 0.003] -> median = (0.001 + 0.002)/2 = 0.0015
    assert_eq!(model._get_text("B1"), *"0.0015");

    // Test with all zeros
    model._set("D1", "0");
    model._set("D2", "0");
    model._set("D3", "0");
    model._set("D4", "0");
    model._set("D5", "0");
    model._set("B2", "=MEDIAN(D1:D5)");
    model.evaluate();
    assert_eq!(model._get_text("B2"), *"0");
}
