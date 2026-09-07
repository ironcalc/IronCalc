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
