#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, UserModel};

#[test]
fn clear_cell_contents_evaluates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 1, 2, "=A1").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("42".to_string())
    );
    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 1,
        })
        .unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("0".to_string()));
}

#[test]
fn clear_cell_all_evaluates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 1, 2, "=A1").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("42".to_string())
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

    assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("0".to_string()));
}
