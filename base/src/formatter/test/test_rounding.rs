#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// Excel rounds what it shows half away from zero. These halves are exact in
// binary (1.25 is 5/4), so rounding half to even would show 1.2, 0.12, 2 and
// 2,345.12 instead.
#[test]
fn exact_halves_round_away_from_zero() {
    let locale = get_default_locale();
    assert_eq!(format_number(1.25, "0.0", locale).text, "1.3");
    assert_eq!(format_number(-1.25, "0.0", locale).text, "-1.3");
    assert_eq!(format_number(0.125, "0.0", locale).text, "0.1");
    assert_eq!(format_number(0.125, "0.00", locale).text, "0.13");
    assert_eq!(format_number(2.5, "0", locale).text, "3");
    assert_eq!(format_number(-2.5, "0", locale).text, "-3");
    assert_eq!(
        format_number(2_345.125, "#,##0.00", locale).text,
        "2,345.13"
    );
    assert_eq!(format_number(1_234.5, "#,##0", locale).text, "1,235");
}

// Excel keeps 15 significant digits and rounds that decimal, not the binary
// number underneath. 1.005 is stored as 1.00499999999999989..., below the
// half, but its 15 digits are 1.00500000000000, so Excel shows 1.01 for "0.00"
// (and TEXT(1.005,"0.00") is "1.01"). So do 2.675 (2.67499999...), 0.15
// (0.14999999...) and 1.45 (1.44999999...).
#[test]
fn rounds_the_fifteen_digit_decimal() {
    let locale = get_default_locale();
    assert_eq!(format_number(1.005, "0.00", locale).text, "1.01");
    assert_eq!(format_number(2.675, "0.00", locale).text, "2.68");
    assert_eq!(format_number(0.15, "0.0", locale).text, "0.2");
    assert_eq!(format_number(1.45, "0.0", locale).text, "1.5");
    // Below the half in its 15 digits too
    assert_eq!(format_number(1.00499999999999, "0.00", locale).text, "1.00");
    // Only 15 digits are shown: 2^53 = 9007199254740992
    assert_eq!(
        format_number(9_007_199_254_740_992.0, "0", locale).text,
        "9007199254740990"
    );
}

// The % and the trailing commas of a format move the decimal point of those 15
// digits. 0.145 is 0.14499999... in binary and 0.145 * 100 is
// 14.499999999999998, but Excel shows 15%.
#[test]
fn percent_and_scaling_round_the_scaled_decimal() {
    let locale = get_default_locale();
    assert_eq!(format_number(0.145, "0%", locale).text, "15%");
    assert_eq!(format_number(0.0125, "0.0%", locale).text, "1.3%");
    assert_eq!(format_number(-0.0125, "0.0%", locale).text, "-1.3%");
    assert_eq!(format_number(2_500.0, "#,##0,", locale).text, "3");
}

#[test]
fn currency_and_accounting() {
    let locale = get_default_locale();
    let accounting = "_(\"$\"* #,##0.00_);_(\"$\"* \\(#,##0.00\\);_(\"$\"* \"-\"??_);_(@_)";
    assert_eq!(format_number(0.125, "$#,##0.00", locale).text, "$0.13");
    assert_eq!(format_number(1.125, accounting, locale).text, " $ 1.13 ");
    assert_eq!(format_number(-1.125, accounting, locale).text, " $ (1.13)");
}

// A digit that rounds up carries into the digits before it: through the
// decimal point, and in scientific notation into the exponent.
#[test]
fn rounding_up_carries() {
    let locale = get_default_locale();
    assert_eq!(format_number(0.96, "0.0", locale).text, "1.0");
    assert_eq!(format_number(0.995, "0.00", locale).text, "1.00");
    assert_eq!(format_number(9.996, "0.00", locale).text, "10.00");
    assert_eq!(format_number(99_960.0, "0.0E+00", locale).text, "1.0E+05");
    assert_eq!(format_number(9.996, "0.00E+00", locale).text, "1.00E+01");
}

// A number that does not round to zero keeps its minus sign: -1 is "-1" and
// -0.05 is "-0.1" in "0.0".
#[test]
fn negative_numbers_keep_their_sign() {
    let locale = get_default_locale();
    assert_eq!(format_number(-1.0, "0", locale).text, "-1");
    assert_eq!(format_number(-0.1, "0.0", locale).text, "-0.1");
    assert_eq!(format_number(-0.05, "0.0", locale).text, "-0.1");
    assert_eq!(
        format_number(-0.000002, "0.00E+00", locale).text,
        "-2.00E-06"
    );
}

// The mantissa rounds half away from zero, and a number from 1 to 10 has the
// exponent +00.
#[test]
fn scientific() {
    let locale = get_default_locale();
    assert_eq!(format_number(125.0, "0.0E+00", locale).text, "1.3E+02");
    assert_eq!(format_number(-125.0, "0.0E+00", locale).text, "-1.3E+02");
    assert_eq!(format_number(5.0, "0.00E+00", locale).text, "5.00E+00");
}

// General rounds the same 15 digits half away from zero: to 9 decimals, or to
// 6 significant digits in scientific notation. 2^-10 = 0.0009765625 is an
// exact half at the tenth decimal, and 123456500000 / 10^11 is
// 1.23456499999... in binary.
#[test]
fn general() {
    let locale = get_default_locale();
    assert_eq!(
        format_number(0.0009765625, "General", locale).text,
        "0.000976563"
    );
    assert_eq!(
        format_number(-0.0009765625, "General", locale).text,
        "-0.000976563"
    );
    assert_eq!(
        format_number(123_456_500_000.0, "General", locale).text,
        "1.23457E+11"
    );
    assert_eq!(
        format_number(-123_456_500_000.0, "General", locale).text,
        "-1.23457E+11"
    );
    assert_eq!(
        format_number(0.0000000099999999999, "General", locale).text,
        "1E-08"
    );
}
