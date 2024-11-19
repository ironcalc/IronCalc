#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use std::io::Read;
use std::{env, fs, io};
use uuid::Uuid;

use ironcalc::compare::{test_file, test_load_and_saving};
use ironcalc::export::save_to_xlsx;
use ironcalc::import::{load_from_icalc, load_from_xlsx, load_from_xlsx_bytes};
use ironcalc_base::types::{HorizontalAlignment, VerticalAlignment};
use ironcalc_base::{Model, UserModel};

// This is a functional test.
// We check that the output of example.xlsx is what we expect.
#[test]
fn test_example() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC").unwrap();
    // We should use the API once it is in place
    let workbook = model.workbook;
    let ws = &workbook.worksheets;
    let expected_names = vec![
        "Sheet1".to_string(),
        "Second".to_string(),
        "Sheet4".to_string(),
        "shared".to_string(),
        "Table".to_string(),
        "Sheet2".to_string(),
        "Created fourth".to_string(),
        "Frozen".to_string(),
        "Split".to_string(),
        "Hidden".to_string(),
    ];
    let names: Vec<String> = ws.iter().map(|s| s.name.clone()).collect();

    // One is not not imported and one is hidden
    assert_eq!(expected_names, names);

    assert_eq!(workbook.views[&0].sheet, 7);

    // Test selection:
    // First sheet (Sheet1)
    // E13 and E13:N20
    assert_eq!(ws[0].frozen_rows, 0);
    assert_eq!(ws[0].frozen_columns, 0);
    assert_eq!(ws[0].views[&0].row, 13);
    assert_eq!(ws[0].views[&0].column, 5);
    assert_eq!(ws[0].views[&0].range, [13, 5, 20, 14]);

    let model2 = load_from_icalc("tests/example.ic").unwrap();
    let _ = bitcode::encode(&model2.workbook);
    assert_eq!(workbook, model2.workbook);
}

