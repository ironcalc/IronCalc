#![allow(clippy::unwrap_used)]

use crate::expressions::types::Area;
use crate::number_format::format_number;
use crate::test::util::new_empty_model;
use crate::UserModel;

#[test]
fn test_simple_format() {
    let formatted = format_number(2.3, "General", "en");
    assert_eq!(formatted.text, "2.3".to_string());
}

#[test]
fn test_maximum_zeros() {
    let formatted = format_number(1.0 / 3.0, "#,##0.0000000000000000000", "en");
    assert_eq!(formatted.text, "0.3333333333333330000".to_string());

    let formatted = format_number(1234.0 + 1.0 / 3.0, "#,##0.0000000000000000000", "en");
    assert_eq!(formatted.text, "1,234.3333333333300000000".to_string());
}

#[test]
fn test_leading_comma_text() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, ",10").unwrap(); // A1
    model.set_user_input(0, 2, 1, ",100").unwrap(); // A2
    model.set_user_input(0, 3, 1, ",1000").unwrap(); // A3

    assert_eq!(
        model.get_formatted_cell_value(0, 1, 1),
        Ok(",10".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 1),
        Ok(",100".to_string())
    );
    assert_eq!(
        model.get_formatted_cell_value(0, 3, 1),
        Ok(",1000".to_string())
    );
}

#[test]
fn test_wrong_locale() {
    let formatted = format_number(2.3, "General", "ens");
    assert_eq!(formatted.text, "#ERROR!".to_string());
}

// A cell's number format and TEXT round as Excel does, half away from zero on
// the 15-digit decimal, as ROUND already did: 1.25 is 1.3 in "0.0", and 1.005
// (1.00499999999999989... in binary) is 1.01 in "0.00".
#[test]
fn test_cell_format_and_text_round_half_away() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "1.25").unwrap(); // A1
    model.set_user_input(0, 2, 1, "-1.25").unwrap(); // A2
    model.set_user_input(0, 3, 1, "0.125").unwrap(); // A3
    model
        .set_user_input(0, 4, 1, "=TEXT(1.25,\"0.0\")")
        .unwrap(); // A4
    model
        .set_user_input(0, 5, 1, "=TEXT(1.005,\"0.00\")")
        .unwrap(); // A5
    model.set_user_input(0, 6, 1, "=ROUND(1.25,1)").unwrap(); // A6
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 3,
    };
    model.update_range_style(&range, "num_fmt", "0.0").unwrap();

    let formatted = |row| model.get_formatted_cell_value(0, row, 1).unwrap();
    assert_eq!(formatted(1), "1.3");
    assert_eq!(formatted(2), "-1.3");
    assert_eq!(formatted(3), "0.1");
    assert_eq!(formatted(4), "1.3");
    assert_eq!(formatted(5), "1.01");
    assert_eq!(formatted(6), "1.3");
}
