#![allow(clippy::unwrap_used)]

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn model_evaluates_automatically() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "=1 + 1").unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("2".to_string()));
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("=1+1".to_string()));
}

#[test]
fn set_user_input_errors() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // Wrong sheet
    assert!(model.set_user_input(1, 1, 1, "1").is_err());
    // Wrong row
    assert!(model.set_user_input(0, 0, 1, "1").is_err());
    // Wrong column
    assert!(model.set_user_input(0, 1, 0, "1").is_err());
    // row too large
    assert!(model.set_user_input(0, LAST_ROW, 1, "1").is_ok());
    assert!(model.set_user_input(0, LAST_ROW + 1, 1, "1").is_err());
    // column too large
    assert!(model.set_user_input(0, 1, LAST_COLUMN, "1").is_ok());
    assert!(model.set_user_input(0, 1, LAST_COLUMN + 1, "1").is_err());
}

#[test]
fn simple_undo_redo() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // at the beginning I cannot undo or redo
    assert!(!model.can_undo());
    assert!(!model.can_redo());
    assert!(model.set_user_input(0, 1, 1, "=1+2").is_ok());

    // Once I enter a value I can undo but not redo
    assert!(model.can_undo());
    assert!(!model.can_redo());
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("3".to_string()));

    // If I undo, I can't undo anymore, but I can redo
    assert!(model.undo().is_ok());
    assert!(!model.can_undo());
    assert!(model.can_redo());
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));

    // If I redo, I have the old value and formula
    assert!(model.redo().is_ok());
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("3".to_string()));
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("=1+2".to_string()));
    assert!(model.can_undo());
    assert!(!model.can_redo());
}

#[test]
fn undo_redo_respect_styles() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    assert!(model.set_user_input(0, 1, 1, "100").is_ok());
    assert!(model.set_user_input(0, 1, 1, "125$").is_ok());
    // The content of the cell is just the number 125
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("125".to_string()));
    assert!(model.undo().is_ok());
    // The cell has no currency number formatting
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("100".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("100".to_string()));
    assert!(model.redo().is_ok());
    // The cell has the number 125 formatted as '125$'
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("125$".to_string())
    );
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("125".to_string()));
}

#[test]
fn insert_remove_rows() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let height = model.get_row_height(0, 5).unwrap();

    // Insert some data in row 5 (and change the style)
    assert!(model.set_user_input(0, 5, 1, "100$").is_ok());
    // Change the height of the column
    assert!(model.set_row_height(0, 5, 3.0 * height).is_ok());

    // remove the row
    assert!(model.delete_row(0, 5).is_ok());
    // Row 5 has now the normal height
    assert_eq!(model.get_row_height(0, 5), Ok(height));
    // There is no value in A5
    assert_eq!(model.get_formatted_cell_value(0, 5, 1), Ok("".to_string()));
    // Setting a value will not format it
    assert!(model.set_user_input(0, 5, 1, "125").is_ok());
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 1),
        Ok("125".to_string())
    );

    // undo twice
    assert!(model.undo().is_ok());
    assert!(model.undo().is_ok());

    assert_eq!(model.get_row_height(0, 5), Ok(3.0 * height));
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 1),
        Ok("100$".to_string())
    );
}

#[test]
fn insert_remove_columns() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // column E
    let column_width = model.get_column_width(0, 5).unwrap();
    println!("{column_width}");

    // Insert some data in row 5 (and change the style) in E1
    assert!(model.set_user_input(0, 1, 5, "100$").is_ok());
    // Change the width of the column
    assert!(model.set_column_width(0, 5, 3.0 * column_width).is_ok());
    assert_eq!(model.get_column_width(0, 5).unwrap(), 3.0 * column_width);

    // remove the column
    assert!(model.delete_column(0, 5).is_ok());
    // Column 5 has now the normal width
    assert_eq!(model.get_column_width(0, 5), Ok(column_width));
    // There is no value in E5
    assert_eq!(model.get_formatted_cell_value(0, 1, 5), Ok("".to_string()));
    // Setting a value will not format it
    assert!(model.set_user_input(0, 1, 5, "125").is_ok());
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 5),
        Ok("125".to_string())
    );

    // undo twice (set_user_input and delete_column)
    assert!(model.undo().is_ok());
    assert!(model.undo().is_ok());

    assert_eq!(model.get_column_width(0, 5), Ok(3.0 * column_width));
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 5),
        Ok("100$".to_string())
    );
}
