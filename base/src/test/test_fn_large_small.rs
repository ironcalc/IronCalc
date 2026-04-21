#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_large_small_wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=LARGE()");
    model._set("A2", "=LARGE(B1:B5)");
    model._set("A3", "=SMALL()");
    model._set("A4", "=SMALL(B1:B5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_large_small_basic_functionality() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "7");
    model._set("B5", "9");
    model._set("A1", "=LARGE(B1:B5,2)");
    model._set("A2", "=SMALL(B1:B5,3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"7");
    assert_eq!(model._get_text("A2"), *"5");
}

#[test]
fn test_fn_large_small_k_equals_zero() {
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("A1", "=LARGE(B1:B2,0)");
    model._set("A2", "=SMALL(B1:B2,0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
}

#[test]
fn test_fn_large_small_k_less_than_one() {
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");

    // Test k < 1 values (should all return #NUM! error)
    model._set("A1", "=LARGE(B1:B3,-1)");
    model._set("A2", "=SMALL(B1:B3,-0.5)");
    model._set("A3", "=LARGE(B1:B3,0.9)");
    model._set("A4", "=SMALL(B1:B3,0)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "#NUM!");
}

#[test]
fn test_fn_large_small_fractional_k() {
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");
    model._set("A1", "=LARGE(B1:B3,2.7)");
    model._set("A2", "=SMALL(B1:B3,1.9)");
    model._set("A3", "=LARGE(B1:B3,2.0)");
    model._set("A4", "=SMALL(B1:B3,3.0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"20"); // truncated to k=2
    assert_eq!(model._get_text("A2"), *"10"); // truncated to k=1
    assert_eq!(model._get_text("A3"), *"20"); // exact integer
    assert_eq!(model._get_text("A4"), *"30"); // exact integer
}

#[test]
fn test_fn_large_small_k_boundary_values() {
    let mut model = new_empty_model();
    model._set("B1", "10");
    model._set("B2", "20");
    model._set("B3", "30");

    model._set("A1", "=LARGE(B1:B3,1)"); // k=1
    model._set("A2", "=SMALL(B1:B3,1)"); // k=1
    model._set("A3", "=LARGE(B1:B3,3)"); // k=array size
    model._set("A4", "=SMALL(B1:B3,3)"); // k=array size
    model._set("A5", "=LARGE(B1:B3,4)"); // k > array size
    model._set("A6", "=SMALL(B1:B3,4)"); // k > array size
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"30"); // largest
    assert_eq!(model._get_text("A2"), *"10"); // smallest
    assert_eq!(model._get_text("A3"), *"10"); // 3rd largest = smallest
    assert_eq!(model._get_text("A4"), *"30"); // 3rd smallest = largest
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

#[test]
fn test_fn_large_small_empty_range() {
    let mut model = new_empty_model();
    model._set("A1", "=LARGE(B1:B3,1)");
    model._set("A2", "=SMALL(B1:B3,1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
}

#[test]
fn test_fn_large_small_no_numeric_values() {
    let mut model = new_empty_model();
    model._set("B1", "Text");
    model._set("B2", "TRUE");
    model._set("B3", "");
    model._set("A1", "=LARGE(B1:B3,1)");
    model._set("A2", "=SMALL(B1:B3,1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
}

#[test]
fn test_fn_large_small_mixed_data_types() {
    let mut model = new_empty_model();
    model._set("B1", "100");
    model._set("B2", "Text");
    model._set("B3", "50");
    model._set("B4", "TRUE");
    model._set("B5", "25");
    model._set("A1", "=LARGE(B1:B5,1)");
    model._set("A2", "=LARGE(B1:B5,3)");
    model._set("A3", "=SMALL(B1:B5,1)");
    model._set("A4", "=SMALL(B1:B5,3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"100");
    assert_eq!(model._get_text("A2"), *"25");
    assert_eq!(model._get_text("A3"), *"25");
    assert_eq!(model._get_text("A4"), *"100");
}

#[test]
fn test_fn_large_small_single_cell() {
    let mut model = new_empty_model();
    model._set("B1", "42");
    model._set("A1", "=LARGE(B1,1)");
    model._set("A2", "=SMALL(B1,1)");
    model._set("A3", "=LARGE(B1,2)");
    model._set("A4", "=SMALL(B1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"42");
    assert_eq!(model._get_text("A2"), *"42");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn test_fn_large_small_duplicate_values() {
    let mut model = new_empty_model();
    model._set("B1", "30");
    model._set("B2", "10");
    model._set("B3", "30");
    model._set("B4", "20");
    model._set("B5", "10");
    model._set("A1", "=LARGE(B1:B5,1)");
    model._set("A2", "=LARGE(B1:B5,2)");
    model._set("A3", "=SMALL(B1:B5,1)");
    model._set("A4", "=SMALL(B1:B5,5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"30");
    assert_eq!(model._get_text("A2"), *"30");
    assert_eq!(model._get_text("A3"), *"10");
    assert_eq!(model._get_text("A4"), *"30");
}

#[test]
fn test_fn_large_small_error_propagation() {
    let mut model = new_empty_model();

    // Error in data range
    model._set("B1", "10");
    model._set("B2", "=1/0");
    model._set("B3", "30");
    model._set("A1", "=LARGE(B1:B3,1)");
    model._set("A2", "=SMALL(B1:B3,1)");

    // Error in k parameter
    model._set("C1", "20");
    model._set("C2", "40");
    model._set("A3", "=LARGE(C1:C2,1/0)");
    model._set("A4", "=SMALL(C1:C2,1/0)");

    model.evaluate();

    assert!(model._get_text("A1").contains("#"));
    assert!(model._get_text("A2").contains("#"));
    assert!(model._get_text("A3").contains("#"));
    assert!(model._get_text("A4").contains("#"));
}
