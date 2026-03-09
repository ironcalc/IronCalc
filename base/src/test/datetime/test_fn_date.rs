//! Test suite for the Excel `DATE(year, month, day)` function.
//!
//! `DATE` converts numeric year/month/day arguments into an Excel serial date
//! number.  It is *permissive*: out-of-range months and days roll over into the
//! adjacent period rather than immediately returning an error.
//!
//! Errors that ARE returned:
//!   - Wrong argument count ->`#ERROR!`
//!   - Non-numeric argument ->`#VALUE!`
//!   - Negative year        ->`#NUM!`
//!   - Result outside the supported range (1899-12-31 … 9999-12-31) -> `#NUM!`
//!
//! Notable IronCalc vs Excel differences tested here:
//!   - IronCalc does NOT add 1900 to year values in [0, 1899].
//!   - IronCalc does NOT treat 1900 as a leap year.

use crate::test::util::{assert_formulas, setup_model};

#[test]
fn test_date_wrong_arg_count() {
    assert_formulas(&[
        ("=DATE()", "#ERROR!"),
        ("=DATE(2023)", "#ERROR!"),
        ("=DATE(2023,6)", "#ERROR!"),
        ("=DATE(2023,6,15,0)", "#ERROR!"),
    ]);
}

#[test]
fn test_date_non_numeric_args() {
    assert_formulas(&[
        (r#"=DATE("foo",1,1)"#, "#VALUE!"),
        (r#"=DATE(2023,"bar",1)"#, "#VALUE!"),
        (r#"=DATE(2023,6,"baz")"#, "#VALUE!"),
    ]);
}

#[test]
fn test_date_normal_dates() {
    assert_formulas(&[
        ("=DATE(2023,6,15)", "6/15/2023"),
        ("=DATE(2023,1,1)", "1/1/2023"),
        ("=DATE(2023,12,31)", "12/31/2023"),
        // Early supported dates
        ("=DATE(1900,1,1)", "1/1/1900"),
        ("=DATE(1900,3,1)", "3/1/1900"),
    ]);
}

#[test]
fn test_date_decimal_args_are_truncated() {
    assert_formulas(&[
        // All fractional parts discarded; result equals DATE(2023,6,15)
        ("=DATE(2023.9,6.8,15.99)", "6/15/2023"),
        // Also holds for fractional parts below 0.5
        ("=DATE(2023.1,6.1,15.1)", "6/15/2023"),
    ]);
}

#[test]
fn test_date_month_overflow() {
    assert_formulas(&[
        // Month 13 rolls into January of the next year
        ("=DATE(2022,13,1)", "1/1/2023"),
        // Month 25 advances two calendar years
        ("=DATE(2022,25,1)", "1/1/2024"),
        // Month 0 rolls back to December of the previous year
        ("=DATE(2023,0,1)", "12/1/2022"),
        // Month -1 rolls back to November of the previous year
        ("=DATE(2023,-1,1)", "11/1/2022"),
    ]);
}

#[test]
fn test_date_day_overflow() {
    assert_formulas(&[
        // Day 0 is the last day of the previous month (Feb - Mar boundary)
        ("=DATE(2023,3,0)", "2/28/2023"),
        // Day 32 in January wraps to February
        ("=DATE(2023,1,32)", "2/1/2023"),
        // Feb 30 (non-existent in 2023) rolls to March
        ("=DATE(2023,2,30)", "3/2/2023"),
        // Negative day rolls further back
        ("=DATE(2023,3,-1)", "2/27/2023"),
    ]);
}

#[test]
fn test_date_negative_year_is_num() {
    assert_formulas(&[("=DATE(-1,1,1)", "#NUM!"), ("=DATE(-100,6,15)", "#NUM!")]);
}

#[test]
fn test_date_year_before_supported_range() {
    assert_formulas(&[
        ("=DATE(0,1,1)", "#NUM!"),
        ("=DATE(1,1,1)", "#NUM!"),
        ("=DATE(1899,1,1)", "#NUM!"),
        // Dec 30, 1899 is one day before the minimum; the general path rejects it
        // because the initial Jan 1, 1899 anchor is already out of range.
        ("=DATE(1899,12,30)", "#NUM!"),
    ]);
}

#[test]
fn test_date_minimum_boundary() {
    assert_formulas(&[("=DATE(1899,12,31)", "12/31/1899")]);
}

#[test]
fn test_date_maximum_boundary() {
    assert_formulas(&[
        ("=DATE(9999,12,31)", "12/31/9999"),
        // Year past the maximum
        ("=DATE(10000,1,1)", "#NUM!"),
        // Permissive day overflow that lands past the maximum is also rejected
        ("=DATE(9999,12,32)", "#NUM!"),
    ]);
}

#[test]
fn test_date_1900_is_not_a_leap_year() {
    assert_formulas(&[("=DATE(1900,2,29)", "3/1/1900")]);
}

#[test]
fn test_date_leap_year_boundaries() {
    assert_formulas(&[
        // 2023 is not a leap year; Feb 29 rolls to Mar 1
        ("=DATE(2023,2,29)", "3/1/2023"),
        // 2024 is a leap year; Feb 29 is valid
        ("=DATE(2024,2,29)", "2/29/2024"),
        // 2100 is NOT a leap year (÷100 but not ÷400)
        ("=DATE(2100,2,29)", "3/1/2100"),
        // 2000 IS a leap year (÷400)
        ("=DATE(2000,2,29)", "2/29/2000"),
    ]);
}

#[test]
fn test_date_roundtrip_with_year_month_day() {
    assert_formulas(&[
        ("=YEAR(DATE(2023,6,15))", "2023"),
        ("=MONTH(DATE(2023,6,15))", "6"),
        ("=DAY(DATE(2023,6,15))", "15"),
        ("=YEAR(DATE(1900,1,1))", "1900"),
        ("=MONTH(DATE(1900,1,1))", "1"),
        ("=DAY(DATE(1900,1,1))", "1"),
        ("=YEAR(DATE(9999,12,31))", "9999"),
        ("=MONTH(DATE(9999,12,31))", "12"),
        ("=DAY(DATE(9999,12,31))", "31"),
    ]);
}

#[test]
fn test_date_cell_references() {
    let model = setup_model(&[
        ("A1", "2023"),
        ("B1", "11"),
        ("C1", "5"),
        // DATE reads its arguments from other cells
        ("D1", "=DATE(A1,B1,C1)"),
        // Works with arithmetic on cell references too (month 12, day 1)
        ("D2", "=DATE(A1,B1+1,C1-4)"),
    ]);
    assert_eq!(model._get_text("D1"), "11/5/2023");
    assert_eq!(model._get_text("D2"), "12/1/2023");
}

#[test]
fn test_date_propagates_errors_from_dependencies() {
    let model = setup_model(&[
        // A1 holds a divide-by-zero error
        ("A1", "=1/0"),
        ("B1", "=DATE(A1,1,1)"),    // error in year
        ("C1", "=DATE(2023,A1,1)"), // error in month
        ("D1", "=DATE(2023,6,A1)"), // error in day
    ]);
    assert_eq!(model._get_text("B1"), "#DIV/0!");
    assert_eq!(model._get_text("C1"), "#DIV/0!");
    assert_eq!(model._get_text("D1"), "#DIV/0!");
}

#[test]
fn test_date_serial_via_value() {
    assert_formulas(&[
        // Well-known anchor: Feb 29, 1976 is serial 27819 (used in many tests)
        ("=VALUE(DATE(1976,2,29))", "27819"),
        // Minimum supported date has serial 1
        ("=VALUE(DATE(1899,12,31))", "1"),
        // Jan 1, 1900 is serial 2 in IronCalc (Excel bug shifts serials by 1
        // before Mar 1, 1900 relative to the proleptic Gregorian calendar)
        ("=VALUE(DATE(1900,1,1))", "2"),
        // A recent date
        ("=VALUE(DATE(2023,6,15))", "45092"),
        // Maximum supported date
        ("=VALUE(DATE(9999,12,31))", "2958465"),
    ]);
}

#[test]
fn test_date_serial_via_value_cell_dependency() {
    let model = setup_model(&[
        ("A1", "=DATE(2023,6,15)"), // displays as "6/15/2023" (date-formatted)
        ("B1", "=VALUE(A1)"),       // strips the date format, exposes serial
    ]);
    assert_eq!(model._get_text("A1"), "6/15/2023");
    assert_eq!(model._get_text("B1"), "45092");
}
