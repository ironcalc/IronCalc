#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_randarray_default() {
    let mut model = new_empty_model();
    model._set("A1", "=RANDARRAY()");
    model.evaluate();
    let val: f64 = model._get_text("A1").parse().unwrap();
    assert!((0.0..1.0).contains(&val));
}

#[test]
fn test_randarray_shape() {
    let mut model = new_empty_model();
    model._set("A1", "=RANDARRAY(2,3)");
    model.evaluate();
    for row in ["A", "B"] {
        for col in ["1", "2", "3"] {
            let cell = format!(
                "{}{}",
                ["A", "B", "C"][col.parse::<usize>().unwrap() - 1],
                ["1", "2"][["A", "B"].iter().position(|&r| r == row).unwrap()]
            );
            let val: f64 = model._get_text(&cell).parse().unwrap();
            assert!((0.0..=1.0).contains(&val));
        }
    }
}

#[test]
fn test_randarray_range() {
    let mut model = new_empty_model();
    model._set("A1", "=RANDARRAY(1,1,5,10)");
    model.evaluate();
    let val: f64 = model._get_text("A1").parse().unwrap();
    assert!((5.0..=10.0).contains(&val));
}

#[test]
fn test_randarray_whole_number() {
    let mut model = new_empty_model();
    model._set("A1", "=RANDARRAY(1,1,1,6,TRUE)");
    model.evaluate();
    let val: f64 = model._get_text("A1").parse().unwrap();
    assert_eq!(val, val.floor());
    assert!((1.0..6.0).contains(&val));
}

#[test]
fn test_randarray_invalid_range() {
    let mut model = new_empty_model();
    model._set("A1", "=RANDARRAY(1,1,10,5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}
