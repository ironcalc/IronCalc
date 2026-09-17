use crate::functions::binary_search::*;

#[test]
fn test_binary_search() {
    let t = vec![1, 2, 3, 40, 55, 155];
    assert_eq!(binary_search_or_smaller(&40, &t), Some(3));
    assert_eq!(binary_search_or_greater(&40, &t), Some(3));
    assert_eq!(binary_search_or_smaller(&45, &t), Some(3));
    assert_eq!(binary_search_or_greater(&45, &t), Some(4));
}

#[test]
fn test_binary_search_descending() {
    let t = vec![100, 33, 23, 14, 5, -155];
    assert_eq!(binary_search_descending_or_smaller(&23, &t), Some(2));
    assert_eq!(binary_search_descending_or_greater(&23, &t), Some(2));
    assert_eq!(binary_search_descending_or_smaller(&25, &t), Some(2));
    assert_eq!(binary_search_descending_or_greater(&25, &t), Some(1));
}

#[test]
fn test_binary_search_multiple() {
    let t = vec![1, 2, 3, 40, 40, 40, 40, 55, 155];
    assert_eq!(binary_search_or_smaller(&40, &t), Some(3));
    assert_eq!(binary_search_or_smaller(&39, &t), Some(2));
    assert_eq!(binary_search_or_greater(&40, &t), Some(3));
    assert_eq!(binary_search_or_greater(&41, &t), Some(7));
}

// --- Legacy approximate lookups skip blanks and type-mismatched entries ---

use crate::calc_result::CalcResult;

fn num(n: f64) -> CalcResult {
    CalcResult::Number(n)
}

fn text(s: &str) -> CalcResult {
    CalcResult::String(s.to_string())
}

fn ascending_with_trailing_blanks() -> Vec<CalcResult> {
    vec![
        num(10.0),
        num(20.0),
        num(30.0),
        num(40.0),
        num(50.0),
        CalcResult::EmptyCell,
        CalcResult::EmptyCell,
        CalcResult::EmptyCell,
        CalcResult::EmptyCell,
        CalcResult::EmptyCell,
    ]
}

#[test]
fn test_binary_search_on_array_trailing_blanks() {
    let t = ascending_with_trailing_blanks();
    // key mid-range -> largest smaller-or-equal non-blank
    assert_eq!(binary_search_on_array(&num(35.0), &t), 2);
    assert_eq!(binary_search_on_array(&num(30.0), &t), 2);
    // key above all -> last non-blank
    assert_eq!(binary_search_on_array(&num(999.0), &t), 4);
    // key below all -> not found
    assert_eq!(binary_search_on_array(&num(5.0), &t), -2);
}

#[test]
fn test_binary_search_on_array_all_blank() {
    let t = vec![CalcResult::EmptyCell; 5];
    assert_eq!(binary_search_on_array(&num(35.0), &t), -2);
}

#[test]
fn test_binary_search_on_array_interior_blank() {
    let t = vec![
        num(10.0),
        num(20.0),
        CalcResult::EmptyCell,
        num(30.0),
        num(40.0),
    ];
    assert_eq!(binary_search_on_array(&num(35.0), &t), 3);
    assert_eq!(binary_search_on_array(&num(20.0), &t), 1);
}

#[test]
fn test_binary_search_on_array_numeric_key_skips_text_and_blanks() {
    // Numeric key ignores text, booleans and blanks: finds last number
    let t = vec![
        num(10.0),
        num(20.0),
        num(30.0),
        text("alpha"),
        text("beta"),
        CalcResult::Boolean(true),
        CalcResult::EmptyCell,
        CalcResult::EmptyCell,
    ];
    assert_eq!(binary_search_on_array(&num(35.0), &t), 2);
    assert_eq!(binary_search_on_array(&num(999.0), &t), 2);
    assert_eq!(binary_search_on_array(&num(5.0), &t), -2);
}

#[test]
fn test_binary_search_on_array_text_key_skips_numbers() {
    // Williams edge: numbers are invisible to a text search, so a text key
    // sorting below every text entry is not found even though numbers sort
    // below all text in the Excel ordering.
    let t = vec![
        num(10.0),
        num(20.0),
        num(30.0),
        text("bravo"),
        text("charlie"),
    ];
    assert_eq!(binary_search_on_array(&text("alpha"), &t), -2);
    assert_eq!(binary_search_on_array(&text("bravo"), &t), 3);
    assert_eq!(binary_search_on_array(&text("delta"), &t), 4);
}

#[test]
fn test_binary_search_on_array_all_mismatched_types() {
    let t = vec![
        text("alpha"),
        text("beta"),
        CalcResult::Boolean(false),
        CalcResult::EmptyCell,
    ];
    assert_eq!(binary_search_on_array(&num(1.0), &t), -2);
}
