#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_hlookup_range_sorted() {
    let mut model = new_empty_model();
    // First row is the (sorted) search row, second row holds the results.
    model._set("A1", "10");
    model._set("B1", "20");
    model._set("C1", "30");
    model._set("A2", "a");
    model._set("B2", "b");
    model._set("C2", "c");
    model._set("E1", "=HLOOKUP(20,A1:C2,2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "b");
}

#[test]
fn test_hlookup_range_exact() {
    let mut model = new_empty_model();
    model._set("A1", "banana");
    model._set("B1", "apple");
    model._set("C1", "cherry");
    model._set("A2", "1");
    model._set("B2", "2");
    model._set("C2", "3");
    model._set("E1", r#"=HLOOKUP("apple",A1:C2,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
}

#[test]
fn test_hlookup_array_sorted() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(20,{10,20,30;\"a\",\"b\",\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_hlookup_array_exact() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=HLOOKUP("banana",{"apple","banana","cherry";1,2,3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_hlookup_array_wildcard() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=HLOOKUP("ban*",{"apple","banana","cherry";1,2,3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_hlookup_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(99,{10,20,30;\"a\",\"b\",\"c\"},2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_hlookup_array_row_index_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=HLOOKUP(20,{10,20,30;\"a\",\"b\",\"c\"},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn test_hlookup_array_sorted_largest_smaller() {
    let mut model = new_empty_model();
    // 25 is not present, sorted search returns the largest value <= 25, i.e. 20.
    model._set("A1", "=HLOOKUP(25,{10,20,30;\"a\",\"b\",\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_hlookup_row_index_below_one() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "one");
    model._set("C1", "10");
    model._set("A2", "2");
    model._set("B2", "two");
    model._set("C2", "20");
    model._set("E1", "=HLOOKUP(1,A1:C2,0,FALSE)");
    model._set("E2", "=HLOOKUP(1,A1:C2,-1,FALSE)");
    model._set("E3", "=HLOOKUP(1,A1:C2,0.5,FALSE)");
    model._set("E4", "=HLOOKUP(1,A1:C2,3,FALSE)");
    model._set("E5", "=HLOOKUP(1,A1:C2,1.9,FALSE)");
    model._set("E6", "=HLOOKUP(\"one\",A1:C2,2.5,FALSE)");
    model._set("E7", "=HLOOKUP(10,A1:C2,TRUE,FALSE)");
    model.evaluate();
    // Below 1 after truncation is #VALUE!, past the table is #REF!
    assert_eq!(model._get_text("E1"), "#VALUE!");
    assert_eq!(model._get_text("E2"), "#VALUE!");
    assert_eq!(model._get_text("E3"), "#VALUE!");
    assert_eq!(model._get_text("E4"), "#REF!");
    assert_eq!(model._get_text("E5"), "1");
    assert_eq!(model._get_text("E6"), "two");
    assert_eq!(model._get_text("E7"), "10");
}

// A whole row is read only as far as the sheet's used area; the cells past it
// are still searched, as empty cells.
#[test]
fn test_hlookup_whole_row() {
    let mut model = new_empty_model();
    model._set("B2", "10");
    model._set("C2", "20");
    model._set("D2", "30");
    model._set("B3", "a");
    model._set("C3", "b");
    model._set("D3", "c");
    model._set("A5", "=HLOOKUP(20,2:3,2,FALSE)");
    model._set("A6", "=HLOOKUP(25,2:3,2,FALSE)");
    model._set("A7", "=HLOOKUP(\"x\",2:3,2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "b");
    assert_eq!(model._get_text("A6"), "#N/A");
    assert_eq!(model._get_text("A7"), "#N/A");
}
