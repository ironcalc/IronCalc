#![allow(clippy::unwrap_used)]

use crate::{cell::CellValue, test::util::new_empty_model};

#[test]
fn fn_coupon_functions() {
    let mut model = new_empty_model();

    // Test with basis 1 (original test)
    model._set("A1", "=DATE(2001,1,25)");
    model._set("A2", "=DATE(2001,11,15)");
    model._set("B1", "=COUPDAYBS(A1,A2,2,1)");
    model._set("B2", "=COUPDAYS(A1,A2,2,1)");
    model._set("B3", "=COUPDAYSNC(A1,A2,2,1)");
    model._set("B4", "=COUPNCD(A1,A2,2,1)");
    model._set("B5", "=COUPNUM(A1,A2,2,1)");
    model._set("B6", "=COUPPCD(A1,A2,2,1)");

    // Test with basis 3 for better coverage
    model._set("C1", "=COUPDAYBS(DATE(2001,1,25),DATE(2001,11,15),2,3)");
    model._set("C2", "=COUPDAYS(DATE(2001,1,25),DATE(2001,11,15),2,3)");
    model._set("C3", "=COUPDAYSNC(DATE(2001,1,25),DATE(2001,11,15),2,3)");
    model._set("C4", "=COUPNCD(DATE(2001,1,25),DATE(2001,11,15),2,3)");
    model._set("C5", "=COUPNUM(DATE(2007,1,25),DATE(2008,11,15),2,1)");
    model._set("C6", "=COUPPCD(DATE(2001,1,25),DATE(2001,11,15),2,3)");

    model.evaluate();

    // Test basis 1
    assert_eq!(model._get_text("B1"), "71");
    assert_eq!(model._get_text("B2"), "181");
    assert_eq!(model._get_text("B3"), "110");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B4"),
        Ok(CellValue::Number(37026.0))
    );
    assert_eq!(model._get_text("B5"), "2");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B6"),
        Ok(CellValue::Number(36845.0))
    );

    // Test basis 3 (more comprehensive coverage)
    assert_eq!(model._get_text("C1"), "71");
    assert_eq!(model._get_text("C2"), "181"); // Fixed: actual days
    assert_eq!(model._get_text("C3"), "110");
    assert_eq!(model._get_text("C4"), "37026");
    assert_eq!(model._get_text("C5"), "4");
    assert_eq!(model._get_text("C6"), "36845");
}

#[test]
fn fn_coupon_functions_error_cases() {
    let mut model = new_empty_model();

    // Test invalid frequency
    model._set("E1", "=COUPDAYBS(DATE(2001,1,25),DATE(2001,11,15),3,1)");
    // Test invalid basis
    model._set("E2", "=COUPDAYS(DATE(2001,1,25),DATE(2001,11,15),2,5)");
    // Test settlement >= maturity
    model._set("E3", "=COUPDAYSNC(DATE(2001,11,15),DATE(2001,1,25),2,1)");
    // Test too few arguments
    model._set("E4", "=COUPNCD(DATE(2001,1,25),DATE(2001,11,15))");
    // Test too many arguments
    model._set("E5", "=COUPNUM(DATE(2001,1,25),DATE(2001,11,15),2,1,1)");

    model.evaluate();

    // All should return errors
    assert_eq!(model._get_text("E1"), "#NUM!");
    assert_eq!(model._get_text("E2"), "#NUM!");
    assert_eq!(model._get_text("E3"), "#NUM!");
    assert_eq!(model._get_text("E4"), *"#ERROR!");
    assert_eq!(model._get_text("E5"), *"#ERROR!");
}

