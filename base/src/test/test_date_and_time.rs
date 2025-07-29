#![allow(clippy::unwrap_used)]

/// Here we add tests that cannot be done in Excel
/// Either because Excel does not have that feature (i.e. wrong number of arguments)
/// or because we differ from Excel throwing #NUM! on invalid dates
/// We can also enter examples that illustrate/document a part of the function
use crate::{cell::CellValue, test::util::new_empty_model};

// Excel uses a serial date system where Jan 1, 1900 = 1 (though it treats 1900 as a leap year)
// Most test dates are documented inline, but we define boundary values here:
const EXCEL_MAX_DATE: f64 = 2958465.0; // Dec 31, 9999 - used in boundary tests
const EXCEL_INVALID_DATE: f64 = 2958466.0; // One day past max - used in error tests

#[test]
fn test_fn_date_arguments() {
    let mut model = new_empty_model();

    // Wrong number of arguments produce #ERROR!
    // NB: Excel does not have this error, but does not let you enter wrong number of arguments in the UI
    model._set("A1", "=DATE()");
    model._set("A2", "=DATE(1975)");
    model._set("A3", "=DATE(1975, 2)");
    model._set("A4", "=DATE(1975, 2, 10, 3)");

    // Arguments are out of rage. This is against Excel
    // Excel will actually compute a date by continuing to the next month, year...
    // We throw #NUM!
    model._set("A5", "=DATE(1975, -2, 10)");
    model._set("A6", "=DATE(1975, 2, -10)");
    model._set("A7", "=DATE(1975, 14, 10)");
    // February doesn't have 30 days
    model._set("A8", "=DATE(1975, 2, 30)");

    // 1975, a great year, wasn't a leap year
    model._set("A9", "=DATE(1975, 2, 29)");
    // 1976 was
    model._set("A10", "=DATE(1976, 2, 29)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"10/10/1974");
    assert_eq!(model._get_text("A6"), *"21/01/1975");
    assert_eq!(model._get_text("A7"), *"10/02/1976");
    assert_eq!(model._get_text("A8"), *"02/03/1975");

    assert_eq!(model._get_text("A9"), *"01/03/1975");
    assert_eq!(model._get_text("A10"), *"29/02/1976");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A10"),
        Ok(CellValue::Number(27819.0))
    );
}

