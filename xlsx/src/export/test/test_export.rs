use std::fs;

use ironcalc_base::types::StyleIncludes;
use ironcalc_base::Model;

use crate::error::XlsxError;
use crate::export::save_to_icalc;
use crate::import::load_from_icalc;
use crate::{export::save_to_xlsx, import::load_from_xlsx};

pub fn new_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
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

        let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
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

        let model = load_from_icalc(temp_file_name, "en").unwrap();
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
fn frozen_rows() {
    let mut model = new_empty_model();
    model.set_frozen_rows(0, 23).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_rows.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 23);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn frozen_columns() {
    let mut model = new_empty_model();
    model.set_frozen_columns(0, 42).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_columns.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 42);
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn frozen_rows_and_columns() {
    let mut model = new_empty_model();
    model.set_frozen_rows(0, 23).unwrap();
    model.set_frozen_columns(0, 42).unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_frozen_rows_and_columns.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(model.get_frozen_rows_count(0).unwrap(), 23);
    assert_eq!(model.get_frozen_columns_count(0).unwrap(), 42);
    fs::remove_file(temp_file_name).unwrap();
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

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
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

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Sheet1", "With space", "Tango & Cash", "你好世界"]
    );
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_move_sheet_order_roundtrips() {
    let mut model = new_empty_model();
    model.add_sheet("Second").unwrap();
    model.add_sheet("Third").unwrap();
    // A cross-sheet reference so we can confirm it survives the reorder + save.
    model.set_user_input(1, 1, 1, "42".to_string()).unwrap(); // Second!A1
    model
        .set_user_input(0, 1, 1, "=Second!A1".to_string())
        .unwrap(); // Sheet1!A1

    // Reorder: Sheet1 goes to the end.
    model.move_sheet(0, 2).unwrap();
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Second", "Third", "Sheet1"]
    );

    let temp_file_name = "temp_file_test_move_sheet_order.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    // The saved order is preserved on reload.
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["Second", "Third", "Sheet1"]
    );
    // Sheet1 (now index 2) still resolves its reference to Second by name.
    assert_eq!(model.get_formatted_cell_value(2, 1, 1).unwrap(), "42");
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_named_styles() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "5.5".to_string()).unwrap();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    style.font.i = true;
    let e = model.workbook.styles.create_named_style(
        "bold & italics",
        &style,
        StyleIncludes::default(),
    );
    assert!(e.is_ok());
    assert!(model
        .set_cell_style_by_name(0, 1, 1, "bold & italics")
        .is_ok());

    // noop
    model.evaluate();

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let mut model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert!(model
        .workbook
        .styles
        .get_style_index_by_name("bold & italics")
        .is_ok());
    // After the roundtrip the style record must still include every formatting
    // category (in cellStyleXfs absent apply* attributes mean "included";
    // exporting applyX="0" would make Excel treat the style as empty).
    let cell_style = model
        .workbook
        .styles
        .cell_styles
        .iter()
        .find(|cs| cs.name == "bold & italics")
        .unwrap();
    let record = &model.workbook.styles.cell_style_xfs[cell_style.xf_id as usize];
    assert!(record.apply_number_format);
    assert!(record.apply_font);
    assert!(record.apply_fill);
    assert!(record.apply_border);
    assert!(record.apply_alignment);
    assert!(record.apply_protection);

    // The cell is still linked to the named style: updating it restyles the cell
    let mut style = model.get_named_style("bold & italics").unwrap();
    assert!(style.font.b);
    style.font.u = true;
    model
        .update_named_style(
            "bold & italics",
            "bold & italics",
            &style,
            StyleIncludes::default(),
        )
        .unwrap();
    assert!(model.get_style_for_cell(0, 1, 1).unwrap().font.u);
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

#[test]
fn test_defined_name_lambda_exports_excel_prefixes() {
    let mut model = new_empty_model();
    model
        .new_defined_name("cal_year", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name(
            "month_start",
            None,
            "LAMBDA(mo,LET(anchor,DATE(cal_year,mo,1),anchor+1))",
        )
        .unwrap();
    model.set_user_input(0, 1, 1, "2024".to_string()).unwrap();
    model
        .set_user_input(0, 2, 1, "=month_start(3)".to_string())
        .unwrap();
    // reference value computed without defined names
    model
        .set_user_input(0, 3, 1, "=DATE(2024,3,1)+1".to_string())
        .unwrap();
    model.evaluate();
    let expected_value = model.get_cell_value_by_index(0, 3, 1).unwrap();
    assert_eq!(
        model.get_cell_value_by_index(0, 2, 1).unwrap(),
        expected_value
    );

    let temp_file_name = "temp_file_test_defined_name_lambda.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    // The workbook part must store the formula in Excel's form: future
    // functions prefixed with `_xlfn.` and lambda/let variables with `_xlpm.`.
    let file = fs::File::open(temp_file_name).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut workbook_xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("xl/workbook.xml").unwrap(),
        &mut workbook_xml,
    )
    .unwrap();
    drop(archive);
    assert!(
        workbook_xml.contains(
            "_xlfn.LAMBDA(_xlpm.mo,_xlfn.LET(_xlpm.anchor,DATE(cal_year,_xlpm.mo,1),_xlpm.anchor+1))"
        ),
        "workbook.xml does not contain the Excel form of the defined name: {workbook_xml}"
    );
    // The plain range name is stored untouched.
    assert!(workbook_xml.contains("<definedName name=\"cal_year\">Sheet1!$A$1</definedName>"));

    // And the file round-trips: the lambda still evaluates and the defined
    // name displays without the prefixes.
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    assert_eq!(
        model.get_cell_value_by_index(0, 2, 1).unwrap(),
        expected_value
    );
    let defined_names = model.get_defined_name_list();
    let month_start = defined_names
        .iter()
        .find(|(name, ..)| name == "month_start")
        .unwrap();
    assert_eq!(
        month_start.2,
        "LAMBDA(mo,LET(anchor,DATE(cal_year,mo,1),anchor+1))"
    );
    fs::remove_file(temp_file_name).unwrap();
}
