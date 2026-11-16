#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::test::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// Excel rounds a date-time to the nearest second and then shows the units the
// format asks for. SheetJS's table of Excel's output for the time formats
// (test/times.tsv in SheetJS/ssf, 1,048,575 values in h, hh:mm, ss, [h], [m],
// [s] and more) follows this rule in every cell, and Excel's SECOND function
// rounds the same way.
//
// 18:58, as Ctrl+Shift+; stores it, is 0.7902777777777777: 68279.99999999999
// seconds, so cutting off the fraction showed 18:57 and 18:57:60.
#[test]
fn rounds_to_the_nearest_second() {
    let locale = get_default_locale();
    let value = 0.790_277_777_777_777_7;
    assert_eq!(format_number(value, "h:mm", locale).text, "18:58");
    assert_eq!(format_number(value, "h:mm AM/PM", locale).text, "6:58 PM");
    assert_eq!(format_number(value, "h:mm:ss", locale).text, "18:58:00");
    assert_eq!(
        format_number(value, "hh:mm:ss AM/PM", locale).text,
        "06:58:00 PM"
    );
}

// The seconds round, the minutes do not: 12:00:36 is 12:00 in h:mm. A time
// 0.4 seconds before a minute rounds up to it whether or not the format shows
// seconds: 12:00:59.6 is 12:01:00, and 12:00:59.4 stays 12:00:59.
#[test]
fn seconds_round_minutes_do_not() {
    let locale = get_default_locale();
    assert_eq!(
        format_number(43_236.0 / 86_400.0, "h:mm", locale).text,
        "12:00"
    );
    assert_eq!(
        format_number(43_259.6 / 86_400.0, "h:mm", locale).text,
        "12:01"
    );
    assert_eq!(
        format_number(43_259.6 / 86_400.0, "h:mm:ss", locale).text,
        "12:01:00"
    );
    assert_eq!(
        format_number(43_259.4 / 86_400.0, "h:mm", locale).text,
        "12:00"
    );
    assert_eq!(
        format_number(43_259.4 / 86_400.0, "h:mm:ss", locale).text,
        "12:00:59"
    );
}

// Elapsed time counts the rounded seconds: 25:59:59.6 is 26:00:00.
#[test]
fn elapsed_time() {
    let locale = get_default_locale();
    let value = 93_599.6 / 86_400.0;
    assert_eq!(format_number(value, "[h]:mm", locale).text, "26:00");
    assert_eq!(format_number(value, "[h]:mm:ss", locale).text, "26:00:00");
    assert_eq!(
        format_number(59.6 / 86_400.0, "[mm]:ss", locale).text,
        "01:00"
    );
    assert_eq!(format_number(59.6 / 86_400.0, "[ss]", locale).text, "60");
    assert_eq!(
        format_number(0.790_277_777_777_777_7, "[h]:mm", locale).text,
        "18:58"
    );
}

// A time that rounds up to midnight is the next day: 2024-01-01 23:59:59.9996
// is 2024-01-02 00:00:00.
#[test]
fn rounding_carries_into_the_next_day() {
    let locale = get_default_locale();
    let value = 45_292.999_999_995_37;
    assert_eq!(
        format_number(value, "yyyy-mm-dd hh:mm:ss", locale).text,
        "2024-01-02 00:00:00"
    );
    assert_eq!(
        format_number(value, "m/d/yyyy h:mm", locale).text,
        "1/2/2024 0:00"
    );
    assert_eq!(format_number(value, "h:mm", locale).text, "0:00");
    assert_eq!(format_number(value, "[h]:mm", locale).text, "1087032:00");
}

// `m` and `mm` just before the seconds are minutes, not months: 00:59:59.6
// rounds to 01:00:00, which is 00:00 in mm:ss.
#[test]
fn minutes_before_seconds() {
    let locale = get_default_locale();
    assert_eq!(
        format_number(3_599.6 / 86_400.0, "mm:ss", locale).text,
        "00:00"
    );
    assert_eq!(
        format_number(754.0 / 86_400.0, "mm:ss", locale).text,
        "12:34"
    );
    assert_eq!(
        format_number(754.0 / 86_400.0, "m:ss", locale).text,
        "12:34"
    );
    // A month is still a month
    assert_eq!(
        format_number(45_292.0, "mm/dd/yyyy", locale).text,
        "01/01/2024"
    );
}

// The fraction of a second the format shows is rounded, not the second:
// 12:00:59.6 is 12:00:59.6 in h:mm:ss.0
#[test]
fn fractions_of_a_second() {
    let locale = get_default_locale();
    let value = 43_259.6 / 86_400.0;
    assert_eq!(format_number(value, "h:mm:ss.0", locale).text, "12:00:59.6");
    assert_eq!(
        format_number(value, "h:mm:ss.00", locale).text,
        "12:00:59.60"
    );
    assert_eq!(
        format_number(43_259.96 / 86_400.0, "h:mm:ss.00", locale).text,
        "12:00:59.96"
    );
    assert_eq!(
        format_number(43_259.96 / 86_400.0, "h:mm:ss.0", locale).text,
        "12:01:00.0"
    );
    assert_eq!(
        format_number(1.234 / 86_400.0, "[ss].000", locale).text,
        "01.234"
    );
    assert_eq!(
        format_number(1.234 / 86_400.0, "mm:ss.00", locale).text,
        "00:01.23"
    );
}

// A time on 9999-12-31 that rounds up to midnight is past the last date, even
// when the format only shows the time.
#[test]
fn rounding_past_the_last_date() {
    let locale = get_default_locale();
    let value = 2_958_465.0 + 86_399.6 / 86_400.0;
    assert_eq!(format_number(value, "h:mm:ss", locale).text, "#VALUE!");
    assert_eq!(
        format_number(2_958_465.0 + 86_399.4 / 86_400.0, "h:mm:ss", locale).text,
        "23:59:59"
    );
}
