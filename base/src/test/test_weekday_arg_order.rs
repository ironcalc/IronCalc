#![allow(clippy::unwrap_used)]

//! WEEKDAY must evaluate the date argument before `return_type` (#1502).
//! Excel returns #VALUE! for an invalid date even when `return_type` is #DIV/0!
//! or #N/A; IronCalc used to resolve `return_type` first and surface that error.

use crate::test::util::new_empty_model;

#[test]
fn weekday_evaluates_date_before_return_type() {
    let mut model = new_empty_model();
    // Unparseable / out-of-range dates fail as #VALUE! before return_type is read.
    model._set("A1", "=WEEKDAY(\"1/1/10000\", 1/0)");
    model._set("A2", "=WEEKDAY(\"1/1/10000\", NA())");
    model._set("A3", "=WEEKDAY(\"abc\", 1/0)");
    model._set("A4", "=WEEKDAY(\"abc\", NA())");
    // A valid date still surfaces the return_type error.
    model._set("A7", "=WEEKDAY(44561, 1/0)");
    model._set("A8", "=WEEKDAY(44561, NA())");
    // The one-argument form is unaffected.
    model._set("A9", "=WEEKDAY(\"abc\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A7"), *"#DIV/0!");
    assert_eq!(model._get_text("A8"), *"#N/A");
    assert_eq!(model._get_text("A9"), *"#VALUE!");
}
