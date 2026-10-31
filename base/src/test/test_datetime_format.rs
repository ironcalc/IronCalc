#![allow(clippy::unwrap_used)]

use crate::formatter::format::format_number;
use crate::locale::get_locale;
use crate::model::Model;

#[test]
fn us_locale() {
    // en-US locale with MM/DD/YYYY format
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();

    model._set("A1", "10/02/2026");
    model._set("A2", "=DAY(A1)");
    model._set("A3", "=MONTH(A1)");
    model._set("A4", "=YEAR(A1)");
    model.evaluate();

    assert_eq!(model._get_text("A2"), "2");
    assert_eq!(model._get_text("A3"), "10");
    assert_eq!(model._get_text("A4"), "2026");
}

#[test]
fn uk_locale() {
    // en-GB locale with DD/MM/YYYY format
    let mut model = Model::new_empty("model", "en-GB", "UTC", "en").unwrap();

    model._set("A1", "10/02/2026");
    model._set("A2", "=DAY(A1)");
    model._set("A3", "=MONTH(A1)");
    model._set("A4", "=YEAR(A1)");
    model.evaluate();

    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "2026");
}

#[test]
fn es_locale() {
    // es-ES locale with DD/MM/YYYY format
    let mut model = Model::new_empty("model", "es", "UTC", "en").unwrap();

    model._set("A1", "10/marzo/2026");
    model._set("A2", "=DAY(A1)");
    model._set("A3", "=MONTH(A1)");
    model._set("A4", "=YEAR(A1)");
    model.evaluate();

    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("A4"), "2026");
}

#[test]
fn fr_locale() {
    // fr-FR locale with DD/MM/YYYY format
    let mut model = Model::new_empty("model", "fr", "UTC", "en").unwrap();
    model._set("A1", "10/mars/2026");
    model._set("A2", "=DAY(A1)");
    model._set("A3", "=MONTH(A1)");
    model._set("A4", "=YEAR(A1)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "3");
    assert_eq!(model._get_text("A4"), "2026");
}

#[test]
fn ja_locale() {
    // ja-JP locale with YYYY/MM/DD format
    let mut model = Model::new_empty("model", "ja", "UTC", "ja").unwrap();
    model._set("A1", "2026/02/10");
    model._set("A2", "=DAY(A1)");
    model._set("A3", "=MONTH(A1)");
    model._set("A4", "=YEAR(A1)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "2026");

    let settings = model.get_fmt_settings();
    assert_eq!(settings.short_date_example, "2025/12/15");
    assert_eq!(settings.long_date_example, "2025年12月15日");
    assert_eq!(settings.currency, "JPY");

    // The full CLDR date format uses the Japanese day names
    let locale = get_locale("ja").unwrap();
    let full = &locale.dates.date_formats.full;
    assert_eq!(full, "yyyy\"年\"m\"月\"d\"日\"dddd");
    assert_eq!(
        format_number(46006.0, full, locale).text,
        "2025年12月15日月曜日"
    );
}
