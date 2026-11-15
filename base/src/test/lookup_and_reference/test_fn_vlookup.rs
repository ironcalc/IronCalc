#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_vlookup_range_sorted() {
    let mut model = new_empty_model();
    // First column is the (sorted) search column, second column holds the results.
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("D1", "=VLOOKUP(20,A1:B3,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "b");
}

#[test]
fn test_vlookup_range_exact() {
    let mut model = new_empty_model();
    model._set("A1", "banana");
    model._set("A2", "apple");
    model._set("A3", "cherry");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", r#"=VLOOKUP("apple",A1:B3,2,FALSE)"#);
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
}

#[test]
fn test_vlookup_array_sorted() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(20,{10,\"a\";20,\"b\";30,\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_vlookup_array_exact() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=VLOOKUP("banana",{"apple",1;"banana",2;"cherry",3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_vlookup_array_wildcard() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=VLOOKUP("ban*",{"apple",1;"banana",2;"cherry",3},2,FALSE)"#,
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_vlookup_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(99,{10,\"a\";20,\"b\";30,\"c\"},2,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_vlookup_array_column_index_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=VLOOKUP(20,{10,\"a\";20,\"b\";30,\"c\"},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn test_vlookup_array_sorted_largest_smaller() {
    let mut model = new_empty_model();
    // 25 is not present, sorted search returns the largest value <= 25, i.e. 20.
    model._set("A1", "=VLOOKUP(25,{10,\"a\";20,\"b\";30,\"c\"},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "b");
}

#[test]
fn test_vlookup_approximate_trailing_blanks() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("A4", "40");
    model._set("A5", "50");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model._set("B4", "d");
    model._set("B5", "e");
    // A6:A10 left blank
    model._set("C1", "=VLOOKUP(35,A1:B10,2,TRUE)");
    model._set("C2", "=VLOOKUP(5,A1:B10,2,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "c");
    assert_eq!(model._get_text("C2"), "#N/A");
}
