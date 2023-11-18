#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::CellType;

#[test]
fn test_cell_get_type() {
    let mut model = new_empty_model();
    model._set("A1", "");
    model._set("A2", "42");
    model._set("A3", "12.34");
    model._set("A4", "foobar");
    model._set("A5", "1+2");
    model._set("A6", "TRUE");
    model._set("A7", "#VALUE!");
    model._set("A8", "=Z100"); // an empty cell, considered to be a CellType::Number
    model._set("A9", "=2*3*7");
    model._set("A10", "=\"foo\"");
    model._set("A11", "=1/0");
    model._set("A12", "=1>0");
    model.evaluate();

    assert_eq!(model._get_cell("A1").get_type(), CellType::Text);
    assert_eq!(model._get_cell("A2").get_type(), CellType::Number);
    assert_eq!(model._get_cell("A3").get_type(), CellType::Number);
    assert_eq!(model._get_cell("A4").get_type(), CellType::Text);
    assert_eq!(model._get_cell("A5").get_type(), CellType::Text);
    assert_eq!(model._get_cell("A6").get_type(), CellType::LogicalValue);
    assert_eq!(model._get_cell("A7").get_type(), CellType::ErrorValue);
    assert_eq!(model._get_cell("A8").get_type(), CellType::Number);
    assert_eq!(model._get_cell("A9").get_type(), CellType::Number);
    assert_eq!(model._get_cell("A10").get_type(), CellType::Text);
    assert_eq!(model._get_cell("A11").get_type(), CellType::ErrorValue);
    assert_eq!(model._get_cell("A12").get_type(), CellType::LogicalValue);
}
