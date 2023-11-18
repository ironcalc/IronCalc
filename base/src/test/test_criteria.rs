use crate::calc_result::CalcResult;
use crate::functions::util::build_criteria;

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
    let fn_criteria = build_criteria(&c);
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String("42".to_string())));
    assert!(fn_criteria(&CalcResult::String("42.00".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));

    let c = CalcResult::String("=42".to_string());
    let fn_criteria = build_criteria(&c);
    assert!(fn_criteria(&CalcResult::Number(42.0)));
    assert!(fn_criteria(&CalcResult::String("42".to_string())));
    assert!(fn_criteria(&CalcResult::String("42.00".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(2.0)));
}

#[test]
fn test_build_criteria_is_bool() {
    let c = CalcResult::Boolean(true);
    let fn_criteria = build_criteria(&c);
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("true".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));

    let c = CalcResult::String("=True".to_string());
    let fn_criteria = build_criteria(&c);
    assert!(fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("true".to_string())));
    assert!(!fn_criteria(&CalcResult::Number(1.0)));
}

#[test]
fn test_build_criteria_is_less_than() {
    let c = CalcResult::String("<100".to_string());
    let fn_criteria = build_criteria(&c);
    assert!(!fn_criteria(&CalcResult::Boolean(true)));
    assert!(!fn_criteria(&CalcResult::String("23".to_string())));
    assert!(fn_criteria(&CalcResult::Number(1.0)));
    assert!(!fn_criteria(&CalcResult::Number(101.0)));
}

#[test]
fn test_build_criteria_is_less_wildcard() {
    let c = CalcResult::String("=D* G*".to_string());
    let fn_criteria = build_criteria(&c);
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
