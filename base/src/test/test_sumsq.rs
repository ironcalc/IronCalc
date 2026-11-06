#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMSQ()");
    model._set("A2", "=SUMSQ(2)");
    model._set("A3", "=SUMSQ(1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"4");
    assert_eq!(model._get_text("A3"), *"5");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,SUMSQ(D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "55");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "55");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

// ── text and booleans typed into the argument list are counted, as in Excel ─────
// Excel: SUMSQ("2","3",4) is 29 (Apache POI's FormulaEvalTestData_Copy.xlsx,
// EverythingTests!N1332, saved by Excel). Values read from cells are left out.
#[test]
fn typed_text_and_booleans() {
    let mut model = new_empty_model();
    model._set("B1", "'2");
    model._set("B2", "TRUE");
    model._set("A1", r#"=SUMSQ("2","3",4)"#);
    model._set("A2", r#"=SUMSQ(B1,B2,4)"#);
    model._set("A3", r#"=SUMSQ(B1:B2,4)"#);
    model._set("A4", r#"=SUMSQ("two",4)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"29");
    assert_eq!(model._get_text("A2"), *"16");
    assert_eq!(model._get_text("A3"), *"16");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
}
