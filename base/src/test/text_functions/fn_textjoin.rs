#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE, A1:A3)");
    model._set("B2", "=TEXTJOIN(\" and \", TRUE, A1:A3)");
    // This formula might have the _xlfn. prefix
    model._set("B3", "=_xlfn.TEXTJOIN(\" or \", , A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Monday, Tuesday, Wednesday");
    assert_eq!(model._get_text("B2"), *"Monday and Tuesday and Wednesday");
    assert_eq!(model._get_text("B3"), *"Monday or Tuesday or Wednesday");
    // Our text version removes the prefix, of course (and some white spaces)
    assert_eq!(model._get_formula("B3"), *"=TEXTJOIN(\" or \",,A1:A3)");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE)");
    model._set("B2", "=TEXTJOIN(\" and \", A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,LEN(TEXTJOIN(\",\",TRUE,D:D)))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

#[test]
fn textjoin_arrays() {
    let mut model = new_empty_model();
    for (i, code) in [72, 101, 108, 108, 111].iter().enumerate() {
        model._set(&format!("A{}", i + 1), &code.to_string());
    }
    // A spilled function result
    model._set("B1", "=TEXTJOIN(\"\", TRUE, CHAR(A1:A5))");
    // An inline array, joined row by row, mixing numbers, booleans and texts
    model._set("B2", "=TEXTJOIN(\"-\", TRUE, {1,\"a\";TRUE,2.5})");
    // Arrays mixed with scalars and ranges
    model._set("B3", "=TEXTJOIN(\",\", TRUE, \"x\", {1,2}, A1:A2)");
    // Empty elements come from empty cells in a range turned into an array
    model._set("B4", "=TEXTJOIN(\",\", FALSE, UPPER(D1:D3))");
    model._set("B5", "=TEXTJOIN(\",\", TRUE, UPPER(D1:D3))");
    // An error element is the error of the function
    model._set("B6", "=TEXTJOIN(\",\", TRUE, CHAR({65,0}))");
    // ignore_empty also skips empty texts, scalar or in a range
    model._set("B7", "=TEXTJOIN(\",\", TRUE, \"a\", \"\", \"b\")");
    model._set("B8", "=TEXTJOIN(\",\", FALSE, \"a\", \"\", \"b\")");
    model._set("B9", "=TEXTJOIN(\",\", TRUE, IF(A1:A5>100, A1:A5, \"\"))");
    model._set("D2", "b");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Hello");
    assert_eq!(model._get_text("B2"), *"1-a-TRUE-2.5");
    assert_eq!(model._get_text("B3"), *"x,1,2,72,101");
    assert_eq!(model._get_text("B4"), *",B,");
    assert_eq!(model._get_text("B5"), *"B");
    assert_eq!(model._get_text("B6"), *"#VALUE!");
    assert_eq!(model._get_text("B7"), *"a,b");
    assert_eq!(model._get_text("B8"), *"a,,b");
    assert_eq!(model._get_text("B9"), *"101,108,108,111");
}
