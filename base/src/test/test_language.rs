#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, Model};

pub fn new_german_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "de").unwrap()
}

#[test]
fn german() {
    let mut model = new_german_empty_model();
    model._set("A1", "=WENN(1>2, 3, 4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
}

#[test]
fn french() {
    let mut model = new_empty_model();
    model._set("A1", "=IF(1>2, 3, 4)");
    model._set("B1", "=TRUE");
    model._set("C1", "=FALSE()");
    model._set("D1", "=FALSE");
    model.evaluate();
    model.set_language("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_formula("A1"), *"=SI(1>2,3,4)");
    assert_eq!(model._get_formula("B1"), *"=VRAI");
    assert_eq!(model._get_formula("C1"), *"=FAUX()");
    assert_eq!(model._get_formula("D1"), *"=FAUX");
    assert_eq!(model._get_text("B1"), *"VRAI");
    assert_eq!(model._get_text("C1"), *"FAUX");
    assert_eq!(model._get_text("D1"), *"FAUX");
}

#[test]
fn spanish() {
    let mut model = new_empty_model();
    model._set("A1", "=TRUE()");
    model.evaluate();
    model.set_language("es").unwrap();
    model._set("B1", "=TRUE()");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=VERDADERO()");
    assert_eq!(model._get_text("A1"), *"VERDADERO");
    assert_eq!(model._get_text("B1"), *"#¿NOMBRE?");
}

#[test]
fn japanese() {
    // Japanese Excel keeps the English function names, booleans and errors
    let mut model = Model::new_empty("model", "ja", "UTC", "ja").unwrap();
    model._set("A1", "=IF(1>2, 3, 4)");
    model._set("B1", "=TRUE");
    model._set("C1", "=1/0");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_formula("A1"), *"=IF(1>2,3,4)");
    assert_eq!(model._get_text("B1"), *"TRUE");
    assert_eq!(model._get_text("C1"), *"#DIV/0!");
    assert_eq!(model.get_worksheets_properties()[0].name, "Sheet1");
}
