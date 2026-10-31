#![allow(clippy::unwrap_used)]

// Furigana (phonetic runs, <rPh>) typed in Japanese Excel. furigana.xlsx is basic_text.xlsx
// with these cells in Sheet1:
//   A10 山田太郎 (ヤマダ / タロウ)   A11 東京都 (トウキョウト)
//   A12 新宿, rich text (シンジュク)  A13 田中さん (タナカ on 田中)
//   A14 大阪, an inline string (オオサカ)

use ironcalc::base::Model;
use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::{load_from_xlsx, load_from_xlsx_bytes};
use std::io::Cursor;

/// The text shown in a cell of the first sheet ("B10" => column 2, row 10)
fn text(model: &Model, cell: &str) -> String {
    let column = (cell.as_bytes()[0] - b'A' + 1) as i32;
    let row: i32 = cell[1..].parse().unwrap();
    model.get_formatted_cell_value(0, row, column).unwrap()
}

fn add_phonetic_formulas(model: &mut Model) {
    for row in 10..=14 {
        model
            .set_user_input(0, row, 2, format!("=PHONETIC(A{row})"))
            .unwrap();
    }
    model
        .set_user_input(0, 10, 3, "=PHONETIC(A10:A13)".to_string())
        .unwrap();
    model.evaluate();
}

fn check(model: &Model) {
    // The furigana are not part of the text
    assert_eq!(text(model, "A10"), "山田太郎");
    assert_eq!(text(model, "A11"), "東京都");
    assert_eq!(text(model, "A12"), "新宿");
    assert_eq!(text(model, "A13"), "田中さん");
    assert_eq!(text(model, "A14"), "大阪");

    assert_eq!(text(model, "B10"), "ヤマダタロウ");
    assert_eq!(text(model, "B11"), "トウキョウト");
    assert_eq!(text(model, "B12"), "シンジュク");
    assert_eq!(text(model, "B13"), "タナカさん");
    // We do not keep the furigana of inline strings (Excel does not write them)
    assert_eq!(text(model, "B14"), "大阪");
    assert_eq!(
        text(model, "C10"),
        "ヤマダタロウトウキョウトシンジュクタナカさん"
    );
}

#[test]
fn imports_furigana() {
    let mut model = load_from_xlsx("tests/furigana.xlsx", "ja", "UTC", "ja").unwrap();
    // The strings that were there before keep their text and have no furigana
    assert_eq!(text(&model, "A1"), "Bold");
    assert_eq!(model.workbook.phonetic_runs.len(), 4);
    add_phonetic_formulas(&mut model);
    check(&model);
}

#[test]
fn furigana_survive_export_and_import() {
    let model = load_from_xlsx("tests/furigana.xlsx", "ja", "UTC", "ja").unwrap();
    let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner();
    let workbook = load_from_xlsx_bytes(&bytes, "furigana", "ja", "UTC").unwrap();
    assert_eq!(workbook.phonetic_runs, model.workbook.phonetic_runs);
    let mut model = Model::from_workbook(workbook, "ja").unwrap();
    add_phonetic_formulas(&mut model);
    check(&model);
}

#[test]
fn furigana_survive_the_ironcalc_format() {
    let model = load_from_xlsx("tests/furigana.xlsx", "ja", "UTC", "ja").unwrap();
    let mut model = Model::from_bytes(&model.to_bytes(), "ja").unwrap();
    add_phonetic_formulas(&mut model);
    check(&model);
}
