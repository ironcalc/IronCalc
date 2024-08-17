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
fn set_sheet_color() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_sheet_color(0, "#343434").unwrap();
    let worksheets_properties = model.get_worksheets_properties();
    assert_eq!(worksheets_properties.len(), 1);
    assert_eq!(worksheets_properties[0].color, Some("#343434".to_owned()));
    model.undo().unwrap();
    assert_eq!(model.get_worksheets_properties()[0].color, None);

    model.redo().unwrap();
    assert_eq!(
        model.get_worksheets_properties()[0].color,
        Some("#343434".to_owned())
    );
    // changes the color if there is one
    model.set_sheet_color(0, "#2534FF").unwrap();
    assert_eq!(
        model.get_worksheets_properties()[0].color,
        Some("#2534FF".to_owned())
    );
    // Setting it back to none
    model.set_sheet_color(0, "").unwrap();
    assert_eq!(model.get_worksheets_properties()[0].color, None);
}

#[test]
fn new_sheet_propagates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();
    let worksheets_properties = model2.get_worksheets_properties();
    assert_eq!(worksheets_properties.len(), 2);
}

#[test]
fn delete_sheet_propagates() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();
    model.delete_sheet(0).unwrap();

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();
    let sheets_info = model2.get_worksheets_properties();
    assert_eq!(sheets_info.len(), 1);
}

#[test]
fn delete_last_sheet() {
    // Deleting the last sheet, selects the previous
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.new_sheet();
    model.new_sheet();
    model.set_selected_sheet(2).unwrap();
    model.delete_sheet(2).unwrap();

    assert_eq!(model.get_selected_sheet(), 1);
}