#[test]
fn test_date_out_of_range() {
    let mut model = new_empty_model();

    // month
    model._set("A1", "=DATE(2022, 0, 10)");
    model._set("A2", "=DATE(2022, 13, 10)");

    // day
    model._set("B1", "=DATE(2042, 5, 0)");
    model._set("B2", "=DATE(2025, 5, 32)");

    // year (actually years < 1900 don't really make sense)
    model._set("C1", "=DATE(-1, 5, 5)");
    // excel is not compatible with years past 9999
    model._set("C2", "=DATE(10000, 5, 5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"10/12/2021");
    assert_eq!(model._get_text("A2"), *"10/01/2023");
    assert_eq!(model._get_text("B1"), *"30/04/2042");
    assert_eq!(model._get_text("B2"), *"01/06/2025");

    assert_eq!(model._get_text("C1"), *"#NUM!");
    assert_eq!(model._get_text("C2"), *"#NUM!");
}

#[test]
fn test_year_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=YEAR()");
    model._set("A2", "=YEAR(27819)");
    model._set("A3", "=YEAR(27819, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"1976");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_month_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=MONTH()");
    model._set("A2", "=MONTH(27819)");
    model._set("A3", "=MONTH(27819, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"2");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_day_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DAY()");
    model._set("A2", "=DAY(27819)");
    model._set("A3", "=DAY(27819, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"29");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_day_small_serial() {
    let mut model = new_empty_model();
    model._set("A1", "=DAY(-1)");
    model._set("A2", "=DAY(0)");
    model._set("A3", "=DAY(60)");

    model._set("A4", "=DAY(61)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    // Excel thinks is Feb 29, 1900
    assert_eq!(model._get_text("A3"), *"28");

    // From now on everyone agrees
    assert_eq!(model._get_text("A4"), *"1");
}

#[test]
fn test_month_small_serial() {
    let mut model = new_empty_model();
    model._set("A1", "=MONTH(-1)");
    model._set("A2", "=MONTH(0)");
    model._set("A3", "=MONTH(60)");

    model._set("A4", "=MONTH(61)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    // We agree with Excel here (We are both in Feb)
    assert_eq!(model._get_text("A3"), *"2");

    // Same as Excel
    assert_eq!(model._get_text("A4"), *"3");
}

#[test]
fn test_year_small_serial() {
    let mut model = new_empty_model();
    model._set("A1", "=YEAR(-1)");
    model._set("A2", "=YEAR(0)");
    model._set("A3", "=YEAR(60)");

    model._set("A4", "=YEAR(61)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");

    assert_eq!(model._get_text("A3"), *"1900");

    // Same as Excel
    assert_eq!(model._get_text("A4"), *"1900");
}

#[test]
fn test_date_early_dates() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(1900, 1, 1)");
    model._set("A2", "=DATE(1900, 2, 28)");
    model._set("B2", "=DATE(1900, 2, 29)");
    model._set("A3", "=DATE(1900, 3, 1)");

    model.evaluate();

    // This is 1 in Excel, we agree with Google Docs
    assert_eq!(model._get_text("A1"), *"01/01/1900");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(2.0))
    );

    // 1900 was not a leap year, this is a bug in EXCEL
    // This would be 60 in Excel
    assert_eq!(model._get_text("A2"), *"28/02/1900");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A2"),
        Ok(CellValue::Number(60.0))
    );

    // This does not agree with Excel, instead of mistakenly allowing
    // for Feb 29, it will auto-wrap to the next day after Feb 28.
    assert_eq!(model._get_text("B2"), *"01/03/1900");

    // This agrees with Excel from he onward
    assert_eq!(model._get_text("A3"), *"01/03/1900");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A3"),
        Ok(CellValue::Number(61.0))
    );
}
#[test]
fn test_days_function() {
    let mut model = new_empty_model();

    // Basic functionality
    model._set("A1", "=DAYS(44570,44561)");
    model._set("A2", "=DAYS(44561,44570)"); // Reversed order
    model._set("A3", "=DAYS(44561,44561)");

    // Edge cases
    model._set("A4", "=DAYS(1,2)"); // Early dates
    model._set(
        "A5",
        &format!("=DAYS({},{})", EXCEL_MAX_DATE, EXCEL_MAX_DATE - 1.0),
    ); // Near max date

    // Error cases - wrong argument count
    model._set("A6", "=DAYS()");
    model._set("A7", "=DAYS(44561)");
    model._set("A8", "=DAYS(44561,44570,1)");

    // Error cases - invalid dates
    model._set("A9", "=DAYS(-1,44561)");
    model._set("A10", &format!("=DAYS(44561,{EXCEL_INVALID_DATE})"));

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9");
    assert_eq!(model._get_text("A2"), *"-9");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"-1"); // DAYS(1,2) = 1-2 = -1
    assert_eq!(model._get_text("A5"), *"1");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
    assert_eq!(model._get_text("A10"), *"#NUM!");
}

