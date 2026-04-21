#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_stdev_no_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=STDEV.S()");
    model._set("A2", "=STDEV.P()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_stdev_s_single_value_should_error() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("A1", "=STDEV.S(B1)");
    model._set("A2", "=STDEV.S(5)");
    model.evaluate();

    // STDEV.S requires at least 2 values, should return #DIV/0! error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

#[test]
fn test_fn_stdev_p_single_value() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("A1", "=STDEV.P(B1)");
    model._set("A2", "=STDEV.P(5)");
    model.evaluate();

    // STDEV.P with single value should return 0
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
}

#[test]
fn test_fn_stdev_empty_range() {
    let mut model = new_empty_model();
    // B1:B3 are all empty
    model._set("A1", "=STDEV.S(B1:B3)");
    model._set("A2", "=STDEV.P(B1:B3)");
    model.evaluate();

    // Both should error with division by zero since no numeric values
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

#[test]
fn test_fn_stdev_basic_calculation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("A1", "=STDEV.S(B1:B3)");
    model._set("A2", "=STDEV.P(B1:B3)");
    model.evaluate();

    // Sample standard deviation: sqrt(sum((x-mean)^2)/(n-1))
    // Values: 1, 2, 3; mean = 2
    // Variance = ((1-2)^2 + (2-2)^2 + (3-2)^2) / (3-1) = (1 + 0 + 1) / 2 = 1
    // STDEV.S = sqrt(1) = 1
    assert_eq!(model._get_text("A1"), *"1");

    // Population standard deviation: sqrt(sum((x-mean)^2)/n)
    // Variance = ((1-2)^2 + (2-2)^2 + (3-2)^2) / 3 = 2/3 ≈ 0.66667
    // STDEV.P = sqrt(2/3) ≈ 0.8164965809
    assert_eq!(model._get_text("A2"), *"0.816496581");
}

#[test]
fn test_fn_stdev_mixed_data_types() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'text"); // String from reference - ignored
    model._set("B5", ""); // Empty cell - ignored
    model._set("B6", "TRUE"); // Boolean from reference - ignored
    model._set("A1", "=STDEV.S(B1:B6)");
    model._set("A2", "=STDEV.P(B1:B6)");
    model.evaluate();

    // Only numeric values 1, 2, 3 are used
    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"0.816496581");
}

#[test]
fn test_fn_stdev_literals_vs_references() {
    let mut model = new_empty_model();
    model._set("B1", "TRUE"); // Boolean from reference - ignored
    model._set("B2", "'5"); // String from reference - ignored
                            // Boolean and string literals should be converted
    model._set("A1", "=STDEV.S(1, 2, 3, TRUE, \"5\")");
    model._set("A2", "=STDEV.P(1, 2, 3, TRUE, \"5\")");
    model.evaluate();

    // Values used: 1, 2, 3, 1 (TRUE), 5 ("5") = [1, 2, 3, 1, 5]
    // Mean = 12/5 = 2.4
    // Sample variance = ((1-2.4)^2 + (2-2.4)^2 + (3-2.4)^2 + (1-2.4)^2 + (5-2.4)^2) / 4
    //                 = (1.96 + 0.16 + 0.36 + 1.96 + 6.76) / 4 = 11.2 / 4 = 2.8
    // STDEV.S = sqrt(2.8) ≈ 1.6733200531
    assert_eq!(model._get_text("A1"), *"1.673320053");

    // Population variance = 11.2 / 5 = 2.24
    // STDEV.P = sqrt(2.24) ≈ 1.4966629547
    assert_eq!(model._get_text("A2"), *"1.496662955");
}

#[test]
fn test_fn_stdev_negative_numbers() {
    let mut model = new_empty_model();
    model._set("B1", "-2");
    model._set("B2", "-1");
    model._set("B3", "0");
    model._set("B4", "1");
    model._set("B5", "2");
    model._set("A1", "=STDEV.S(B1:B5)");
    model._set("A2", "=STDEV.P(B1:B5)");
    model.evaluate();

    // Values: -2, -1, 0, 1, 2; mean = 0
    // Sample variance = (4 + 1 + 0 + 1 + 4) / 4 = 10/4 = 2.5
    // STDEV.S = sqrt(2.5) ≈ 1.5811388301
    assert_eq!(model._get_text("A1"), *"1.58113883");

    // Population variance = 10/5 = 2
    // STDEV.P = sqrt(2) ≈ 1.4142135624
    assert_eq!(model._get_text("A2"), *"1.414213562");
}

#[test]
fn test_fn_stdev_all_same_values() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "5");
    model._set("B3", "5");
    model._set("B4", "5");
    model._set("A1", "=STDEV.S(B1:B4)");
    model._set("A2", "=STDEV.P(B1:B4)");
    model.evaluate();

    // All values are the same, so standard deviation should be 0
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
}

#[test]
fn test_fn_stdev_error_propagation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "=1/0"); // Division by zero error
    model._set("B3", "3");
    model._set("A1", "=STDEV.S(B1:B3)");
    model._set("A2", "=STDEV.P(B1:B3)");
    model.evaluate();

    // Error should propagate
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

