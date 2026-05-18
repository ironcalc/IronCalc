#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn mmult_2x2() {
    let mut model = new_empty_model();
    // A = [[1, 2],
    //      [3, 4]]
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    // B = [[5, 6],
    //      [7, 8]]
    model._set("D1", "5");
    model._set("E1", "6");
    model._set("D2", "7");
    model._set("E2", "8");

    model._set("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    // A*B = [[19, 22],
    //        [43, 50]]
    assert_eq!(model._get_text("G1"), "19");
    assert_eq!(model._get_text("H1"), "22");
    assert_eq!(model._get_text("G2"), "43");
    assert_eq!(model._get_text("H2"), "50");
}

#[test]
fn mmult_2x3_times_3x2() {
    let mut model = new_empty_model();
    // A = [[1, 2, 3],
    //      [4, 5, 6]]
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    // B = [[7,  8],
    //      [9,  10],
    //      [11, 12]]
    model._set("E1", "7");
    model._set("F1", "8");
    model._set("E2", "9");
    model._set("F2", "10");
    model._set("E3", "11");
    model._set("F3", "12");

    model._set("H1", "=MMULT(A1:C2, E1:F3)");
    model.evaluate();
    // A*B = [[58,  64],
    //        [139, 154]]
    assert_eq!(model._get_text("H1"), "58");
    assert_eq!(model._get_text("I1"), "64");
    assert_eq!(model._get_text("H2"), "139");
    assert_eq!(model._get_text("I2"), "154");
}

#[test]
fn mmult_dimension_mismatch() {
    let mut model = new_empty_model();
    // 1x2 times 3x1 -> mismatch
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("D1", "3");
    model._set("D2", "4");
    model._set("D3", "5");
    model._set("F1", "=MMULT(A1:B1, D1:D3)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#VALUE!");
}

#[test]
fn mmult_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=MMULT(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn mmult_non_numeric() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "\"x\"");
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("F1", "=MMULT(A1:B1, D1:D2)");
    model.evaluate();
    assert_eq!(model._get_text("F1"), "#VALUE!");
}
