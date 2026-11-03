#![allow(clippy::unwrap_used)]

use crate::number_format::number_to_text;
use crate::test::util::new_empty_model;

#[test]
fn fn_concatenate_args_number() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCATENATE()");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn fn_concatenate() {
    let mut model = new_empty_model();
    model._set("A1", "Hello");
    model._set("A2", " my ");
    model._set("A3", "World");

    model._set("B1", r#"=CONCATENATE(A1, A2, A3, "!")"#);
    model._set("C2", r#"=CONCATENATE(@A1:A3, "!")"#);
    // FIXME: this should be an array ["Hello!", " my!", "World!"] and spill
    // Instead we are doing implicit intersection
    model._set("B2", r#"=CONCATENATE(A1:A3, "!")"#);
    model._set("D3", r#"=CONCAT(A1:A3, "!")"#);

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Hello my World!");
    // FIXME: this is wrong
    assert_eq!(model._get_text("B2"), *" my !");
    assert_eq!(model._get_text("D3"), *"Hello my World!");
    assert_eq!(model._get_text("C2"), *" my !");
}

#[test]
fn concat_arrays() {
    let mut model = new_empty_model();
    for (i, code) in [72, 101, 108, 108, 111].iter().enumerate() {
        model._set(&format!("A{}", i + 1), &code.to_string());
    }
    model._set("D2", "b");
    // A spilled function result
    model._set("B1", "=CONCAT(CHAR(A1:A5))");
    // An inline array, row by row, mixing numbers, booleans and texts
    model._set("B2", "=CONCAT({1,\"a\";TRUE,2.5})");
    // Arrays mixed with scalars and ranges; empty elements are skipped
    model._set("B3", "=CONCAT(\"x\", {1,2}, A1:A2, UPPER(D1:D3))");
    // An error element is the error of the function
    model._set("B4", "=CONCAT(CHAR({65,0}))");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"Hello");
    assert_eq!(model._get_text("B2"), *"1aTRUE2.5");
    assert_eq!(model._get_text("B3"), *"x1272101B");
    assert_eq!(model._get_text("B4"), *"#VALUE!");
}

// CONCAT / TEXTJOIN must stringify numbers like Excel does, not with Rust's
// `{}` (https://github.com/ironcalc/IronCalc/issues/1665)
#[test]
fn concat_number_to_text() {
    let mut model = new_empty_model();
    model._set("B1", "=CONCAT(0.1+0.2)");
    model._set("B2", "=CONCAT(1E+20)");
    model._set("B3", "=1E+15&\"\"");
    model._set("B4", "=1E-15&\"\"");
    model._set("B5", r#"=TEXTJOIN("-", TRUE, 0.1+0.2, 1E+20)"#);
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"0.3");
    assert_eq!(model._get_text("B2"), *"1E+20");
    assert_eq!(model._get_text("B3"), *"1000000000000000");
    assert_eq!(model._get_text("B4"), *"0.000000000000001");
    assert_eq!(model._get_text("B5"), *"0.3-1E+20");
}

// Excel switches to scientific notation when the plain representation (sign
// excluded) does not fit its 20-character window, not at a fixed exponent
// (https://github.com/ironcalc/IronCalc/pull/1677#issuecomment)
#[test]
fn concat_number_to_text_window() {
    assert_eq!(number_to_text(1e-18), "0.000000000000000001");
    assert_eq!(number_to_text(1e-19), "1E-19");
    assert_eq!(number_to_text(1.5e-18), "1.5E-18");
    assert_eq!(number_to_text(1.5e-19), "1.5E-19");
    assert_eq!(number_to_text(1.23456789012345e-18), "1.23456789012345E-18");
    assert_eq!(number_to_text(1.23456789012345e19), "12345678901234500000");
    assert_eq!(number_to_text(-1e-19), "-1E-19");
}
