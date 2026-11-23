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

    model._set("B1", "=VAR.P(A2:A15)");
    model._set("B2", "=VAR.S(A2:A15)");
    model._set("B3", "=VARA(A2:A15)");
    model._set("B4", "=VARPA(A2:A15)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"71.9625");
    assert_eq!(model._get_text("B2"), *"79.958333333");
    assert_eq!(model._get_text("B3"), *"240.248626374");
    assert_eq!(model._get_text("B4"), *"223.088010204");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [
        ("VAR.P(D:D)", "2"),
        ("VAR.S(D:D)", "2.5"),
        ("VARA(D:D)", "2.5"),
        ("VARPA(D:D)", "2"),
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

// ── values far from zero ────────────────────────────────────────────────────

// Adding a constant to every value does not change the variance. Excel (since
// 2003) keeps all the digits here.
#[test]
fn values_far_from_zero() {
    let cases = [
        ("VAR.P(A1:A4)", "1.25"),
        ("VAR.S(A1:A4)", "1.666666667"),
        ("VARA(A1:A4)", "1.666666667"),
        ("VARPA(A1:A4)", "1.25"),
        ("STDEV.P(A1:A4)", "1.118033989"),
        ("STDEV.S(A1:A4)", "1.290994449"),
        ("STDEVA(A1:A4)", "1.290994449"),
        ("STDEVPA(A1:A4)", "1.118033989"),
        ("DEVSQ(A1:A4)", "5"),
    ];
    for offset in ["0", "1000000000", "-1000000000"] {
        let mut model = new_empty_model();
        for row in 1..=4 {
            model._set(&format!("A{row}"), &format!("={offset}+{row}"));
        }
        for (row, (formula, _)) in cases.iter().enumerate() {
            model._set(&format!("B{}", row + 1), &format!("={formula}"));
        }
        model.evaluate();
        for (row, (formula, expected)) in cases.iter().enumerate() {
            let cell = format!("B{}", row + 1);
            assert_eq!(model._get_text(&cell), *expected, "{formula} {offset}");
        }
    }
}

// The sum of the squares is not exact here, but the variance is still zero
#[test]
fn equal_values() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model._set(&format!("A{row}"), "123456789.1");
    }
    model._set("B1", "=VAR.P(A1:A10)");
    model._set("B2", "=STDEV.S(A1:A10)");
    model._set("B3", "=DEVSQ(A1:A10)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"0");
    assert_eq!(model._get_text("B2"), *"0");
    assert_eq!(model._get_text("B3"), *"0");
}
