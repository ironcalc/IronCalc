#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── SORT ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_single_column_ascending() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "1");
    model._set("A3", "2");
    model._set("B1", "=SORT(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "3");
}

#[test]
fn sort_single_column_descending() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("A2", "1");
    model._set("A3", "2");
    model._set("B1", "=SORT(A1:A3,1,-1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "1");
}

#[test]
fn sort_multi_column_by_second_col() {
    let mut model = new_empty_model();
    // Col A: names, Col B: scores
    model._set("A1", "Charlie");
    model._set("B1", "3");
    model._set("A2", "Alice");
    model._set("B2", "1");
    model._set("A3", "Bob");
    model._set("B3", "2");
    model._set("C1", "=SORT(A1:B3,2,1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Alice");
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("C2"), "Bob");
    assert_eq!(model._get_text("D2"), "2");
    assert_eq!(model._get_text("C3"), "Charlie");
    assert_eq!(model._get_text("D3"), "3");
}

#[test]
fn sort_strings_ascending() {
    let mut model = new_empty_model();
    model._set("A1", "banana");
    model._set("A2", "apple");
    model._set("A3", "cherry");
    model._set("B1", "=SORT(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "apple");
    assert_eq!(model._get_text("B2"), "banana");
    assert_eq!(model._get_text("B3"), "cherry");
}

#[test]
fn sort_by_col_true() {
    let mut model = new_empty_model();
    // A row of numbers to sort horizontally
    model._set("A1", "3");
    model._set("B1", "1");
    model._set("C1", "2");
    model._set("A3", "=SORT(A1:C1,1,1,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("B3"), "2");
    assert_eq!(model._get_text("C3"), "3");
}

#[test]
fn sort_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SORT()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn sort_invalid_sort_order() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("B1", "=SORT(A1:A2,1,2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "#VALUE!");
}

// ── SORTBY ────────────────────────────────────────────────────────────────────

#[test]
fn sortby_basic() {
    let mut model = new_empty_model();
    model._set("A1", "Charlie");
    model._set("A2", "Alice");
    model._set("A3", "Bob");
    model._set("B1", "3");
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("C1", "=SORTBY(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Alice");
    assert_eq!(model._get_text("C2"), "Bob");
    assert_eq!(model._get_text("C3"), "Charlie");
}

#[test]
fn sortby_descending() {
    let mut model = new_empty_model();
    model._set("A1", "Charlie");
    model._set("A2", "Alice");
    model._set("A3", "Bob");
    model._set("B1", "3");
    model._set("B2", "1");
    model._set("B3", "2");
    model._set("C1", "=SORTBY(A1:A3,B1:B3,-1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Charlie");
    assert_eq!(model._get_text("C2"), "Bob");
    assert_eq!(model._get_text("C3"), "Alice");
}

#[test]
fn sortby_two_keys() {
    let mut model = new_empty_model();
    // Rows: (Dept, Score) — sort by Dept asc, then Score desc
    model._set("A1", "B");
    model._set("B1", "2");
    model._set("A2", "A");
    model._set("B2", "1");
    model._set("A3", "A");
    model._set("B3", "3");
    model._set("C1", "=SORTBY(A1:B3,A1:A3,1,B1:B3,-1)");
    model.evaluate();
    // First: Dept A with score 3
    assert_eq!(model._get_text("C1"), "A");
    assert_eq!(model._get_text("D1"), "3");
    // Second: Dept A with score 1
    assert_eq!(model._get_text("C2"), "A");
    assert_eq!(model._get_text("D2"), "1");
    // Third: Dept B with score 2
    assert_eq!(model._get_text("C3"), "B");
    assert_eq!(model._get_text("D3"), "2");
}

#[test]
fn sortby_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SORTBY(A2:A3,B2:B3,1,C2:C3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── UNIQUE ────────────────────────────────────────────────────────────────────

#[test]
fn unique_basic() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("A3", "apple");
    model._set("A4", "cherry");
    model._set("B1", "=UNIQUE(A1:A4)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "apple");
    assert_eq!(model._get_text("B2"), "banana");
    assert_eq!(model._get_text("B3"), "cherry");
}

#[test]
fn unique_exactly_once() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("A3", "apple");
    model._set("A4", "cherry");
    model._set("B1", "=UNIQUE(A1:A4,FALSE,TRUE)");
    model.evaluate();
    // Only banana and cherry appear once
    assert_eq!(model._get_text("B1"), "banana");
    assert_eq!(model._get_text("B2"), "cherry");
}

#[test]
fn unique_multi_column_rows() {
    let mut model = new_empty_model();
    model._set("A1", "Alice");
    model._set("B1", "1");
    model._set("A2", "Bob");
    model._set("B2", "2");
    model._set("A3", "Alice");
    model._set("B3", "1");
    model._set("C1", "=UNIQUE(A1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Alice");
    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("C2"), "Bob");
    assert_eq!(model._get_text("D2"), "2");
}

#[test]
fn unique_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    model._set("A2", "3");
    model._set("A3", "5");
    model._set("A4", "7");
    model._set("A5", "3");
    model._set("B1", "=UNIQUE(A1:A5)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "5");
    assert_eq!(model._get_text("B2"), "3");
    assert_eq!(model._get_text("B3"), "7");
}

#[test]
fn unique_by_col() {
    let mut model = new_empty_model();
    // A row where some columns are duplicates
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "1");
    model._set("D1", "3");
    model._set("A3", "=UNIQUE(A1:D1,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A3"), "1");
    assert_eq!(model._get_text("B3"), "2");
    assert_eq!(model._get_text("C3"), "3");
}

