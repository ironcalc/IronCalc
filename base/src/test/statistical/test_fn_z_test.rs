#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_z_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "6");
    model._set("A4", "7");
    model._set("A5", "8");
    model._set("A6", "6");
    model._set("A7", "5");
    model._set("A8", "4");
    model._set("A9", "2");
    model._set("A10", "1");
    model._set("A11", "9");

    model._set("G1", "=Z.TEST(A2:A11, 4)");
    model._set("G2", "=Z.TEST(A2:A11, 6)");
    model.evaluate();

    assert_eq!(model._get_text("G1"), *"0.090574197");
    assert_eq!(model._get_text("G2"), *"0.863043389");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model._set("D1", "=Z.TEST({5,2,3,4}, 4, 123)");
    model._set("D2", "=Z.TEST({5,2,3,4}, 4)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"0.503243397");
    assert_eq!(model._get_text("D2"), *"0.780710987");
}

// A whole column is read only as far as the used area: the cells past it are
// empty, and Z.TEST skips empty cells.
#[test]
fn test_fn_z_test_whole_column() {
    let mut model = new_empty_model();
    for (row, x) in [3, 5, 2, 8, 6].iter().enumerate() {
        model._set(&format!("D{}", row + 1), &x.to_string());
    }
    model._set("A1", "=Z.TEST(D:D,4)=Z.TEST(D1:D5,4)");
    model._set("A2", "=Z.TEST(D:D,4,2)=Z.TEST(D1:D5,4,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "TRUE");
}
