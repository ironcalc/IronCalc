#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, UserModel};

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
    assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("".to_string()));
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
    assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("".to_string()));
    model.redo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("35".to_string())
    );
}

#[test]
fn mixed_spills() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // D9 => ={1,2,3}
    model.set_user_input(0, 9, 4, "={34,35,3}").unwrap();
    // F6 => ={1;2;3;4}
    model.set_user_input(0, 6, 6, "={1;2;3;4}").unwrap();

    // F6 should be #SPILL!
    assert_eq!(
        model.get_formatted_cell_value(0, 6, 6),
        Ok("#SPILL!".to_string())
    );

    // We delete D9
    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 9,
            column: 4,
            width: 1,
            height: 1,
        })
        .unwrap();

    // F6 should be 1
    assert_eq!(model.get_formatted_cell_value(0, 6, 6), Ok("1".to_string()));

    // Now we undo that
    model.undo().unwrap();
    // F6 should be #SPILL!
    assert_eq!(
        model.get_formatted_cell_value(0, 6, 6),
        Ok("#SPILL!".to_string())
    );
}

#[test]
fn spill_order_d9_f6() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // D9 => ={1,2,3}
    model.set_user_input(0, 9, 4, "={34,35,3}").unwrap();
    // F6 => ={1;2;3;4}
    model.set_user_input(0, 6, 6, "={1;2;3;4}").unwrap();

    // F6 should be #SPILL!
    assert_eq!(
        model.get_formatted_cell_value(0, 6, 6),
        Ok("#SPILL!".to_string())
    );
}

#[test]
fn spill_order_f6_d9() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // F6 => ={1;2;3;4}
    model.set_user_input(0, 6, 6, "={1;2;3;4}").unwrap();
    // D9 => ={1,2,3}
    model.set_user_input(0, 9, 4, "={34,35,3}").unwrap();

    // D9 should be #SPILL!
    assert_eq!(
        model.get_formatted_cell_value(0, 9, 4),
        Ok("#SPILL!".to_string())
    );
}
