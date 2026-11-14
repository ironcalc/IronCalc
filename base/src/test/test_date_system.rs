#![allow(clippy::unwrap_used)]

use crate::{
    cell::CellValue,
    model::Model,
    test::util::{new_empty_model, new_empty_model_excel_1900},
    types::DateSystem,
};

/// The raw value of a cell as text. Cells with a DATE formula show a formatted
/// date in `_get_text`, so the serial itself is read here.
fn raw(model: &Model, cell: &str) -> String {
    match model
        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
        .unwrap()
    {
        CellValue::Number(n) => format!("{n}"),
        CellValue::String(s) => s,
        CellValue::Boolean(b) => format!("{b}").to_uppercase(),
        CellValue::None => "".to_string(),
    }
}

fn new_empty_model_excel_1904<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.workbook.settings.date_system = DateSystem::Excel1904;
    model
}

#[test]
fn new_workbooks_use_the_ironcalc_date_system() {
    let model = new_empty_model();
    assert_eq!(model.workbook.settings.date_system, DateSystem::IronCalc);
}

#[test]
fn date_system_default_is_ironcalc() {
    assert_eq!(DateSystem::default(), DateSystem::IronCalc);
}

#[test]
fn date_system_survives_a_round_trip_through_bytes() {
    let mut model = new_empty_model();
    model.workbook.settings.date_system = DateSystem::Excel1904;
    let bytes = model.to_bytes();
    let model2 = Model::from_bytes(&bytes, "en").unwrap();
    assert_eq!(model2.workbook.settings.date_system, DateSystem::Excel1904);
}

// ---------------------------------------------------------------------------
// IronCalc (LibreOffice) date system: serial 0 is 1899-12-30, any serial is a
// date, DATE takes years literally
// ---------------------------------------------------------------------------

