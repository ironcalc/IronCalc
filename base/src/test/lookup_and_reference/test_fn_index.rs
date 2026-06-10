#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── INDEX with inline arrays ──────────────────────────────────────────────────

#[test]
fn test_index_array_row_vector() {
    let mut model = new_empty_model();
    // A 1×4 row vector: the single index is a column index -> scalar.
    model._set("A1", "=INDEX({1,2,3,4},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}

#[test]
fn test_index_array_column_vector() {
    let mut model = new_empty_model();
    // A 4×1 column vector: the single index is a row index -> scalar.
    model._set("A1", "=INDEX({1;2;3;4},3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_index_array_whole_row() {
    let mut model = new_empty_model();
    // A 2×2 array: a single index selects an entire row -> array (spills).
    model._set("A1", "=INDEX({1,2;3,4},1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
}

#[test]
fn test_index_array_whole_row_second() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1,2;3,4},2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("B1"), "4");
}

#[test]
fn test_index_array_row_and_col() {
    let mut model = new_empty_model();
    // Both row_num and col_num given -> single cell.
    model._set("A1", "=INDEX({1,2;3,4},1,1)");
    model._set("A2", "=INDEX({1,2;3,4},2,1)");
    model._set("A3", "=INDEX({1,2;3,4},2,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "4");
}

#[test]
fn test_index_array_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1,2,3,4},9)");
    model._set("A2", "=INDEX({1,2;3,4},3,1)");
    model._set("A3", "=INDEX({1,2;3,4},1,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#REF!");
    assert_eq!(model._get_text("A2"), "#REF!");
    assert_eq!(model._get_text("A3"), "#REF!");
}

#[test]
fn test_index_array_strings_and_booleans() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({\"a\",\"b\";TRUE,FALSE},2,1)");
    model._set("A2", "=INDEX({\"a\",\"b\";TRUE,FALSE},1,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "b");
}
