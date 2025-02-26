#![allow(clippy::unwrap_used)]

use crate::{
    constants::{DEFAULT_COLUMN_WIDTH, DEFAULT_ROW_HEIGHT, LAST_COLUMN},
    test::util::new_empty_model,
    UserModel,
};

#[test]
fn simple_insert_row() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let (sheet, column) = (0, 5);
    for row in 1..5 {
        assert!(model.set_user_input(sheet, row, column, "123").is_ok());
    }
    assert!(model.insert_row(sheet, 3).is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        ""
    );

    assert!(model.undo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        "123"
    );
    assert!(model.redo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, 3, column).unwrap(),
        ""
    );
}

#[test]
fn simple_insert_column() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    let (sheet, row) = (0, 5);
    for column in 1..5 {
        assert!(model.set_user_input(sheet, row, column, "123").is_ok());
    }
    assert!(model.insert_column(sheet, 3).is_ok());
    assert_eq!(model.get_formatted_cell_value(sheet, row, 3).unwrap(), "");

    assert!(model.undo().is_ok());
    assert_eq!(
        model.get_formatted_cell_value(sheet, row, 3).unwrap(),
        "123"
    );
    assert!(model.redo().is_ok());
    assert_eq!(model.get_formatted_cell_value(sheet, row, 3).unwrap(), "");
}

#[test]
fn simple_delete_column() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 5, "3").unwrap();
    model.set_user_input(0, 2, 5, "=E1*2").unwrap();
    model
        .set_columns_width(0, 5, 5, DEFAULT_COLUMN_WIDTH * 3.0)
        .unwrap();

    model.delete_column(0, 5).unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 2, 5), Ok("".to_string()));
    assert_eq!(model.get_column_width(0, 5), Ok(DEFAULT_COLUMN_WIDTH));

    model.undo().unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 2, 5), Ok("6".to_string()));
    assert_eq!(model.get_column_width(0, 5), Ok(DEFAULT_COLUMN_WIDTH * 3.0));

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(
        model2.get_formatted_cell_value(0, 2, 5),
        Ok("6".to_string())
    );
    assert_eq!(
        model2.get_column_width(0, 5),
        Ok(DEFAULT_COLUMN_WIDTH * 3.0)
    );
}

#[test]
fn delete_column_errors() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    assert_eq!(
        model.delete_column(1, 1),
        Err("Invalid sheet index".to_string())
    );

    assert_eq!(
        model.delete_column(0, 0),
        Err("Column number '0' is not valid.".to_string())
    );
    assert_eq!(
        model.delete_column(0, LAST_COLUMN + 1),
        Err("Column number '16385' is not valid.".to_string())
    );

    assert_eq!(model.delete_column(0, LAST_COLUMN), Ok(()));
}

#[test]
fn simple_delete_row() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 15, 4, "3").unwrap();
    model.set_user_input(0, 15, 6, "=D15*2").unwrap();

    model
        .set_rows_height(0, 15, 15, DEFAULT_ROW_HEIGHT * 3.0)
        .unwrap();

    model.delete_row(0, 15).unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 15, 6), Ok("".to_string()));
    assert_eq!(model.get_row_height(0, 15), Ok(DEFAULT_ROW_HEIGHT));

    model.undo().unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 15, 6),
        Ok("6".to_string())
    );
    assert_eq!(model.get_row_height(0, 15), Ok(DEFAULT_ROW_HEIGHT * 3.0));

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(
        model2.get_formatted_cell_value(0, 15, 6),
        Ok("6".to_string())
    );
    assert_eq!(model2.get_row_height(0, 15), Ok(DEFAULT_ROW_HEIGHT * 3.0));
}

#[test]
fn simple_delete_row_no_style() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 15, 4, "3").unwrap();
    model.set_user_input(0, 15, 6, "=D15*2").unwrap();
    model.delete_row(0, 15).unwrap();

    assert_eq!(model.get_formatted_cell_value(0, 15, 6), Ok("".to_string()));
}

#[test]
fn row_heigh_increases_automatically() {
    let mut model = UserModel::new_empty("Workbook1", "en", "UTC").unwrap();
    assert_eq!(model.get_row_height(0, 1), Ok(DEFAULT_ROW_HEIGHT));

    // Entering a single line does not change the height
    model
        .set_user_input(0, 1, 1, "My home in Canada had horses")
        .unwrap();
    assert_eq!(model.get_row_height(0, 1), Ok(DEFAULT_ROW_HEIGHT));

    // entering a two liner does:
    model
        .set_user_input(0, 1, 1, "My home in Canada had horses\nAnd monkeys!")
        .unwrap();
    assert_eq!(model.get_row_height(0, 1), Ok(40.5));
}

#[test]
fn insert_row_evaluates() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 1, 2, "=A1*2").unwrap();

    assert!(model.insert_row(0, 1).is_ok());
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "84");
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "84");
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "84");

    model.delete_row(0, 1).unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "84");
    assert_eq!(model.get_cell_content(0, 1, 2).unwrap(), "=A1*2");
}

#[test]
fn insert_column_evaluates() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "42").unwrap();
    model.set_user_input(0, 10, 1, "=A1*2").unwrap();

    assert!(model.insert_column(0, 1).is_ok());
    assert_eq!(model.get_formatted_cell_value(0, 10, 2).unwrap(), "84");

    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 10, 1).unwrap(), "84");
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 10, 2).unwrap(), "84");

    model.delete_column(0, 1).unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 10, 1).unwrap(), "84");
    assert_eq!(model.get_cell_content(0, 10, 1).unwrap(), "=A1*2");
}
