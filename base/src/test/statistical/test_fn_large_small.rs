#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn large_and_small_over_a_range() {
    let mut model = new_empty_model();
    model._set("D1", "5");
    model._set("D2", "text");
    model._set("D4", "1");
    model._set("D5", "3");
    model._set("A1", "=LARGE(D1:D5,1)");
    model._set("A2", "=SMALL(D1:D5,2)");
    model._set("A3", "=LARGE(D1:D5,4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "#NUM!");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole column is read only as far as the sheet's used area: the cells past
// it are empty, and LARGE and SMALL skip empty cells.
#[test]
fn whole_column() {
    let mut model = new_empty_model();
    model._set("D1", "5");
    model._set("D2", "text");
    model._set("D4", "1");
    model._set("D5", "3");
    model._set("A1", "=LARGE(D:D,1)");
    model._set("A2", "=SMALL(D:D,2)");
    model._set("A3", "=LARGE(D:D,4)");
    model._set("A4", "=SMALL(7:7,1)");
    model._set("B7", "7");
    model._set("C7", "9");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "7");
}

// The skipped part is on record, so a spill into it restarts the pass and the
// second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,LARGE(D:D,1))");
    model._set("B1", "=SEQUENCE(1,1,SMALL(D:D,5))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("B1"), "5");
    assert!(model.evaluation.restarts_in_last_evaluation >= 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}
