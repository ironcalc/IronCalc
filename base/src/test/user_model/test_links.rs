#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::Link;
use crate::UserModel;

fn example_link() -> Link {
    Link::External {
        target: "https://www.ironcalc.com/".to_string(),
        tooltip: None,
    }
}

#[test]
fn add_update_delete() {
    let mut model = UserModel::from_model(new_empty_model());

    // no link in an empty cell
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));

    // add
    model.set_cell_link(0, 2, 2, example_link()).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    assert_eq!(model.get_links(0).unwrap().len(), 1);

    // update
    let updated = Link::Internal {
        location: "Sheet1!A30".to_string(),
        tooltip: Some("An internal link".to_string()),
    };
    model.set_cell_link(0, 2, 2, updated.clone()).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated)));
    assert_eq!(model.get_links(0).unwrap().len(), 1);

    // delete
    model.delete_cell_link(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert!(model.get_links(0).unwrap().is_empty());
}

#[test]
fn invalid_references() {
    let mut model = UserModel::from_model(new_empty_model());
    assert!(model.set_cell_link(1, 1, 1, example_link()).is_err());
    assert!(model.set_cell_link(0, 0, 1, example_link()).is_err());
    assert!(model.set_cell_link(0, 1, 20_000, example_link()).is_err());
    assert!(model.get_cell_link(0, -1, 1).is_err());
}

#[test]
fn undo_redo() {
    let mut model = UserModel::from_model(new_empty_model());

    model.set_cell_link(0, 2, 2, example_link()).unwrap();
    let updated = Link::Internal {
        location: "Sheet1!A30".to_string(),
        tooltip: None,
    };
    model.set_cell_link(0, 2, 2, updated.clone()).unwrap();

    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));

    model.redo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    model.redo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated.clone())));

    // deleting is undoable too
    model.delete_cell_link(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated)));
}

#[test]
fn no_op_operations_do_not_pollute_history() {
    let mut model = UserModel::from_model(new_empty_model());

    // deleting a link that is not there is a no-op
    model.delete_cell_link(0, 5, 5).unwrap();
    assert!(!model.can_undo());

    // setting the same link twice records only one history entry
    model.set_cell_link(0, 2, 2, example_link()).unwrap();
    model.set_cell_link(0, 2, 2, example_link()).unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert!(!model.can_undo());
}

#[test]
fn diffs_are_sent_to_other_models() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_cell_link(0, 2, 2, example_link()).unwrap();

    let send_queue = model.flush_send_queue();
    let mut model2 = UserModel::from_model(new_empty_model());
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(model2.get_cell_link(0, 2, 2), Ok(Some(example_link())));
}
