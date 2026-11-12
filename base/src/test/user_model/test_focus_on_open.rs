#![allow(clippy::unwrap_used)]

use crate::{expressions::types::Area, test::util::new_empty_model, UserModel};

#[test]
fn off_by_default() {
    let model = UserModel::from_model(new_empty_model());
    assert_eq!(model.get_focus_on_open(), None);
    assert_eq!(model.get_selected_cell(), (0, 1, 1));
}

#[test]
fn opens_on_the_cell() {
    let mut model = UserModel::from_model(new_empty_model());
    model.new_sheet().unwrap();
    model.set_focus_on_open(Some((1, 5, 3))).unwrap();
    assert_eq!(model.get_focus_on_open(), Some((1, 5, 3)));
    // Setting it does not move the selection
    model.set_selected_sheet(0).unwrap();
    assert_eq!(model.get_selected_cell(), (0, 1, 1));

    let model = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(model.get_focus_on_open(), Some((1, 5, 3)));
    assert_eq!(model.get_selected_cell(), (1, 5, 3));
    let view = model.get_selected_view();
    assert_eq!(view.range, [5, 3, 5, 3]);
    // The cell fits in the window: there is no need to scroll
    assert_eq!((view.top_row, view.left_column), (1, 1));
}

#[test]
fn scrolls_to_a_far_away_cell() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_focus_on_open(Some((0, 5000, 300))).unwrap();

    let model = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(model.get_selected_cell(), (0, 5000, 300));
    let view = model.get_selected_view();
    assert_eq!((view.top_row, view.left_column), (5000, 300));
}

#[test]
fn selects_the_whole_merged_cell() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .merge_cells(&Area {
            sheet: 0,
            row: 2,
            column: 2,
            width: 2,
            height: 3,
        })
        .unwrap();
    model.set_focus_on_open(Some((0, 3, 3))).unwrap();

    let model = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(model.get_selected_cell(), (0, 2, 2));
    assert_eq!(model.get_selected_view().range, [2, 2, 4, 3]);
}

#[test]
fn invalid_arguments() {
    let mut model = UserModel::from_model(new_empty_model());
    assert!(model.set_focus_on_open(Some((1, 1, 1))).is_err());
    assert!(model.set_focus_on_open(Some((0, 0, 1))).is_err());
    assert!(model.set_focus_on_open(Some((0, 1, 0))).is_err());
    assert_eq!(model.get_focus_on_open(), None);
    // Nothing went to the history
    assert!(!model.can_undo());
}

#[test]
fn undo_redo() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_focus_on_open(Some((0, 5, 3))).unwrap();
    model.set_focus_on_open(Some((0, 7, 7))).unwrap();
    // Setting the same value is not added to the history
    model.set_focus_on_open(Some((0, 7, 7))).unwrap();
    model.set_focus_on_open(None).unwrap();

    model.undo().unwrap();
    assert_eq!(model.get_focus_on_open(), Some((0, 7, 7)));
    model.undo().unwrap();
    assert_eq!(model.get_focus_on_open(), Some((0, 5, 3)));
    model.undo().unwrap();
    assert_eq!(model.get_focus_on_open(), None);
    assert!(!model.can_undo());

    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(model.get_focus_on_open(), Some((0, 7, 7)));
    model.redo().unwrap();
    assert_eq!(model.get_focus_on_open(), None);
}

#[test]
fn follows_the_sheet() {
    let mut model = UserModel::from_model(new_empty_model());
    model.new_sheet().unwrap();
    model.new_sheet().unwrap();
    model.set_focus_on_open(Some((2, 5, 3))).unwrap();

    // Deleting a sheet before it shifts its index
    model.delete_sheet(0).unwrap();
    assert_eq!(model.get_focus_on_open(), Some((1, 5, 3)));

    // Deleting the sheet turns the setting off...
    model.delete_sheet(1).unwrap();
    assert_eq!(model.get_focus_on_open(), None);
    let reopened = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(reopened.get_selected_cell(), (0, 1, 1));

    // ...and it is back if the sheet is back
    model.undo().unwrap();
    assert_eq!(model.get_focus_on_open(), Some((1, 5, 3)));
}

#[test]
fn hidden_sheet_is_ignored() {
    let mut model = UserModel::from_model(new_empty_model());
    model.new_sheet().unwrap();
    model.set_focus_on_open(Some((1, 5, 3))).unwrap();
    model.hide_sheet(1).unwrap();
    // The setting is kept
    assert_eq!(model.get_focus_on_open(), Some((1, 5, 3)));

    let model = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(model.get_selected_cell(), (0, 1, 1));
}

#[test]
fn applies_to_other_peers() {
    let mut model1 = UserModel::from_model(new_empty_model());
    let mut model2 = UserModel::from_model(new_empty_model());
    model1.set_focus_on_open(Some((0, 5, 3))).unwrap();
    model2
        .apply_external_diffs(&model1.flush_send_queue())
        .unwrap();
    assert_eq!(model2.get_focus_on_open(), Some((0, 5, 3)));
}
