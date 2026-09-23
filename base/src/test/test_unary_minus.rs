#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn eval(cells: &[(&str, &str)]) -> crate::Model<'static> {
    let mut model = new_empty_model();
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model.evaluate();
    model
}

#[test]
fn minus_signs_work_on_lists_and_each_one_counts() {
    let model = eval(&[
        ("A1", "a"),
        ("A2", "b"),
        ("A3", "a"),
        ("B1", "=SUM(--(A1:A3=\"a\"))"),
        ("B2", "=SUMPRODUCT(--(A1:A3=\"a\"))"),
        ("B3", "=--TRUE"),
        ("B4", "=---1"),
        ("C1", "=-(A1:A3=\"a\")"),
    ]);
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("B3"), "1");
    assert_eq!(model._get_text("B4"), "-1");
    assert_eq!(model._get_text("C1"), "-1");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_formula("B1"), "=SUM(--(A1:A3=\"a\"))");
}