#[test]
fn test_days360_function() {
    let mut model = new_empty_model();

    // Basic functionality with different basis values
    model._set("A1", "=DAYS360(44196,44560)"); // Default basis (US 30/360)
    model._set("A2", "=DAYS360(44196,44560,FALSE)"); // US 30/360 explicitly
    model._set("A3", "=DAYS360(44196,44560,TRUE)"); // European 30/360

    // Same date
    model._set("A4", "=DAYS360(44561,44561)");
    model._set("A5", "=DAYS360(44561,44561,TRUE)");

    // Reverse order (negative result)
    model._set("A6", "=DAYS360(44560,44196)");
    model._set("A7", "=DAYS360(44560,44196,TRUE)");

    // Edge cases
    model._set("A8", "=DAYS360(1,2)");
    model._set("A9", "=DAYS360(1,2,FALSE)");

    // Error cases - wrong argument count
    model._set("A10", "=DAYS360()");
    model._set("A11", "=DAYS360(44561)");
    model._set("A12", "=DAYS360(44561,44570,TRUE,1)");

    // Error cases - invalid dates
    model._set("A13", "=DAYS360(-1,44561)");
    model._set("A14", &format!("=DAYS360(44561,{EXCEL_INVALID_DATE})"));

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"360");
    assert_eq!(model._get_text("A2"), *"360");
    assert_eq!(model._get_text("A3"), *"360");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
    assert_eq!(model._get_text("A6"), *"-360");
    assert_eq!(model._get_text("A7"), *"-360");
    assert_eq!(model._get_text("A8"), *"1");
    assert_eq!(model._get_text("A9"), *"1");
    assert_eq!(model._get_text("A10"), *"#ERROR!");
    assert_eq!(model._get_text("A11"), *"#ERROR!");
    assert_eq!(model._get_text("A12"), *"#ERROR!");
    assert_eq!(model._get_text("A13"), *"#NUM!");
    assert_eq!(model._get_text("A14"), *"#NUM!");
}

#[test]
fn test_weekday_function() {
    let mut model = new_empty_model();

    // Test return_type parameter variations with one known date (Friday 44561)
    model._set("A1", "=WEEKDAY(44561)"); // Default: Sun=1, Fri=6
    model._set("A2", "=WEEKDAY(44561,2)"); // Mon=1, Fri=5
    model._set("A3", "=WEEKDAY(44561,3)"); // Mon=0, Fri=4

    // Test boundary days (Sun/Mon) to verify return_type logic
    model._set("A4", "=WEEKDAY(44556,1)"); // Sunday: should be 1
    model._set("A5", "=WEEKDAY(44556,2)"); // Sunday: should be 7
    model._set("A6", "=WEEKDAY(44557,2)"); // Monday: should be 1

    // Error cases
    model._set("A7", "=WEEKDAY()"); // Wrong arg count
    model._set("A8", "=WEEKDAY(44561,0)"); // Invalid return_type
    model._set("A9", "=WEEKDAY(-1)"); // Invalid date

    model.evaluate();

    // Core functionality
    assert_eq!(model._get_text("A1"), *"6"); // Friday default
    assert_eq!(model._get_text("A2"), *"5"); // Friday Mon=1
    assert_eq!(model._get_text("A3"), *"4"); // Friday Mon=0

    // Boundary verification
    assert_eq!(model._get_text("A4"), *"1"); // Sunday Sun=1
    assert_eq!(model._get_text("A5"), *"7"); // Sunday Mon=1
    assert_eq!(model._get_text("A6"), *"1"); // Monday Mon=1

    // Error cases
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#VALUE!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
}

