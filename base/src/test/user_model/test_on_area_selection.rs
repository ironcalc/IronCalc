#![allow(clippy::unwrap_used)]

use crate::{
    constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_WINDOW_WIDTH},
    test::util::new_empty_model,
    UserModel,
};

#[test]
fn basic_test() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);

    model.on_area_selecting(2, 4).unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [1, 1, 2, 4]);
}

// this checks that is we select in the boundary we automatically scroll
#[test]
fn scroll_right() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let window_width = DEFAULT_WINDOW_WIDTH as f64;
    let column_width = DEFAULT_COLUMN_WIDTH;
    let column_count = f64::floor(window_width / column_width) as i32;
    model.set_selected_cell(3, column_count).unwrap();

    model.on_area_selecting(3, column_count + 3).unwrap();
    let view = model.get_selected_view();
    assert_eq!(view.range, [3, column_count, 3, column_count + 3]);
    assert_eq!(view.left_column, 4);
}
