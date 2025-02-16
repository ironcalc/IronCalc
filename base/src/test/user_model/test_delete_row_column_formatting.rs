#![allow(clippy::unwrap_used)]

use crate::{
    constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, LAST_COLUMN, LAST_ROW},
    expressions::types::Area,
    UserModel,
};

#[test]
fn delete_column_formatting() {
    // We are going to delete formatting in column G (7)
    // There are cells with their own styles
    // There are rows with their own styles
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let cell_g123 = Area {
        sheet: 0,
        row: 123,
        column: 7,
        width: 1,
        height: 1,
    };

    let column_g_range = Area {
        sheet: 0,
        row: 1,
        column: 7,
        width: 1,
        height: LAST_ROW,
    };

    let row_3_range = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: LAST_COLUMN,
        height: 1,
    };

    // Set the style of the whole column
    model
        .update_range_style(&column_g_range, "fill.bg_color", "#555666")
        .unwrap();

    // Set G123 background to red
    model
        .update_range_style(&cell_g123, "fill.bg_color", "#FF5533")
        .unwrap();

    // Set the style of the whole row
    model
        .update_range_style(&row_3_range, "fill.bg_color", "#333444")
        .unwrap();

    // Delete the column formatting
    model.range_clear_formatting(&column_g_range).unwrap();

    // Check the style of G123 is now what it was before
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    // Check the style of the whole row is still there
    let style = model.get_cell_style(0, 3, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#333444".to_owned()));

    // Check the style of the whole column is now gone
    let style = model.get_cell_style(0, 3, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    let style = model.get_cell_style(0, 40, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    model.undo().unwrap();

    // Check the style of G123 is now what it was before
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, Some("#FF5533".to_owned()));

    // Check G3 is the row style
    let style = model.get_cell_style(0, 3, 7).unwrap();
    assert_eq!(style.fill.bg_color, Some("#333444".to_owned()));

    // Check G40 is the column style
    let style = model.get_cell_style(0, 40, 7).unwrap();
    assert_eq!(style.fill.bg_color, Some("#555666".to_owned()));

    model.redo().unwrap();

    // Check the style of G123 is now what it was before
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    // Check the style of the whole row is still there
    let style = model.get_cell_style(0, 3, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#333444".to_owned()));

    // Check the style of the whole column is now gone
    let style = model.get_cell_style(0, 3, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    let style = model.get_cell_style(0, 40, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);
}

#[test]
fn column_width() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model
        .set_columns_width(0, 7, 7, DEFAULT_COLUMN_WIDTH * 2.0)
        .unwrap();

    let column_g_range = Area {
        sheet: 0,
        row: 1,
        column: 7,
        width: 1,
        height: LAST_ROW,
    };

    // Set the style of the whole column
    model
        .update_range_style(&column_g_range, "fill.bg_color", "#555666")
        .unwrap();

    // Delete the column formatting
    model.range_clear_formatting(&column_g_range).unwrap();
    // This does not change the column width
    assert_eq!(
        model.get_column_width(0, 7).unwrap(),
        2.0 * DEFAULT_COLUMN_WIDTH
    );

    model.undo().unwrap();
    assert_eq!(
        model.get_column_width(0, 7).unwrap(),
        2.0 * DEFAULT_COLUMN_WIDTH
    );
    model.redo().unwrap();
    assert_eq!(
        model.get_column_width(0, 7).unwrap(),
        2.0 * DEFAULT_COLUMN_WIDTH
    );
}

#[test]
fn column_row_style_undo() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    model
        .set_columns_width(0, 7, 7, DEFAULT_COLUMN_WIDTH * 2.0)
        .unwrap();

    let column_g_range = Area {
        sheet: 0,
        row: 1,
        column: 7,
        width: 1,
        height: LAST_ROW,
    };

    let row_123_range = Area {
        sheet: 0,
        row: 123,
        column: 1,
        width: LAST_COLUMN,
        height: 1,
    };

    let delete_range = Area {
        sheet: 0,
        row: 120,
        column: 5,
        width: 20,
        height: 20,
    };

    // Set the style of the whole column
    model
        .update_range_style(&column_g_range, "fill.bg_color", "#555666")
        .unwrap();

    model
        .update_range_style(&row_123_range, "fill.bg_color", "#111222")
        .unwrap();

    model.range_clear_formatting(&delete_range).unwrap();

    // check G123 is empty
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);

    // uno clear formatting
    model.undo().unwrap();

    // G123 has the row style
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, Some("#111222".to_owned()));

    // undo twice
    model.undo().unwrap();
    model.undo().unwrap();

    // check G123 is empty
    let style = model.get_cell_style(0, 123, 7).unwrap();
    assert_eq!(style.fill.bg_color, None);
}

#[test]
fn column_row_row_height_undo() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();

    let column_g_range = Area {
        sheet: 0,
        row: 1,
        column: 7,
        width: 1,
        height: LAST_ROW,
    };

    let row_3_range = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: LAST_COLUMN,
        height: 1,
    };

    model
        .update_range_style(&column_g_range, "fill.bg_color", "#555666")
        .unwrap();

    model
        .set_rows_height(0, 3, 3, DEFAULT_ROW_HEIGHT * 2.0)
        .unwrap();

    model
        .update_range_style(&row_3_range, "fill.bg_color", "#111222")
        .unwrap();

    model.undo().unwrap();

    // check G3 has the column style
    let style = model.get_cell_style(0, 3, 7).unwrap();
    assert_eq!(style.fill.bg_color, Some("#555666".to_string()));
}
