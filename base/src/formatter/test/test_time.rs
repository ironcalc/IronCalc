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
fn rounded_seconds_carry_into_minutes() {
    let locale = get_default_locale();
    // =DATE(2026,10,27)+TIME(14,30,0) is stored as 46322.604166666664: the nearest
    // double to 14:30 sits a whisker below the whole minute and must display 14:30
    let value = 46_322.604_166_666_664_f64;
    let formatted = format_number(value, "m/d/yy h:mm", locale);
    assert_eq!(formatted.text, "10/27/26 14:30");

    let formatted = format_number(value, "h:mm:ss", locale);
    assert_eq!(formatted.text, "14:30:00");

    let formatted = format_number(46322.0 + 14.5 / 24.0, "h:mm", locale);
    assert_eq!(formatted.text, "14:30");
}

#[test]
fn fractional_seconds_round_to_the_displayed_minute() {
    let locale = get_default_locale();
    // 14:29:59.7 with a minute-only format rounds to the next minute
    let value = (14.0 * 3600.0 + 29.0 * 60.0 + 59.7) / (24.0 * 60.0 * 60.0);
    let formatted = format_number(value, "h:mm", locale);
    assert_eq!(formatted.text, "14:30");
}

#[test]
fn rounding_at_midnight_carries_into_the_date() {
    let locale = get_default_locale();
    // 10/27/26 23:59:59.7 formatted with h:mm rounds to midnight of the 28th
    let value = 46322.0 + ((23.0 * 60.0 + 59.0) * 60.0 + 59.7) / (24.0 * 60.0 * 60.0);
    let formatted = format_number(value, "m/d/yy h:mm", locale);
    assert_eq!(formatted.text, "10/28/26 0:00");
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
