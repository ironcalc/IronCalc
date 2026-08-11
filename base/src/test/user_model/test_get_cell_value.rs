#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::user_model::util::new_empty_user_model;

#[test]
fn get_cell_value_returns_raw_typed_values() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "=1/3").unwrap();
    model.set_user_input(0, 2, 1, "hello").unwrap();
    model.set_user_input(0, 3, 1, "=TRUE()").unwrap();
    model.evaluate();

    // full-precision f64, not the 15-digit display rounding
    assert_eq!(
        model.get_cell_value(0, 1, 1).unwrap(),
        CellValue::Number(1.0 / 3.0)
    );
    assert_eq!(
        model.get_cell_value(0, 2, 1).unwrap(),
        CellValue::String("hello".to_string())
    );
    assert_eq!(
        model.get_cell_value(0, 3, 1).unwrap(),
        CellValue::Boolean(true)
    );
    // empty cell
    assert_eq!(model.get_cell_value(0, 4, 1).unwrap(), CellValue::None);
    // wrong sheet is an error
    assert!(model.get_cell_value(1, 1, 1).is_err());
}
