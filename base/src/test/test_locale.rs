#![allow(clippy::unwrap_used)]

use crate::Model;

pub fn new_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "de", "UTC", "de").unwrap()
}

#[test]
fn german_functions() {
    let mut model = new_empty_model();
    model._set("A1", "=WENN(1>2; 3; 4)");
    model._set("A2", "=SUMME({1;2;3\\4;5;6})");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"21");
}

#[test]
fn german_numbers() {
    let mut model = new_empty_model();
    model._set("A1", "=SUMME(1,23; 3,45; 4,56)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9,24");
}

fn new_hebrew_model<'a>() -> Model<'a> {
    Model::new_empty("model", "he", "UTC", "he").unwrap()
}

#[test]
fn hebrew_functions_and_numbers() {
    // Hebrew Excel keeps English function names, booleans and the comma argument separator
    let mut model = new_hebrew_model();
    model._set("A1", "=IF(1>2, 3, 4)");
    model._set("A2", "=SUM(1.23, 3.45, 4.56)");
    model._set("A3", "=AND(TRUE, FALSE)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"4");
    assert_eq!(model._get_text("A2"), *"9.24");
    assert_eq!(model._get_text("A3"), *"FALSE");
}

#[test]
fn hebrew_dates() {
    let mut model = new_hebrew_model();
    model._set("A1", "=TEXT(DATE(2025, 12, 15), \"mmmm\")");
    model._set("A2", "=TEXT(DATE(2025, 12, 15), \"dddd\")");
    model._set("A3", "=TEXT(DATE(2025, 12, 15), \"d \"\"ב\"\"mmmm yyyy\")");
    model._set("A4", "15.12.2025");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"דצמבר");
    assert_eq!(model._get_text("A2"), *"יום שני");
    assert_eq!(model._get_text("A3"), *"15 בדצמבר 2025");
    assert_eq!(
        model.get_cell_value_by_ref("גיליון1!A4"),
        Ok(crate::cell::CellValue::Number(46006.0))
    );
}

#[test]
fn hebrew_settings() {
    let model = new_hebrew_model();
    let settings = model.get_fmt_settings();

    assert_eq!(settings.currency, "ILS");
    assert_eq!(settings.short_date, "d.m.yyyy");
    assert_eq!(settings.long_date, "d \"ב\"mmmm yyyy");
    assert_eq!(settings.short_date_example, "15.12.2025");
    assert_eq!(settings.long_date_example, "15 בדצמבר 2025");
    // CLDR puts RLM/LRM bidi marks in the Hebrew currency pattern; they must not leak in
    assert!(!settings.currency_format.contains(['\u{200E}', '\u{200F}']));
    assert_eq!(
        model.workbook.get_worksheet_names(),
        vec!["גיליון1".to_string()]
    );
}
