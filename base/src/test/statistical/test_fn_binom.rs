#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_binom_dist_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST(6, 10, 0.5, TRUE)");
    model._set("A2", "=BINOM.DIST(6, 10, 0.5, FALSE)");
    model._set("A3", "=BINOM.DIST(6, 10, 0.5)"); // wrong args
    model._set("A4", "=BINOM.DIST(6, 10, 0.5, TRUE, FALSE)"); // too many args
    model.evaluate();

    // P(X <= 6) for X ~ Bin(10, 0.5) = 0.828125
    assert_eq!(model._get_text("A1"), *"0.828125");

    // P(X = 6) for X ~ Bin(10, 0.5) = 0.205078125
    assert_eq!(model._get_text("A2"), *"0.205078125");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_binom_dist_range_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.DIST.RANGE(60, 0.75, 48)");
    model._set("A2", "=BINOM.DIST.RANGE(60, 0.75, 45, 50)");
    model._set("A3", "=BINOM.DIST.RANGE(60, 1.2, 45, 50)"); // p > 1 -> #NUM!
    model._set("A4", "=BINOM.DIST.RANGE(60, 0.75, 50, 45)"); // lower > upper -> #NUM!");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.083974967");

    assert_eq!(model._get_text("A2"), *"0.523629793");

    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn test_fn_binom_inv_smoke() {
    let mut model = new_empty_model();
    model._set("A1", "=BINOM.INV(6, 0.5, 0.75)");
    model._set("A2", "=BINOM.INV(6, 0.5, -0.1)"); // alpha < 0 -> #NUM!
    model._set("A3", "=BINOM.INV(6, 1.2, 0.75)"); // p > 1 -> #NUM!
    model._set("A4", "=BINOM.INV(6, 0.5)"); // args error
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_negbinom_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: PMF (non-cumulative) and CDF (cumulative)
    model._set("A1", "=NEGBINOM.DIST(10, 5, 0.25, FALSE)");
    model._set("A2", "=NEGBINOM.DIST(10, 5, 0.25, TRUE)");

    // Wrong number of arguments -> #ERROR!
    model._set("A3", "=NEGBINOM.DIST(10, 5, 0.25)");
    model._set("A4", "=NEGBINOM.DIST(10, 5, 0.25, TRUE, FALSE)");

    // Domain errors:
    // p < 0 or p > 1 -> #NUM!
    model._set("A5", "=NEGBINOM.DIST(10, 5, 1.5, TRUE)");
    // number_f < 0 -> #NUM!
    model._set("A6", "=NEGBINOM.DIST(-1, 5, 0.25, TRUE)");
    // number_s < 1 -> #NUM!
    model._set("A7", "=NEGBINOM.DIST(10, 0, 0.25, TRUE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.05504866");
    assert_eq!(model._get_text("A2"), *"0.313514058");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}
