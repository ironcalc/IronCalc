#![allow(clippy::unwrap_used)]

use crate::UserModel;

// Tests basic behavour.
#[test]
fn basic() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We put a value by the dynamic array to check the border conditions
    model.set_user_input(0, 2, 1, "22").unwrap();
    model.set_user_input(0, 1, 1, "={34,35,3}").unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("34".to_string())
    );
}

// Test that overwriting a dynamic array with a single value dissolves the array
#[test]
fn sett_user_input_mother() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "={34,35,3}").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("35".to_string())
    );
    model.set_user_input(0, 1, 1, "123").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("".to_string())
    );
}

#[test]
fn set_user_input_sibling() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "={43,55,34}").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("55".to_string())
    );
    // This does nothing
    model.set_user_input(0, 1, 2, "123").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("55".to_string())
    );
}

#[test]
fn basic_undo_redo() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "={34,35,3}").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("35".to_string())
    );
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("".to_string())
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("35".to_string())
    );
}