#[test]
fn test_weeknum_function() {
    let mut model = new_empty_model();

    // Test different return_type values (1=week starts Sunday, 2=week starts Monday)
    model._set("A1", "=WEEKNUM(44561)"); // Default return_type=1
    model._set("A2", "=WEEKNUM(44561,1)"); // Sunday start
    model._set("A3", "=WEEKNUM(44561,2)"); // Monday start

    // Test year boundaries
    model._set("A4", "=WEEKNUM(43831,1)"); // Jan 1, 2020 (Wednesday)
    model._set("A5", "=WEEKNUM(43831,2)"); // Jan 1, 2020 (Wednesday)
    model._set("A6", "=WEEKNUM(44196,1)"); // Dec 31, 2020 (Thursday)
    model._set("A7", "=WEEKNUM(44196,2)"); // Dec 31, 2020 (Thursday)

    // Test first and last weeks of year
    model._set("A8", "=WEEKNUM(44197,1)"); // Jan 1, 2021 (Friday)
    model._set("A9", "=WEEKNUM(44197,2)"); // Jan 1, 2021 (Friday)
    model._set("A10", "=WEEKNUM(44561,1)"); // Dec 31, 2021 (Friday)
    model._set("A11", "=WEEKNUM(44561,2)"); // Dec 31, 2021 (Friday)

    // Error cases - wrong argument count
    model._set("A12", "=WEEKNUM()");
    model._set("A13", "=WEEKNUM(44561,1,1)");

    // Error cases - invalid return_type
    model._set("A14", "=WEEKNUM(44561,0)");
    model._set("A15", "=WEEKNUM(44561,3)");
    model._set("A16", "=WEEKNUM(44561,-1)");

    // Error cases - invalid dates
    model._set("A17", "=WEEKNUM(-1)");
    model._set("A18", &format!("=WEEKNUM({EXCEL_INVALID_DATE})"));

    model.evaluate();

    // Basic functionality
    assert_eq!(model._get_text("A1"), *"53"); // Week 53
    assert_eq!(model._get_text("A2"), *"53"); // Week 53 (Sunday start)
    assert_eq!(model._get_text("A3"), *"53"); // Week 53 (Monday start)

    // Year boundary tests
    assert_eq!(model._get_text("A4"), *"1"); // Jan 1, 2020 (Sunday start)
    assert_eq!(model._get_text("A5"), *"1"); // Jan 1, 2020 (Monday start)
    assert_eq!(model._get_text("A6"), *"53"); // Dec 31, 2020 (Sunday start)
    assert_eq!(model._get_text("A7"), *"53"); // Dec 31, 2020 (Monday start)

    // 2021 tests
    assert_eq!(model._get_text("A8"), *"1"); // Jan 1, 2021 (Sunday start)
    assert_eq!(model._get_text("A9"), *"1"); // Jan 1, 2021 (Monday start)
    assert_eq!(model._get_text("A10"), *"53"); // Dec 31, 2021 (Sunday start)
    assert_eq!(model._get_text("A11"), *"53"); // Dec 31, 2021 (Monday start)

    // Error cases
    assert_eq!(model._get_text("A12"), *"#ERROR!");
    assert_eq!(model._get_text("A13"), *"#ERROR!");
    assert_eq!(model._get_text("A14"), *"#VALUE!");
    assert_eq!(model._get_text("A15"), *"#VALUE!");
    assert_eq!(model._get_text("A16"), *"#VALUE!");
    assert_eq!(model._get_text("A17"), *"#NUM!");
    assert_eq!(model._get_text("A18"), *"#NUM!");
}

