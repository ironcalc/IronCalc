use std::{env, fs, io};
use uuid::Uuid;

use ironcalc::compare::{test_file, test_load_and_saving};
use ironcalc::export::save_to_xlsx;
use ironcalc::import::{load_from_excel, load_model_from_xlsx};
use ironcalc_base::types::{HorizontalAlignment, VerticalAlignment, Workbook};
use ironcalc_base::Model;

// This is a functional test.
// We check that the output of example.xlsx is what we expect.
#[test]
fn test_example() {
    let model = load_from_excel("tests/example.xlsx", "en", "UTC").unwrap();
    assert_eq!(model.worksheets[0].frozen_rows, 0);
    assert_eq!(model.worksheets[0].frozen_columns, 0);
    let contents =
        fs::read_to_string("tests/example.json").expect("Something went wrong reading the file");
    let model2: Workbook = serde_json::from_str(&contents).unwrap();
    let s = serde_json::to_string(&model).unwrap();
    assert_eq!(model, model2, "{s}");
}

#[test]
fn test_save_to_xlsx() {
    let mut model = load_model_from_xlsx("tests/example.xlsx", "en", "UTC").unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_example.xlsx";
    // test can safe
    save_to_xlsx(&model, temp_file_name).unwrap();
    // test can open
    let model = load_model_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    let metadata = &model.workbook.metadata;
    assert_eq!(metadata.application, "IronCalc Sheets");
    // FIXME: This will need to be updated once we fix versioning
    assert_eq!(metadata.app_version, "10.0000");
    // TODO: can we show it is the 'same' model?
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_freeze() {
    // freeze has 3 frozen columns and 2 frozen rows
    let model = load_from_excel("tests/freeze.xlsx", "en", "UTC").unwrap();
    assert_eq!(model.worksheets[0].frozen_rows, 2);
    assert_eq!(model.worksheets[0].frozen_columns, 3);
}

#[test]
fn test_split() {
    // We test that a workbook with split panes do not produce frozen rows and columns
    let model = load_from_excel("tests/split.xlsx", "en", "UTC").unwrap();
    assert_eq!(model.worksheets[0].frozen_rows, 0);
    assert_eq!(model.worksheets[0].frozen_columns, 0);
}

fn test_model_has_correct_styles(model: &Model) {
    // A1 is bold
    let style_a1 = model.get_style_for_cell(0, 1, 1);
    assert!(style_a1.font.b);
    assert!(!style_a1.font.i);
    assert!(!style_a1.font.u);

    // B1 is Italics
    let style_b1 = model.get_style_for_cell(0, 1, 2);
    assert!(style_b1.font.i);
    assert!(!style_b1.font.b);
    assert!(!style_b1.font.u);

    // C1 Underlined
    let style_c1 = model.get_style_for_cell(0, 1, 3);
    assert!(style_c1.font.u);
    assert!(!style_c1.font.b);
    assert!(!style_c1.font.i);

    // D1 Bold and Italics
    let style_d1 = model.get_style_for_cell(0, 1, 4);
    assert!(style_d1.font.b);
    assert!(style_d1.font.i);
    assert!(!style_d1.font.u);

    // E1 Bold, italics and underlined
    let style_e1 = model.get_style_for_cell(0, 1, 5);
    assert!(style_e1.font.b);
    assert!(style_e1.font.i);
    assert!(style_e1.font.u);
    assert!(!style_e1.font.strike);

    // F1 strikethrough
    let style_f1 = model.get_style_for_cell(0, 1, 6);
    assert!(style_f1.font.strike);

    // G1 Double underlined just get simple underlined
    let style_g1 = model.get_style_for_cell(0, 1, 7);
    assert!(style_g1.font.u);

    let height_row_3 = model.workbook.worksheet(0).unwrap().row_height(3).unwrap();
    assert_eq!(height_row_3, 136.0);

    let height_row_5 = model.workbook.worksheet(0).unwrap().row_height(5).unwrap();
    assert_eq!(height_row_5, 62.0);

    // Second sheet has alignment
    // Horizontal
    let alignment = model.get_style_for_cell(1, 2, 1).alignment;
    assert_eq!(alignment, None);

    let alignment = model.get_style_for_cell(1, 3, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Left);

    let alignment = model.get_style_for_cell(1, 4, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Distributed);

    let alignment = model.get_style_for_cell(1, 5, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Right);

    let alignment = model.get_style_for_cell(1, 6, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Center);

    let alignment = model.get_style_for_cell(1, 7, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Fill);

    let alignment = model.get_style_for_cell(1, 8, 1).alignment.unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Justify);

    // Vertical
    let alignment = model.get_style_for_cell(1, 2, 2).alignment;
    assert_eq!(alignment, None);

    let alignment = model.get_style_for_cell(1, 3, 2).alignment;
    assert_eq!(alignment, None);

    let alignment = model.get_style_for_cell(1, 4, 2).alignment.unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Top);

    let alignment = model.get_style_for_cell(1, 5, 2).alignment.unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Center);

    let alignment = model.get_style_for_cell(1, 6, 2).alignment.unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Justify);

    let alignment = model.get_style_for_cell(1, 7, 2).alignment.unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Distributed);
}

#[test]
fn test_simple_text() {
    let model = load_model_from_xlsx("tests/basic_text.xlsx", "en", "UTC").unwrap();

    test_model_has_correct_styles(&model);

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_model_from_xlsx(temp_file_name, "en", "UTC").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    test_model_has_correct_styles(&model);
}

#[test]
fn test_defined_names_casing() {
    let test_file_path = "tests/calc_tests/defined_names_for_unit_test.xlsx";
    let loaded_workbook = load_from_excel(test_file_path, "en", "UTC").unwrap();
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
        model.set_user_input(0, row, column, formula.to_string());
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
    for file_path in entries {
        let file_name_str = file_path.file_name().unwrap().to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();
        println!("Testing file: {}", file_path_str);
        if file_name_str.ends_with(".xlsx") && !file_name_str.starts_with('~') {
            if let Err(message) = test_file(file_path_str) {
                println!("{}", message);
                panic!("Model was evaluated inconsistently with XLSX data.")
            }
            assert!(test_load_and_saving(file_path_str, &dir).is_ok());
        } else {
            println!("skipping");
        }
    }
    fs::remove_dir_all(&dir).unwrap();
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
    for file_path in entries {
        let file_name_str = file_path.file_name().unwrap().to_str().unwrap();
        let file_path_str = file_path.to_str().unwrap();
        println!("Testing file: {}", file_path_str);
        if file_name_str.ends_with(".xlsx") && !file_name_str.starts_with('~') {
            if let Err(message) = test_file(file_path_str) {
                println!("{}", message);
                panic!("Model was evaluated inconsistently with XLSX data.")
            }
        } else {
            println!("skipping");
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}