#[test]
fn ironcalc_system_serials_around_the_epoch() {
    let mut model = new_empty_model();
    model._set("A1", "=DAY(0)");
    model._set("A2", "=MONTH(0)");
    model._set("A3", "=YEAR(0)");
    model._set("A4", "=DAY(1)");
    model._set("A5", "=DAY(2)");
    model._set("A6", "=MONTH(2)");
    // There is no 29 February 1900: serial 60 is 28 February and 61 is 1 March
    model._set("A7", "=DAY(60)");
    model._set("A8", "=DAY(61)");
    model._set("A9", "=MONTH(61)");
    // Negative serials
    model._set("B1", "=DAY(-100)");
    model._set("B2", "=MONTH(-100)");
    model._set("B3", "=YEAR(-100)");
    model._set("B4", "=YEAR(-100000)");
    // Years before 1 AD have no year 0, as in LibreOffice
    model._set("B5", "=YEAR(-1000000)");
    model._set("B6", "=YEAR(-4000000)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "30");
    assert_eq!(model._get_text("A2"), "12");
    assert_eq!(model._get_text("A3"), "1899");
    assert_eq!(model._get_text("A4"), "31");
    assert_eq!(model._get_text("A5"), "1");
    assert_eq!(model._get_text("A6"), "1");
    assert_eq!(model._get_text("A7"), "28");
    assert_eq!(model._get_text("A8"), "1");
    assert_eq!(model._get_text("A9"), "3");

    assert_eq!(model._get_text("B1"), "21");
    assert_eq!(model._get_text("B2"), "9");
    assert_eq!(model._get_text("B3"), "1899");
    assert_eq!(model._get_text("B4"), "1626");
    assert_eq!(model._get_text("B5"), "-839");
    assert_eq!(model._get_text("B6"), "-9053");
}

#[test]
fn ironcalc_system_date_function() {
    let mut model = new_empty_model();
    // Years are literal
    model._set("A1", "=DATE(1899,12,30)");
    model._set("A2", "=DATE(1899,9,21)");
    model._set("A3", "=DATE(1616,4,22)");
    model._set("A4", "=DATE(1879,3,14)");
    // Two-digit years: 0-29 are 2000-2029, 30-99 are 1930-1999
    model._set("A5", "=DATE(29,1,1)");
    model._set("A6", "=DATE(30,1,1)");
    // No fictional leap day: rolls over to 1 March
    model._set("A7", "=DATE(1900,2,29)");
    // Only Gregorian dates (from 1582-10-15), as Err:502 in LibreOffice
    model._set("A8", "=DATE(1582,10,15)");
    model._set("A9", "=DATE(1582,10,14)");
    model._set("A10", "=DATE(531,1,16)");
    model._set("A11", "=DATE(-1,1,1)");
    model._set("A12", "=DATE(10000,1,1)");
    // DATE and DAY/MONTH/YEAR are inverses for negative serials
    model._set("A13", "=DATE(YEAR(-557),MONTH(-557),DAY(-557))");
    // 23 April 1616 is a Saturday in the (proleptic) Gregorian calendar;
    // Einstein was born on a Friday
    model._set("A14", "=TEXT(DATE(1616,4,23),\"dddd\")");
    model._set("A15", "=WEEKDAY(DATE(1879,3,14))");
    model.evaluate();

    assert_eq!(raw(&model, "A1"), "0");
    assert_eq!(raw(&model, "A2"), "-100");
    assert_eq!(raw(&model, "A3"), "-103615");
    assert_eq!(raw(&model, "A4"), "-7596");
    assert_eq!(raw(&model, "A5"), "47119"); // 2029-01-01
    assert_eq!(raw(&model, "A6"), "10959"); // 1930-01-01
    assert_eq!(raw(&model, "A7"), "61");
    assert_eq!(raw(&model, "A8"), "-115858");
    assert_eq!(model._get_text("A9"), "#VALUE!");
    assert_eq!(model._get_text("A10"), "#VALUE!");
    assert_eq!(model._get_text("A11"), "#VALUE!");
    assert_eq!(model._get_text("A12"), "#VALUE!");
    assert_eq!(raw(&model, "A13"), "-557");
    assert_eq!(model._get_text("A14"), "Saturday");
    assert_eq!(model._get_text("A15"), "6");
}

#[test]
fn ironcalc_system_other_functions_accept_negative_serials() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE(\"1899-12-30\")");
    model._set("A2", "=DATEVALUE(\"1900-02-29\")");
    model._set("A3", "=TEXT(-100,\"yyyy-mm-dd\")");
    model._set("A4", "=TEXT(-0.25,\"yyyy-mm-dd hh:mm\")");
    model._set("A5", "=HOUR(-0.25)");
    model._set("A6", "=EDATE(0,-1)");
    model._set("A7", "=EOMONTH(0,-1)");
    model._set("A8", "=DAYS360(0,61)");
    model._set("A9", "=DAYS(0,-10)");
    model._set("A10", "=NETWORKDAYS(-10,0)");
    model._set("A11", "=WORKDAY(0,-1)");
    model._set("A12", "=DATEDIF(-365,0,\"d\")");
    model._set("A13", "=YEARFRAC(-365,0)");
    model.evaluate();

    assert_eq!(raw(&model, "A1"), "0");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "1899-09-21");
    assert_eq!(model._get_text("A4"), "1899-12-29 18:00");
    assert_eq!(model._get_text("A5"), "18");
    assert_eq!(raw(&model, "A6"), "-30"); // 1899-11-30
    assert_eq!(raw(&model, "A7"), "-30"); // 1899-11-30
    assert_eq!(model._get_text("A8"), "61"); // 1899-12-30 to 1900-03-01
    assert_eq!(model._get_text("A9"), "10");
    assert_eq!(model._get_text("A10"), "8"); // Wed 1899-12-20 to Sat 1899-12-30
    assert_eq!(raw(&model, "A11"), "-1"); // Friday 1899-12-29
    assert_eq!(model._get_text("A12"), "365");
    assert_eq!(model._get_text("A13"), "1");
}

