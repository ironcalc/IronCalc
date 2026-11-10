#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn simple_test() {
    let locale = get_default_locale();
    let format = "h:mm AM/PM";
    let value = 16.001_423_611_111_11; // =1/86400 => 12:02 AM
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "12:02 AM");
}

#[test]
fn padded_vs_unpadded() {
    let locale = get_default_locale();
    let padded_format = "hh:mm:ss AM/PM";
    let unpadded_format = "h:m:s AM/PM";
    let value = 0.25351851851851853; // => 6:05:04 AM (21904/(24*60*60)) where 21904 = 6 * 3600 + 5*60 + 4
    let formatted = format_number(value, padded_format, locale);
    assert_eq!(formatted.text, "06:05:04 AM");

    let formatted = format_number(value, unpadded_format, locale);
    assert_eq!(formatted.text, "6:5:4 AM");
}

#[test]
fn elapsed_hour() {
    let locale = get_default_locale();
    let format = "[hh]:mm:ss";
    // 1.5 days = 36 hours
    let value = 1.5;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "36:00:00");
}

#[test]
fn elapsed_minute() {
    let locale = get_default_locale();
    let format = "[mm]:ss";
    // 2 days = 2880 minutes
    let value = 2.0;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "2880:00");
}

#[test]
fn elapsed_second() {
    let locale = get_default_locale();
    let format = "[ss]";
    // 0.5 days = 43200 seconds
    let value = 0.5;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "43200");
}

#[test]
fn elapsed_hour_padded() {
    let locale = get_default_locale();
    let format = "[hh]:mm:ss";
    // 0.1 days = 2.4 hours
    let value = 0.1;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "02:24:00");
}

#[test]
fn fractions_of_a_second() {
    // Excel's results, from Apache POI's ElapsedFormatTests.xlsx (Tests!A2:A5) and
    // DateFormatNumberTests.xlsx (Tests!A4:A10, the time part), both saved by Excel.
    let locale = get_default_locale();
    let value = 3.14159; // 75 hours, 23 minutes and 53.376 seconds
    assert_eq!(format_number(value, "[h]:m:s.000", locale).text, "75:23:53.376");
    assert_eq!(format_number(value, "[m]:s.000", locale).text, "4523:53.376");
    assert_eq!(format_number(value, "[s].000", locale).text, "271433.376");
    assert_eq!(format_number(value, "[hh]:mm:ss.0", locale).text, "75:23:53.4");

    let value = 1.0032134017094001;
    assert_eq!(format_number(value, "hh:mm:ss.000", locale).text, "00:04:37.638");
    assert_eq!(format_number(value, "hh:mm:ss.00", locale).text, "00:04:37.64");
    assert_eq!(format_number(value, "hh:mm:ss.0", locale).text, "00:04:37.6");
    assert_eq!(format_number(value, "hh:mm:ss", locale).text, "00:04:38");
    let value = 1.0000047870370301;
    assert_eq!(format_number(value, "hh:mm:ss.000", locale).text, "00:00:00.414");
    assert_eq!(format_number(value, "hh:mm:ss.00", locale).text, "00:00:00.41");
    assert_eq!(format_number(value, "hh:mm:ss.0", locale).text, "00:00:00.4");
}