#[test]
fn test_workday_function() {
    let mut model = new_empty_model();

    // Basic functionality
    model._set("A1", "=WORKDAY(44560,1)");
    model._set("A2", "=WORKDAY(44561,-1)");
    model._set("A3", "=WORKDAY(44561,0)");
    model._set("A4", "=WORKDAY(44560,5)");

    // Test with holidays
    model._set("B1", "44561");
    model._set("A5", "=WORKDAY(44560,1,B1)"); // Should skip the holiday
    model._set("B2", "44562");
    model._set("B3", "44563");
    model._set("A6", "=WORKDAY(44560,3,B1:B3)"); // Multiple holidays

    // Test starting on weekend
    model._set("A7", "=WORKDAY(44562,1)"); // Saturday start
    model._set("A8", "=WORKDAY(44563,1)"); // Sunday start

    // Test negative workdays
    model._set("A9", "=WORKDAY(44565,-3)"); // Go backwards 3 days
    model._set("A10", "=WORKDAY(44565,-5,B1:B3)"); // Backwards with holidays

    // Edge cases
    model._set("A11", "=WORKDAY(1,1)"); // Early date
    model._set("A12", "=WORKDAY(100000,10)"); // Large numbers

    // Error cases - wrong argument count
    model._set("A13", "=WORKDAY()");
    model._set("A14", "=WORKDAY(44560)");
    model._set("A15", "=WORKDAY(44560,1,B1,B2)");

    // Error cases - invalid dates
    model._set("A16", "=WORKDAY(-1,1)");
    model._set("A17", &format!("=WORKDAY({EXCEL_INVALID_DATE},1)"));

    // Error cases - invalid holiday dates
    model._set("B4", "-1");
    model._set("A18", "=WORKDAY(44560,1,B4)");

    model.evaluate();

    // Basic functionality
    assert_eq!(model._get_text("A1"), *"44561"); // 1 day forward
    assert_eq!(model._get_text("A2"), *"44560"); // 1 day backward
    assert_eq!(model._get_text("A3"), *"44561"); // 0 days
    assert_eq!(model._get_text("A4"), *"44567"); // 5 days forward

    // With holidays
    assert_eq!(model._get_text("A5"), *"44564"); // Skip holiday, go to Monday
    assert_eq!(model._get_text("A6"), *"44566"); // Skip multiple holidays

    // Weekend starts
    assert_eq!(model._get_text("A7"), *"44564"); // From Saturday
    assert_eq!(model._get_text("A8"), *"44564"); // From Sunday

    // Negative workdays
    assert_eq!(model._get_text("A9"), *"44560"); // 3 days back
    assert_eq!(model._get_text("A10"), *"44557"); // 5 days back with holidays

    // Edge cases
    assert_eq!(model._get_text("A11"), *"2"); // Early date
    assert_eq!(model._get_text("A12"), *"100014"); // Large numbers

    // Error cases
    assert_eq!(model._get_text("A13"), *"#ERROR!");
    assert_eq!(model._get_text("A14"), *"#ERROR!");
    assert_eq!(model._get_text("A15"), *"#ERROR!");
    assert_eq!(model._get_text("A16"), *"#NUM!");
    assert_eq!(model._get_text("A17"), *"#NUM!");
    assert_eq!(model._get_text("A18"), *"#NUM!"); // Invalid holiday
}

#[test]
fn test_workday_intl_function() {
    let mut model = new_empty_model();

    // Test key weekend mask types
    model._set("A1", "=WORKDAY.INTL(44560,1,1)"); // Numeric: standard (Sat-Sun)
    model._set("A2", "=WORKDAY.INTL(44560,1,2)"); // Numeric: Sun-Mon
    model._set("A3", "=WORKDAY.INTL(44560,1,\"0000001\")"); // String: Sunday only
    model._set("A4", "=WORKDAY.INTL(44560,1,\"1100000\")"); // String: Mon-Tue

    // Test with holidays
    model._set("B1", "44561");
    model._set("A5", "=WORKDAY.INTL(44560,2,1,B1)"); // Standard + holiday
    model._set("A6", "=WORKDAY.INTL(44560,2,7,B1)"); // Fri-Sat + holiday

    // Basic edge cases
    model._set("A7", "=WORKDAY.INTL(44561,0,1)"); // Zero days
    model._set("A8", "=WORKDAY.INTL(44565,-1,1)"); // Negative days

    // Error cases
    model._set("A9", "=WORKDAY.INTL()"); // Wrong arg count
    model._set("A10", "=WORKDAY.INTL(44560,1,0)"); // Invalid weekend mask
    model._set("A11", "=WORKDAY.INTL(44560,1,\"123\")"); // Invalid string mask
    model._set("A12", "=WORKDAY.INTL(-1,1,1)"); // Invalid date

    model.evaluate();

    // Weekend mask functionality
    assert_eq!(model._get_text("A1"), *"44561"); // Standard weekend
    assert_eq!(model._get_text("A2"), *"44561"); // Sun-Mon weekend
    assert_eq!(model._get_text("A3"), *"44561"); // Sunday only
    assert_eq!(model._get_text("A4"), *"44561"); // Mon-Tue weekend

    // With holidays
    assert_eq!(model._get_text("A5"), *"44565"); // Skip holiday + standard weekend
    assert_eq!(model._get_text("A6"), *"44564"); // Skip holiday + Fri-Sat weekend

    // Edge cases
    assert_eq!(model._get_text("A7"), *"44561"); // Zero days
    assert_eq!(model._get_text("A8"), *"44564"); // Negative days

    // Error cases
    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"#NUM!");
    assert_eq!(model._get_text("A11"), *"#VALUE!");
    assert_eq!(model._get_text("A12"), *"#NUM!");
}

