use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDate;

use crate::constants::EXCEL_DATE_BASE;

pub fn from_excel_date(days: i64) -> NaiveDate {
    let dt = NaiveDate::from_ymd_opt(1900, 1, 1).expect("problem with chrono::NaiveDate");
    dt + Duration::days(days - 2)
}

pub fn date_to_serial_number(day: u32, month: u32, year: i32) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => Ok(native_date.num_days_from_ce() - EXCEL_DATE_BASE),
        None => Err("Out of range parameters for date".to_string()),
    }
}
