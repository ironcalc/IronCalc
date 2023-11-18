#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_cell_currency_dollar() {
    let mut model = new_empty_model();
    model._set("A1", "=PMT(8/1200,10,10000)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "-$1,037.03");

    assert!(model.set_currency("EUR").is_ok());
}

#[test]
fn test_cell_currency_euro() {
    let mut model = new_empty_model();
    assert!(model.set_currency("EUR").is_ok());
    model._set("A1", "=PMT(8/1200,10,10000)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "-€1,037.03");
}
