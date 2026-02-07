#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn cun_n_paste_same_area() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // B3:D5 with data
    model.set_user_input(0, 3, 2, "A").unwrap();
    model.set_user_input(0, 3, 3, "B").unwrap();
    model.set_user_input(0, 3, 4, "C").unwrap();
    model.set_user_input(0, 4, 2, "D").unwrap();
    model.set_user_input(0, 4, 3, "E").unwrap();
    model.set_user_input(0, 4, 4, "F").unwrap();
    model.set_user_input(0, 5, 2, "G").unwrap();
    model.set_user_input(0, 5, 3, "H").unwrap();
    model.set_user_input(0, 5, 4, "I").unwrap();

    // Cut it and paste it in C4
    model.set_selected_cell(3, 2).unwrap();
    model.set_selected_range(3, 2, 5, 4).unwrap();
    let cp = model.copy_to_clipboard().unwrap();

    // C4
    model.set_selected_cell(4, 3).unwrap();

    let source_range = (3, 2, 5, 4);
    model
        .paste_from_clipboard(0, source_range, &cp.data, true)
        .unwrap();

    // Check data is in C4:E6
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(0, 4, 4).unwrap(), "B");
    assert_eq!(model.get_formatted_cell_value(0, 4, 5).unwrap(), "C");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "D");
    assert_eq!(model.get_formatted_cell_value(0, 5, 4).unwrap(), "E");
    assert_eq!(model.get_formatted_cell_value(0, 5, 5).unwrap(), "F");
    assert_eq!(model.get_formatted_cell_value(0, 6, 3).unwrap(), "G");
    assert_eq!(model.get_formatted_cell_value(0, 6, 4).unwrap(), "H");
    assert_eq!(model.get_formatted_cell_value(0, 6, 5).unwrap(), "I");
}

#[test]
fn cun_n_paste_different_sheet() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // B3:D5 with data
    model.set_user_input(0, 3, 2, "A").unwrap();
    model.set_user_input(0, 3, 3, "B").unwrap();
    model.set_user_input(0, 3, 4, "C").unwrap();
    model.set_user_input(0, 4, 2, "D").unwrap();
    model.set_user_input(0, 4, 3, "E").unwrap();
    model.set_user_input(0, 4, 4, "F").unwrap();
    model.set_user_input(0, 5, 2, "G").unwrap();
    model.set_user_input(0, 5, 3, "H").unwrap();
    model.set_user_input(0, 5, 4, "I").unwrap();

    // Cut it and paste it in C4
    model.set_selected_cell(3, 2).unwrap();
    model.set_selected_range(3, 2, 5, 4).unwrap();
    let cp = model.copy_to_clipboard().unwrap();

    // New sheet and select it
    model.new_sheet().unwrap();
    model.set_selected_sheet(1).unwrap();

    // C4
    model.set_selected_cell(4, 3).unwrap();

    let source_range = (3, 2, 5, 4);
    model
        .paste_from_clipboard(0, source_range, &cp.data, true)
        .unwrap();

    // Check data is in Sheet2!C4:E6
    assert_eq!(model.get_formatted_cell_value(1, 4, 3).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(1, 4, 4).unwrap(), "B");
    assert_eq!(model.get_formatted_cell_value(1, 4, 5).unwrap(), "C");
    assert_eq!(model.get_formatted_cell_value(1, 5, 3).unwrap(), "D");
    assert_eq!(model.get_formatted_cell_value(1, 5, 4).unwrap(), "E");
    assert_eq!(model.get_formatted_cell_value(1, 5, 5).unwrap(), "F");
    assert_eq!(model.get_formatted_cell_value(1, 6, 3).unwrap(), "G");
    assert_eq!(model.get_formatted_cell_value(1, 6, 4).unwrap(), "H");
    assert_eq!(model.get_formatted_cell_value(1, 6, 5).unwrap(), "I");

    // Check original range is empty Sheet1!B3:D5
    assert_eq!(model.get_formatted_cell_value(0, 3, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 3, 4).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 4, 4).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 2).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 3).unwrap(), "");
    assert_eq!(model.get_formatted_cell_value(0, 5, 4).unwrap(), "");
}
