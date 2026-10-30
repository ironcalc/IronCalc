#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::Comment;
use crate::UserModel;

fn text_at(model: &UserModel, row: i32, column: i32) -> Option<String> {
    model
        .get_cell_comment(0, row, column)
        .unwrap()
        .map(|c| c.text)
}

#[test]
fn add_update_delete() {
    let mut model = UserModel::from_model(new_empty_model());
    assert_eq!(model.get_cell_comment(0, 2, 3), Ok(None));

    model.set_cell_comment(0, 2, 3, "First", "Ann").unwrap();
    assert_eq!(
        model.get_cell_comment(0, 2, 3),
        Ok(Some(Comment {
            text: "First".to_string(),
            author_name: "Ann".to_string(),
            author_id: None,
            cell_ref: "C2".to_string(),
        }))
    );

    model.set_cell_comment(0, 2, 3, "Second", "Bo").unwrap();
    assert_eq!(text_at(&model, 2, 3), Some("Second".to_string()));
    assert_eq!(model.get_comments(0).unwrap().len(), 1);

    model.delete_cell_comment(0, 2, 3).unwrap();
    assert_eq!(model.get_cell_comment(0, 2, 3), Ok(None));
    assert!(model.get_comments(0).unwrap().is_empty());

    // deleting nothing is fine
    model.delete_cell_comment(0, 2, 3).unwrap();
}

#[test]
fn invalid_cells_are_errors() {
    let mut model = UserModel::from_model(new_empty_model());
    assert!(model.set_cell_comment(0, 0, 1, "x", "").is_err());
    assert!(model.set_cell_comment(0, 1, 20_000, "x", "").is_err());
    assert!(model.set_cell_comment(1, 1, 1, "x", "").is_err());
    assert!(model.get_cell_comment(0, -1, 1).is_err());
}

#[test]
fn undo_redo() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 2, 3, "First", "Ann").unwrap();
    model.set_cell_comment(0, 2, 3, "Second", "Ann").unwrap();
    model.delete_cell_comment(0, 2, 3).unwrap();

    model.undo().unwrap();
    assert_eq!(text_at(&model, 2, 3), Some("Second".to_string()));
    model.undo().unwrap();
    assert_eq!(text_at(&model, 2, 3), Some("First".to_string()));
    model.undo().unwrap();
    assert_eq!(text_at(&model, 2, 3), None);

    model.redo().unwrap();
    assert_eq!(text_at(&model, 2, 3), Some("First".to_string()));
    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(text_at(&model, 2, 3), None);
}

#[test]
fn setting_the_same_comment_adds_no_history() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 2, 3, "Same", "Ann").unwrap();
    model.set_cell_comment(0, 2, 3, "Same", "Ann").unwrap();
    model.undo().unwrap();
    assert_eq!(text_at(&model, 2, 3), None);
}

#[test]
fn comments_follow_inserted_rows_and_columns() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 5, 2, "B5", "").unwrap();
    model.set_cell_comment(0, 1, 1, "A1", "").unwrap();

    model.insert_rows(0, 3, 2).unwrap();
    assert_eq!(text_at(&model, 7, 2), Some("B5".to_string()));
    assert_eq!(text_at(&model, 5, 2), None);
    assert_eq!(text_at(&model, 1, 1), Some("A1".to_string()));

    model.insert_columns(0, 2, 1).unwrap();
    assert_eq!(text_at(&model, 7, 3), Some("B5".to_string()));
    assert_eq!(
        model.get_cell_comment(0, 7, 3).unwrap().unwrap().cell_ref,
        "C7"
    );

    model.undo().unwrap();
    model.undo().unwrap();
    assert_eq!(text_at(&model, 5, 2), Some("B5".to_string()));
}

#[test]
fn deleting_rows_removes_their_comments_and_undo_restores_them() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 3, 1, "deleted", "Ann").unwrap();
    model.set_cell_comment(0, 6, 1, "below", "").unwrap();

    model.delete_rows(0, 2, 3).unwrap();
    assert_eq!(text_at(&model, 3, 1), Some("below".to_string()));
    assert_eq!(model.get_comments(0).unwrap().len(), 1);

    model.undo().unwrap();
    assert_eq!(text_at(&model, 3, 1), Some("deleted".to_string()));
    assert_eq!(text_at(&model, 6, 1), Some("below".to_string()));
    assert_eq!(model.get_comments(0).unwrap().len(), 2);

    model.redo().unwrap();
    assert_eq!(text_at(&model, 3, 1), Some("below".to_string()));
    assert_eq!(model.get_comments(0).unwrap().len(), 1);
}

#[test]
fn deleting_columns_removes_their_comments_and_undo_restores_them() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 1, 2, "deleted", "").unwrap();
    model.set_cell_comment(0, 1, 4, "right", "").unwrap();

    model.delete_columns(0, 2, 1).unwrap();
    assert_eq!(text_at(&model, 1, 3), Some("right".to_string()));
    assert_eq!(model.get_comments(0).unwrap().len(), 1);

    model.undo().unwrap();
    assert_eq!(text_at(&model, 1, 2), Some("deleted".to_string()));
    assert_eq!(text_at(&model, 1, 4), Some("right".to_string()));
}

#[test]
fn comments_follow_moved_rows_and_columns() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 2, 1, "moved", "").unwrap();
    model.set_cell_comment(0, 3, 1, "between", "").unwrap();

    // row 2 moves two rows down, row 3 moves up into its place
    model.move_rows_action(0, 2, 1, 2).unwrap();
    assert_eq!(text_at(&model, 4, 1), Some("moved".to_string()));
    assert_eq!(text_at(&model, 2, 1), Some("between".to_string()));

    model.undo().unwrap();
    assert_eq!(text_at(&model, 2, 1), Some("moved".to_string()));
    assert_eq!(text_at(&model, 3, 1), Some("between".to_string()));

    model.set_cell_comment(0, 1, 3, "column C", "").unwrap();
    model.move_columns_action(0, 3, 1, -2).unwrap();
    assert_eq!(text_at(&model, 1, 1), Some("column C".to_string()));
}

#[test]
fn comments_survive_to_bytes() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_comment(0, 2, 3, "Kept", "Ann").unwrap();
    let model = UserModel::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(text_at(&model, 2, 3), Some("Kept".to_string()));
}
