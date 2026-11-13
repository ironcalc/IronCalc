#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn var_leaves_out_hidden_rows() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "8");
    model._set("A3", "4");
    model.set_row_hidden(0, 2, true).unwrap();
    model._set("B1", "=SUBTOTAL(10,A1:A3)");
    model._set("B2", "=SUBTOTAL(11,A1:A3)");
    model._set("B3", "=SUBTOTAL(110,A1:A3)");
    model._set("B4", "=SUBTOTAL(111,A1:A3)");
    model.evaluate();
    // 10 and 11 take the three numbers
    assert_eq!(model._get_text("B1"), "9.333333333");
    assert_eq!(model._get_text("B2"), "6.222222222");
    // 110 and 111 only 2 and 4
    assert_eq!(model._get_text("B3"), "2");
    assert_eq!(model._get_text("B4"), "1");
}

#[test]
fn product_of_no_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "2");
    model._set("A2", "8");
    // B1:B2 holds no number
    model._set("B1", "x");
    model._set("C1", "=SUBTOTAL(6,A1:A2)");
    model._set("C2", "=SUBTOTAL(6,B1:B2)");
    model._set("C3", "=SUBTOTAL(106,B1:B2)");
    model._set("C4", "=PRODUCT(B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "16");
    assert_eq!(model._get_text("C2"), "0");
    assert_eq!(model._get_text("C3"), "0");
    assert_eq!(model._get_text("C4"), "0");
}
