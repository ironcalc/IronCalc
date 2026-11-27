#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_if_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=IF()");
    model._set("A2", "=IF(1, 2, 3, 4)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn fn_if_2_args() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(2 > 3, TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
}

#[test]
fn fn_if_missing_args() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(2 > 3, TRUE, )");
    model._set("A2", "=IF(2 > 3, , 5)");
    model._set("A3", "=IF(2 < 3, , 5)");

    model.evaluate();

    // assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"5");
    assert_eq!(model._get_text("A3"), *"0");
}

// Issue 1749: a range in a branch of IF is handed to the function around it.
// B1:B4 is 1, 3, 5, 8 and C1:C4 is 10, 20, 30, 40. Legacy import must not put
// `@` on those ranges, or SUM/COUNT/MAX/SUBTOTAL/AVERAGE see one cell.
#[test]
fn if_range_branch_reaches_the_function_around_it() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "8");
    model._set("C1", "10");
    model._set("C2", "20");
    model._set("C3", "30");
    model._set("C4", "40");

    model._set("F2", "=SUM(IF(TRUE,B1:B4))");
    model._set("F6", "=SUM(IF(TRUE,B1:B4))");
    model._set("F7", "=COUNT(IF(TRUE,B1:B4))");
    model._set("F8", "=MAX(IF(TRUE,B1:B4))");
    model._set("F11", "=SUBTOTAL(9,IF(TRUE,B1:B4))");
    model._set("F12", "=SUM(IF(B1>0,B1:B4,C1:C4))");
    model._set("F13", "=SUM(IF(B1>5,B1:B4,C1:C4))");
    model._set("F17", "=AVERAGE(IF(TRUE,B1:B4))");
    model.evaluate();

    assert_eq!(model._get_text("F2"), "17");
    assert_eq!(model._get_text("F6"), "17");
    assert_eq!(model._get_text("F7"), "4");
    assert_eq!(model._get_text("F8"), "8");
    assert_eq!(model._get_text("F11"), "17");
    assert_eq!(model._get_text("F12"), "17");
    assert_eq!(model._get_text("F13"), "100");
    assert_eq!(model._get_text("F17"), "4.25");
}

// A bare IF that returns a range is intersected at the formula's row
// (legacy import writes `=@IF(TRUE,B1:B4)`). Row 2 meets B2; row 6 meets nothing.
#[test]
fn bare_if_of_a_range_intersects_at_the_formula_row() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "3");
    model._set("B3", "5");
    model._set("B4", "8");
    model._set("F2", "=@IF(TRUE,B1:B4)");
    model._set("F6", "=@IF(TRUE,B1:B4)");
    model.evaluate();

    assert_eq!(model._get_text("F2"), "3");
    assert_eq!(model._get_text("F6"), "#VALUE!");
    // The intersection is one cell: nothing spills.
    assert_eq!(model._get_text("F3"), "");
    assert_eq!(model._get_text("F7"), "");
}
