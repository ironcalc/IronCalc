#![allow(clippy::unwrap_used)]

use bitcode::decode;

use crate::{
    test::util::new_empty_model,
    user_model::history::{Diff, QueueDiffs},
    UserModel,
};

fn last_diff_list(model: &mut UserModel) -> Vec<Diff> {
    let bytes = model.flush_send_queue();
    let queue: Vec<QueueDiffs> = decode(&bytes).unwrap();
    // Get the last operation's diff list
    queue.last().unwrap().list.clone()
}

#[test]
fn diff_invariant_insert_rows() {
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    assert!(model.insert_rows(0, 5, 3).is_ok());

    let list = last_diff_list(&mut model);
    assert_eq!(list.len(), 3);
    for diff in list {
        match diff {
            Diff::InsertRow { sheet, row } => {
                assert_eq!(sheet, 0);
                assert_eq!(row, 5);
            }
            _ => panic!("Unexpected diff variant"),
        }
    }
}

#[test]
fn diff_invariant_insert_columns() {
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    assert!(model.insert_columns(0, 2, 4).is_ok());
    let list = last_diff_list(&mut model);
    assert_eq!(list.len(), 4);
    for diff in list {
        match diff {
            Diff::InsertColumn { sheet, column } => {
                assert_eq!(sheet, 0);
                assert_eq!(column, 2);
            }
            _ => panic!("Unexpected diff variant"),
        }
    }
}

#[test]
fn undo_redo_after_batch_delete() {
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    // Place values that will shift.
    model.set_user_input(0, 20, 1, "A").unwrap();
    model.set_user_input(0, 1, 20, "B").unwrap();

    // Fill the rows we are about to delete so the diff list can be produced
    for r in 10..15 {
        model.set_user_input(0, r, 1, "tmp").unwrap();
    }

    // Delete rows 10..14 and columns 5..8
    assert!(model.delete_rows(0, 10, 5).is_ok());
    assert!(model.delete_columns(0, 5, 4).is_ok());

    // Verify shift
    assert_eq!(model.get_formatted_cell_value(0, 15, 1).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(0, 1, 16).unwrap(), "B");

    // Undo
    model.undo().unwrap(); // columns
    model.undo().unwrap(); // rows
    assert_eq!(model.get_formatted_cell_value(0, 20, 1).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(0, 1, 20).unwrap(), "B");

    // Redo
    model.redo().unwrap(); // rows
    model.redo().unwrap(); // columns
    assert_eq!(model.get_formatted_cell_value(0, 15, 1).unwrap(), "A");
    assert_eq!(model.get_formatted_cell_value(0, 1, 16).unwrap(), "B");
}

#[test]
fn diff_order_delete_rows() {
    // Verifies that delete diffs are generated bottom-to-top
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    // Populate rows to delete
    for r in 5..10 {
        model.set_user_input(0, r, 1, &r.to_string()).unwrap();
    }

    assert!(model.delete_rows(0, 5, 5).is_ok());

    let list = last_diff_list(&mut model);
    assert_eq!(list.len(), 5);

    // Diffs should be in reverse order: 9, 8, 7, 6, 5
    let mut expected_row = 9;
    for diff in list {
        match diff {
            Diff::DeleteRow { row, .. } => {
                assert_eq!(row, expected_row);
                expected_row -= 1;
            }
            _ => panic!("Unexpected diff variant"),
        }
    }
}

#[test]
fn batch_operations_with_formulas() {
    // Verifies formulas update correctly after batch ops
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    model.set_user_input(0, 1, 1, "10").unwrap();
    model.set_user_input(0, 5, 1, "=A1*2").unwrap(); // Will become A3 after insert

    assert!(model.insert_rows(0, 2, 2).is_ok());

    // Formula should now reference A1 (unchanged) but be in row 7
    assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "20");
    assert_eq!(model.get_cell_content(0, 7, 1).unwrap(), "=A1*2");

    // Undo and verify formula is back at original position
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "20");
}

#[test]
fn edge_case_single_operation() {
    // Single row/column operations should still work correctly
    let base = new_empty_model();
    let mut model = UserModel::from_model(base);

    assert!(model.insert_rows(0, 1, 1).is_ok());
    let list = last_diff_list(&mut model);
    assert_eq!(list.len(), 1);

    assert!(model.insert_columns(0, 1, 1).is_ok());
    let list = last_diff_list(&mut model);
    assert_eq!(list.len(), 1);
}
