use std::fs;

use ironcalc_base::Model;

use crate::error::XlsxError;
use crate::export::save_to_icalc;
use crate::import::load_from_icalc;
use crate::{export::save_to_xlsx, import::load_from_xlsx};

pub fn new_empty_model() -> Model {
    Model::new_empty("model", "en", "UTC").unwrap()
}

#[test]
fn test_values() {
    let mut model = new_empty_model();
    // numbers
    model
        .set_user_input(0, 1, 1, "123.456".to_string())
        .unwrap();
    // strings
    model
        .set_user_input(0, 2, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 1, "Hello world!".to_string())
        .unwrap();
    model
        .set_user_input(0, 4, 1, "你好世界！".to_string())
        .unwrap();
    // booleans
    model.set_user_input(0, 5, 1, "TRUE".to_string()).unwrap();
    model.set_user_input(0, 6, 1, "FALSE".to_string()).unwrap();
    // errors
    model
        .set_user_input(0, 7, 1, "#VALUE!".to_string())
        .unwrap();

    // noop
    model.evaluate();
    {
        let temp_file_name = "temp_file_test_values.xlsx";
        save_to_xlsx(&model, temp_file_name).unwrap();

        let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
    {
        let temp_file_name = "temp_file_test_values.ic";
        save_to_icalc(&model, temp_file_name).unwrap();

        let model = load_from_icalc(temp_file_name).unwrap();
        assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "123.456");
        assert_eq!(
            model.get_formatted_cell_value(0, 2, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 3, 1).unwrap(),
            "Hello world!"
        );
        assert_eq!(
            model.get_formatted_cell_value(0, 4, 1).unwrap(),
            "你好世界！"
        );
        assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "TRUE");
        assert_eq!(model.get_formatted_cell_value(0, 6, 1).unwrap(), "FALSE");
        assert_eq!(model.get_formatted_cell_value(0, 7, 1).unwrap(), "#VALUE!");

        fs::remove_file(temp_file_name).unwrap();
    }
}

#[test]
fn test_formulas() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5.5".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "6.5".to_string()).unwrap();
    model.set_user_input(0, 3, 1, "7.5".to_string()).unwrap();

    model.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
    model.set_user_input(0, 2, 2, "=A2*2".to_string()).unwrap();
    model.set_user_input(0, 3, 2, "=A3*2".to_string()).unwrap();
    model
        .set_user_input(0, 4, 2, "=SUM(A1:B3)".to_string())
        .unwrap();

    model.evaluate();
    let temp_file_name = "temp_file_test_formulas.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 2).unwrap(), "11");
    assert_eq!(model.get_formatted_cell_value(0, 2, 2).unwrap(), "13");
    assert_eq!(model.get_formatted_cell_value(0, 3, 2).unwrap(), "15");
    assert_eq!(model.get_formatted_cell_value(0, 4, 2).unwrap(), "58.5");
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_sheets() {
    let mut model = new_empty_model();
    model.add_sheet("With space").unwrap();
    // xml escaped
    model.add_sheet("Tango & Cash").unwrap();
    model.add_sheet("你好世界").unwrap();

    // noop
    model.evaluate();

    let temp_file_name = "temp_file_test_sheets.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Sheet1", "With space", "Tango & Cash", "你好世界"]
    );
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_named_styles() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5.5".to_string()).unwrap();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    style.font.i = true;
    assert!(model.set_cell_style(0, 1, 1, &style).is_ok());
    let bold_style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let e = model
        .workbook
        .styles
        .add_named_cell_style("bold & italics", bold_style_index);
    assert!(e.is_ok());

    // noop
    model.evaluate();

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    assert!(model
        .workbook
        .styles
        .get_style_index_by_name("bold & italics")
        .is_ok());
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_existing_file() {
    let file_name = "existing_file.xlsx";
    fs::File::create(file_name).unwrap();

    assert_eq!(
        save_to_xlsx(&new_empty_model(), file_name),
        Err(XlsxError::IO(
            "file existing_file.xlsx already exists".to_string()
        )),
    );

    fs::remove_file(file_name).unwrap();
}
