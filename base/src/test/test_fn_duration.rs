#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::{cell::CellValue, test::util::new_empty_model};

// Test constants for realistic bond scenarios
const BOND_SETTLEMENT: &str = "=DATE(2020,1,1)";
const BOND_MATURITY_4Y: &str = "=DATE(2024,1,1)";
const BOND_MATURITY_INVALID: &str = "=DATE(2016,1,1)"; // Before settlement
const BOND_MATURITY_SAME: &str = "=DATE(2020,1,1)"; // Same as settlement
const BOND_MATURITY_1DAY: &str = "=DATE(2020,1,2)"; // Very short term

// Standard investment-grade corporate bond parameters
const STD_COUPON: f64 = 0.08; // 8% annual coupon rate
const STD_YIELD: f64 = 0.09; // 9% yield (discount bond scenario)
const STD_FREQUENCY: i32 = 2; // Semi-annual payments (most common)

// Helper function to reduce test repetition
fn assert_numerical_result(model: &crate::Model, cell_ref: &str, should_be_positive: bool) {
    if let Ok(CellValue::Number(v)) = model.get_cell_value_by_ref(cell_ref) {
        if should_be_positive {
            assert!(v > 0.0, "Expected positive value at {cell_ref}, got {v}");
        }
        // Value is valid - test passes
    } else {
        panic!("Expected numerical result at {cell_ref}");
    }
}

#[test]
fn fn_duration_mduration_arguments() {
    let mut model = new_empty_model();

    // Test argument count validation
    model._set("A1", "=DURATION()");
    model._set("A2", "=DURATION(1,2,3,4)");
    model._set("A3", "=DURATION(1,2,3,4,5,6,7)");

    model._set("B1", "=MDURATION()");
    model._set("B2", "=MDURATION(1,2,3,4)");
    model._set("B3", "=MDURATION(1,2,3,4,5,6,7)");

    model.evaluate();

    // Too few or too many arguments should result in errors
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_duration_mduration_settlement_maturity_errors() {
    let mut model = new_empty_model();

    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_INVALID); // Before settlement
    model._set("A3", BOND_MATURITY_SAME); // Same as settlement

    // Both settlement > maturity and settlement = maturity should error
    model._set(
        "B1",
        &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B2",
        &format!("=DURATION(A1,A3,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B3",
        &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B4",
        &format!("=MDURATION(A1,A3,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("B3"), *"#NUM!");
    assert_eq!(model._get_text("B4"), *"#NUM!");
}

#[test]
fn fn_duration_mduration_negative_values_errors() {
    let mut model = new_empty_model();

    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_4Y);

    // Test negative coupon (coupons must be >= 0)
    model._set(
        "B1",
        &format!("=DURATION(A1,A2,-0.01,{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B2",
        &format!("=MDURATION(A1,A2,-0.01,{STD_YIELD},{STD_FREQUENCY})"),
    );

    // Test negative yield (yields must be >= 0)
    model._set(
        "C1",
        &format!("=DURATION(A1,A2,{STD_COUPON},-0.01,{STD_FREQUENCY})"),
    );
    model._set(
        "C2",
        &format!("=MDURATION(A1,A2,{STD_COUPON},-0.01,{STD_FREQUENCY})"),
    );

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#NUM!");
    assert_eq!(model._get_text("B2"), *"#NUM!");
    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
}

#[test]
fn fn_duration_mduration_invalid_frequency_errors() {
    let mut model = new_empty_model();

    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_4Y);

    // Only 1, 2, and 4 are valid frequencies (annual, semi-annual, quarterly)
    let invalid_frequencies = [0, 3, 5, 12]; // Common invalid values

    for (i, &freq) in invalid_frequencies.iter().enumerate() {
        let row = i + 1;
        model._set(
            &format!("B{row}"),
            &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{freq})"),
        );
        model._set(
            &format!("C{row}"),
            &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{freq})"),
        );
    }

    model.evaluate();

    for i in 1..=invalid_frequencies.len() {
        assert_eq!(model._get_text(&format!("B{i}")), *"#NUM!");
        assert_eq!(model._get_text(&format!("C{i}")), *"#NUM!");
    }
}

#[test]
fn fn_duration_mduration_frequency_variations() {
    let mut model = new_empty_model();
    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_4Y);

    // Test all valid frequencies: 1=annual, 2=semi-annual, 4=quarterly
    let valid_frequencies = [1, 2, 4];

    for (i, &freq) in valid_frequencies.iter().enumerate() {
        let row = i + 1;
        model._set(
            &format!("B{row}"),
            &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{freq})"),
        );
        model._set(
            &format!("C{row}"),
            &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{freq})"),
        );
    }

    model.evaluate();

    // All should return positive numerical values
    for i in 1..=valid_frequencies.len() {
        assert_numerical_result(&model, &format!("Sheet1!B{i}"), true);
        assert_numerical_result(&model, &format!("Sheet1!C{i}"), true);
    }
}