#[test]
fn test_load_from_xlsx_bytes() {
    let file_path = std::path::Path::new("tests/example.xlsx");
    let mut file = fs::File::open(file_path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let workbook = load_from_xlsx_bytes(&bytes, "home", "en", "UTC").unwrap();
    assert_eq!(workbook.views[&0].sheet, 7);
}

#[test]
fn no_grid() {
    let model = load_from_xlsx("tests/NoGrid.xlsx", "en", "UTC").unwrap();
    {
        let workbook = &model.workbook;
        let ws = &workbook.worksheets;

        // NoGrid does not show grid lines
        let no_grid_sheet = &ws[0];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(!no_grid_sheet.show_grid_lines);

        let sheet2 = &ws[1];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(sheet2.show_grid_lines);

        let no_grid_no_headers_sheet = &ws[2];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        // There is also no headers
        assert!(!no_grid_no_headers_sheet.show_grid_lines);
    }
    {
        // save it and check again
        let temp_file_name = "temp_file_no_grid.xlsx";
        save_to_xlsx(&model, temp_file_name).unwrap();
        let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
        let workbook = &model.workbook;
        let ws = &workbook.worksheets;

        // NoGrid does not show grid lines
        let no_grid_sheet = &ws[0];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(!no_grid_sheet.show_grid_lines);

        let sheet2 = &ws[1];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(sheet2.show_grid_lines);

        let no_grid_no_headers_sheet = &ws[2];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        // There is also no headers
        assert!(!no_grid_no_headers_sheet.show_grid_lines);
        fs::remove_file(temp_file_name).unwrap();
    }
}

#[test]
fn test_save_to_xlsx() {
    let mut model = load_from_xlsx("tests/example.xlsx", "en", "UTC").unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_example.xlsx";
    // test can safe
    save_to_xlsx(&model, temp_file_name).unwrap();
    // test can open
    let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    let metadata = &model.workbook.metadata;
    assert_eq!(metadata.application, "IronCalc Sheets");
    // FIXME: This will need to be updated once we fix versioning
    assert_eq!(metadata.app_version, "10.0000");

    let workbook = model.workbook;
    let ws = &workbook.worksheets;

    assert_eq!(workbook.views[&0].sheet, 7);

    // Test selection:
    // First sheet (Sheet1)
    // E13 and E13:N20
    assert_eq!(ws[0].frozen_rows, 0);
    assert_eq!(ws[0].frozen_columns, 0);
    assert_eq!(ws[0].views[&0].row, 13);
    assert_eq!(ws[0].views[&0].column, 5);
    assert_eq!(ws[0].views[&0].range, [13, 5, 20, 14]);
    // TODO: can we show it is the 'same' model?
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_freeze() {
    // freeze has 3 frozen columns and 2 frozen rows
    let model = load_from_xlsx("tests/freeze.xlsx", "en", "UTC")
        .unwrap()
        .workbook;
    assert_eq!(model.worksheets[0].frozen_rows, 2);
    assert_eq!(model.worksheets[0].frozen_columns, 3);
}

#[test]
fn test_split() {
    // We test that a workbook with split panes do not produce frozen rows and columns
    let model = load_from_xlsx("tests/split.xlsx", "en", "UTC")
        .unwrap()
        .workbook;
    assert_eq!(model.worksheets[0].frozen_rows, 0);
    assert_eq!(model.worksheets[0].frozen_columns, 0);
}

fn test_model_has_correct_styles(model: &Model) {
    // A1 is bold
    let style_a1 = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(style_a1.font.b);
    assert!(!style_a1.font.i);
    assert!(!style_a1.font.u);

    // B1 is Italics
    let style_b1 = model.get_style_for_cell(0, 1, 2).unwrap();
    assert!(style_b1.font.i);
    assert!(!style_b1.font.b);
    assert!(!style_b1.font.u);

    // C1 Underlined
    let style_c1 = model.get_style_for_cell(0, 1, 3).unwrap();
    assert!(style_c1.font.u);
    assert!(!style_c1.font.b);
    assert!(!style_c1.font.i);

    // D1 Bold and Italics
    let style_d1 = model.get_style_for_cell(0, 1, 4).unwrap();
    assert!(style_d1.font.b);
    assert!(style_d1.font.i);
    assert!(!style_d1.font.u);

    // E1 Bold, italics and underlined
    let style_e1 = model.get_style_for_cell(0, 1, 5).unwrap();
    assert!(style_e1.font.b);
    assert!(style_e1.font.i);
    assert!(style_e1.font.u);
    assert!(!style_e1.font.strike);

    // F1 strikethrough
    let style_f1 = model.get_style_for_cell(0, 1, 6).unwrap();
    assert!(style_f1.font.strike);

    // G1 Double underlined just get simple underlined
    let style_g1 = model.get_style_for_cell(0, 1, 7).unwrap();
    assert!(style_g1.font.u);

    let height_row_3 = model.workbook.worksheet(0).unwrap().row_height(3).unwrap();
    assert_eq!(height_row_3, 136.0);

    let height_row_5 = model.workbook.worksheet(0).unwrap().row_height(5).unwrap();
    assert_eq!(height_row_5, 62.0);

    // Second sheet has alignment
    // Horizontal
    let alignment = model.get_style_for_cell(1, 2, 1).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model
        .get_style_for_cell(1, 3, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Left);

    let alignment = model
        .get_style_for_cell(1, 4, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Distributed);

    let alignment = model
        .get_style_for_cell(1, 5, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Right);

    let alignment = model
        .get_style_for_cell(1, 6, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Center);

    let alignment = model
        .get_style_for_cell(1, 7, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Fill);

    let alignment = model
        .get_style_for_cell(1, 8, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Justify);

    // Vertical
    let alignment = model.get_style_for_cell(1, 2, 2).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model.get_style_for_cell(1, 3, 2).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model
        .get_style_for_cell(1, 4, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Top);

    let alignment = model
        .get_style_for_cell(1, 5, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Center);

    let alignment = model
        .get_style_for_cell(1, 6, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Justify);

    let alignment = model
        .get_style_for_cell(1, 7, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Distributed);
}

#[test]
fn test_simple_text() {
    let model = load_from_xlsx("tests/basic_text.xlsx", "en", "UTC").unwrap();

    test_model_has_correct_styles(&model);

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    test_model_has_correct_styles(&model);
}

#[test]
fn test_defined_names_casing() {
    let test_file_path = "tests/calc_tests/defined_names_for_unit_test.xlsx";
    let loaded_workbook = load_from_xlsx(test_file_path, "en", "UTC")
        .unwrap()
        .workbook;
    let mut model = Model::from_bytes(&bitcode::encode(&loaded_workbook)).unwrap();

    let (row, column) = (2, 13); // B13
    let test_cases = [
        ("=named1", "11"),
        ("=NAMED1", "11"),
        ("=NaMeD1", "11"),
        ("=named2", "22"),
        ("=NAMED2", "22"),
        ("=NaMeD2", "22"),
        ("=named3", "33"),
        ("=NAMED3", "33"),
        ("=NaMeD3", "33"),
    ];
    for (formula, expected_value) in test_cases {
        model
            .set_user_input(0, row, column, formula.to_string())
            .unwrap();
        model.evaluate();
        assert_eq!(
            model.get_formatted_cell_value(0, row, column).unwrap(),
            expected_value
        );
    }
}

#[test]
fn test_xlsx() {
    let mut entries = fs::read_dir("tests/calc_tests/")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.sort();
    let temp_folder = env::temp_dir();
    let path = format!("{}", Uuid::new_v4());
    let dir = temp_folder.join(path);
    fs::create_dir(&dir).unwrap();
    let mut is_error = false;
    for file_path in entries {
        let file_name_str = file_path.file_name().unwrap().to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();
        println!("Testing file: {}", file_path_str);
        if file_name_str.ends_with(".xlsx") && !file_name_str.starts_with('~') {
            if let Err(message) = test_file(file_path_str) {
                println!("Error with file: '{file_path_str}'");
                println!("{}", message);
                is_error = true;
            }
            let t = test_load_and_saving(file_path_str, &dir);
            if t.is_err() {
                println!("Error while load and saving file: {file_path_str}");
                is_error = true;
            }
        } else {
            println!("skipping");
        }
    }
    fs::remove_dir_all(&dir).unwrap();
    assert!(
        !is_error,
        "Models were evaluated inconsistently with XLSX data."
    );
}

#[test]
fn no_export() {
    let mut entries = fs::read_dir("tests/calc_test_no_export/")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.sort();
    let temp_folder = env::temp_dir();
    let path = format!("{}", Uuid::new_v4());
    let dir = temp_folder.join(path);
    fs::create_dir(&dir).unwrap();
    let mut is_error = false;
    for file_path in entries {
        let file_name_str = file_path.file_name().unwrap().to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();
        println!("Testing file: {}", file_path_str);
        if file_name_str.ends_with(".xlsx") && !file_name_str.starts_with('~') {
            if let Err(message) = test_file(file_path_str) {
                println!("Error with file: '{file_path_str}'");
                println!("{}", message);
                is_error = true;
            }
        } else {
            println!("skipping");
        }
    }
    fs::remove_dir_all(&dir).unwrap();
    assert!(
        !is_error,
        "Models were evaluated inconsistently with XLSX data."
    );
}

// This test verifies whether exporting the merged cells functionality is happening properly or not.
// It first loads the Excel having the merged cell and exports it to another xlsx and verifies whether merged
// cell node is same in both of the xlsx file or not.
#[test]
fn test_exporting_merged_cells() {
    let temp_file_name = "temp_file_test_export_merged_cells.xlsx";
    let expected_merge_cell_ref = {
        // loading the xlsx file containing merged cells
        let example_file_name = "tests/example.xlsx";
        let mut model = load_from_xlsx(example_file_name, "en", "UTC").unwrap();
        let expected_merge_cell_ref = model
            .workbook
            .worksheets
            .first()
            .unwrap()
            .merge_cells
            .clone();
        // exporting and saving it in another xlsx
        model.evaluate();
        save_to_xlsx(&model, temp_file_name).unwrap();
        expected_merge_cell_ref
    };
    {
        let mut temp_model = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
        {
            // loading the previous file back and verifying whether
            // merged cells got exported properly or not
            let got_merge_cell_ref = &temp_model
                .workbook
                .worksheets
                .first()
                .unwrap()
                .merge_cells
                .clone();
            assert_eq!(expected_merge_cell_ref, *got_merge_cell_ref);
            fs::remove_file(temp_file_name).unwrap();
        }
        {
            // this block is to verify that if there are no
            // merged cells, exported xml should not have the
            // <mergeCells/> xml node
            temp_model
                .workbook
                .worksheets
                .get_mut(0)
                .unwrap()
                .merge_cells
                .clear();

            save_to_xlsx(&temp_model, temp_file_name).unwrap();
            let temp_model2 = load_from_xlsx(temp_file_name, "en", "UTC").unwrap();
            let got_merge_cell_ref_cnt = &temp_model2
                .workbook
                .worksheets
                .first()
                .unwrap()
                .merge_cells
                .len();
            assert!(*got_merge_cell_ref_cnt == 0);
        }
    }

    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_documentation_xlsx() {
    let mut entries = fs::read_dir("tests/docs/")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    entries.sort();
    // We can't test volatiles
    let mut skip = vec!["DATE.xlsx", "DAY.xlsx", "MONTH.xlsx", "YEAR.xlsx"];
    // Numerically unstable
    skip.push("TAN.xlsx");
    let skip: Vec<String> = skip.iter().map(|s| format!("tests/docs/{s}")).collect();
    println!("{:?}", skip);
    // dumb counter to make sure we are actually testing the files
    assert!(entries.len() > 7);
    let temp_folder = env::temp_dir();
    let path = format!("{}", Uuid::new_v4());
    let dir = temp_folder.join(path);
    fs::create_dir(&dir).unwrap();
    let mut is_error = false;
    for file_path in entries {
        let file_name_str = file_path.file_name().unwrap().to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();
        if skip.contains(&file_path_str.to_string()) {
            println!("Skipping file: {}", file_path_str);
            continue;
        }
        println!("Testing file: {}", file_path_str);
        if file_name_str.ends_with(".xlsx") && !file_name_str.starts_with('~') {
            if let Err(message) = test_file(file_path_str) {
                println!("{}", message);
                is_error = true;
            }
            assert!(test_load_and_saving(file_path_str, &dir).is_ok());
        } else {
            println!("skipping");
        }
    }
    fs::remove_dir_all(&dir).unwrap();
    assert!(
        !is_error,
        "Models were evaluated inconsistently with XLSX data."
    )
}

#[test]
fn test_user_model() {
    let temp_file_name = "temp_file_test_user_model.xlsx";
    let mut model = UserModel::new_empty("my_model", "en", "UTC").unwrap();
    model.set_user_input(0, 1, 1, "=1+1").unwrap();

    // test we can use `get_model` to save the model
    save_to_xlsx(model.get_model(), temp_file_name).unwrap();
    fs::remove_file(temp_file_name).unwrap();

    // we can still use the model afterwards
    model.set_rows_height(0, 1, 1, 100.0).unwrap();
}