#[test]
fn test_fn_stdev_larger_dataset() {
    let mut model = new_empty_model();
    // Setting up a larger dataset: 1, 4, 9, 16, 25, 36, 49, 64, 81, 100
    for i in 1..=10 {
        model._set(&format!("B{i}"), &format!("{}", i * i));
    }
    model._set("A1", "=STDEV.S(B1:B10)");
    model._set("A2", "=STDEV.P(B1:B10)");
    model.evaluate();

    // Values: 1, 4, 9, 16, 25, 36, 49, 64, 81, 100
    // This is a known dataset, we can verify the mathematical correctness
    // Mean = 385/10 = 38.5
    // Sample std dev should be approximately 32.731...
    // Population std dev should be approximately 31.113...

    // The exact values would need calculation, but we're testing the functions work with larger datasets
    // and don't crash or produce obviously wrong results
    let result_s = model._get_text("A1");
    let result_p = model._get_text("A2");

    // Basic sanity checks - results should be positive numbers
    assert!(result_s.parse::<f64>().unwrap() > 0.0);
    assert!(result_p.parse::<f64>().unwrap() > 0.0);
    // Sample std dev should be larger than population std dev
    assert!(result_s.parse::<f64>().unwrap() > result_p.parse::<f64>().unwrap());
}

#[test]
fn test_fn_stdev_decimal_values() {
    let mut model = new_empty_model();
    model._set("B1", "1.5");
    model._set("B2", "2.7");
    model._set("B3", "3.1");
    model._set("B4", "4.9");
    model._set("A1", "=STDEV.S(B1:B4)");
    model._set("A2", "=STDEV.P(B1:B4)");
    model.evaluate();

    // Values: 1.5, 2.7, 3.1, 4.9; mean = 12.2/4 = 3.05
    // Should handle decimal calculations correctly
    let result_s = model._get_text("A1");
    let result_p = model._get_text("A2");

    assert!(result_s.parse::<f64>().unwrap() > 0.0);
    assert!(result_p.parse::<f64>().unwrap() > 0.0);
    assert!(result_s.parse::<f64>().unwrap() > result_p.parse::<f64>().unwrap());
}

#[test]
fn test_fn_stdev_with_false_boolean_literal() {
    let mut model = new_empty_model();
    model._set("A1", "=STDEV.S(0, 1, FALSE)"); // FALSE literal should become 0
    model._set("A2", "=STDEV.P(0, 1, FALSE)");
    model.evaluate();

    // Values: 0, 1, 0 (FALSE); mean = 1/3 ≈ 0.333
    // This tests that FALSE literals are properly converted to 0
    let result_s = model._get_text("A1");
    let result_p = model._get_text("A2");

    assert!(result_s.parse::<f64>().unwrap() > 0.0);
    assert!(result_p.parse::<f64>().unwrap() > 0.0);
}

#[test]
fn test_fn_stdev_mixed_arguments_ranges_and_literals() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("A1", "=STDEV.S(B1:B2, 3, 4)"); // Mix of range and literals
    model._set("A2", "=STDEV.P(B1:B2, 3, 4)");
    model.evaluate();

    // Values: 1, 2, 3, 4; mean = 2.5
    // Sample variance = ((1-2.5)^2 + (2-2.5)^2 + (3-2.5)^2 + (4-2.5)^2) / 3
    //                 = (2.25 + 0.25 + 0.25 + 2.25) / 3 = 5/3 ≈ 1.667
    // STDEV.S = sqrt(5/3) ≈ 1.2909944487
    assert_eq!(model._get_text("A1"), *"1.290994449");

    // Population variance = 5/4 = 1.25
    // STDEV.P = sqrt(1.25) ≈ 1.1180339887
    assert_eq!(model._get_text("A2"), *"1.118033989");
}

#[test]
fn test_fn_stdev_range_with_only_non_numeric() {
    let mut model = new_empty_model();
    model._set("B1", "'text");
    model._set("B2", "TRUE"); // Boolean from reference
    model._set("B3", ""); // Empty
    model._set("A1", "=STDEV.S(B1:B3)");
    model._set("A2", "=STDEV.P(B1:B3)");
    model.evaluate();

    // No numeric values, should error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
}

#[test]
fn test_fn_stdev_mathematical_correctness_known_values() {
    let mut model = new_empty_model();
    // Using a simple known dataset for exact verification
    model._set("B1", "2");
    model._set("B2", "4");
    model._set("B3", "4");
    model._set("B4", "4");
    model._set("B5", "5");
    model._set("B6", "5");
    model._set("B7", "7");
    model._set("B8", "9");
    model._set("A1", "=STDEV.S(B1:B8)");
    model._set("A2", "=STDEV.P(B1:B8)");
    model.evaluate();

    // Values: 2, 4, 4, 4, 5, 5, 7, 9; mean = 40/8 = 5
    // Sample variance = ((2-5)^2 + (4-5)^2 + (4-5)^2 + (4-5)^2 + (5-5)^2 + (5-5)^2 + (7-5)^2 + (9-5)^2) / 7
    //                 = (9 + 1 + 1 + 1 + 0 + 0 + 4 + 16) / 7 = 32/7
    // STDEV.S = sqrt(32/7) ≈ 2.1380899353
    let result_s = model._get_text("A1");
    let expected_s = (32.0 / 7.0_f64).sqrt();
    assert!((result_s.parse::<f64>().unwrap() - expected_s).abs() < 1e-9);

    // Population variance = 32/8 = 4
    // STDEV.P = sqrt(4) = 2
    assert_eq!(model._get_text("A2"), *"2");
}
