#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn iserror_over_array() {
    let mut model = new_empty_model();
    model._set("A1", "=ISERROR(CHAR({65,0,256,66}))");
    model._set("A2", "=SUM(--ISERROR(CHAR({65,0,256,66})))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("C1"), "TRUE");
    assert_eq!(model._get_text("D1"), "FALSE");
    assert_eq!(model._get_text("A2"), "2");
}

#[test]
fn error_kinds_over_array() {
    let mut model = new_empty_model();
    model._set("A1", "=ISNA({1,#N/A,#DIV/0!})");
    model._set("A2", "=ISERR({1,#N/A,#DIV/0!})");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("C1"), "FALSE");
    assert_eq!(model._get_text("A2"), "FALSE");
    assert_eq!(model._get_text("B2"), "FALSE");
    assert_eq!(model._get_text("C2"), "TRUE");
}

#[test]
fn types_over_range() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "hello");
    model._set("A3", "TRUE");
    // A4 is empty
    model._set("A5", "=1/0");

    model._set("B1", "=ISNUMBER(A1:A5)");
    model._set("C1", "=ISTEXT(A1:A5)");
    model._set("D1", "=ISNONTEXT(A1:A5)");
    model._set("E1", "=ISLOGICAL(A1:A5)");
    model._set("F1", "=ISBLANK(A1:A5)");
    model._set("G1", "=ISERROR(A1:A5)");
    model.evaluate();

    let column = |model: &crate::Model, c: &str| -> Vec<String> {
        (1..=5)
            .map(|r| model._get_text(&format!("{c}{r}")))
            .collect()
    };
    assert_eq!(
        column(&model, "B"),
        ["TRUE", "FALSE", "FALSE", "FALSE", "FALSE"]
    );
    assert_eq!(
        column(&model, "C"),
        ["FALSE", "TRUE", "FALSE", "FALSE", "FALSE"]
    );
    assert_eq!(
        column(&model, "D"),
        ["TRUE", "FALSE", "TRUE", "TRUE", "TRUE"]
    );
    assert_eq!(
        column(&model, "E"),
        ["FALSE", "FALSE", "TRUE", "FALSE", "FALSE"]
    );
    assert_eq!(
        column(&model, "F"),
        ["FALSE", "FALSE", "FALSE", "TRUE", "FALSE"]
    );
    assert_eq!(
        column(&model, "G"),
        ["FALSE", "FALSE", "FALSE", "FALSE", "TRUE"]
    );
}

#[test]
fn single_values_are_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=ISBLANK(C1)");
    model._set("B2", "=ISERROR(1/0)");
    model._set("B3", "=ISNUMBER(A1)");
    model._set("B4", "=ISBLANK(OFFSET(C1,0,0))");
    model._set("B5", "=ISNUMBER(INDEX(A1:A3,1))");
    model._set("B6", "=IF(ISBLANK(OFFSET(C1,0,0)),\"empty\",\"full\")");
    model.evaluate();

    assert_eq!(model._get_text("B1"), "TRUE");
    assert_eq!(model._get_text("B2"), "TRUE");
    assert_eq!(model._get_text("B3"), "TRUE");
    assert_eq!(model._get_text("B4"), "TRUE");
    assert_eq!(model._get_text("B5"), "TRUE");
    assert_eq!(model._get_text("B6"), "empty");
}
