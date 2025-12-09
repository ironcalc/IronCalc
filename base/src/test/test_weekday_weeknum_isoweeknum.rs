#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    // Day 9 Dec 2025 = serial number 46000, Tuesday, week 50
    model._set("A1", "=WEEKDAY()");
    model._set("A2", "=WEEKDAY(46000)");
    model._set("A3", "=WEEKDAY(46000, 11)");
    model._set("A4", "=WEEKDAY(46000, 11, 2)");

    model._set("B1", "=WEEKNUM()");
    model._set("B2", "=WEEKNUM(46000)");
    model._set("B3", "=WEEKNUM(46000, 11)");
    model._set("B4", "=WEEKNUM(46000, 11, 2)");

    model._set("C1", "=ISOWEEKNUM()");
    model._set("C2", "=ISOWEEKNUM(46000)");
    model._set("C3", "=ISOWEEKNUM(46000, 11)");
    model._set("C4", "=ISOWEEKNUM(46000, 11, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"3");
    assert_eq!(model._get_text("A3"), *"2");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"50");
    assert_eq!(model._get_text("B3"), *"50");
    assert_eq!(model._get_text("B4"), *"#ERROR!");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"50");
    assert_eq!(model._get_text("C3"), *"#ERROR!");
    assert_eq!(model._get_text("C4"), *"#ERROR!");
}