#[test]
fn test_yearfrac_function() {
    let mut model = new_empty_model();

    // Test key basis values (not exhaustive - just verify parameter works)
    model._set("A1", "=YEARFRAC(44561,44926)"); // Default (30/360)
    model._set("A2", "=YEARFRAC(44561,44926,1)"); // Actual/actual
    model._set("A3", "=YEARFRAC(44561,44926,4)"); // European 30/360

    // Edge cases
    model._set("A4", "=YEARFRAC(44561,44561,1)"); // Same date = 0
    model._set("A5", "=YEARFRAC(44926,44561,1)"); // Reverse = negative
    model._set("A6", "=YEARFRAC(44197,44562,1)"); // Exact year (2021)

    // Error cases
    model._set("A7", "=YEARFRAC()"); // Wrong arg count
    model._set("A8", "=YEARFRAC(44561,44926,5)"); // Invalid basis
    model._set("A9", "=YEARFRAC(-1,44926,1)"); // Invalid date

    model.evaluate();

    // Basic functionality (approximate values expected)
    assert_eq!(model._get_text("A1"), *"1"); // About 1 year
    assert_eq!(model._get_text("A2"), *"1"); // About 1 year
    assert_eq!(model._get_text("A3"), *"1"); // About 1 year

    // Edge cases
    assert_eq!(model._get_text("A4"), *"0"); // Same date
    assert_eq!(model._get_text("A5"), *"-1"); // Negative
    assert_eq!(model._get_text("A6"), *"1"); // Exact year

    // Error cases
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#NUM!"); // Invalid basis should return #NUM!
    assert_eq!(model._get_text("A9"), *"#NUM!");
}

#[test]
fn test_isoweeknum_function() {
    let mut model = new_empty_model();

    // Basic functionality
    model._set("A1", "=ISOWEEKNUM(44563)"); // Mid-week date
    model._set("A2", "=ISOWEEKNUM(44561)"); // Year-end date

    // Key ISO week boundaries (just critical cases)
    model._set("A3", "=ISOWEEKNUM(44197)"); // Jan 1, 2021 (Fri) -> Week 53 of 2020
    model._set("A4", "=ISOWEEKNUM(44200)"); // Jan 4, 2021 (Mon) -> Week 1 of 2021
    model._set("A5", "=ISOWEEKNUM(44564)"); // Jan 3, 2022 (Mon) -> Week 1 of 2022

    // Error cases
    model._set("A6", "=ISOWEEKNUM()"); // Wrong arg count
    model._set("A7", "=ISOWEEKNUM(-1)"); // Invalid date

    model.evaluate();

    // Basic functionality
    assert_eq!(model._get_text("A1"), *"52");
    assert_eq!(model._get_text("A2"), *"52");

    // ISO week boundaries
    assert_eq!(model._get_text("A3"), *"53"); // Week 53 of previous year
    assert_eq!(model._get_text("A4"), *"1"); // Week 1 of current year
    assert_eq!(model._get_text("A5"), *"1"); // Week 1 of next year

    // Error cases
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}
