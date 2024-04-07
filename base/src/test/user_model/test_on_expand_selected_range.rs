#![allow(clippy::unwrap_used)]

use crate::{
    constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_WINDOW_WIDTH, LAST_COLUMN},
    test::util::new_empty_model,
    UserModel,
};

#[test]
fn arrow_right() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.on_expand_selected_range("ArrowRight").unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [1, 1, 1, 2]);
}

#[test]
fn arrow_right_decreases() {
    // if the selected cell is on the upper right corner, right-arrow will decrease the size of teh area
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let (start_row, start_column, end_row, end_column) = (5, 3, 10, 8);
    model.set_selected_cell(start_row, end_column).unwrap();
    model
        .set_selected_range(start_row, start_column, end_row, end_column)
        .unwrap();

    model.on_expand_selected_range("ArrowRight").unwrap();
    let view = model.get_selected_view();
    assert_eq!(
        view.range,
        [start_row, start_column + 1, end_row, end_column]
    );
}

#[test]
fn arrow_right_last_column() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_selected_cell(1, LAST_COLUMN).unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [1, LAST_COLUMN, 1, LAST_COLUMN]);
}

#[test]
fn arrow_right_scroll_right() {
    let window_width = DEFAULT_WINDOW_WIDTH as f64;
    let column_width = DEFAULT_COLUMN_WIDTH;
    let column_count = f64::floor(window_width / column_width) as i32;

    let model = new_empty_model();
    let mut model = UserModel::from_model(model);

    // initially the column to the left is A
    let view = model.get_selected_view();
    assert_eq!(view.left_column, 1);

    // We select all columns from 1 to the last visible
    let (start_row, start_column, end_row, end_column) = (1, 1, 1, column_count);
    model.set_selected_cell(start_row, start_column).unwrap();
    model
        .set_selected_range(start_row, start_column, end_row, end_column)
        .unwrap();

    // Now we select one more column
    model.on_expand_selected_range("ArrowRight").unwrap();

    // The view has updated and the first visible column is B
    let view = model.get_selected_view();
    assert_eq!(
        view.range,
        [start_row, start_column, end_row, end_column + 1]
    );
    assert_eq!(view.left_column, 2);

    // now we click on cell B2 and we
    model.set_selected_cell(2, 2).unwrap();
    model.on_expand_selected_range("ArrowLeft").unwrap();

    let view = model.get_selected_view();
    assert_eq!(view.range, [2, 1, 2, 2]);
    assert_eq!(view.left_column, 1);

    // a second arrow left won't do anything
    model.on_expand_selected_range("ArrowLeft").unwrap();

    let view = model.get_selected_view();
    assert_eq!(view.range, [2, 1, 2, 2]);
    assert_eq!(view.left_column, 1);
}

#[test]
fn arrow_left() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_selected_cell(5, 3).unwrap();
    model.set_selected_range(5, 3, 10, 8).unwrap();
    model.on_expand_selected_range("ArrowLeft").unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [5, 3, 10, 7]);
}

#[test]
fn arrow_left_left_border() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.on_expand_selected_range("ArrowLeft").unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [1, 1, 1, 1]);
}

#[test]
fn arrow_left_increases() {
    // If the selected cell is on the top right corner
    // arrow left increases the selected area by
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);

    let (start_row, start_column, end_row, end_column) = (4, 10, 4, 20);
    model.set_selected_cell(start_row, end_column).unwrap();
    model
        .set_selected_range(start_row, start_column, end_row, end_column)
        .unwrap();
    model.on_expand_selected_range("ArrowLeft").unwrap();
    let view = model.get_selected_view();
    assert_eq!(
        view.range,
        [start_row, start_column - 1, end_row, end_column]
    );
}

#[test]
fn arrow_left_scrolls_left() {
    // If the selected cell is on the top right corner
    // arrow left increases the selected area by
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);

    model.set_top_left_visible_cell(1, 50).unwrap();

    model.set_selected_cell(1, 50).unwrap();
    // arrow left x 2
    model.on_expand_selected_range("ArrowLeft").unwrap();
    model.on_expand_selected_range("ArrowLeft").unwrap();

    let view = model.get_selected_view();
    assert_eq!(view.range, [1, 48, 1, 50]);
    assert_eq!(view.left_column, 48);
    assert_eq!(view.column, 50);
}
