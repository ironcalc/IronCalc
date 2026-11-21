#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn smoke_test() {
    let mut model = new_empty_model();
    model._set("A1", "=STDEV.P(10, 12, 23, 23, 16, 23, 21)");
    model._set("A2", "=STDEV.S(10, 12, 23, 23, 16, 23, 21)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"5.174505793");

    assert_eq!(model._get_text("A2"), *"5.589105048");
}

#[test]
fn numbers() {
    let mut model = new_empty_model();

    model._set("A2", "24");
    model._set("A3", "25");
    model._set("A4", "27");
    model._set("A5", "23");
    model._set("A6", "45");
    model._set("A7", "23.5");
    model._set("A8", "34");
    model._set("A9", "23");
    model._set("A10", "23");
    model._set("A11", "TRUE");
    model._set("A12", "'23");
    model._set("A13", "Text");
    model._set("A14", "FALSE");
    model._set("A15", "45");

    model._set("B1", "=STDEV.P(A2:A15)");
    model._set("B2", "=STDEV.S(A2:A15)");
    model._set("B3", "=STDEVA(A2:A15)");
    model._set("B4", "=STDEVPA(A2:A15)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"8.483071378");
    assert_eq!(model._get_text("B2"), *"8.941942369");
    assert_eq!(model._get_text("B3"), *"15.499955689");
    assert_eq!(model._get_text("B4"), *"14.936131032");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [
        ("STDEV.P(D:D)", "1.414213562"),
        ("STDEV.S(D:D)", "1.58113883"),
        ("STDEVA(D:D)", "1.58113883"),
        ("STDEVPA(D:D)", "1.414213562"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model._set("A1", &format!("=SEQUENCE(1,1,{formula})"));
        model._set("C1", "=SEQUENCE(3,1,10)");
        model._set("D1", "=SEQUENCE(5)");
        model.evaluate();
        assert_eq!(model._get_text("A1"), expected, "{formula}");
        assert_eq!(model.evaluation.restarts_in_last_evaluation, 1, "{formula}");
    }
}

// Text and booleans typed into the argument list are counted, as in Excel; values read
// from cells are not. Excel: STDEV("1","2") is 0.7071067811865476 (Apache POI's
// FormulaEvalTestData_Copy.xlsx, EverythingTests!G1288, saved by Excel).
#[test]
fn typed_text_and_booleans() {
    let mut model = new_empty_model();
    model._set("B1", "'1");
    model._set("B2", "TRUE");
    model._set("A1", r#"=STDEV("1","2")"#);
    model._set("A2", r#"=STDEV.S(TRUE,FALSE)"#);
    model._set("A3", r#"=STDEV.P("1","3")"#);
    model._set("A4", r#"=STDEV(B1,B2,1,2)"#);
    model._set("A5", r#"=STDEV("one",1,2)"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.707106781");
    assert_eq!(model._get_text("A2"), *"0.707106781");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model._get_text("A4"), *"0.707106781");
    assert_eq!(model._get_text("A5"), *"#VALUE!");
}