#[test]
fn ironcalc_system_typed_dates_before_the_epoch() {
    let mut model = new_empty_model();
    model._set("A1", "3/14/1879");
    model._set("B1", "=A1");
    model._set("C1", "=ISNUMBER(A1)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), "TRUE");
    assert_eq!(raw(&model, "A1"), "-7596");
    assert_eq!(model._get_text("B1"), "3/14/1879");
    assert_eq!(model._get_text("A1"), "3/14/1879");
}

// ---------------------------------------------------------------------------
// Excel 1900 date system: the two months before 1900-03-01 follow Excel where
// an explicit year/month/day is given and are #N/IMPL! otherwise
// ---------------------------------------------------------------------------

#[test]
fn excel_1900_system_date_and_datevalue() {
    let mut model = new_empty_model_excel_1900();
    model._set("A1", "=DATE(1900,1,1)");
    model._set("A2", "=DATE(1900,2,28)");
    model._set("A3", "=DATE(1900,2,29)");
    model._set("A4", "=DATE(1900,3,1)");
    model._set("A5", "=DATE(1900,1,0)");
    model._set("A6", "=DATE(1900,1,-1)");
    // Years below 1900 are offsets from 1900
    model._set("A7", "=DATE(100,1,1)");
    model._set("A8", "=DATE(1899,12,30)");
    model._set("A9", "=DATE(-1,1,1)");
    model._set("B1", "=DATEVALUE(\"1900-01-01\")");
    model._set("B2", "=DATEVALUE(\"1900-02-29\")");
    model._set("B3", "=DATEVALUE(\"1899-12-30\")");
    model.evaluate();

    assert_eq!(raw(&model, "A1"), "1");
    assert_eq!(raw(&model, "A2"), "59");
    assert_eq!(raw(&model, "A3"), "60");
    assert_eq!(raw(&model, "A4"), "61");
    assert_eq!(raw(&model, "A5"), "0");
    assert_eq!(model._get_text("A6"), "#NUM!");
    assert_eq!(raw(&model, "A7"), "36526"); // 2000-01-01
    assert_eq!(raw(&model, "A8"), "693961"); // 3799-12-30
    assert_eq!(model._get_text("A9"), "#NUM!");
    assert_eq!(raw(&model, "B1"), "1");
    assert_eq!(raw(&model, "B2"), "60");
    assert_eq!(model._get_text("B3"), "#VALUE!");
}

#[test]
fn excel_1900_system_calendar_below_61() {
    let mut model = new_empty_model_excel_1900();
    // Serial 0 is "0 January 1900", serial 60 is 29 February 1900
    model._set("A1", "=DAY(0)");
    model._set("A2", "=MONTH(0)");
    model._set("A3", "=YEAR(0)");
    model._set("A4", "=DAY(1)");
    model._set("A5", "=DAY(60)");
    model._set("A6", "=MONTH(60)");
    model._set("A7", "=DAY(61)");
    model._set("A8", "=TEXT(0,\"m/d/yyyy\")");
    model._set("A9", "=TEXT(1,\"yyyy-mm-dd\")");
    model._set("A10", "=TEXT(60,\"mmmm d, dddd\")");
    model._set("A11", "=TEXT(60,\"yy\")");
    // Month arithmetic on Excel's calendar
    model._set("B1", "=EDATE(1,1)");
    model._set("B2", "=EDATE(89,-1)"); // 29 March to 28 February: Gregorian month lengths
    model._set("B3", "=EDATE(0,1)"); // "0 February" is 31 January
    model._set("B4", "=EOMONTH(61,-1)");
    model._set("B5", "=EOMONTH(32,0)");
    model._set("B6", "=EOMONTH(0,0)");
    model._set("B7", "=DATEDIF(1,61,\"d\")");
    model._set("B8", "=DATEDIF(1,61,\"m\")");
    model._set("B9", "=DATEDIF(1,366,\"yd\")");
    model._set("B10", "=DATEDIF(32,61,\"md\")");
    // 1900 has 366 serials but YEARFRAC uses the Gregorian year length
    model._set("C1", "=YEARFRAC(1,367,1)");
    model._set("C2", "=YEARFRAC(1,367)");
    model._set("C3", "=DATE(1900,12,31)");
    // Weeks
    model._set("D1", "=WEEKDAY(1)"); // Excel: 1 January 1900 is a Sunday
    model._set("D2", "=WEEKDAY(60)"); // Excel: 29 February 1900 is a Wednesday
    model._set("D3", "=WEEKNUM(1)");
    model._set("D4", "=WEEKNUM(2)");
    model._set("D5", "=WEEKNUM(8)");
    model._set("D6", "=WEEKNUM(2,2)");
    model._set("D7", "=ISOWEEKNUM(1)");
    model._set("D8", "=ISOWEEKNUM(2)");
    model._set("D9", "=NETWORKDAYS(1,61)");
    model._set("D10", "=WORKDAY(61,-1)");
    model._set("D11", "=WORKDAY(1,1)");
    // Negative serials are #NUM!, as in Excel
    model._set("E1", "=DAY(-1)");
    model._set("E2", "=HOUR(-0.5)");
    model._set("E3", "=WEEKDAY(-1)");
    model._set("E4", "=EDATE(1,-1)");
    model._set("E5", "=WORKDAY(1,-1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "0");
    assert_eq!(model._get_text("A2"), "1");
    assert_eq!(model._get_text("A3"), "1900");
    assert_eq!(model._get_text("A4"), "1");
    assert_eq!(model._get_text("A5"), "29");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "1");
    assert_eq!(model._get_text("A8"), "1/0/1900");
    assert_eq!(model._get_text("A9"), "1900-01-01");
    assert_eq!(model._get_text("A10"), "February 29, Wednesday");
    assert_eq!(model._get_text("A11"), "00");

    assert_eq!(raw(&model, "B1"), "32");
    assert_eq!(raw(&model, "B2"), "59");
    assert_eq!(raw(&model, "B3"), "31");
    assert_eq!(raw(&model, "B4"), "59");
    assert_eq!(raw(&model, "B5"), "59");
    assert_eq!(raw(&model, "B6"), "31");
    assert_eq!(model._get_text("B7"), "60");
    assert_eq!(model._get_text("B8"), "2");
    assert_eq!(model._get_text("B9"), "365");
    assert_eq!(model._get_text("B10"), "0");

    assert_eq!(raw(&model, "C1"), "1.0027397260273974");
    assert_eq!(model._get_text("C2"), "1");
    assert_eq!(raw(&model, "C3"), "366");

    assert_eq!(model._get_text("D1"), "1");
    assert_eq!(model._get_text("D2"), "4");
    assert_eq!(model._get_text("D3"), "1");
    assert_eq!(model._get_text("D4"), "1");
    assert_eq!(model._get_text("D5"), "2");
    assert_eq!(model._get_text("D6"), "2");
    assert_eq!(model._get_text("D7"), "52");
    assert_eq!(model._get_text("D8"), "1");
    assert_eq!(model._get_text("D9"), "44");
    assert_eq!(raw(&model, "D10"), "60");
    assert_eq!(raw(&model, "D11"), "2");

    for cell in ["E1", "E2", "E3", "E4", "E5"] {
        assert_eq!(model._get_text(cell), "#NUM!", "{cell}");
    }
}

#[test]
fn excel_1900_system_days_and_days360() {
    let mut model = new_empty_model_excel_1900();
    model._set("A1", "=DAYS(61,0)");
    model._set("A2", "=DAYS(61,-1)");
    model._set("A3", "=DAYS360(1,61)"); // 1 January to 1 March 1900
    model._set("A4", "=DAYS360(0,1)"); // "0 January" to 1 January
    model._set("A5", "=DAYS360(60,61)"); // 29 February (last day of the month) to 1 March
    model._set("A6", "=DAYS360(-1,61)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "61");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "60");
    assert_eq!(model._get_text("A4"), "1");
    assert_eq!(model._get_text("A5"), "1");
    assert_eq!(model._get_text("A6"), "#NUM!");
}

// ---------------------------------------------------------------------------
// Excel 1904 date system: not implemented
// ---------------------------------------------------------------------------

#[test]
fn excel_1904_system_is_not_implemented() {
    let mut model = new_empty_model_excel_1904();
    model._set("A1", "=DATE(2024,1,1)");
    model._set("A2", "=DAY(45000)");
    model._set("A3", "=DATEVALUE(\"2024-01-01\")");
    model._set("A4", "=TODAY()");
    model._set("A5", "=NOW()");
    model._set("A6", "=TEXT(45000,\"yyyy\")");
    model._set("A7", "=DAYS(45000,44000)");
    model._set("A8", "=WEEKDAY(45000)");
    model._set("A9", "=HOUR(-0.5)");
    // Times of day do not depend on the date system
    model._set("B1", "=HOUR(0.75)");
    model._set("B2", "=TIME(12,30,0)");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9"] {
        assert_eq!(model._get_text(cell), "#N/IMPL!", "{cell}");
    }
    assert_eq!(model._get_text("B1"), "18");
    assert_eq!(model._get_text("B2"), "0.520833333");
}
