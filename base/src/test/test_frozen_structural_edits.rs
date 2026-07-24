#![allow(clippy::unwrap_used)]

//! Structural edits (insert/delete rows and columns) track the frozen-pane boundary the way
//! Excel does: inserting within or above the frozen band grows it, deleting within it shrinks
//! it, and edits entirely past the band leave it unchanged. The adjustment rides the same single
//! undo step as the insert/delete itself.

use crate::test::util::new_empty_model;
use crate::UserModel;

fn frozen_model() -> UserModel<'static> {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_frozen_rows_count(0, 3).unwrap();
    model.set_frozen_columns_count(0, 2).unwrap();
    model
}

#[test]
fn insert_rows_above_band_grows_and_round_trips() {
    let mut model = frozen_model();
    // Insert one row above the whole band (row 1, 1-based).
    model.insert_rows(0, 1, 1).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 4);
    // Columns are untouched by a row insert.
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);

    model.undo().unwrap();
    assert_eq!(
        model.get_frozen_rows_count(0).unwrap(),
        3,
        "undo restores M"
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_frozen_rows_count(0).unwrap(),
        4,
        "redo re-applies"
    );
}

#[test]
fn insert_rows_within_band_grows_by_count() {
    let mut model = frozen_model();
    // Insert two rows within the band (before row 2).
    model.insert_rows(0, 2, 2).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 5);
}

#[test]
fn insert_rows_at_last_frozen_track_grows() {
    let mut model = frozen_model();
    // Row 3 is the last frozen track (pos == frozen_rows, the `<=` upper edge): still within
    // the band, so the boundary grows.
    model.insert_rows(0, 3, 1).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 4);
}

#[test]
fn insert_rows_just_below_band_leaves_it() {
    let mut model = frozen_model();
    // Row 4 is the first scrollable row (M + 1); inserting there is below the band.
    model.insert_rows(0, 4, 1).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);
}

#[test]
fn insert_rows_far_below_band_leaves_it() {
    let mut model = frozen_model();
    model.insert_rows(0, 50, 3).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);
}

#[test]
fn delete_rows_within_band_shrinks_and_round_trips() {
    let mut model = frozen_model();
    // Delete one frozen row (row 2).
    model.delete_rows(0, 2, 1).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 2);

    model.undo().unwrap();
    assert_eq!(
        model.get_frozen_rows_count(0).unwrap(),
        3,
        "undo restores the shrunk band"
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_frozen_rows_count(0).unwrap(),
        2,
        "redo re-shrinks"
    );
}

#[test]
fn delete_entire_band_collapses_to_zero_and_undo_restores() {
    let mut model = frozen_model();
    model.delete_rows(0, 1, 3).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 0);
    // Undo must regrow the fully-collapsed band — insert_rows alone cannot, so the snapshot does.
    model.undo().unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);
}

#[test]
fn delete_rows_spanning_band_boundary_removes_only_frozen_part() {
    let mut model = frozen_model();
    // Delete rows 2..=4: rows 2 and 3 are frozen (2 of them), row 4 is not.
    model.delete_rows(0, 2, 3).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 1);
    model.undo().unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);
}

#[test]
fn delete_rows_below_band_leaves_it() {
    let mut model = frozen_model();
    model.delete_rows(0, 10, 2).unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);
}

#[test]
fn insert_columns_within_band_grows_and_round_trips() {
    let mut model = frozen_model();
    model.insert_columns(0, 1, 2).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 4);
    // Rows untouched by a column insert.
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 3);

    model.undo().unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
    model.redo().unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 4);
}

#[test]
fn insert_columns_at_last_frozen_track_grows() {
    let mut model = frozen_model();
    // Column 2 is the last frozen track (pos == frozen_columns, the `<=` upper edge): still
    // within the band, so the boundary grows.
    model.insert_columns(0, 2, 1).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 3);
}

#[test]
fn insert_columns_below_band_leaves_it() {
    let mut model = frozen_model();
    // Column 3 is the first scrollable column (K + 1).
    model.insert_columns(0, 3, 1).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
}

#[test]
fn delete_columns_spanning_band_boundary_removes_only_frozen_part() {
    let mut model = frozen_model();
    // Delete columns 2..=3: column 2 is frozen (1 of them), column 3 is not.
    model.delete_columns(0, 2, 2).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 1);
    model.undo().unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
}

#[test]
fn delete_columns_within_band_shrinks_and_undo_restores() {
    let mut model = frozen_model();
    model.delete_columns(0, 1, 1).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 1);
    model.undo().unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
}

#[test]
fn delete_entire_column_band_collapses_and_undo_restores() {
    let mut model = frozen_model();
    model.delete_columns(0, 1, 2).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 0);
    model.undo().unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
}

#[test]
fn delete_columns_right_of_band_leaves_it() {
    let mut model = frozen_model();
    model.delete_columns(0, 10, 2).unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 2);
}
