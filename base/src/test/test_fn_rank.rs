#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

#[test]
fn test_rank_basic_functionality() {
    let mut model = new_empty_model();
    model._set("B1", "3");
    model._set("B2", "3");
    model._set("B3", "2");
    model._set("B4", "1");

    // Test basic rank calculations
    model._set("A1", "=RANK(2,B1:B4)"); // Legacy function
    model._set("A2", "=RANK.AVG(3,B1:B4)"); // Average rank for duplicates
    model._set("A3", "=RANK.EQ(3,B1:B4)"); // Equal rank for duplicates
    model._set("A4", "=RANK(3,B1:B4,1)"); // Ascending order
    model.evaluate();

    assert_eq!(model._get_text("A1"), "3"); // Descending rank of 2
    assert_eq!(model._get_text("A2"), "1.5"); // Average of ranks 1,2 for value 3
    assert_eq!(model._get_text("A3"), "1"); // Highest rank for value 3
    assert_eq!(model._get_text("A4"), "3"); // Ascending rank of 3
}

#[test]
fn test_rank_sort_order_and_duplicates() {
    let mut model = new_empty_model();
    // Data: 1, 3, 5, 7, 9 (no duplicates)
    for (i, val) in [1, 3, 5, 7, 9].iter().enumerate() {
        model._set(&format!("B{}", i + 1), &val.to_string());
    }

    // Test sort orders
    model._set("A1", "=RANK(5,B1:B5)"); // Descending (default)
    model._set("A2", "=RANK(5,B1:B5,1)"); // Ascending

    // Data with many duplicates: 1, 2, 2, 3, 3, 3, 4
    model._set("C1", "1");
    model._set("C2", "2");
    model._set("C3", "2");
    model._set("C4", "3");
    model._set("C5", "3");
    model._set("C6", "3");
    model._set("C7", "4");

    // Test duplicate handling
    model._set("A3", "=RANK.EQ(3,C1:C7)"); // Highest rank for duplicates
    model._set("A4", "=RANK.AVG(3,C1:C7)"); // Average rank for duplicates
    model._set("A5", "=RANK.AVG(2,C1:C7)"); // Average of ranks 5,6

    model.evaluate();

    assert_eq!(model._get_text("A1"), "3"); // 5 is 3rd largest
    assert_eq!(model._get_text("A2"), "3"); // 5 is 3rd smallest
    assert_eq!(model._get_text("A3"), "2"); // Highest rank for value 3
    assert_eq!(model._get_text("A4"), "3"); // Average rank for value 3: (2+3+4)/3
    assert_eq!(model._get_text("A5"), "5.5"); // Average rank for value 2: (5+6)/2
}

#[test]
fn test_rank_not_found() {
    let mut model = new_empty_model();
    model._set("B1", "3");
    model._set("B2", "2");
    model._set("B3", "1");

    // Test cases where target number is not in range
    model._set("A1", "=RANK(5,B1:B3)"); // Not in range
    model._set("A2", "=RANK.AVG(0,B1:B3)"); // Not in range
    model._set("A3", "=RANK.EQ(2.5,B1:B3)"); // Close but not exact
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#N/A");
    assert_eq!(model._get_text("A2"), "#N/A");
    assert_eq!(model._get_text("A3"), "#N/A");
}

#[test]
fn test_rank_single_element() {
    let mut model = new_empty_model();
    model._set("B1", "5");

    model._set("A1", "=RANK(5,B1)");
    model._set("A2", "=RANK.EQ(5,B1)");
    model._set("A3", "=RANK.AVG(5,B1)");
    model.evaluate();

    // All should return rank 1 for single element
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "1");
    assert_eq!(model._get_text("A3"), "1");
}

#[test]
fn test_rank_identical_values() {
    let mut model = new_empty_model();
    // All values are the same
    for i in 1..=4 {
        model._set(&format!("C{i}"), "7");
    }

    model._set("A1", "=RANK.EQ(7,C1:C4)"); // Should be rank 1
    model._set("A2", "=RANK.AVG(7,C1:C4)"); // Should be average: 2.5
    model.evaluate();

    assert_eq!(model._get_text("A1"), "1"); // All identical - highest rank
    assert_eq!(model._get_text("A2"), "2.5"); // All identical - average rank
}

#[test]
fn test_rank_mixed_data_types() {
    let mut model = new_empty_model();
    // Mixed data types (only numbers counted)
    model._set("D1", "1");
    model._set("D2", "text"); // Ignored
    model._set("D3", "3");
    model._set("D4", "TRUE"); // Ignored
    model._set("D5", "5");

    model._set("A1", "=RANK(3,D1:D5)"); // Rank in [1,3,5]
    model._set("A2", "=RANK(1,D1:D5)"); // Rank of smallest
    model.evaluate();

    assert_eq!(model._get_text("A1"), "2"); // 3 is 2nd largest in [1,3,5]
    assert_eq!(model._get_text("A2"), "3"); // 1 is smallest
}

#[test]
fn test_rank_extreme_values() {
    let mut model = new_empty_model();
    // Extreme values
    model._set("E1", "1e10");
    model._set("E2", "0");
    model._set("E3", "-1e10");

    model._set("A1", "=RANK(0,E1:E3)"); // Rank of 0
    model._set("A2", "=RANK(1e10,E1:E3)"); // Rank of largest
    model._set("A3", "=RANK(-1e10,E1:E3)"); // Rank of smallest
    model.evaluate();

    assert_eq!(model._get_text("A1"), "2"); // 0 is 2nd largest
    assert_eq!(model._get_text("A2"), "1"); // 1e10 is largest
    assert_eq!(model._get_text("A3"), "3"); // -1e10 is smallest
}

#[test]
fn test_rank_invalid_arguments() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");

    // Invalid argument count
    model._set("A1", "=RANK(1)"); // Too few
    model._set("A2", "=RANK(1,B1:B2,0,1)"); // Too many
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn test_rank_invalid_parameters() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");

    // Non-numeric search value
    model._set("A1", "=RANK(\"text\",B1:B2)");
    model._set("A2", "=RANK.EQ(TRUE,B1:B2)"); // Boolean

    // Invalid order parameter
    model._set("A3", "=RANK(2,B1:B2,\"text\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
}

#[test]
fn test_rank_invalid_data_ranges() {
    let mut model = new_empty_model();

    // Empty range
    model._set("A1", "=RANK(1,C1:C3)"); // Empty cells

    // Text-only range
    model._set("D1", "text1");
    model._set("D2", "text2");
    model._set("A2", "=RANK(1,D1:D2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
}

#[test]
fn test_rank_error_propagation() {
    let mut model = new_empty_model();

    // Error propagation from cell references
    model._set("E1", "=1/0");
    model._set("E2", "2");
    model._set("A1", "=RANK(2,E1:E2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#VALUE!");
}