// ── FILTER ────────────────────────────────────────────────────────────────────

#[test]
fn filter_basic() {
    let mut model = new_empty_model();
    model._set("A1", "Alice");
    model._set("B1", "TRUE");
    model._set("A2", "Bob");
    model._set("B2", "FALSE");
    model._set("A3", "Charlie");
    model._set("B3", "TRUE");
    model._set("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "Alice");
    assert_eq!(model._get_text("C2"), "Charlie");
}

#[test]
fn filter_with_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("B1", "1");
    model._set("A2", "20");
    model._set("B2", "0");
    model._set("A3", "30");
    model._set("B3", "1");
    model._set("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "10");
    assert_eq!(model._get_text("C2"), "30");
}

#[test]
fn filter_no_match_returns_calc_error() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "FALSE");
    model._set("C1", "=FILTER(A1:A1,B1:B1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#CALC!");
}

#[test]
fn filter_no_match_with_if_empty() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "FALSE");
    model._set("C1", "=FILTER(A1:A1,B1:B1,\"No results\")");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "No results");
}

#[test]
fn filter_multi_column() {
    let mut model = new_empty_model();
    model._set("A1", "Alice");
    model._set("B1", "100");
    model._set("C1", "TRUE");
    model._set("A2", "Bob");
    model._set("B2", "200");
    model._set("C2", "FALSE");
    model._set("A3", "Charlie");
    model._set("B3", "300");
    model._set("C3", "TRUE");
    model._set("D1", "=FILTER(A1:B3,C1:C3)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "Alice");
    assert_eq!(model._get_text("E1"), "100");
    assert_eq!(model._get_text("D2"), "Charlie");
    assert_eq!(model._get_text("E2"), "300");
}

#[test]
fn filter_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=FILTER(A2:A3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

// ── Combination tests ─────────────────────────────────────────────────────────

#[test]
fn sort_after_filter() {
    let mut model = new_empty_model();
    model._set("A1", "Charlie");
    model._set("B1", "TRUE");
    model._set("A2", "Alice");
    model._set("B2", "TRUE");
    model._set("A3", "Bob");
    model._set("B3", "FALSE");
    // FILTER first, then SORT the result using a helper: place FILTER in a helper col
    // (direct nesting not yet tested — use separate cells)
    model._set("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    // C1=Charlie, C2=Alice (filtered)
    model._set("D1", "=SORT(C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "Alice");
    assert_eq!(model._get_text("D2"), "Charlie");
}