#[test]
fn fn_duration_mduration_basis_variations() {
    let mut model = new_empty_model();
    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_4Y);

    // Test all valid basis values (day count conventions)
    // 0=30/360 US, 1=Actual/actual, 2=Actual/360, 3=Actual/365, 4=30/360 European
    for basis in 0..=4 {
        let row = basis + 1;
        model._set(
            &format!("B{row}"),
            &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY},{basis})"),
        );
        model._set(
            &format!("C{row}"),
            &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY},{basis})"),
        );
    }

    // Test default basis (should be 0)
    model._set(
        "D1",
        &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "D2",
        &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );

    model.evaluate();

    // All basis values should work
    for row in 1..=5 {
        assert_numerical_result(&model, &format!("Sheet1!B{row}"), true);
        assert_numerical_result(&model, &format!("Sheet1!C{row}"), true);
    }

    // Default basis should match basis 0
    if let (Ok(CellValue::Number(d1)), Ok(CellValue::Number(b1))) = (
        model.get_cell_value_by_ref("Sheet1!D1"),
        model.get_cell_value_by_ref("Sheet1!B1"),
    ) {
        assert!(
            (d1 - b1).abs() < 1e-10,
            "Default basis should match basis 0"
        );
    }
}

#[test]
fn fn_duration_mduration_edge_cases() {
    let mut model = new_empty_model();

    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_1DAY); // Very short term (1 day)
    model._set("A3", BOND_MATURITY_4Y); // Standard term

    // Edge case scenarios with explanations
    let test_cases = [
        ("B", "A1", "A2", STD_COUPON, STD_YIELD, "short_term"), // 1-day bond
        ("C", "A1", "A3", 0.0, STD_YIELD, "zero_coupon"),       // Zero coupon bond
        ("D", "A1", "A3", STD_COUPON, 0.0, "zero_yield"),       // Zero yield
        ("E", "A1", "A3", 1.0, 0.5, "high_rates"),              // High coupon/yield (100%/50%)
    ];

    for (col, settlement, maturity, coupon, yield_rate, _scenario) in test_cases {
        model._set(
            &format!("{col}1"),
            &format!("=DURATION({settlement},{maturity},{coupon},{yield_rate},{STD_FREQUENCY})"),
        );
        model._set(
            &format!("{col}2"),
            &format!("=MDURATION({settlement},{maturity},{coupon},{yield_rate},{STD_FREQUENCY})"),
        );
    }

    model.evaluate();

    // All edge cases should return positive values
    for col in ["B", "C", "D", "E"] {
        assert_numerical_result(&model, &format!("Sheet1!{col}1"), true);
        assert_numerical_result(&model, &format!("Sheet1!{col}2"), true);
    }
}

#[test]
fn fn_duration_mduration_relationship() {
    let mut model = new_empty_model();
    model._set("A1", BOND_SETTLEMENT);
    model._set("A2", BOND_MATURITY_4Y);

    // Test mathematical relationship: MDURATION = DURATION / (1 + yield/frequency)
    model._set(
        "B1",
        &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B2",
        &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set("B3", &format!("=B1/(1+{STD_YIELD}/{STD_FREQUENCY})")); // Manual calculation

    // Test with quarterly frequency and different yield
    model._set("C1", &format!("=DURATION(A1,A2,{STD_COUPON},0.12,4)"));
    model._set("C2", &format!("=MDURATION(A1,A2,{STD_COUPON},0.12,4)"));
    model._set("C3", "=C1/(1+0.12/4)"); // Manual calculation for quarterly

    model.evaluate();

    // MDURATION should equal DURATION / (1 + yield/frequency) for both scenarios
    if let (Ok(CellValue::Number(md)), Ok(CellValue::Number(manual))) = (
        model.get_cell_value_by_ref("Sheet1!B2"),
        model.get_cell_value_by_ref("Sheet1!B3"),
    ) {
        assert!(
            (md - manual).abs() < 1e-10,
            "MDURATION should equal DURATION/(1+yield/freq)"
        );
    }

    if let (Ok(CellValue::Number(md)), Ok(CellValue::Number(manual))) = (
        model.get_cell_value_by_ref("Sheet1!C2"),
        model.get_cell_value_by_ref("Sheet1!C3"),
    ) {
        assert!(
            (md - manual).abs() < 1e-10,
            "MDURATION should equal DURATION/(1+yield/freq) for quarterly"
        );
    }
}

#[test]
fn fn_duration_mduration_regression() {
    // Original regression test with known expected values
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2016,1,1)");
    model._set("A2", "=DATE(2020,1,1)");
    model._set(
        "B1",
        &format!("=DURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );
    model._set(
        "B2",
        &format!("=MDURATION(A1,A2,{STD_COUPON},{STD_YIELD},{STD_FREQUENCY})"),
    );

    model.evaluate();

    // Verify exact values for regression testing
    if let Ok(CellValue::Number(v1)) = model.get_cell_value_by_ref("Sheet1!B1") {
        assert!(
            (v1 - 3.410746844012284).abs() < 1e-9,
            "DURATION regression test failed"
        );
    } else {
        panic!("Unexpected value for DURATION");
    }
    if let Ok(CellValue::Number(v2)) = model.get_cell_value_by_ref("Sheet1!B2") {
        assert!(
            (v2 - 3.263872578002186).abs() < 1e-9,
            "MDURATION regression test failed"
        );
    } else {
        panic!("Unexpected value for MDURATION");
    }
}
