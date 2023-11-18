#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_count_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=COUNT()");
    model._set("A2", "=COUNTA()");
    model._set("A3", "=COUNTBLANK()");
    model._set("A4", "=COUNTBLANK(C1:D1, H3:H4)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_count_minimal() {
    let mut model = new_empty_model();
    model._set("B1", "3.1415926");
    model._set("B2", "Tomorrow's the day my bride's gonna come");
    model._set("B3", "");
    model._set("A1", "=COUNT(B1:B5)");
    model._set("A2", "=COUNTA(B1:B5)");
    model._set("A3", "=COUNTBLANK(B1:B5)");
    model.evaluate();

    // There is only one number
    assert_eq!(model._get_text("A1"), *"1");
    // Thre are three non-empty cells
    assert_eq!(model._get_text("A2"), *"3");
    // There are 3 blank cells B4, B5 and B3 that contains the empty string
    assert_eq!(model._get_text("A3"), *"3");
}
