#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_match_array_row_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(69,{24,43.5,52.8,69,269,387,770},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
}

#[test]
fn test_match_array_column_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(52.8,{24;43.5;52.8;69;269;387;770},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_match_array_exact_string() {
    let mut model = new_empty_model();
    model._set("A1", r#"=MATCH("banana",{"apple","banana","cherry"},0)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_match_array_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(100,{24,43.5,52.8,69},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
}

#[test]
fn test_match_array_ascending_default() {
    let mut model = new_empty_model();
    // match_type 1 (default): largest value <= target in an ascending array
    model._set("A1", "=MATCH(70,{24,43.5,52.8,69,269,387,770})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4");
}

#[test]
fn test_match_array_descending() {
    let mut model = new_empty_model();
    // match_type -1: smallest value >= target in a descending array.
    // Values >= 70 are 770, 387, 269; the smallest of those is 269 at position 3.
    model._set("A1", "=MATCH(70,{770,387,269,69,52.8,43.5,24},-1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_match_array_wildcard() {
    let mut model = new_empty_model();
    model._set("A1", r#"=MATCH("ban*",{"apple","banana","cherry"},0)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_match_array_not_a_vector() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(1,{1,2;3,4},0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_match_approximate_skips_trailing_blanks() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("A4", "40");
    model._set("A5", "50");
    // A6:A10 left blank
    model._set("B1", "=MATCH(35,A1:A10,1)");
    model._set("B2", "=MATCH(35,A1:A5,1)");
    model._set("B3", "=INDEX(A1:A10,MATCH(35,A1:A10,1))");
    model._set("B4", "=MATCH(999,A1:A10,1)");
    model._set("B5", "=MATCH(5,A1:A10,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "30");
    assert_eq!(model._get_text("B4"), "5");
    assert_eq!(model._get_text("B5"), "#N/A");
}

#[test]
fn test_match_approximate_all_blank_range() {
    let mut model = new_empty_model();
    model._set("B1", "=MATCH(35,A1:A10,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#N/A");
}

#[test]
fn test_match_approximate_interior_blank() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    // A3 blank
    model._set("A4", "30");
    model._set("A5", "40");
    model._set("B1", "=MATCH(35,A1:A5,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "4");
}

#[test]
fn test_match_descending_trailing_blanks() {
    let mut model = new_empty_model();
    model._set("A1", "50");
    model._set("A2", "40");
    model._set("A3", "30");
    model._set("A4", "20");
    model._set("A5", "10");
    // A6:A10 left blank
    model._set("B1", "=MATCH(35,A1:A10,-1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2");
}

#[test]
fn test_match_approximate_numeric_key_skips_text() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("A4", "alpha");
    model._set("A5", "beta");
    // A6:A7 left blank
    model._set("B1", "=MATCH(999,A1:A7,1)");
    model._set("B2", "=MATCH(35,A1:A7,1)");
    model.evaluate();
    // text entries and blanks are invisible to a numeric key
    assert_eq!(model._get_text("B1"), "3");
    assert_eq!(model._get_text("B2"), "3");
}

#[test]
fn test_match_approximate_text_key_skips_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "bravo");
    model._set("A4", "charlie");
    // text key sorting below all text entries: numbers are invisible
    model._set("B1", r#"=MATCH("alpha",A1:A4,1)"#);
    model._set("B2", r#"=MATCH("bravo",A1:A4,1)"#);
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#N/A");
    assert_eq!(model._get_text("B2"), "3");
}

#[test]
fn test_match_approximate_all_mismatched_types() {
    let mut model = new_empty_model();
    model._set("A1", "alpha");
    model._set("A2", "beta");
    model._set("B1", "=MATCH(35,A1:A2,1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#N/A");
}
