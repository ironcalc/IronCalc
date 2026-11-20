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

// ── whole columns ────────────────────────────────────────────────────────────

// A whole column is read only as far as the sheet's used area; the rest is
// put on record as one rectangle read empty, not as one entry per cell. With a
// dynamic anchor in the workbook every read is recorded, and a million records
// for each MATCH made a sheet of a few hundred INDEX/MATCH formulas over whole
// columns take a minute to evaluate.
#[test]
fn a_whole_column_match_records_one_rectangle() {
    let mut model = new_empty_model();
    model._set("A1", "=MATCH(2,D:D,0)");
    model._set("F1", "=SEQUENCE(1)");
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("D3", "3");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert!(
        model.evaluation.seen.len() <= 4,
        "{:?}",
        model.evaluation.seen
    );
    assert_eq!(model.evaluation.seen_ranges.len(), 1);
    let range = model.evaluation.seen_ranges[0];
    assert_eq!((range.column1, range.column2), (4, 4));
    assert_eq!(range.row2, crate::constants::LAST_ROW);
}

// The cells past the used area are not read but they are still searched, as
// empty cells, so every match type answers as it did when each cell was read.
// An approximate search meets them first and runs to the last row. (Excel's
// binary search skips trailing empty cells and would answer 2 to A1; that is
// a separate difference.)
#[test]
fn a_whole_column_match_still_searches_the_cells_past_the_used_area() {
    let mut model = new_empty_model();
    model._set("D1", "10");
    model._set("D2", "20");
    model._set("D3", "30");
    model._set("A1", "=MATCH(25,D:D)");
    model._set("A2", "=MATCH(20,D:D,0)");
    model._set("A3", "=MATCH(25,D:D,0)");
    model._set("A4", "=MATCH(25,D:D,-1)");
    model._set("A5", "=MATCH(20,1:1,0)");
    model._set("E1", "20");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1048576");
    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(model._get_text("A5"), "5");
}

// The skipped part of the column is on record: a spill into it, made on demand
// while the anchor walks the column, restarts the pass.
#[test]
fn a_whole_column_match_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,MATCH(5,D:D,0))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}
