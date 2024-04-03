#![allow(clippy::unwrap_used)]

use crate::{constants::DEFAULT_COLUMN_WIDTH, UserModel};

#[test]
fn add_undo_redo() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();
    model.set_user_input(1, 1, 1, "=1 + 1").unwrap();
    model.set_user_input(1, 1, 2, "=A1*3").unwrap();
    model
        .set_column_width(1, 5, 5.0 * DEFAULT_COLUMN_WIDTH)
        .unwrap();
    model.new_sheet();
    model.set_user_input(2, 1, 1, "=Sheet2!B1").unwrap();

    model.undo().unwrap();
    model.undo().unwrap();

    assert!(model.get_formatted_cell_value(2, 1, 1).is_err());

    model.redo().unwrap();
    model.redo().unwrap();

    assert_eq!(model.get_formatted_cell_value(2, 1, 1), Ok("6".to_string()));

    model.delete_sheet(1).unwrap();

    assert!(!model.can_undo());
    assert!(!model.can_redo());
}

#[test]
fn new_sheet_propagates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();
    let sheets_info = model2.get_sheets_info();
    assert_eq!(sheets_info.len(), 2);
}

#[test]
fn delete_sheet_propagates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();
    model.delete_sheet(0).unwrap();

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();
    let sheets_info = model2.get_sheets_info();
    assert_eq!(sheets_info.len(), 1);
}
