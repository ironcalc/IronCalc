#![allow(clippy::unwrap_used)]

//! Moving a row or a column changes where its cells are and nothing else.
//! A value is not typed in again at its new place: a text that looks like a
//! number, a boolean, a percentage or a formula stays a text, a number keeps
//! all its digits, and no link is attached to a text that looks like a URL.
//! This holds for the cells of the line that is moved and for those of the
//! lines it moves past.

use crate::model::Model;
use crate::test::util::new_empty_model;
use crate::types::Cell;
use crate::UserModel;

/// Fills `cells` with values that change when they are typed in again
fn fill(model: &mut Model, cells: &[&str; 8]) {
    let [quoted_number, quoted_bool, quoted_formula, quoted_percent, text, number, url, error] =
        cells;
    model._set(quoted_number, "'0630");
    model._set(quoted_bool, "'TRUE");
    model._set(quoted_formula, "'=B1");
    model._set(quoted_percent, "'12%");
    // A text that looks like a number, without a quote prefix, as it can be
    // read from a file
    let r = model._parse_reference(text);
    model
        .set_cell_with_string(r.sheet, r.row, r.column, "0630", 0)
        .unwrap();
    // A number that takes 17 significant digits to write
    let r = model._parse_reference(number);
    model
        .update_cell_with_number(r.sheet, r.row, r.column, 0.1 + 0.2)
        .unwrap();
    let r = model._parse_reference(url);
    model
        .update_cell_with_text(r.sheet, r.row, r.column, "https://example.com")
        .unwrap();
    model._set(error, "#N/A");
}

fn cells_of(model: &Model, cells: &[&str]) -> Vec<Cell> {
    cells.iter().map(|c| model._get_cell(c)).collect()
}

fn assert_moved(model: &Model, before: &[Cell], cells: &[&str]) {
    assert_eq!(cells_of(model, cells), before);
    assert!(model.workbook.worksheet(0).unwrap().links.is_empty());
}

const ROW_3: [&str; 8] = ["A3", "B3", "C3", "D3", "E3", "F3", "G3", "H3"];
const ROW_4: [&str; 8] = ["A4", "B4", "C4", "D4", "E4", "F4", "G4", "H4"];
const COLUMN_C: [&str; 8] = ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8"];
const COLUMN_D: [&str; 8] = ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8"];

#[test]
fn move_row_down_keeps_values() {
    let mut model = new_empty_model();
    fill(&mut model, &ROW_3);
    let moved = cells_of(&model, &ROW_3);
    assert_eq!(model._get_text("A3"), "0630");
    assert_eq!(model._get_text("F3"), "0.3");

    model.move_rows_action(0, 3, 1, 1).unwrap();
    assert_moved(&model, &moved, &ROW_4);
    assert_eq!(model._get_text("A4"), "0630");
    assert_eq!(model._get_text("C4"), "=B1");
    assert_eq!(model._get_text("E4"), "0630");
}

#[test]
fn move_row_up_keeps_values() {
    let mut model = new_empty_model();
    fill(&mut model, &ROW_4);
    let moved = cells_of(&model, &ROW_4);

    model.move_rows_action(0, 4, 1, -1).unwrap();
    assert_moved(&model, &moved, &ROW_3);
}

#[test]
fn move_row_keeps_values_of_rows_it_passes() {
    let mut model = new_empty_model();
    fill(&mut model, &ROW_4);
    let passed = cells_of(&model, &ROW_4);

    // Row 3 is empty: moving it down moves row 4 up
    model.move_rows_action(0, 3, 1, 1).unwrap();
    assert_moved(&model, &passed, &ROW_3);

    // and back
    model.move_rows_action(0, 4, 1, -1).unwrap();
    assert_moved(&model, &passed, &ROW_4);
}

#[test]
fn move_column_right_keeps_values() {
    let mut model = new_empty_model();
    fill(&mut model, &COLUMN_C);
    let moved = cells_of(&model, &COLUMN_C);

    model.move_columns_action(0, 3, 1, 1).unwrap();
    assert_moved(&model, &moved, &COLUMN_D);
}

#[test]
fn move_column_left_keeps_values() {
    let mut model = new_empty_model();
    fill(&mut model, &COLUMN_D);
    let moved = cells_of(&model, &COLUMN_D);

    model.move_columns_action(0, 4, 1, -1).unwrap();
    assert_moved(&model, &moved, &COLUMN_C);
}

#[test]
fn move_column_keeps_values_of_columns_it_passes() {
    let mut model = new_empty_model();
    fill(&mut model, &COLUMN_D);
    let passed = cells_of(&model, &COLUMN_D);

    model.move_columns_action(0, 3, 1, 1).unwrap();
    assert_moved(&model, &passed, &COLUMN_C);

    model.move_columns_action(0, 4, 1, -1).unwrap();
    assert_moved(&model, &passed, &COLUMN_D);
}

#[test]
fn moved_values_keep_their_style() {
    let mut model = new_empty_model();
    model._set("B3", "'0630");
    let mut style = model.get_style_for_cell(0, 3, 2).unwrap();
    style.font.b = true;
    model.set_cell_style(0, 3, 2, &style).unwrap();
    model._set("B4", "12");

    model.move_rows_action(0, 3, 1, 1).unwrap();
    let style = model.get_style_for_cell(0, 4, 2).unwrap();
    assert!(style.font.b);
    assert!(style.quote_prefix);
    assert_eq!(model._get_text("B4"), "0630");
    assert_eq!(model._get_text("B3"), "12");
}

#[test]
fn moved_formulas_still_follow_their_references() {
    let mut model = new_empty_model();
    model._set("A3", "'0630");
    model._set("B3", "=A3&\"!\"");
    model._set("C5", "=A3");
    model.evaluate();

    model.move_rows_action(0, 3, 1, 2).unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("B5"), "=A5&\"!\"");
    assert_eq!(model._get_formula("C4"), "=A5");
    assert_eq!(model._get_text("B5"), "0630!");
    assert_eq!(model._get_text("C4"), "0630");
}

#[test]
fn move_and_undo_keep_a_quoted_text() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 4, 3, "'007").unwrap();

    model.move_rows_action(0, 4, 1, -2).unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "007");
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 4, 3).unwrap(), "007");
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "007");

    model.move_columns_action(0, 3, 1, 2).unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 5).unwrap(), "007");
    model.undo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "007");
}
