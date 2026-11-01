use chrono::Datelike;
use chrono::Days;
use chrono::Duration;
use chrono::Months;
use chrono::NaiveDate;

use crate::constants::EXCEL_DATE_BASE;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;

pub const DATE_OUT_OF_RANGE_MESSAGE: &str = "Out of range parameters for date";

#[inline]
fn convert_to_serial_number(date: NaiveDate) -> i32 {
    date.num_days_from_ce() - EXCEL_DATE_BASE
}

fn is_date_within_range(date: NaiveDate) -> bool {
    convert_to_serial_number(date) >= MINIMUM_DATE_SERIAL_NUMBER
        && convert_to_serial_number(date) <= MAXIMUM_DATE_SERIAL_NUMBER
}

/// Serial 0 in the 1904 date system (1904-01-01) is serial 1462 in the 1900 date system.
const DATE_1904_OFFSET: f64 = 1462.0;

/// Converts a serial of the workbook's date system to the 1900-system serial that the date code
/// computes with. A negative serial in the 1904 system is kept negative, so that range checks
/// still reject it.
pub(crate) fn to_1900_serial(serial: f64, date1904: bool) -> f64 {
    if date1904 && serial >= 0.0 {
        serial + DATE_1904_OFFSET
    } else {
        serial
    }
}

/// Converts a 1900-system serial back to the workbook's date system.
/// Returns `None` for a date before 1904-01-01 in the 1904 system.
pub(crate) fn from_1900_serial(serial: f64, date1904: bool) -> Option<f64> {
    if !date1904 {
        return Some(serial);
    }
    let serial = serial - DATE_1904_OFFSET;
    if serial < 0.0 {
        None
    } else {
        Some(serial)
    }
}

pub fn from_excel_date(days: i64) -> Result<NaiveDate, String> {
    if days < MINIMUM_DATE_SERIAL_NUMBER as i64 {
        return Err(format!(
            "Excel date must be greater than {MINIMUM_DATE_SERIAL_NUMBER}"
        ));
    };
    if days > MAXIMUM_DATE_SERIAL_NUMBER as i64 {
        return Err(format!(
            "Excel date must be less than {MAXIMUM_DATE_SERIAL_NUMBER}"
        ));
    };
    #[allow(clippy::expect_used)]
    let dt = NaiveDate::from_ymd_opt(1900, 1, 1).expect("problem with chrono::NaiveDate");
    Ok(dt + Duration::days(days - 2))
}

pub fn date_to_serial_number(day: u32, month: u32, year: i32) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => Ok(convert_to_serial_number(native_date)),
        None => Err(DATE_OUT_OF_RANGE_MESSAGE.to_string()),
    }
}

pub fn permissive_date_to_serial_number(day: i32, month: i32, year: i32) -> Result<i32, String> {
    // Excel parses `DATE` very permissively. It allows not just for valid date values, but it
    // allows for invalid dates as well. If you for example enter `DATE(1900, 1, 32)` it will
    // return the date `1900-02-01`. Despite giving a day that is out of range it will just
    // wrap the month and year around.
    //
    // This function applies that same logic to dates. And does it in the most compatible way as
    // possible.

    // Special case for the minimum date
    if year == 1899 && month == 12 && day == 31 {
        return Ok(MINIMUM_DATE_SERIAL_NUMBER);
    }
    let Some(mut date) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    };

    // One thing to note for example is that even if you started with a year out of range
    // but tried to increment the months so that it wraps around into within range, excel
    // would still return an error.
    //
    // I.E. DATE(0,13,-1) will return an error, despite it being equivalent to DATE(1,1,0) which
    // is within range.
    //
    // As a result, we have to run range checks as we parse the date from the biggest unit to the
    // smallest unit.
    if !is_date_within_range(date) {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    }

    date = {
        let month_diff = month - 1;
        let abs_month = month_diff.unsigned_abs();
        if month_diff <= 0 {
            date = date - Months::new(abs_month);
        } else {
            date = date + Months::new(abs_month);
        }
        if !is_date_within_range(date) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    date = {
        let day_diff = day - 1;
        let abs_day = day_diff.unsigned_abs() as u64;
        if day_diff <= 0 {
            date = date - Days::new(abs_day);
        } else {
            date = date + Days::new(abs_day);
        }
        if !is_date_within_range(date) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    Ok(convert_to_serial_number(date))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_date_to_serial_number() {
        assert_eq!(
            permissive_date_to_serial_number(42, 42, 2002),
            date_to_serial_number(12, 7, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 42, 2002),
            date_to_serial_number(1, 6, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 15, 2000),
            date_to_serial_number(1, 3, 2001)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 49, 2000),
            date_to_serial_number(31, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 49, 2000),
            date_to_serial_number(12, 9, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 1, 2004),
            date_to_serial_number(12, 9, 2004)
        );
    }

    #[test]
    fn test_max_and_min_dates() {
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 9999),
            Ok(MAXIMUM_DATE_SERIAL_NUMBER),
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 1899),
            Ok(MINIMUM_DATE_SERIAL_NUMBER),
        );
    }
}
