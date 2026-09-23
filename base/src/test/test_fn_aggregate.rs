#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn aggregate_functions() {
    let mut model = new_empty_model();
    for i in 1..=5 {
        model._set(&format!("A{i}"), &i.to_string());
    }
    model._set("A6", "=1/0");
    let cases = [
        ("=AGGREGATE(1,6,A1:A6)", "3"),
        ("=AGGREGATE(2,6,A1:A6)", "5"),
        ("=AGGREGATE(3,6,A1:A6)", "5"),
        ("=AGGREGATE(4,6,A1:A6)", "5"),
        ("=AGGREGATE(5,6,A1:A6)", "1"),
        ("=AGGREGATE(6,6,A1:A6)", "120"),
        ("=AGGREGATE(9,6,A1:A6)", "15"),
        ("=AGGREGATE(9,4,A1:A6)", "#DIV/0!"),
        ("=AGGREGATE(11,6,A1:A6)", "2"),
        ("=AGGREGATE(12,6,A1:A6)", "3"),
        ("=AGGREGATE(14,6,A1:A6,2)", "4"),
        ("=AGGREGATE(15,6,A1:A6,1)", "1"),
        ("=AGGREGATE(16,6,A1:A6,0.5)", "3"),
        ("=AGGREGATE(17,6,A1:A6,1)", "2"),
        ("=AGGREGATE(18,6,A1:A6,0.5)", "3"),
        ("=AGGREGATE(19,6,A1:A6,1)", "1.5"),
        ("=AGGREGATE(14,6,A1:A5/(A1:A5>2),3)", "3"),
        ("=AGGREGATE(14,6,A1:A6,9)", "#NUM!"),
        ("=AGGREGATE(20,6,A1:A6)", "#VALUE!"),
        ("=AGGREGATE(9,8,A1:A6)", "#VALUE!"),
        ("=AGGREGATE(9,6)", "#ERROR!"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("C{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("C{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
    assert_eq!(model._get_formula("C1"), "=AGGREGATE(1,6,A1:A6)");
}

// Options 0-3 leave out nested SUBTOTAL and AGGREGATE; 1, 3, 5 and 7 hidden rows.
#[test]
fn aggregate_nested_and_hidden() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "=SUBTOTAL(9,A1:A2)");
    model._set("A4", "=AGGREGATE(9,0,A1:A2)");
    model._set("A5", "10");
    model._set("B1", "=AGGREGATE(9,0,A1:A5)");
    model._set("B2", "=AGGREGATE(9,4,A1:A5)");
    model._set("B3", "=AGGREGATE(9,5,A1:A5)");
    model._set("B4", "=AGGREGATE(9,1,A1:A5)");
    model.set_row_hidden(0, 2, true).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B1"), "13");
    assert_eq!(model._get_text("B2"), "19");
    assert_eq!(model._get_text("B3"), "17");
    assert_eq!(model._get_text("B4"), "11");
}
