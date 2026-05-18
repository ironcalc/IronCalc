#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE, A1:A3)");
    model._set("B2", "=TEXTJOIN(\" and \", TRUE, A1:A3)");
    // This formula might have the _xlfn. prefix
    model._set("B3", "=_xlfn.TEXTJOIN(\" or \", , A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Monday, Tuesday, Wednesday");
    assert_eq!(model._get_text("B2"), *"Monday and Tuesday and Wednesday");
    assert_eq!(model._get_text("B3"), *"Monday or Tuesday or Wednesday");
    // Our text version removes the prefix, of course (and some white spaces)
    assert_eq!(model._get_formula("B3"), *"=TEXTJOIN(\" or \",,A1:A3)");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "Monday");
    model._set("A2", "Tuesday");
    model._set("A3", "Wednesday");

    model._set("B1", "=TEXTJOIN(\", \", TRUE)");
    model._set("B2", "=TEXTJOIN(\" and \", A1:A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}
