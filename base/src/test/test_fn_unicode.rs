#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICODE(\"1,00\")");
    model._set("A2", "=UNICODE(\"1\")");
    model._set("A3", "=UNICODE(1)");
    model._set("A4", "=UNICODE(\"T\")");
    model._set("A5", "=UNICODE(\"TRUE\")");
    model._set("A6", "=UNICODE(TRUE)");
    model._set("A7", "=UNICODE(FALSE)");
    model._set("A8", "=UNICODE(\"の\")");
    model._set("A9", "=UNICODE(\" \")");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"49");
    assert_eq!(model._get_text("A2"), *"49");
    assert_eq!(model._get_text("A3"), *"49");
    assert_eq!(model._get_text("A4"), *"84");
    assert_eq!(model._get_text("A5"), *"84");
    assert_eq!(model._get_text("A6"), *"84");
    assert_eq!(model._get_text("A7"), *"70");
    assert_eq!(model._get_text("A8"), *"12398");
    assert_eq!(model._get_text("A9"), *"32");
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
