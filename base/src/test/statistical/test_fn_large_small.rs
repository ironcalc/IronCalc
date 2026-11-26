#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// B1:B4 = 1, 3, 5, 8. Every row is one of the 24 formulas checked in Excel
/// for Microsoft 365 (build 16.0.20527.20150) in issue #1750.
///
/// SMALL rejects k below 1 or above the count before it truncates k toward
/// zero. LARGE uses the same bounds, then ceilings k.
#[test]
fn issue_1750_non_integer_and_boolean_k() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "8");

    // (k, LARGE, SMALL)
    let cases = [
        ("0.5", "#NUM!", "#NUM!"),
        ("1", "8", "1"),
        ("1.2", "5", "1"),
        ("1.5", "5", "1"),
        ("1.9", "5", "1"),
        ("2", "5", "3"),
        ("2.5", "3", "3"),
        ("3.999", "1", "5"),
        ("4", "1", "8"),
        ("4.5", "#NUM!", "#NUM!"),
        ("TRUE", "8", "1"),
        ("\"2\"", "5", "3"),
    ];

    for (row, (k, _, _)) in cases.iter().enumerate() {
        let row = row + 1;
        model._set(&format!("D{row}"), &format!("=LARGE(B1:B4,{k})"));
        model._set(&format!("E{row}"), &format!("=SMALL(B1:B4,{k})"));
    }
    model.evaluate();

    for (row, (k, large, small)) in cases.iter().enumerate() {
        let row = row + 1;
        assert_eq!(model._get_text(&format!("D{row}")), *large, "LARGE k={k}");
        assert_eq!(model._get_text(&format!("E{row}")), *small, "SMALL k={k}");
    }
}

/// Edges that follow the same rule but were not in the issue #1750 table.
/// `k = 0` is the cached Excel result in
/// `xlsx/tests/calc_tests/STATISTICAL/SMALL_LARGE.xlsx`. The other rows are
/// the scalar coercion `get_number` already applies; they are not from that
/// Excel session.
#[test]
fn k_bounds_errors_and_scalar_coercion() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "8");

    // Literal FALSE is 0, which is below 1.
    model._set("A1", "=LARGE(B1:B4,FALSE)");
    model._set("A2", "=SMALL(B1:B4,FALSE)");
    // k = 0 and a negative k are below 1.
    model._set("A3", "=LARGE(B1:B4,0)");
    model._set("A4", "=SMALL(B1:B4,-3.2)");
    // An error in k propagates. A non-numeric text is #VALUE!.
    model._set("A5", "=LARGE(B1:B4,1/0)");
    model._set("A6", "=SMALL(B1:B4,NA())");
    model._set("A7", "=LARGE(B1:B4,\"abc\")");
    // No numbers in the array.
    model._set("A8", "=LARGE(Z1:Z4,1)");
    model._set("A9", "=SMALL(Z1:Z4,1)");
    // k below 1 is #NUM! even when the array is empty.
    model._set("A10", "=SMALL(Z1:Z4,0.5)");

    // k taken from a cell: a number, TRUE, FALSE, numeric text, a blank.
    model._set("C1", "1.2");
    model._set("C2", "TRUE");
    model._set("C3", "FALSE");
    model._set("C4", "'2");
    model._set("A11", "=LARGE(B1:B4,C1)");
    model._set("A12", "=LARGE(B1:B4,C2)");
    model._set("A13", "=SMALL(B1:B4,C2)");
    model._set("A14", "=SMALL(B1:B4,C3)");
    model._set("A15", "=LARGE(B1:B4,C4)");
    model._set("A16", "=SMALL(B1:B4,C4)");
    model._set("A17", "=LARGE(B1:B4,C5)");
    // An omitted k is empty, which coerces to 0.
    model._set("A18", "=SMALL(B1:B4,)");
    // Whole-number k still picks from either end, including an array constant.
    model._set("A19", "=LARGE({8,1,5,3},4)");
    model._set("A20", "=SMALL({8,1,5,3},1.9)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#N/A");
    assert_eq!(model._get_text("A7"), *"#VALUE!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
    assert_eq!(model._get_text("A10"), *"#NUM!");
    assert_eq!(model._get_text("A11"), *"5");
    assert_eq!(model._get_text("A12"), *"8");
    assert_eq!(model._get_text("A13"), *"1");
    assert_eq!(model._get_text("A14"), *"#NUM!");
    assert_eq!(model._get_text("A15"), *"5");
    assert_eq!(model._get_text("A16"), *"3");
    assert_eq!(model._get_text("A17"), *"#NUM!");
    assert_eq!(model._get_text("A18"), *"#NUM!");
    assert_eq!(model._get_text("A19"), *"1");
    assert_eq!(model._get_text("A20"), *"1");
}
