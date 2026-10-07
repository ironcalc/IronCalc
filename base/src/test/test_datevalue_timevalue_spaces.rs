#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel ignores spaces around the date separators (#1380).
#[test]
fn datevalue_spaces_around_separators() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE(\"2026 - 01 - 01\")");
    model._set("A2", "=DATEVALUE(\"1 / 2 / 2026\")");
    model._set("A3", "=DATEVALUE(\"4 - Feb - 2026\")");
    model._set("A4", "=DATEVALUE(\"2026 - 01 - 01 12:00\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"46023");
    assert_eq!(model._get_text("A2"), *"46024");
    assert_eq!(model._get_text("A3"), *"46057");
    assert_eq!(model._get_text("A4"), *"46023");
}

// Excel ignores spaces around the time separator (#1402).
#[test]
fn timevalue_spaces_around_separators() {
    let mut model = new_empty_model();
    model._set("A1", "=TIMEVALUE(\"4 : 35\")");
    model._set("A2", "=TIMEVALUE(\"4:35\")");
    model._set("A3", "=TIMEVALUE(\"12 : 00 : 00\")");
    model._set("A4", "=TIMEVALUE(\"3 : 30 PM\")");
    model._set("A5", "=TIMEVALUE(\"3:30 PM\")");
    model.evaluate();

    assert_ne!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A1"), model._get_text("A2"));
    assert_eq!(model._get_text("A3"), *"0.5");
    assert_eq!(model._get_text("A4"), model._get_text("A5"));
}

// A long whitespace run next to a separator is removed in one pass.
#[test]
fn long_space_runs_around_separators() {
    let mut model = new_empty_model();
    model._set("A1", "=TIMEVALUE(\"4\"&REPT(\" \",32000)&\":35\")");
    model._set("A2", "=TIMEVALUE(\"4:35\")");
    model._set("A3", "=DATEVALUE(\"2026-\"&REPT(\" \",32000)&\"01-01\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), model._get_text("A2"));
    assert_eq!(model._get_text("A3"), *"46023");
}
