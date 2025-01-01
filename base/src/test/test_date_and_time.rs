#![allow(clippy::unwrap_used)]

/// Here we add tests that cannot be done in Excel
/// Either because Excel does not have that feature (i.e. wrong number of arguments)
/// or because we differ from Excel throwing #NUM! on invalid dates
/// We can also enter examples that illustrate/document a part of the function
use crate::{cell::CellValue, test::util::new_empty_model};

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
