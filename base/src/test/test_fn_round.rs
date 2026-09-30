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
fn fn_round_acts_on_the_value_rounded_to_15_digits() {
    let mut model = new_empty_model();
    model._set("A1", "=ROUNDUP(1.1*3,1)");
    model._set("A2", "=ROUND(1.005,2)");
    model._set("A3", "=ROUNDDOWN(0.29,2)");
    model._set("A4", "=ROUNDUP(0.285,2)");

    model.evaluate();

    // 1.1*3 is 3.3000000000000003 in floating point; Excel rounds the
    // value as if it were exactly 3.3, not 3.4
    assert_eq!(model._get_text("A1"), "3.3");
    assert_eq!(model._get_text("A2"), "1.01");
    assert_eq!(model._get_text("A3"), "0.29");
    assert_eq!(model._get_text("A4"), "0.29");
}
