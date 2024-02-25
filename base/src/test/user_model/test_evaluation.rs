#![allow(clippy::unwrap_used)]

use crate::UserModel;

#[test]
fn model_evaluates_automatically() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "=1 + 1").unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("2".to_string()));
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("=1+1".to_string()));
}

#[test]
fn pause_resume_evaluation() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model.pause_evaluation();
    model.set_user_input(0, 1, 1, "=1+1").unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok("#ERROR!".to_string())
    );
    model.evaluate();

    assert_eq!(model.get_formatted_cell_value(0, 1, 1), Ok("2".to_string()));
    assert_eq!(model.get_cell_content(0, 1, 1), Ok("=1+1".to_string()));

    model.resume_evaluation();
    model.set_user_input(0, 2, 1, "=1+4").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 1), Ok("5".to_string()));
}