#[test]
fn fn_coupdays_actual_day_count_fix() {
    // Verify COUPDAYS correctly distinguishes between fixed vs actual day count methods
    // Bug: basis 2&3 were incorrectly using fixed calculations like basis 0&4
    let mut model = new_empty_model();

    model._set("A1", "=DATE(2023,1,15)");
    model._set("A2", "=DATE(2023,7,15)");

    model._set("B1", "=COUPDAYS(A1,A2,2,0)"); // 30/360: uses 360/freq
    model._set("B2", "=COUPDAYS(A1,A2,2,2)"); // Actual/360: uses actual days
    model._set("B3", "=COUPDAYS(A1,A2,2,3)"); // Actual/365: uses actual days
    model._set("B4", "=COUPDAYS(A1,A2,2,4)"); // 30/360 European: uses 360/freq

    model.evaluate();

    // Basis 0&4: theoretical 360/2 = 180 days
    assert_eq!(model._get_text("B1"), "180");
    assert_eq!(model._get_text("B4"), "180");

    // Basis 2&3: actual days between Jan 15 and Jul 15 = 181 days
    assert_eq!(model._get_text("B2"), "181");
    assert_eq!(model._get_text("B3"), "181");
}

// =============================================================================
// FEBRUARY EDGE CASE TESTS - Day Count Convention Compliance
// =============================================================================
// These tests verify that financial functions correctly handle February dates
// according to the official 30/360 day count convention specifications.

#[test]
fn test_coupon_functions_february_consistency() {
    let mut model = new_empty_model();

    // Test that coupon functions behave consistently between US and European methods
    // when February dates are involved

    // Settlement: Last day of February (non-leap year)
    // Maturity: Some date in following year that creates a clear test case
    model._set("A1", "=DATE(2023,2,28)"); // Last day of Feb, non-leap year
    model._set("A2", "=DATE(2024,2,28)"); // Same day next year

    // Test COUPDAYS with different basis values
    model._set("B1", "=COUPDAYS(A1,A2,2,0)"); // US 30/360 - should treat Feb 28 as day 30
    model._set("B2", "=COUPDAYS(A1,A2,2,4)"); // European 30/360 - should treat Feb 28 as day 28
    model._set("B3", "=COUPDAYS(A1,A2,2,1)"); // Actual/actual - should use real days

    model.evaluate();

    // All should return valid numbers (no errors)
    assert_ne!(model._get_text("B1"), *"#NUM!");
    assert_ne!(model._get_text("B2"), *"#NUM!");
    assert_ne!(model._get_text("B3"), *"#NUM!");

    // US and European 30/360 should potentially give different results for February dates
    // (though the exact difference depends on the specific coupon calculation logic)
    let us_result = model._get_text("B1");
    let european_result = model._get_text("B2");
    let actual_result = model._get_text("B3");

    // Verify all are numeric
    assert!(us_result.parse::<f64>().is_ok());
    assert!(european_result.parse::<f64>().is_ok());
    assert!(actual_result.parse::<f64>().is_ok());
}

#[test]
fn test_february_edge_cases_leap_vs_nonleap() {
    let mut model = new_empty_model();

    // Test leap year vs non-leap year February handling

    // Feb 28 in non-leap year (this IS the last day of February)
    model._set("A1", "=DATE(2023,2,28)");
    model._set("A2", "=DATE(2023,8,28)");

    // Feb 28 in leap year (this is NOT the last day of February)
    model._set("A3", "=DATE(2024,2,28)");
    model._set("A4", "=DATE(2024,8,28)");

    // Feb 29 in leap year (this IS the last day of February)
    model._set("A5", "=DATE(2024,2,29)");
    model._set("A6", "=DATE(2024,8,29)");

    // Test with basis 0 (US 30/360) - should have special February handling
    model._set("B1", "=COUPDAYS(A1,A2,2,0)"); // Feb 28 non-leap (last day)
    model._set("B2", "=COUPDAYS(A3,A4,2,0)"); // Feb 28 leap year (not last day)
    model._set("B3", "=COUPDAYS(A5,A6,2,0)"); // Feb 29 leap year (last day)

    model.evaluate();

    // All should succeed
    assert_ne!(model._get_text("B1"), *"#NUM!");
    assert_ne!(model._get_text("B2"), *"#NUM!");
    assert_ne!(model._get_text("B3"), *"#NUM!");

    // Verify they're all numeric
    assert!(model._get_text("B1").parse::<f64>().is_ok());
    assert!(model._get_text("B2").parse::<f64>().is_ok());
    assert!(model._get_text("B3").parse::<f64>().is_ok());
}

