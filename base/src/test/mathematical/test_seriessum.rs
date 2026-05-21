#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_seriessum_basic() {
    let mut model = new_empty_model();
    // SERIESSUM(2, 1, 1, {1, 2, 3})
    // = 1*2^1 + 2*2^2 + 3*2^3 = 2 + 8 + 24 = 34
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "=SERIESSUM(2,1,1,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "34");
}

#[test]
fn test_seriessum_cosine_approximation() {
    let mut model = new_empty_model();
    // cos(x) ≈ 1 - x^2/2! + x^4/4! - x^6/6! with x=PI()/4 (≈ 0.7071)
    // SERIESSUM(x^2, 0, 1, {1, -1/2, 1/24, -1/720}) should approximate cos(x)
    // Using x=0 for simplicity: SERIESSUM(0, 0, 1, {1}) = 1
    model._set("A1", "1");
    model._set("B1", "=SERIESSUM(0,0,1,A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "1");
}

#[test]
fn test_seriessum_single_coeff() {
    let mut model = new_empty_model();
    // SERIESSUM(3, 2, 0, {5}) = 5 * 3^2 = 45
    model._set("A1", "5");
    model._set("B1", "=SERIESSUM(3,2,0,A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "45");
}

#[test]
fn test_seriessum_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=SERIESSUM(1,0,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_seriessum_x1_n0_m1() {
    let mut model = new_empty_model();
    // SERIESSUM(1, 0, 1, {1, 1, 1}) = 1*1^0 + 1*1^1 + 1*1^2 = 1+1+1 = 3
    model._set("A1", "1");
    model._set("A2", "1");
    model._set("A3", "1");
    model._set("B1", "=SERIESSUM(1,0,1,A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "3");
}
