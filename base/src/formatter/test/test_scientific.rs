#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn scientific_minus_negative() {
    let locale = get_default_locale();
    let b = format_number(0.000002, "0.00E-00", locale);
    assert_eq!(b.text, "2.00E-06");
}

#[test]
fn scientific_minus_positive() {
    let locale = get_default_locale();
    let b = format_number(2_000_000.0, "0.00E-00", locale);
    assert_eq!(b.text, "2.00E06");
}

#[test]
fn scientific_positive() {
    let locale = get_default_locale();
    let b = format_number(2_000_000.0, "0.00E+00", locale);
    assert_eq!(b.text, "2.00E+06");
}

#[test]
fn scientific_negative() {
    let locale = get_default_locale();
    let b = format_number(0.000002, "0.00E+00", locale);
    assert_eq!(b.text, "2.00E-06");
}

// Excel switches the mantissa/exponent as a pair when the mantissa rounds up
// (https://github.com/ironcalc/IronCalc/issues/1628)
#[test]
fn scientific_mantissa_rounding_carries_exponent() {
    let locale = get_default_locale();
    assert_eq!(format_number(99.96, "0.00E+00", locale).text, "1.00E+02");
    assert_eq!(format_number(9_996.0, "0.00E+00", locale).text, "1.00E+04");
    assert_eq!(format_number(99_999.5, "0.00E+00", locale).text, "1.00E+05");
    assert_eq!(format_number(99.94, "0.0E+0", locale).text, "1.0E+2");
    assert_eq!(format_number(995.0, "0E+00", locale).text, "1E+03");
    // The exponent sign is E+ for zero and positive exponents
    assert_eq!(format_number(1.0, "0.00E+00", locale).text, "1.00E+00");
    // Three-digit exponents overflow the two exponent digits of the format
    assert_eq!(
        format_number(1e15 - 1.0, "0.00E+00", locale).text,
        "1.00E+15"
    );
    assert_eq!(format_number(1e100, "0.00E+00", locale).text, "1.00E+100");
    assert_eq!(format_number(1e-300, "0.00E+00", locale).text, "1.00E-300");
    // Subnormal numbers must not produce inf (Excel has none, we still show 5)
    assert_eq!(
        format_number(f64::MAX, "0.00E+00", locale).text,
        "1.80E+308"
    );
    // Engineering formats keep the exponent a multiple of three
    assert_eq!(format_number(12.0, "##0.0E+0", locale).text, "12.0E+0");
    assert_eq!(format_number(0.0123, "##0.0E+0", locale).text, "12.3E-3");
}
