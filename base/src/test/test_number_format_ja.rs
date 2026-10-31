#![allow(clippy::unwrap_used)]

use crate::formatter::dates::date_to_serial_number;
use crate::formatter::lexer::is_likely_date_number_format;
use crate::number_format::format_number;
use crate::test::util::new_empty_model;

fn format_date(year: i32, month: u32, day: u32, format: &str) -> String {
    let serial = date_to_serial_number(day, month, year).unwrap() as f64;
    format_number(serial, format, "ja").text
}

const WAREKI: &str = "ggge\"年\"m\"月\"d\"日\"";

#[test]
fn japanese_era_codes() {
    // 2025-12-15 is a Monday
    assert_eq!(format_date(2025, 12, 15, WAREKI), "令和7年12月15日");
    assert_eq!(format_date(2025, 12, 15, "gge\"年\""), "令7年");
    assert_eq!(format_date(2025, 12, 15, "ge.m.d"), "R7.12.15");
    assert_eq!(format_date(2025, 12, 15, "gggee\"年\""), "令和07年");
    assert_eq!(format_date(2025, 12, 15, "yyyy/m/d(aaa)"), "2025/12/15(月)");
    assert_eq!(format_date(2025, 12, 15, "aaaa"), "月曜日");
    // Japanese weekday names do not depend on the locale
    let serial = date_to_serial_number(14, 12, 2025).unwrap() as f64;
    assert_eq!(format_number(serial, "aaa", "en").text, "日");
}

#[test]
fn japanese_era_boundaries() {
    let cases = [
        ((1912, 7, 29), "明治45年7月29日"),
        ((1912, 7, 30), "大正1年7月30日"),
        ((1926, 12, 24), "大正15年12月24日"),
        ((1926, 12, 25), "昭和1年12月25日"),
        ((1989, 1, 7), "昭和64年1月7日"),
        ((1989, 1, 8), "平成1年1月8日"),
        ((2019, 4, 30), "平成31年4月30日"),
        ((2019, 5, 1), "令和1年5月1日"),
        ((1900, 1, 1), "明治33年1月1日"),
    ];
    for ((year, month, day), expected) in cases {
        assert_eq!(format_date(year, month, day, WAREKI), expected);
    }
}

#[test]
fn locale_tags() {
    // Japanese Excel writes these prefixes in the files
    assert_eq!(
        format_date(2025, 12, 15, "[$-411]ggge\"年\"m\"月\"d\"日\""),
        "令和7年12月15日"
    );
    assert_eq!(
        format_date(2025, 12, 15, "[$-ja-JP]yyyy\"年\"m\"月\"d\"日\";@"),
        "2025年12月15日"
    );
    assert_eq!(
        format_date(2025, 12, 15, "[$-x-sysdate]dddd, mmmm d, yyyy"),
        "月曜日, 12月 15, 2025"
    );
    // A currency with a locale still works
    assert_eq!(format_number(1234.0, "[$¥-411]#,##0", "ja").text, "¥1,234");
    assert_eq!(format_number(1234.0, "[$-411]#,##0", "ja").text, "1,234");
}

#[test]
fn general_and_scientific_still_work() {
    assert_eq!(format_number(2.5, "General", "en").text, "2.5");
    assert_eq!(format_number(2.5, "general", "en").text, "2.5");
    assert_eq!(format_number(12345.0, "0.00E+00", "en").text, "1.23E+04");
    assert_eq!(format_number(12345.0, "0.00e+00", "en").text, "1.23E+04");
    assert_eq!(format_number(0.5, "h:mm AM/PM", "en").text, "12:00 PM");
}

#[test]
fn era_formats_are_dates() {
    assert!(is_likely_date_number_format(WAREKI));
    assert!(is_likely_date_number_format("[$-411]ge.m.d"));
    assert!(!is_likely_date_number_format("[$-411]#,##0"));
}

#[test]
fn text_function_with_era() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXT(DATE(2025,12,15),\"ggge年m月d日(aaa)\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "令和7年12月15日(月)");
}
