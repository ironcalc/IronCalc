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

#[test]
fn lowercase_e_and_comma() {
    // Excel's results, from Apache POI's NumberFormatApproxTests.xlsx (Tests!A2, A3, A42 and
    // A43), saved by Excel: a lowercase e is written as such, and a comma does not scale.
    let locale = get_default_locale();
    assert_eq!(format_number(123456.789, "|#,e-#|", locale).text, "|1e5|");
    assert_eq!(format_number(123456.789, "|#e-#,|", locale).text, "|1e5|");
    assert_eq!(format_number(1.23456789E-5, "|#,e-#|", locale).text, "|1e-5|");
    assert_eq!(format_number(1.23456789E-5, "|#e-#,|", locale).text, "|1e-5|");
}
