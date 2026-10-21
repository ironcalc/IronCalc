#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_round_approximation() {
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(1.05*(0.0284+0.0046)-0.0284,4)");
    model._set("A2", "=ROUNDDOWN(1.05*(0.0284+0.0046)-0.0284,5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.0063");
    assert_eq!(model._get_text("A2"), *"0.00625");
}

#[test]
fn fn_round_binary_noise_after_scaling() {
    // Values from Excel. 1.005 * 100 is 100.49999999999999 in binary, and
    // -(0.1 + 0.7) is -0.7999999999999999.
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(1.005,2)");
    model._set("A2", "=ROUNDDOWN((0.1+0.7)*10,0)");
    model._set("A3", "=TRUNC(-(0.1+0.7),1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.01");
    assert_eq!(model._get_text("A2"), *"8");
    assert_eq!(model._get_text("A3"), *"-0.8");
}

#[test]
fn fn_round_digit_counts_beyond_f64_range() {
    // Values from Excel. Excel does not parse 1E+308 as a literal, so it is computed.
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(1E+307*10,10)");
    model._set("A2", "=ROUNDUP(1E+307*10,5)");
    model._set("A3", "=ROUNDDOWN(1E+307*10,5)");
    model._set("A4", "=ROUND(123.456,-400)");
    model._set("A5", "=ROUNDDOWN(123.456,-400)");
    model._set("A6", "=TRUNC(123.456,-400)");
    model._set("A7", "=TRUNC(123.456,-16)");
    model._set("A8", "=TRUNC(-1E+300,20)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1E+308");
    assert_eq!(model._get_text("A2"), *"1E+308");
    assert_eq!(model._get_text("A3"), *"1E+308");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"0");
    assert_eq!(model._get_text("A7"), *"0");
    assert_eq!(model._get_text("A8"), *"-1E+300");
}

#[test]
fn fn_round_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=ROUND()");
    model._set("A2", "=ROUND(1.234)");
    model._set("A3", "=ROUND(1.234,1)");
    model._set("A4", "=ROUND(1.234,1,2)");

    model._set("B1", "=ROUNDUP()");
    model._set("B2", "=ROUNDUP(1.234)");
    model._set("B3", "=ROUNDUP(1.234,1)");
    model._set("B4", "=ROUNDUP(1.234,1,2)");

    model._set("C1", "=ROUNDDOWN()");
    model._set("C2", "=ROUNDDOWN(1.234)");
    model._set("C3", "=ROUNDDOWN(1.234,1)");
    model._set("C4", "=ROUNDDOWN(1.234,1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"1.2");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"1.3");
    assert_eq!(model._get_text("B4"), *"#ERROR!");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
    assert_eq!(model._get_text("C3"), *"1.2");
    assert_eq!(model._get_text("C4"), *"#ERROR!");
}
