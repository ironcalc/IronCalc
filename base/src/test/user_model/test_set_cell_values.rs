#![allow(clippy::unwrap_used)]

use crate::expressions::token::Error;
use crate::expressions::types::Area;
use crate::test::user_model::util::new_empty_user_model;
use crate::types::FormulaValue;
use crate::UserModel;

#[test]
fn set_cell_values_clear_spills() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "=SEQUENCE(3)").unwrap();
    model.set_user_input(0, 1, 3, "=SEQUENCE(3)").unwrap();

    // A value in a cell the formula spills into blocks the spill
    model.set_cell_number(0, 2, 1, 7.0).unwrap();
    // A value over the formula itself removes its spill
    model.set_cell_text(0, 1, 3, "x").unwrap();
    model.evaluate();

    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#SPILL!");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "7");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "x");
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");
}

#[test]
fn set_cell_values_refuse_cells_that_cannot_be_written() {
    let mut model = new_empty_user_model();
    let merged = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 2,
        height: 1,
    };
    model.merge_cells(&merged).unwrap();
    model
        .set_user_array_formula(0, 3, 1, 1, 2, "={1;2}")
        .unwrap();

    // Covered by a merged cell
    assert!(model.set_cell_text(0, 1, 2, "x").is_err());
    // Inside an array formula
    assert!(model.set_cell_number(0, 4, 1, 1.0).is_err());
    // Out of the sheet
    assert!(model.set_cell_boolean(0, 0, 1, true).is_err());
    assert!(model.set_cell_input(0, 1, 0, "1").is_err());

    // Not a formula: the formula that was there stays, with its spill
    model.set_user_input(0, 10, 1, "=SEQUENCE(2)").unwrap();
    assert!(model.set_cell_formula(0, 10, 1, "1+1").is_err());
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 11, 1).unwrap(), "2");
}

#[test]
fn set_cell_values_are_not_undoable() {
    let mut model = new_empty_user_model();
    model.set_cell_text(0, 1, 1, "a").unwrap();
    model.set_cell_number(0, 1, 2, 1.0).unwrap();
    model.set_cell_boolean(0, 1, 3, true).unwrap();
    model.set_cell_formula(0, 1, 4, "=B1+1").unwrap();
    model.set_cell_input(0, 1, 5, "12%").unwrap();
    model.evaluate();

    assert!(!model.can_undo());
    assert_eq!(model.get_formatted_cell_value(0, 1, 4).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 1, 5).unwrap(), "12%");
}

#[test]
fn formulas_are_english_whatever_the_locale() {
    let mut model = UserModel::new_empty("model", "fr", "UTC", "fr").unwrap();
    model.set_cell_number(0, 1, 1, 1.5).unwrap();
    model.set_cell_formula(0, 1, 2, "=SUM(A1,0.5)").unwrap();
    // The language and locale of the model are back for what the user types
    model.set_user_input(0, 2, 2, "=SOMME(A1;0,5)").unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "2");
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "2");

    // Evaluating a formula in a cell leaves the model as it is
    let value = model.evaluate_formula(0, 1, 1, "=A1*2+SUM(1,0.5)").unwrap();
    assert_eq!(value, FormulaValue::Number(4.5));
    let value = model.evaluate_formula(0, 1, 1, "=1/0").unwrap();
    assert!(matches!(value, FormulaValue::Error { ei: Error::DIV, .. }));
    assert!(model.evaluate_formula(0, 1, 1, "=SUM(1,").is_err());
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "1,5");
}
