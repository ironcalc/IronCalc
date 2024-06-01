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
    // We autofill from A3 to A5
    model
        .auto_fill_rows(
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
    assert_eq!(
        model.get_formatted_cell_value(0, 4, 1),
        Ok("alpha".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 1),
        Ok("alpha".to_string())
    );
}

#[test]
fn one_cell_down() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "23").unwrap();
    model
        .auto_fill_rows(
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
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok("23".to_string())
    );
}

#[test]
fn alpha_beta_gamma() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A1:B3
    model.set_user_input(0, 1, 1, "Alpher").unwrap();
    model.set_user_input(0, 2, 1, "Bethe").unwrap();
    model.set_user_input(0, 3, 1, "Gamow").unwrap();
    model.set_user_input(0, 1, 2, "=A1").unwrap();
    model.set_user_input(0, 2, 2, "=A2").unwrap();
    model.set_user_input(0, 3, 2, "=A3").unwrap();
    // We autofill from A1:B3 to A9
    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 2,
                height: 3,
            },
            9,
        )
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 4, 1),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 1),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 6, 1),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 7, 1),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 8, 1),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 9, 1),
        Ok("Gamow".to_string())
    );

    assert_eq!(
        model.get_formatted_cell_value(0, 4, 2),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 2),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 6, 2),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 7, 2),
        Ok("Alpher".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 8, 2),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 9, 2),
        Ok("Gamow".to_string())
    );

    assert_eq!(model.get_cell_content(0, 4, 2), Ok("=A4".to_string()));
}

#[test]
fn styles() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A1:B3
    model.set_user_input(0, 1, 1, "Alpher").unwrap();
    model.set_user_input(0, 2, 1, "Bethe").unwrap();
    model.set_user_input(0, 3, 1, "Gamow").unwrap();

    let a2 = Area {
        sheet: 0,
        row: 2,
        column: 1,
        width: 1,
        height: 1,
    };

    let a3 = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: 1,
        height: 1,
    };

    model.update_range_style(&a2, "font.i", "true").unwrap();
    model
        .update_range_style(&a3, "fill.bg_color", "#334455")
        .unwrap();

    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 3,
            },
            9,
        )
        .unwrap();

    // Check that cell A5 has A2 style
    let style = model.get_cell_style(0, 5, 1).unwrap();
    assert!(style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 6, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#334455".to_string()));

    model.undo().unwrap();

    assert_eq!(model.get_cell_content(0, 4, 1), Ok("".to_string()));
    // Check that cell A5 has A2 style
    let style = model.get_cell_style(0, 5, 1).unwrap();
    assert!(!style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 6, 1).unwrap();
    assert_eq!(style.fill.bg_color, None);

    model.redo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 4, 1),
        Ok("Alpher".to_string())
    );
    // Check that cell A5 has A2 style
    let style = model.get_cell_style(0, 5, 1).unwrap();
    assert!(style.font.i);
    // A6 would have the style of A3
    let style = model.get_cell_style(0, 6, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#334455".to_string()));
}

#[test]
fn upwards() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A10:A12
    model.set_user_input(0, 10, 1, "Alpher").unwrap();
    model.set_user_input(0, 11, 1, "Bethe").unwrap();
    model.set_user_input(0, 12, 1, "Gamow").unwrap();

    // We fill upwards to row 5
    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 10,
                column: 1,
                width: 1,
                height: 3,
            },
            5,
        )
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 9, 1),
        Ok("Gamow".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 8, 1),
        Ok("Bethe".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 7, 1),
        Ok("Alpher".to_string())
    );
}

#[test]
fn upwards_4() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A10:A13
    model.set_user_input(0, 10, 1, "Margaret Burbidge").unwrap();
    model.set_user_input(0, 11, 1, "Geoffrey Burbidge").unwrap();
    model.set_user_input(0, 12, 1, "Willy Fowler").unwrap();
    model.set_user_input(0, 13, 1, "Fred Hoyle").unwrap();

    // We fill upwards to row 5
    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 10,
                column: 1,
                width: 1,
                height: 4,
            },
            5,
        )
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 9, 1),
        Ok("Fred Hoyle".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 8, 1),
        Ok("Willy Fowler".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 1),
        Ok("Fred Hoyle".to_string())
    );
}

#[test]
fn errors() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    // cells A10:A13
    model.set_user_input(0, 4, 1, "Margaret Burbidge").unwrap();

    // Invalid sheet
    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 3,
                row: 4,
                column: 1,
                width: 1,
                height: 1,
            },
            10,
        ),
        Err("Invalid worksheet index: '3'".to_string())
    );

    // invalid row
    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: -1,
                column: 1,
                width: 1,
                height: 1,
            },
            10,
        ),
        Err("Invalid row: '-1'".to_string())
    );

    // invalid row
    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: LAST_ROW - 1,
                column: 1,
                width: 1,
                height: 10,
            },
            10,
        ),
        Err("Invalid row: '1048584'".to_string())
    );

    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: LAST_COLUMN + 1,
                width: 1,
                height: 10,
            },
            10,
        ),
        Err("Invalid column: '16385'".to_string())
    );

    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: LAST_COLUMN - 2,
                width: 10,
                height: 1,
            },
            10,
        ),
        Err("Invalid column: '16391'".to_string())
    );

    assert_eq!(
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: 5,
                column: 1,
                width: 1,
                height: 10,
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
        model.auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 2,
            },
            2,
        ),
        Err("Invalid parameters for autofill".to_string())
    );
}
