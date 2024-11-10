#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(\"1,00\")");
    model._set("A2", "=UNICODE(\"1\")");
    model._set("A3", "=UNICODE(\"T\")");
    model._set("A4", "=UNICODE(\"TRUE\")");
    model._set("A5", "=UNICODE(\"の\")");
    model._set("A6", "=UNICODE(\" \")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"49");
    assert_eq!(model._get_text("A2"), *"49");
    assert_eq!(model._get_text("A3"), *"84");
    assert_eq!(model._get_text("A4"), *"84");
    assert_eq!(model._get_text("A5"), *"12398");
    assert_eq!(model._get_text("A6"), *"32");
}

#[test]
fn value_errors() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(\"\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}
