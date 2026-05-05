#![allow(clippy::unwrap_used)]

use crate::calc_result::CalcResult;
use crate::functions::util::build_criteria;
use crate::locale::{get_locale, Locale};

fn en_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// Tests for build_criteria
// ------------------------
//
// Note that any test here is mostly for documentation purposes.
// A real test must be done in Excel.
//
// `build_criteria` takes a string ('criteria') and returns a function ('fn_criteria') that takes a CalcResult and returns a boolean.
//
// For instance if criteria is "123" we want all cells that contain the number "123". Then
//
// let fn_criteria = build_criteria(&CalcResult::Number(123));
//
// Then fn_criteria(calc_result) will return true every time calc_result is the number "123"
//
// There are different types of criteria
//
// * We want the cells that are equal to a value (say a number, string, bool or an error).
//   We can build those with a calc_result of the type (i.e CalcResult::Number(123))
//   or we can use a string preceded by "=" like CalcResult::String("=123")
// * We can use inequality signs "<", ">", "<=", ">=" or "<>"
// * If you use "=" or "<>" you can use wildcards (like "=*brown")
//
// All of them are case insensitive.

#[test]
fn test_build_criteria_is_number() {
    let c = CalcResult::Number(42.0);
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String("42".to_string())));
    assert!(fn_criteria(&CalcResult::String("42.00".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));

    let c = CalcResult::String("=42".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String("42".to_string())));
    assert!(fn_criteria(&CalcResult::String("42.00".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));
}

#[test]
fn test_build_criteria_is_bool() {
    let c = CalcResult::Boolean(true);
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("true".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));

    let c = CalcResult::String("=True".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("true".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));
}

#[test]
fn test_build_criteria_is_less_than() {
    let c = CalcResult::String("<100".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(!fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("23".to_string())));
    assert!(fn_criteria(&CalcResult::Number(1.0)));
    assert!(!fn_criteria(&CalcResult::Number(101.0)));
}

#[test]
fn test_build_criteria_is_less_wildcard() {
    let c = CalcResult::String("=D* G*".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::String(
        "Diarmuid Glynn".to_string()
    )));
    assert!(fn_criteria(&CalcResult::String(
        "Daniel Gonzalez".to_string()
    )));
    assert!(!fn_criteria(&CalcResult::String(
        "DanielGonzalez".to_string()
    )));
    assert!(!fn_criteria(&CalcResult::String(
        " Daniel Gonzalez".to_string()
    )));
}

// Date-string criteria. Excel parses literals like "7/31/2023" into a serial
// (45138 in this case) and applies the comparison numerically against
// date-serial cells. These tests cover all six branches of build_criteria.
//
// The neighbouring serials used below:
//   45137 = 7/30/2023
//   45138 = 7/31/2023  (the criterion target)
//   45139 = 8/1/2023

#[test]
fn test_build_criteria_date_less_than() {
    let c = CalcResult::String("<7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_less_or_equal() {
    let c = CalcResult::String("<=7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_greater_than() {
    let c = CalcResult::String(">7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_greater_or_equal() {
    let c = CalcResult::String(">=7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_not_equal() {
    let c = CalcResult::String("<>7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(fn_criteria(&CalcResult::Number(45137.0)));
    assert!(!fn_criteria(&CalcResult::Number(45138.0)));
    assert!(fn_criteria(&CalcResult::Number(45139.0)));
}

#[test]
fn test_build_criteria_date_equal() {
    let c = CalcResult::String("7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));

    // Same with a leading "=" prefix.
    let c = CalcResult::String("=7/31/2023".to_string());
    let fn_criteria = build_criteria(&c, en_locale());
    assert!(!fn_criteria(&CalcResult::Number(45137.0)));
    assert!(fn_criteria(&CalcResult::Number(45138.0)));
    assert!(!fn_criteria(&CalcResult::Number(45139.0)));
}
