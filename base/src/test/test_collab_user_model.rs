#![allow(clippy::unwrap_used)]

use crate::collab::model::{CollabModel, Stable};
use crate::UserModel;

#[test]
fn stable_user_model_reads_and_selects() {
    let mut model = CollabModel::new(1);
    model.new_sheet();
    // Input is driven on the model, since the wrapper's writers are still ordinal-only.
    model.set_user_input(0, 1, 1, "=1+1".to_string()).unwrap();
    model.evaluate();

    let mut user_model = UserModel::<Stable>::from_model(model);
    assert_eq!(user_model.get_formatted_cell_value(0, 1, 1).unwrap(), "2");

    // Every replica seeds the default view, so selection is live local state.
    user_model.set_selected_cell(2, 3).unwrap();
    assert_eq!(user_model.get_selected_sheet(), 0);
    assert_eq!(user_model.get_selected_cell(), (0, 2, 3));
}

#[test]
fn move_sheet_keeps_selection_and_undoes() {
    let mut model = CollabModel::new(1);
    for _ in 0..3 {
        model.new_sheet();
    }
    let mut user_model = UserModel::<Stable>::from_model(model);
    let names = |m: &UserModel<Stable>| -> Vec<String> {
        m.get_worksheets_properties()
            .into_iter()
            .map(|p| p.name)
            .collect()
    };

    user_model.set_selected_sheet(0).unwrap();
    user_model.move_sheet(0, 2).unwrap();
    assert_eq!(names(&user_model), ["Sheet2", "Sheet3", "Sheet1"]);
    // The selection follows the sheet, not the index it used to sit at.
    assert_eq!(user_model.get_selected_sheet(), 2);

    user_model.undo().unwrap();
    assert_eq!(names(&user_model), ["Sheet1", "Sheet2", "Sheet3"]);
    user_model.redo().unwrap();
    assert_eq!(names(&user_model), ["Sheet2", "Sheet3", "Sheet1"]);
}
