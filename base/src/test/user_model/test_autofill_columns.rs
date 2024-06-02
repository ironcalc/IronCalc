#![allow(clippy::unwrap_used)]

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::Area;
use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn basic_tests() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // This is cell A3
    model.set_user_input(0, 3, 1, "alpha").unwrap();
    // We autofill from A3 to C3
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 3,
                column: 1,
                width: 1,
                height: 1,
            },
            5,
        )
        .unwrap();
    // B3
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 2),
        Ok("alpha".to_string())
    );
    // C3
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 3),
        Ok("alpha".to_string())
    );
}

#[test]
fn one_cell_right() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "23").unwrap();
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 1,
            },
            2,
        )
        .unwrap();
    // B1
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 2),
        Ok("23".to_string())
    );
}

#[test]
fn alpha_beta_gamma() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A1:B3
    model.set_user_input(0, 1, 1, "Alpher").unwrap(); // A1
    model.set_user_input(0, 1, 2, "Bethe").unwrap(); // B1
    model.set_user_input(0, 1, 3, "Gamow").unwrap(); // C1
    model.set_user_input(0, 2, 1, "=A1").unwrap(); // A2
    model.set_user_input(0, 2, 2, "=B1").unwrap(); // B2
    model.set_user_input(0, 2, 3, "=C1").unwrap(); // C2

    // We autofill from A1:C2 to I2
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 3,
                height: 2,
            },
            9,
        )
        .unwrap();

    // D1
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 4),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 5),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 6),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 7),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 8),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 9),
        Ok("Gamow".to_string())
    );

    assert_eq!(
        model.get_formatted_cell_value(0, 2, 4),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 5),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 6),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 7),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 8),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 9),
        Ok("Gamow".to_string())
    );

    assert_eq!(model.get_cell_content(0, 2, 4), Ok("=D1".to_string()));
}

#[test]
fn styles() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A1:C1
    model.set_user_input(0, 1, 1, "Alpher").unwrap();
    model.set_user_input(0, 2, 1, "Bethe").unwrap();
    model.set_user_input(0, 3, 1, "Gamow").unwrap();

    let b1 = Area {
        sheet: 0,
        row: 1,
        column: 2,
        width: 1,
        height: 1,
    };

    let c1 = Area {
        sheet: 0,
        row: 1,
        column: 3,
        width: 1,
        height: 1,
    };

    model.update_range_style(&b1, "font.i", "true").unwrap();
    model
        .update_range_style(&c1, "fill.bg_color", "#334455")
        .unwrap();

    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 3,
                height: 1,
            },
            9,
        )
        .unwrap();

    // Check that cell E1 has B1 style
    let style = model.get_cell_style(0, 1, 5).unwrap();
    assert!(style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 1, 6).unwrap();
    assert_eq!(style.fill.bg_color, Some("#334455".to_string()));

    model.undo().unwrap();

    assert_eq!(model.get_cell_content(0, 1, 4), Ok("".to_string()));
    // Check that cell A5 has A2 style
    let style = model.get_cell_style(0, 1, 5).unwrap();
    assert!(!style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 1, 6).unwrap();
    assert_eq!(style.fill.bg_color, None);

    model.redo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 4),
        Ok("Alpher".to_string())
    );
    // Check that cell A5 has A2 style
    let style = model.get_cell_style(0, 1, 5).unwrap();
    assert!(style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 1, 6).unwrap();
    assert_eq!(style.fill.bg_color, Some("#334455".to_string()));
}

#[test]
fn left() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A10:A12
    model.set_user_input(0, 1, 10, "Alpher").unwrap();
    model.set_user_input(0, 1, 11, "Bethe").unwrap();
    model.set_user_input(0, 1, 12, "Gamow").unwrap();

    // We fill upwards to row 5
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 10,
                width: 3,
                height: 1,
            },
            5,
        )
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 9),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 8),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 7),
        Ok("Alpher".to_string())
    );
}

#[test]
fn left_4() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A10:A13
    model.set_user_input(0, 1, 10, "Margaret Burbidge").unwrap();
    model.set_user_input(0, 1, 11, "Geoffrey Burbidge").unwrap();
    model.set_user_input(0, 1, 12, "Willy Fowler").unwrap();
    model.set_user_input(0, 1, 13, "Fred Hoyle").unwrap();

    // We fill left to row 5
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 10,
                width: 4,
                height: 1,
            },
            5,
        )
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 9),
        Ok("Fred Hoyle".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 8),
        Ok("Willy Fowler".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 1, 5),
        Ok("Fred Hoyle".to_string())
    );
}

#[test]
fn errors() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);

    model.set_user_input(0, 1, 4, "Margaret Burbidge").unwrap();

    // Invalid sheet
    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 3,
                row: 1,
                column: 4,
                width: 1,
                height: 1,
            },
            10,
        ),
        Err("Invalid worksheet index: '3'".to_string())
    );

    // invalid column
    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: -1,
                width: 1,
                height: 1,
            },
            10,
        ),
        Err("Invalid column: '-1'".to_string())
    );

    // invalid column
    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: LAST_COLUMN - 1,
                width: 10,
                height: 1,
            },
            10,
        ),
        Err("Invalid column: '16392'".to_string())
    );

    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: LAST_ROW + 1,
                column: 1,
                width: 10,
                height: 1,
            },
            10,
        ),
        Err("Invalid row: '1048577'".to_string())
    );

    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: LAST_ROW - 2,
                column: 1,
                width: 1,
                height: 10,
            },
            10,
        ),
        Err("Invalid row: '1048583'".to_string())
    );

    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 5,
                width: 10,
                height: 1,
            },
            -10,
        ),
        Err("Invalid row: '-10'".to_string())
    );
}

#[test]
fn invalid_parameters() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "23").unwrap();
    assert_eq!(
        model.auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 2,
                height: 1,
            },
            2,
        ),
        Err("Invalid parameters for autofill".to_string())
    );
}
