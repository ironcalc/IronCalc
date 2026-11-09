#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// Excel rounds what it displays half away from zero, on the number's 15
// significant digit decimal value, and carries into the integer part.
// 0.96, 0.996, =19.99*0.05, 2.5, 9.995, 0.125 and 1234.5678 were checked in
// desktop Excel (Microsoft 365, build 20527); the other cases follow the rule.

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
    test::util::new_empty_model,
};

fn en() -> &'static Locale {
    get_locale("en").unwrap()
}

fn show(value: f64, format: &str) -> String {
    format_number(value, format, en()).text
}

#[test]
fn carries_into_the_integer_part() {
    assert_eq!(show(0.96, "0.0"), "1.0");
    assert_eq!(show(0.996, "0.00"), "1.00");
    assert_eq!(show(19.99 * 0.05, "$#,##0.00"), "$1.00");
    assert_eq!(show(9.995, "0.00"), "10.00");
    assert_eq!(show(99.95, "0.0"), "100.0");
    assert_eq!(show(0.6, "0"), "1");
    assert_eq!(show(0.06, "0.0"), "0.1");
}

#[test]
fn carries_add_a_group_separator() {
    assert_eq!(show(999.995, "#,##0.00"), "1,000.00");
    assert_eq!(show(999_999.5, "#,##0"), "1,000,000");
}

#[test]
fn halves_round_away_from_zero() {
    assert_eq!(show(2.5, "0"), "3");
    assert_eq!(show(0.5, "0"), "1");
    assert_eq!(show(1.5, "0"), "2");
    assert_eq!(show(0.125, "0.00"), "0.13");
    assert_eq!(show(1234.5, "#,##0"), "1,235");
    assert_eq!(show(-2.5, "0"), "-3");
    assert_eq!(show(-0.125, "0.00"), "-0.13");
}

#[test]
fn rounds_the_decimal_value_not_the_binary_one() {
    // The nearest doubles are just below the halfway point
    assert_eq!(show(0.95, "0.0"), "1.0");
    assert_eq!(show(1.005, "0.00"), "1.01");
    assert_eq!(show(2.675, "0.00"), "2.68");
    assert_eq!(
        show(1.0000000000000002, "0.000000000000000"),
        "1.000000000000000"
    );
    assert_eq!(show(0.1 + 0.2, "0.0000000000000000"), "0.3000000000000000");
}

#[test]
fn negative_numbers() {
    assert_eq!(show(-0.96, "0.0"), "-1.0");
    assert_eq!(show(-9.995, "#,##0.00"), "-10.00");
    assert_eq!(show(-0.005, "0.00"), "-0.01");
    assert_eq!(show(-0.5, "0"), "-1");
    assert_eq!(show(-0.96, "0.0;(0.0)"), "(1.0)");
    // A number that rounds to zero shows no sign, as before (not checked in Excel)
    assert_eq!(show(-0.004, "0.00"), "0.00");
}

#[test]
fn percentages() {
    assert_eq!(show(0.995, "0%"), "100%");
    assert_eq!(show(0.0005, "0.0%"), "0.1%");
    assert_eq!(show(0.12345, "0.00%"), "12.35%");
}

#[test]
fn optional_digits() {
    assert_eq!(show(1.996, "0.0#"), "2.0");
    assert_eq!(show(1.25, "0.0#"), "1.25");
    assert_eq!(show(0.996, "#.##"), "1.");
    assert_eq!(show(1.996, "0.0?"), "2.0 ");
}

#[test]
fn unchanged_cases() {
    assert_eq!(show(1234.5678, "#,##0.00"), "1,234.57");
    assert_eq!(show(1_234_567.891, "#,##0.00"), "1,234,567.89");
    assert_eq!(show(0.4, "0"), "0");
    assert_eq!(show(1e-10, "0.00"), "0.00");
    assert_eq!(show(0.0, "0.00"), "0.00");
    assert_eq!(show(3.1, "0.00"), "3.10");
    assert_eq!(show(1e20, "0"), "100000000000000000000");
}

#[test]
fn keeps_15_significant_digits() {
    assert_eq!(show(1234567890123456.7, "0.00"), "1234567890123460.00");
}

#[test]
fn scientific_carries_into_the_exponent() {
    assert_eq!(show(9.996, "0.00E+00"), "1.00E+01");
    assert_eq!(show(99.96, "0.00E+00"), "1.00E+02");
    assert_eq!(show(99_999.0, "0.0E+00"), "1.0E+05");
    assert_eq!(show(0.000_999_999, "0.00E+00"), "1.00E-03");
    assert_eq!(show(-9.996, "0.00E+00"), "-1.00E+01");
}

#[test]
fn scientific_exponent_sign() {
    assert_eq!(show(5.0, "0.00E+00"), "5.00E+00");
    assert_eq!(show(12345.0, "0.00E+00"), "1.23E+04");
    assert_eq!(show(0.5, "0.00E+00"), "5.00E-01");
    assert_eq!(show(0.0, "0.00E+00"), "0.00E+00");
}

#[test]
fn cells_and_text_function() {
    let mut model = new_empty_model();
    model._set("A1", "=19.99*0.05");
    model._set("A2", "=TEXT(9.995,\"0.00\")");
    model._set("A3", "=TEXT(0.96,\"0.0\")");
    model.evaluate();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = "$#,##0.00".to_string();
    model.set_cell_style(0, 1, 1, &style).unwrap();
    assert_eq!(model._get_text("A1"), "$1.00");
    assert_eq!(model._get_text("A2"), "10.00");
    assert_eq!(model._get_text("A3"), "1.0");
}
