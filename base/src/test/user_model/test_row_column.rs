#![allow(clippy::unwrap_used)]

use crate::{test::util::new_empty_model, UserModel};

#[test]
fn simple_insert_row() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let (sheet, column) = (0, 5);
    for row in 1..5 {
        assert!(model.set_user_input(sheet, row, column, "123").is_ok());
    }
    assert!(model.insert_row(sheet, 3).is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        ""
    );

    assert!(model.undo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        "123"
    );
    assert!(model.redo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        ""
    );
}

#[test]
fn simple_insert_column() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let (sheet, row) = (0, 5);
    for column in 1..5 {
        assert!(model.set_user_input(sheet, row, column, "123").is_ok());
    }
    assert!(model.insert_column(sheet, 3).is_ok());
    assert_eq!(model.get_formatted_cell_value(sheet, row, 3).unwrap(), "");

    assert!(model.undo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, row, 3).unwrap(),
        "123"
    );
    assert!(model.redo().is_ok());
    assert_eq!(model.get_formatted_cell_value(sheet, row, 3).unwrap(), "");
}
