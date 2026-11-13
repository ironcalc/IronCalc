#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model._set("A1", "Brian Wiles");
    model._set("A2", "Jeden,dwa,trzy,cztery");

    model._set("B1", "=TEXTAFTER(A1, \" \")");
    model._set("B2", "=TEXTAFTER(A2, \",\")");
    model._set("C2", "=TEXTAFTER(A2, \",\", 2)");

    model._set("H1", "=TEXTBEFORE(A1, \" \")");
    model._set("H2", "=TEXTBEFORE(A2, \",\")");
    model._set("I2", "=_xlfn.TEXTBEFORE(A2, \",\", 2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Wiles");
    assert_eq!(model._get_text("B2"), *"dwa,trzy,cztery");
    assert_eq!(model._get_text("C2"), *"trzy,cztery");

    assert_eq!(model._get_text("H1"), *"Brian");
    assert_eq!(model._get_text("H2"), *"Jeden");
    assert_eq!(model._get_text("I2"), *"Jeden,dwa");
    assert_eq!(model._get_formula("I2"), *"=TEXTBEFORE(A2,\",\",2)");
}

// "When searching with an empty delimiter value, TEXTAFTER matches immediately. It returns
// the entire text when searching from the front (if instance_num is positive) and empty
// text when searching from the end (if instance_num is negative)." TEXTBEFORE mirrors it.
// Expected values captured from Excel for Mac 16.114.1 (en-US).
#[test]
fn empty_delimiter() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTAFTER(\"a-b-c\", \"\")");
    model._set("A2", "=TEXTAFTER(\"a-b-c\", \"\", 2)");
    model._set("A3", "=TEXTAFTER(\"a-b-c\", \"\", 100)");
    model._set("A4", "=LEN(TEXTAFTER(\"a-b-c\", \"\", -1))");
    model._set("A5", "=TEXTAFTER(\"a-b-c\", \"\", 1, 0, 1)");
    model._set("A6", "=TEXTAFTER(\"a-b-c\", \"\", 0)");
    model._set("A7", "=LEN(TEXTAFTER(\"\", \"\"))");

    model._set("B1", "=LEN(TEXTBEFORE(\"a-b-c\", \"\"))");
    model._set("B2", "=TEXTBEFORE(\"a-b-c\", \"\", -1)");
    model._set("B3", "=TEXTBEFORE(\"a-b-c\", \"\", -100)");
    model._set("B4", "=TEXTBEFORE(\"a-b-c\", \"\", -1, 0, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"a-b-c");
    assert_eq!(model._get_text("A2"), *"a-b-c");
    assert_eq!(model._get_text("A3"), *"a-b-c");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"a-b-c");
    assert_eq!(model._get_text("A6"), *"#VALUE!");
    assert_eq!(model._get_text("A7"), *"0");

    assert_eq!(model._get_text("B1"), *"0");
    assert_eq!(model._get_text("B2"), *"a-b-c");
    assert_eq!(model._get_text("B3"), *"a-b-c");
    assert_eq!(model._get_text("B4"), *"a-b-c");
}
