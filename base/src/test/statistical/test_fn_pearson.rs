#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_chisq_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "48");
    model._set("A3", "32");
    model._set("A4", "12");
    model._set("A5", "1");
    model._set("A6", "'13");
    model._set("A7", "TRUE");
    model._set("A8", "1");
    model._set("A9", "13");
    model._set("A10", "15");

    model._set("B2", "55");
    model._set("B3", "34");
    model._set("B4", "13");
    model._set("B5", "blah");
    model._set("B6", "13");
    model._set("B7", "1");
    model._set("B8", "TRUE");
    model._set("B9", "'14");
    model._set("B10", "16");

    model._set("C1", "=PEARSON(A2:A10, B2:B10)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"0.998381439");
}

// ── whole columns ────────────────────────────────────────────────────────────

// Two whole columns are read only as far as the used area; the pairs left
// out are pairs of empty cells, which these functions give no weight.
#[test]
fn test_whole_columns() {
    let mut model = new_empty_model();
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("D3", "3");
    model._set("D5", "text");
    model._set("E1", "2");
    model._set("E2", "4");
    model._set("E3", "7");
    model._set("A1", "=CORREL(D:D,E:E)=CORREL(D1:D3,E1:E3)");
    model._set("A2", "=PEARSON(D:D,E:E)=PEARSON(D1:D3,E1:E3)");
    model._set("A3", "=SLOPE(E:E,D:D)");
    model._set("A4", "=INTERCEPT(E:E,D:D)");
    model._set("A5", "=RSQ(E:E,D:D)=RSQ(E1:E3,D1:D3)");
    model._set("A6", "=SUMX2MY2(D:D,E:E)");
    model._set("A7", "=SUMXMY2(D:D,E:E)");
    model._set("A8", "=FORECAST.LINEAR(4,E:E,D:D)");
    model._set("A9", "=STEYX(E:E,D:D)=STEYX(E1:E3,D1:D3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "2.5");
    assert_eq!(model._get_text("A4"), "-0.666666667");
    assert_eq!(model._get_text("A5"), "TRUE");
    assert_eq!(model._get_text("A6"), "-55");
    assert_eq!(model._get_text("A7"), "21");
    assert_eq!(model._get_text("A8"), "9.333333333");
    assert_eq!(model._get_text("A9"), "TRUE");
}
