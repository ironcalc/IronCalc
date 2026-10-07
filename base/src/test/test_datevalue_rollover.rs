#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel rolls the date over when the time part reaches 24 hours (#1379).
#[test]
fn datevalue_time_part_rolls_over() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE(\"2026-01-01 24:00:00\")");
    model._set("A2", "=DATEVALUE(\"2026-01-01 25:00:00\")");
    model._set("A3", "=DATEVALUE(\"2026-01-01 23:59:59\")");
    model._set("A4", "=DATEVALUE(\"2026-01-01T24:00:00\")");
    model._set(
        "A5",
        "=DATEDIF(\"2026-01-01\",\"2026-01-01 24:00:00\",\"d\")",
    );
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"46024");
    assert_eq!(model._get_text("A2"), *"46024");
    assert_eq!(model._get_text("A3"), *"46023");
    assert_eq!(model._get_text("A4"), *"46024");
    assert_eq!(model._get_text("A5"), *"1");
}
