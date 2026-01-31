#![allow(clippy::unwrap_used)]

use crate::{formatter::format::parse_formatted_number, locale::get_default_locale};

const PARSE_ERROR_MSG: &str = "Could not parse number";

fn parse(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_default_locale();
    parse_formatted_number(input, currencies, locale)
}

#[test]
fn numbers() {
    // whole numbers
    assert_eq!(parse("400", &["$"]), Ok((400.0, None)));

    // decimal numbers
    assert_eq!(parse("4.456", &["$"]), Ok((4.456, None)));

    // scientific notation
    assert_eq!(
        parse("23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.123456789e-11", &["$"]),
        Ok((2.123456789e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E-9", &["$"]),
        Ok((4.5e-9, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e+2", &["$"]),
        Ok((2300.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E9", &["$"]),
        Ok((4.5e9, Some("0.00E+00".to_string())))
    );

    // negative numbers
    assert_eq!(parse("-400", &["$"]), Ok((-400.0, None)));
    assert_eq!(parse("-4.456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse("-23e-12", &["$"]),
        Ok((-2.3e-11, Some("0.00E+00".to_string())))
    );

    // trims space
    assert_eq!(parse(" 400  ", &["$"]), Ok((400.0, None)));
}

#[test]
fn percentage() {
    // whole numbers
    assert_eq!(parse("400%", &["$"]), Ok((4.0, Some("#,##0%".to_string()))));
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Percentage in scientific notation will not be formatted as percentage
    assert_eq!(
        parse("23e-12%", &["$"]),
        Ok((23e-12 / 100.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3E4%", &["$"]),
        Ok((230.0, Some("0.00E+00".to_string())))
    );
}

#[test]
fn currency() {
    // whole numbers
    assert_eq!(
        parse("400$", &["$"]),
        Ok((400.0, Some("#,##0$".to_string())))
    );
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Currencies in scientific notation will not be formatted as currencies
    assert_eq!(
        parse("23e-12$", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3e-12$", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("€23e-12", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );

    // switch side of currencies
    assert_eq!(
        parse("$400", &["$"]),
        Ok((400.0, Some("$#,##0".to_string())))
    );
    assert_eq!(
        parse("$4.456", &["$"]),
        Ok((4.456, Some("$#,##0.00".to_string())))
    );
    assert_eq!(
        parse("$23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("$2.3e-12", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e-12€", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
}

#[test]
fn negative_currencies() {
    assert_eq!(
        parse("-400$", &["$"]),
        Ok((-400.0, Some("#,##0$".to_string())))
    );
    assert_eq!(
        parse("-$400", &["$"]),
        Ok((-400.0, Some("$#,##0".to_string())))
    );
    assert_eq!(
        parse("$-400", &["$"]),
        Ok((-400.0, Some("$#,##0".to_string())))
    );
}

#[test]
fn errors() {
    // Strings are not numbers
    assert_eq!(parse("One", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // Not partial parsing
    assert_eq!(parse("23 Hello", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("Hello 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("2 3", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // No space between
    assert_eq!(parse("- 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

#[test]
fn errors_wrong_currency() {
    assert_eq!(parse("123€", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

#[test]
fn long_dates_en_us() {
    assert_eq!(
        parse("03/02/2024", &["$"]),
        Ok((45353.0, Some("mm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/02/2024", &["$"]),
        Ok((45353.0, Some("m/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("Mar/02/2024", &["$"]),
        Ok((45353.0, Some("mmm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("March/02/2024", &["$"]),
        Ok((45353.0, Some("mmmm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/2/24", &["$"]),
        Ok((45353.0, Some("m/d/yy".to_string())))
    );

    assert_eq!(
        parse("02-10-1975", &["$"]),
        Ok((27435.0, Some("mm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-1975", &["$"]),
        Ok((27435.0, Some("m-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("Feb-10-1975", &["$"]),
        Ok((27435.0, Some("mmm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("February-10-1975", &["$"]),
        Ok((27435.0, Some("mmmm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-75", &["$"]),
        Ok((27435.0, Some("m-dd-yy".to_string())))
    );
}

#[test]
fn iso_dates() {
    assert_eq!(
        parse("2024/03/02", &["$"]),
        Ok((45353.0, Some("yyyy/mm/dd".to_string())))
    );
    assert_eq!(
        parse("2024/March/02", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
}

#[test]
fn long_dates_with_dots() {
    assert_eq!(
        parse("03.02.2024", &["$"]),
        Ok((45353.0, Some("mm.dd.yyyy".to_string())))
    );
}
