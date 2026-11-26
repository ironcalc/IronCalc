#![allow(clippy::unwrap_used)]

//! Deleting rows or columns at the start or the end of a range shrinks the
//! range to what is left of it; only a range deleted entirely becomes `#REF!`.
//! This is what Excel does: with `=SUM(B2:D2)`, deleting a column of the range
//! adjusts the range ("How to correct a #REF! error", Microsoft Support).
//! LibreOffice, Apache POI's `FormulaShifter` and HyperFormula do the same.

use crate::test::util::new_empty_model;

#[test]
fn deleting_rows() {
    // formula, (first row deleted, how many), formula after
    let cases = [
        ("=SUM(A5:A8)", (5, 1), "=SUM(A5:A7)"),
        ("=SUM(A5:A8)", (8, 1), "=SUM(A5:A7)"),
        ("=SUM(A5:A8)", (6, 2), "=SUM(A5:A6)"),
        ("=SUM(A5:A8)", (5, 4), "=SUM(#REF!)"),
        ("=SUM(A5:A8)", (3, 4), "=SUM(A3:A4)"),
        ("=SUM(A5:A8)", (7, 4), "=SUM(A5:A6)"),
        ("=SUM(A5:A8)", (4, 6), "=SUM(#REF!)"),
        ("=SUM(A5:A8)", (2, 2), "=SUM(A3:A6)"),
        ("=SUM(A5:A8)", (10, 1), "=SUM(A5:A8)"),
        ("=SUM(A5:B8)", (8, 1), "=SUM(A5:B7)"),
        ("=SUM($A$5:$A$8)", (5, 1), "=SUM($A$5:$A$7)"),
        ("=SUM(A$5:A8)", (8, 1), "=SUM(A$5:A7)"),
        ("=SUM($A5:A$8)", (5, 1), "=SUM($A5:A$7)"),
        ("=SUM(5:8)", (5, 1), "=SUM(5:7)"),
        ("=SUM(5:8)", (5, 4), "=SUM(#REF!)"),
        ("=SUM(A:A)", (5, 1), "=SUM(A:A)"),
        ("=A5*2", (5, 1), "=#REF!*2"),
        ("=SUM(Sheet2!A5:A8)", (5, 1), "=SUM(Sheet2!A5:A8)"),
    ];
    for (formula, (row, count), expected) in cases {
        // One formula above the deleted rows (F1), one below them (F20, which moves)
        let mut model = new_empty_model();
        model.new_sheet();
        for r in 1..=12 {
            model._set(&format!("A{r}"), &format!("{r}"));
        }
        model._set("F1", formula);
        model._set("F20", formula);
        model.evaluate();
        model.delete_rows(0, row, count).unwrap();
        model.evaluate();
        let case = format!("'{formula}', {count} rows deleted from row {row}");
        assert_eq!(model._get_formula("F1"), *expected, "{case}");
        let moved = format!("F{}", 20 - count);
        assert_eq!(model._get_formula(&moved), *expected, "{case} (moved)");
    }
}

#[test]
fn deleting_columns() {
    // formula, (first column deleted, how many), formula after
    let cases = [
        ("=SUM(B1:E1)", (2, 1), "=SUM(B1:D1)"),
        ("=SUM(B1:E1)", (5, 1), "=SUM(B1:D1)"),
        ("=SUM(B1:E1)", (3, 2), "=SUM(B1:C1)"),
        ("=SUM(B1:E1)", (2, 4), "=SUM(#REF!)"),
        ("=SUM(B1:E1)", (4, 4), "=SUM(B1:C1)"),
        ("=SUM($B$1:$E$1)", (2, 1), "=SUM($B$1:$D$1)"),
        ("=SUM(B:E)", (5, 1), "=SUM(B:D)"),
        ("=SUM(1:1)", (2, 1), "=SUM(1:1)"),
    ];
    for (formula, (column, count), expected) in cases {
        // One formula left of the deleted columns (A9), one right of them (K9, which moves)
        let mut model = new_empty_model();
        model._set("A9", formula);
        model._set("K9", formula);
        model.evaluate();
        model.delete_columns(0, column, count).unwrap();
        model.evaluate();
        let case = format!("'{formula}', {count} columns deleted from column {column}");
        assert_eq!(model._get_formula("A9"), *expected, "{case}");
        let moved = crate::expressions::utils::number_to_column(11 - count).unwrap();
        assert_eq!(
            model._get_formula(&format!("{moved}9")),
            *expected,
            "{case} (moved)"
        );
    }
}

#[test]
fn the_shrunk_range_is_evaluated() {
    let mut model = new_empty_model();
    for r in 1..=8 {
        model._set(&format!("A{r}"), &format!("{r}"));
    }
    model._set("C1", "=SUM(A5:A8)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "26");
    // Row 5 deleted: rows 6, 7 and 8 are left
    model.delete_rows(0, 5, 1).unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("C1"), "=SUM(A5:A7)");
    assert_eq!(model._get_text("C1"), "21");
}

#[test]
fn deleting_rows_on_another_sheet() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("B2", "=SUM(Sheet2!A5:A8)+SUM(A5:A8)");
    model.evaluate();
    model.delete_rows(1, 5, 1).unwrap();
    assert_eq!(model._get_formula("B2"), "=SUM(Sheet2!A5:A7)+SUM(A5:A8)");
    model.delete_rows(1, 5, 3).unwrap();
    assert_eq!(model._get_formula("B2"), "=SUM(#REF!)+SUM(A5:A8)");
}
