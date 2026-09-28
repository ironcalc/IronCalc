#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn rows_columns_not_take_lists() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");

    model._set("B1", "=ROWS(UNIQUE(A1:A2))");
    model._set("B2", "=COLUMNS(SEQUENCE(1,4))");
    model._set("B3", "=ROWS({1;2;3})");
    model._set("B4", "=ROWS(5)");

    model._set("C1", "=NOT(A1:A2>1)");
    model._set("E1", "=NOT({TRUE,FALSE})");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "4");
    assert_eq!(model._get_text("B3"), "3");
    assert_eq!(model._get_text("B4"), "1");

    // NOT spills down over the two rows of A1:A2>1
    assert_eq!(model._get_text("C1"), "TRUE");
    assert_eq!(model._get_text("C2"), "FALSE");

    // NOT spills across the two columns of {TRUE,FALSE}
    assert_eq!(model._get_text("E1"), "FALSE");
    assert_eq!(model._get_text("F1"), "TRUE");
}
