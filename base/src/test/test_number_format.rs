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

// A time shows its nearest second, in a cell's format and in TEXT: 18:58, as
// Ctrl+Shift+; stores it (0.7902777777777777, a hair under 18:58), is 6:58 PM.
#[test]
fn test_cell_format_and_text_round_time_to_the_second() {
    let model = new_empty_model();
    let mut model = UserModel::from_model(model);
    model.set_user_input(0, 1, 1, "0.7902777777777777").unwrap(); // A1
    model
        .set_user_input(0, 2, 1, "=TEXT(A1,\"h:mm AM/PM\")")
        .unwrap(); // A2
    model
        .set_user_input(0, 3, 1, "=TEXT(A1,\"h:mm:ss\")")
        .unwrap(); // A3
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    model
        .update_range_style(&range, "num_fmt", "h:mm AM/PM")
        .unwrap();

    let formatted = |row| model.get_formatted_cell_value(0, row, 1).unwrap();
    assert_eq!(formatted(1), "6:58 PM");
    assert_eq!(formatted(2), "6:58 PM");
    assert_eq!(formatted(3), "18:58:00");
}
