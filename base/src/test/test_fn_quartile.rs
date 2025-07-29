#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

#[test]
fn test_quartile_basic_functionality() {
    let mut model = new_empty_model();
    for i in 1..=8 {
        model._set(&format!("B{i}"), &i.to_string());
    }

    // Test basic quartile calculations
    model._set("A1", "=QUARTILE(B1:B8,1)"); // Legacy function
    model._set("A2", "=QUARTILE.INC(B1:B8,3)"); // Inclusive method
    model._set("A3", "=QUARTILE.EXC(B1:B8,1)"); // Exclusive method
    model.evaluate();

    assert_eq!(model._get_text("A1"), "2.75");
    assert_eq!(model._get_text("A2"), "6.25");
    assert_eq!(model._get_text("A3"), "2.25");
}

#[test]
fn test_quartile_all_parameters() {
    let mut model = new_empty_model();
    for i in 1..=8 {
        model._set(&format!("B{i}"), &i.to_string());
    }

    // Test all valid quartile parameters
    model._set("A1", "=QUARTILE.INC(B1:B8,0)"); // Min
    model._set("A2", "=QUARTILE.INC(B1:B8,2)"); // Median
    model._set("A3", "=QUARTILE.INC(B1:B8,4)"); // Max
    model._set("A4", "=QUARTILE.EXC(B1:B8,2)"); // EXC median
    model.evaluate();

    assert_eq!(model._get_text("A1"), "1"); // Min
    assert_eq!(model._get_text("A2"), "4.5"); // Median
    assert_eq!(model._get_text("A3"), "8"); // Max
    assert_eq!(model._get_text("A4"), "4.5"); // EXC median
}

#[test]
fn test_quartile_data_filtering() {
    let mut model = new_empty_model();

    // Mixed data types - only numbers should be considered
    model._set("B1", "1");
    model._set("B2", "text"); // Ignored
    model._set("B3", "3");
    model._set("B4", "TRUE"); // Ignored
    model._set("B5", "5");
    model._set("B6", ""); // Ignored

    model._set("A1", "=QUARTILE.INC(B1:B6,2)"); // Median of [1,3,5]
    model.evaluate();

    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_quartile_single_element() {
    let mut model = new_empty_model();
    model._set("B1", "5");

    model._set("A1", "=QUARTILE.INC(B1,0)"); // Min
    model._set("A2", "=QUARTILE.INC(B1,2)"); // Median
    model._set("A3", "=QUARTILE.INC(B1,4)"); // Max
    model.evaluate();

    // All quartiles should return the single value
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "5");
}

#[test]
fn test_quartile_duplicate_values() {
    let mut model = new_empty_model();
    // Data with duplicates: 1, 1, 3, 3
    model._set("C1", "1");
    model._set("C2", "1");
    model._set("C3", "3");
    model._set("C4", "3");

    model._set("A1", "=QUARTILE.INC(C1:C4,1)"); // Q1
    model._set("A2", "=QUARTILE.INC(C1:C4,2)"); // Q2
    model._set("A3", "=QUARTILE.INC(C1:C4,3)"); // Q3
    model.evaluate();

    assert_eq!(model._get_text("A1"), "1"); // Q1 with duplicates
    assert_eq!(model._get_text("A2"), "2"); // Median with duplicates
    assert_eq!(model._get_text("A3"), "3"); // Q3 with duplicates
}

#[test]
fn test_quartile_exc_boundary_conditions() {
    let mut model = new_empty_model();

    // Small dataset for EXC - should work for median but fail for Q1/Q3
    model._set("D1", "1");
    model._set("D2", "2");

    model._set("A1", "=QUARTILE.EXC(D1:D2,1)"); // Should fail
    model._set("A2", "=QUARTILE.EXC(D1:D2,2)"); // Should work (median)
    model._set("A3", "=QUARTILE.EXC(D1:D2,3)"); // Should fail
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!"); // EXC Q1 fails
    assert_eq!(model._get_text("A2"), "1.5"); // EXC median works
    assert_eq!(model._get_text("A3"), "#NUM!"); // EXC Q3 fails
}

#[test]
fn test_quartile_invalid_arguments() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");

    // Invalid argument count
    model._set("A1", "=QUARTILE.INC(B1:B2)"); // Too few
    model._set("A2", "=QUARTILE.INC(B1:B2,1,2)"); // Too many
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn test_quartile_invalid_quartile_values() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");

    // Invalid quartile values for QUARTILE.INC
    model._set("A1", "=QUARTILE.INC(B1:B2,-1)"); // Below 0
    model._set("A2", "=QUARTILE.INC(B1:B2,5)"); // Above 4

    // Invalid quartile values for QUARTILE.EXC
    model._set("A3", "=QUARTILE.EXC(B1:B2,0)"); // Below 1
    model._set("A4", "=QUARTILE.EXC(B1:B2,4)"); // Above 3

    // Non-numeric quartile
    model._set("A5", "=QUARTILE.INC(B1:B2,\"text\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "#NUM!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
}

#[test]
fn test_quartile_invalid_data_ranges() {
    let mut model = new_empty_model();

    // Empty range
    model._set("A1", "=QUARTILE.INC(B1:B3,1)"); // Empty range

    // Text-only range
    model._set("C1", "text");
    model._set("A2", "=QUARTILE.INC(C1,1)"); // Text-only
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
}

#[test]
fn test_quartile_error_propagation() {
    let mut model = new_empty_model();

    // Error propagation from cell references
    model._set("E1", "=1/0");
    model._set("E2", "2");
    model._set("A1", "=QUARTILE.INC(E1:E2,1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#VALUE!");
}
