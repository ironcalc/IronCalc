#![allow(clippy::unwrap_used)]

use crate::Model;

pub fn new_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "de", "UTC", "de").unwrap()
}

#[test]
fn german_functions() {
    let mut model = new_empty_model();
    model._set("A1", "=WENN(1>2; 3; 4)");
    model._set("A2", "=SUMME({1;2;3\\4;5;6})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"21");
}

#[test]
fn german_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMME(1,23; 3,45; 4,56)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9,24");
}

#[test]
fn croatian_numbers() {
    let mut model = Model::new_empty("model", "hr", "UTC", "en").unwrap();
    model._set("A1", "=SUM(1,23; 3,45; 4,56)");
    model._set("A2", "1234,5");
    model._set("A3", "=A2*2");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9,24");
    assert_eq!(model._get_text("A3"), *"2469");
}

#[test]
fn croatian_dates() {
    let mut model = Model::new_empty("model", "hr", "UTC", "en").unwrap();
    model._set("A1", "=DATE(2026; 10; 17)");
    model._set("A2", "=TEXT(A1; \"mmmm\")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"17. 10. 2026.");
    assert_eq!(model._get_text("A2"), *"listopad");
}