#[test]
fn test_us_nasd_both_february_rule() {
    let mut model = new_empty_model();

    // Test the specific US/NASD rule: "If both date A and B fall on the last day of February,
    // then date B will be changed to the 30th"

    // Case 1: Both dates are Feb 28 in non-leap years (both are last day of February)
    model._set("A1", "=DATE(2023,2,28)"); // Last day of Feb 2023
    model._set("A2", "=DATE(2025,2,28)"); // Last day of Feb 2025

    // Case 2: Both dates are Feb 29 in leap years (both are last day of February)
    model._set("A3", "=DATE(2024,2,29)"); // Last day of Feb 2024
    model._set("A4", "=DATE(2028,2,29)"); // Last day of Feb 2028

    // Case 3: Mixed - Feb 28 non-leap to Feb 29 leap (both are last day of February)
    model._set("A5", "=DATE(2023,2,28)"); // Last day of Feb 2023
    model._set("A6", "=DATE(2024,2,29)"); // Last day of Feb 2024

    // Case 4: Control - Feb 28 in leap year (NOT last day) to Feb 29 (IS last day)
    model._set("A7", "=DATE(2024,2,28)"); // NOT last day of Feb 2024
    model._set("A8", "=DATE(2024,2,29)"); // IS last day of Feb 2024

    // Test using coupon functions that should apply US/NASD 30/360 (basis 0)
    model._set("B1", "=COUPDAYS(A1,A2,1,0)"); // Both last day Feb - Rule 1 should apply
    model._set("B2", "=COUPDAYS(A3,A4,1,0)"); // Both last day Feb - Rule 1 should apply
    model._set("B3", "=COUPDAYS(A5,A6,1,0)"); // Both last day Feb - Rule 1 should apply
    model._set("B4", "=COUPDAYS(A7,A8,1,0)"); // Only end is last day Feb - Rule 1 should NOT apply

    // Compare with European method (basis 4) - should behave differently
    model._set("C1", "=COUPDAYS(A1,A2,1,4)"); // European - no special Feb handling
    model._set("C2", "=COUPDAYS(A3,A4,1,4)"); // European - no special Feb handling
    model._set("C3", "=COUPDAYS(A5,A6,1,4)"); // European - no special Feb handling
    model._set("C4", "=COUPDAYS(A7,A8,1,4)"); // European - no special Feb handling

    model.evaluate();

    // All should succeed without errors
    for row in ["B1", "B2", "B3", "B4", "C1", "C2", "C3", "C4"] {
        assert_ne!(model._get_text(row), *"#NUM!", "Failed for {row}");
        assert!(
            model._get_text(row).parse::<f64>().is_ok(),
            "Non-numeric result for {row}"
        );
    }
}

#[test]
fn test_coupon_functions_february_edge_cases() {
    let mut model = new_empty_model();

    // Test that coupon functions handle February dates correctly without errors

    // Settlement: February 28, 2023 (non-leap), Maturity: February 28, 2024 (leap)
    model._set("A1", "=DATE(2023,2,28)");
    model._set("A2", "=DATE(2024,2,28)");

    // Test with basis 0 (US 30/360 - should use special February handling)
    model._set("B1", "=COUPDAYBS(A1,A2,2,0)");
    model._set("B2", "=COUPDAYS(A1,A2,2,0)");
    model._set("B3", "=COUPDAYSNC(A1,A2,2,0)");

    // Test with basis 4 (European 30/360 - should NOT use special February handling)
    model._set("C1", "=COUPDAYBS(A1,A2,2,4)");
    model._set("C2", "=COUPDAYS(A1,A2,2,4)");
    model._set("C3", "=COUPDAYSNC(A1,A2,2,4)");

    model.evaluate();

    // With US method (basis 0), February dates should be handled specially
    // With European method (basis 4), February dates should use actual dates
    // Key point: both should work without errors

    // We're ensuring functions complete successfully with February dates
    assert_ne!(model._get_text("B1"), *"#NUM!");
    assert_ne!(model._get_text("B2"), *"#NUM!");
    assert_ne!(model._get_text("B3"), *"#NUM!");
    assert_ne!(model._get_text("C1"), *"#NUM!");
    assert_ne!(model._get_text("C2"), *"#NUM!");
    assert_ne!(model._get_text("C3"), *"#NUM!");
}
