#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_average_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=AVERAGE()");
    model._set("A2", "=AVERAGEA()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_fn_average_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "'2");
    // B5 is empty
    model._set("B6", "true");
    model._set("A1", "=AVERAGE(B1:B6)");
    model._set("A2", "=AVERAGEA(B1:B6)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"1.4");
}

// ── whole columns ────────────────────────────────────────────────────────────

// A whole column is walked only as far as the sheet's used area: the cells
// past it are empty, and these functions skip empty cells. AVERAGEA, MAXA and
// MINA count text as 0.
#[test]
fn whole_column() {
    let mut model = new_empty_model();
    model._set("D1", "5");
    model._set("D2", "text");
    model._set("D4", "-2");
    model._set("D5", "3");
    model._set("A1", "=AVERAGE(D:D)");
    model._set("A2", "=AVERAGEA(D:D)");
    model._set("A3", "=MAXA(D:D)");
    model._set("A4", "=MINA(D:D)");
    model._set("A5", "=MEDIAN(D:D)");
    model._set("A6", "=GEOMEAN(E:E)");
    model._set("E1", "2");
    model._set("E2", "8");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "1.5");
    assert_eq!(model._get_text("A3"), "5");
    assert_eq!(model._get_text("A4"), "-2");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "4");
}

#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,AVERAGE(D:D))");
    model._set("B1", "=SEQUENCE(1,1,AVERAGEA(D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("B1"), "3");
    assert!(model.evaluation.restarts_in_last_evaluation >= 1);
}
