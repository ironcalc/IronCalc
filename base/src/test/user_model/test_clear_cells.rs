#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, UserModel};

#[test]
fn basic() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "100$").unwrap();
    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        })
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("100$".to_string())
    );
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));

    model.set_user_input(0, 1, 1, "300").unwrap();
    // clear contents keeps the formatting
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("300$".to_string())
    );

    model
        .range_clear_all(&Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        })
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("300$".to_string())
    );
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
    model.set_user_input(0, 1, 1, "400").unwrap();
    // clear contents keeps the formatting
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("400".to_string())
    );
}

#[test]
fn clear_empty_cell() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        })
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
}

#[test]
fn clear_all_empty_cell() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model
        .range_clear_all(&Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        })
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("".to_string()));
}
