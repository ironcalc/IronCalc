#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;

#[test]
fn batch_write_is_one_undo_step() {
    let mut model = new_empty_user_model();
    model
        .set_user_inputs(&[
            (0, 1, 1, "cat".to_string()),  // A1
            (0, 1, 2, "dog".to_string()),  // B1
            (0, 3, 5, "bird".to_string()), // E3 (scattered, non-contiguous)
        ])
        .unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "cat");
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "dog");
    assert_eq!(model.get_cell_content(0, 3, 5).unwrap(), "bird");

    // The defining property: a SINGLE undo reverts the whole batch, and it is the
    // only history entry.
    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "");
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "");
    assert_eq!(model.get_cell_content(0, 3, 5).unwrap(), "");
    assert!(!model.can_undo(), "the batch is a single history entry");

    // A SINGLE redo re-applies the whole batch.
    model.redo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "cat");
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "dog");
    assert_eq!(model.get_cell_content(0, 3, 5).unwrap(), "bird");
}

#[test]
fn batch_overwrite_undo_restores_prior_values() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "old-a").unwrap();
    model.set_user_input(0, 2, 2, "old-b").unwrap();

    model
        .set_user_inputs(&[
            (0, 1, 1, "new-a".to_string()),
            (0, 2, 2, "new-b".to_string()),
        ])
        .unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "new-a");
    assert_eq!(model.get_cell_content(0, 2, 2).unwrap(), "new-b");

    // A single undo restores BOTH prior (non-empty) values.
    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "old-a");
    assert_eq!(model.get_cell_content(0, 2, 2).unwrap(), "old-b");
    // The two seed edits are still individually undoable — the batch was one entry.
    assert!(model.can_undo());
}

#[test]
fn batch_recomputes_dependent_formula() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 3, "=A1+B1").unwrap(); // C1 = A1 + B1 -> 0
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "0");

    model
        .set_user_inputs(&[
            (0, 1, 1, "2".to_string()), // A1
            (0, 1, 2, "3".to_string()), // B1
        ])
        .unwrap();
    // One evaluate at the end of the batch reflects both inputs.
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "5");

    // A single undo reverts both inputs and the dependent recomputes back to 0.
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "0");
}

#[test]
fn empty_batch_is_a_noop() {
    let mut model = new_empty_user_model();
    model.set_user_inputs(&[]).unwrap();
    // An empty batch records no history entry.
    assert!(!model.can_undo());
}

#[test]
fn out_of_range_batch_is_rejected_without_mutating() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "seed").unwrap();

    // The second entry is out of range (row 0 is invalid — rows are 1-based).
    let result = model.set_user_inputs(&[
        (0, 2, 2, "written?".to_string()),
        (0, 0, 1, "bad".to_string()),
    ]);
    assert!(result.is_err());
    // All-or-nothing: the valid entry was NOT written, and no batch history entry
    // exists beyond the seed edit.
    assert_eq!(model.get_cell_content(0, 2, 2).unwrap(), "");
    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "");
    assert!(!model.can_undo(), "only the seed edit was ever recorded");
}

#[test]
fn batch_across_sheets_is_one_undo_step() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap(); // Sheet2 = index 1

    model
        .set_user_inputs(&[
            (0, 1, 1, "on-sheet1".to_string()),
            (1, 1, 1, "on-sheet2".to_string()),
        ])
        .unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "on-sheet1");
    assert_eq!(model.get_cell_content(1, 1, 1).unwrap(), "on-sheet2");

    // A single undo clears both sheets' cells (one history entry spans sheets).
    model.undo().unwrap();
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "");
    assert_eq!(model.get_cell_content(1, 1, 1).unwrap(), "");
    // Only the new_sheet remains undoable.
    assert!(model.can_undo());
}

#[test]
fn batch_propagates_as_one_diff_list() {
    let mut model = new_empty_user_model();
    model
        .set_user_inputs(&[(0, 1, 1, "a".to_string()), (0, 2, 2, "b".to_string())])
        .unwrap();
    let send_queue = model.flush_send_queue();

    // A fresh peer replays the batch as one external diff list.
    let mut peer = new_empty_user_model();
    peer.apply_external_diffs(&send_queue).unwrap();
    assert_eq!(peer.get_cell_content(0, 1, 1).unwrap(), "a");
    assert_eq!(peer.get_cell_content(0, 2, 2).unwrap(), "b");
}
