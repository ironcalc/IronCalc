#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn value_and_datevalue_read_dates_and_times() {
    let mut model = new_empty_model();
    model._set("A1", "=VALUE(\"12:00\")");
    model._set("A2", "=VALUE(\"2026-09-23 12:00\")");
    model._set("A3", "=DATEVALUE(\"Sep 23, 2026\")");
    model._set("A4", "=DATEVALUE(\"23 September 2026\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.5");
    assert_eq!(model._get_text("A2"), "46288.5");
    assert_eq!(model._get_text("A3"), "46288");
    assert_eq!(model._get_text("A4"), "46288");
}
