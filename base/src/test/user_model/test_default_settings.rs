#![allow(clippy::unwrap_used)]

use crate::{
    constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, LAST_COLUMN, LAST_ROW},
    expressions::types::Area,
    test::user_model::util::new_empty_user_model,
    types::{Border, Fill, Font, Style},
};

#[test]
fn basic_defaults() {
    let model = new_empty_user_model();
    assert_eq!(model.get_default_column_width(), DEFAULT_COLUMN_WIDTH);
    assert_eq!(model.get_default_row_height(), DEFAULT_ROW_HEIGHT);
    let style = model.get_default_cell_style();
    let sty = Style {
        font: Font {
            name: "Calibri".to_string(),
            strike: false,
            u: false,
            b: false,
            i: false,
            sz: 13,
            color: Some("#000000".to_string()),
            family: 2,
            scheme: crate::types::FontScheme::Minor,
        },
        fill: Fill {
            fg_color: None,
            bg_color: None,
            pattern_type: "none".to_string(),
        },
        num_fmt: "general".to_string(),
        quote_prefix: false,
        alignment: None,
        border: Border {
            left: None,
            right: None,
            top: None,
            bottom: None,
            diagonal_up: false,
            diagonal_down: false,
            diagonal: None,
        },
    };
    assert_eq!(style, sty);
}

#[test]
fn default_column_width_and_row_height_can_be_changed() {
    let mut model = new_empty_user_model();
    // sanity check
    assert_eq!(DEFAULT_COLUMN_WIDTH, 125.0);
    assert_eq!(DEFAULT_ROW_HEIGHT, 28.0);

    // change column 40 and row 40
    model.set_columns_width(0, 40, 40, 50.0).unwrap();
    model.set_rows_height(0, 40, 40, 30.0).unwrap();

    // change styles of column 50
    let range_column50 = Area {
        sheet: 0,
        row: 1,
        column: 50,
        width: 1,
        height: LAST_ROW,
    };
    model
        .update_range_style(&range_column50, "font.b", "true")
        .unwrap();

    let range_row50 = Area {
        sheet: 0,
        row: 50,
        column: 1,
        width: LAST_COLUMN,
        height: 1,
    };
    model
        .update_range_style(&range_row50, "font.b", "true")
        .unwrap();

    // Check default values
    let column_c = model.get_column_width(0, 3).unwrap();
    let row_c = model.get_row_height(0, 3).unwrap();
    assert_eq!(column_c, DEFAULT_COLUMN_WIDTH);
    assert_eq!(row_c, DEFAULT_ROW_HEIGHT);

    // change default values and check that they are updated
    model.set_default_column_width(42.0).unwrap();
    model.set_default_row_height(24.0).unwrap();

    assert_eq!(model.get_default_column_width(), 42.0);
    assert_eq!(model.get_default_row_height(), 24.0);

    // Check that the new default values are returned for cells without specific widths/heights
    let column_c = model.get_column_width(0, 3).unwrap();
    let row_c = model.get_row_height(0, 3).unwrap();
    assert_eq!(column_c, 42.0);
    assert_eq!(row_c, 24.0);

    // Check that specific widths/heights are unchanged
    let column40 = model.get_column_width(0, 40).unwrap();
    let row40 = model.get_row_height(0, 40).unwrap();
    assert_eq!(column40, 50.0);
    assert_eq!(row40, 30.0);

    // Column 50 has the default width but a specific style
    let column50 = model.get_column_width(0, 50).unwrap();
    let style50 = model.get_cell_style(0, 1, 50).unwrap();
    assert_eq!(column50, 42.0);
    assert!(style50.font.b);
}

#[test]
fn default_sheet_settings() {
    let mut model = new_empty_user_model();
    assert!(model.get_default_sheet_settings(0).unwrap().is_none());

    model.set_default_sheet_column_width(0, 50.0).unwrap();
    model.set_default_sheet_row_height(0, 30.0).unwrap();

    let settings = model.get_default_sheet_settings(0).unwrap().unwrap();

    assert_eq!(settings.column_width, 50.0);
    assert_eq!(settings.row_height, 30.0);

    assert_eq!(model.get_column_width(0, 5).unwrap(), 50.0);
    assert_eq!(model.get_row_height(0, 5).unwrap(), 30.0);

    model.undo().unwrap();
    model.undo().unwrap();
    assert!(model.get_default_sheet_settings(0).unwrap().is_none());

    model.redo().unwrap();
    model.redo().unwrap();
    let settings = model.get_default_sheet_settings(0).unwrap().unwrap();
    assert_eq!(settings.column_width, 50.0);
    assert_eq!(settings.row_height, 30.0);
}

#[test]
fn errors() {
    let mut model = new_empty_user_model();
    assert!(model.set_default_column_width(-1.0).is_err());
    assert!(model.set_default_row_height(-1.0).is_err());

    assert!(model.set_default_sheet_column_width(0, -1.0).is_err());
    assert!(model.set_default_sheet_row_height(0, -1.0).is_err());
}
