#![allow(clippy::unwrap_used)]

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

#[test]
fn test_round_half_away_from_zero() {
    // Excel rounds halves away from zero (Rust formatting rounds them to even)
    let cases = [
        (0.5, "0", "1"),
        (1.5, "0", "2"),
        (2.5, "0", "3"),
        (-2.5, "0", "-3"),
        (1234.5, "#,##0", "1,235"),
        (1234.5, "$#,##0", "$1,235"),
        (1.25, "0.0", "1.3"),
        (0.125, "0.00", "0.13"),
        (0.125, "0.0%", "12.5%"),
        (0.0125, "0.0%", "1.3%"),
        // 1.005 and 2.675 are stored just below the half
        (1.005, "0.00", "1.01"),
        (2.675, "0.00", "2.68"),
        (0.0049, "0.00", "0.00"),
        (-0.4, "0", "0"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, "en");
        assert_eq!(formatted.text, expected, "{value} with '{format}'");
    }
    let formatted = format_number(1234.5, "\"￥\"#,##0", "ja");
    assert_eq!(formatted.text, "￥1,235");
}
