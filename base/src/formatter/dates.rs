//! Conversions between date serial numbers and calendar dates.
//!
//! Every conversion takes the [`DateSystem`] of the workbook:
//!
//! * `IronCalc` (the LibreOffice system): serial 0 is 1899-12-30 and every
//!   integer is a valid serial. Negative serials reach back into the proleptic
//!   Gregorian calendar.
//! * `Excel1900`: serial 0 is "0 January 1900", serial 1 is 1900-01-01 and
//!   serial 60 is the fictional 29 February 1900, so 1900 has 366 days. From
//!   serial 61 (1900-03-01) onwards the serials agree with the IronCalc system.
//!   Calendar work goes through [`Ymd`], which can hold those two days that
//!   `chrono` cannot; [`serial_to_date`] and [`date_to_serial`] are only for
//!   code that needs a real `NaiveDate` and refuse the two odd months.
//! * `Excel1904`: the Gregorian calendar with serial 0 on 1904-01-01, so every
//!   serial is 1462 below the IronCalc one. Serials below 0 are not dates.
//!   Excel still formats a negative serial, as the date of its absolute value
//!   with a leading minus; see the formatter.
//!
//! The upper limit in every system is 9999-12-31.

use std::fmt::Display;

use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::Weekday;

use crate::constants::EXCEL_DATE_BASE;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::token::Error;
use crate::types::DateSystem;

pub const DATE_OUT_OF_RANGE_MESSAGE: &str = "Out of range parameters for date";
pub const DATE_SYSTEM_NOT_IMPLEMENTED_MESSAGE: &str = "Date system not implemented";

/// Why a conversion between a serial number and a date failed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateError {
    /// Not a valid date, or outside the range of the date system
    OutOfRange,
    /// The conversion is not implemented for the date system
    NotImplemented,
}

impl Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateError::OutOfRange => write!(f, "{DATE_OUT_OF_RANGE_MESSAGE}"),
            DateError::NotImplemented => write!(f, "{DATE_SYSTEM_NOT_IMPLEMENTED_MESSAGE}"),
        }
    }
}

impl From<DateError> for Error {
    fn from(e: DateError) -> Error {
        match e {
            DateError::OutOfRange => Error::NUM,
            DateError::NotImplemented => Error::NIMPL,
        }
    }
}

/// Serial of 1900-03-01, the first day on which the Excel 1900 system and the
/// IronCalc system agree.
const EXCEL_1900_FIRST_SHARED_SERIAL: i64 = 61;

/// Serial 0 of the Excel 1904 system, 1904-01-01, in the IronCalc system
pub const EXCEL_1904_OFFSET: i64 = 1462;

/// The largest serial of the system, 9999-12-31
pub fn maximum_serial(system: DateSystem) -> i64 {
    match system {
        DateSystem::IronCalc | DateSystem::Excel1900 => MAXIMUM_DATE_SERIAL_NUMBER as i64,
        DateSystem::Excel1904 => MAXIMUM_DATE_SERIAL_NUMBER as i64 - EXCEL_1904_OFFSET,
    }
}

/// A serial of the IronCalc system (what the clock gives) in `system`. The
/// fractional part, the time of day, is kept.
pub fn from_ironcalc_serial(serial: f64, system: DateSystem) -> f64 {
    match system {
        DateSystem::IronCalc | DateSystem::Excel1900 => serial,
        DateSystem::Excel1904 => serial - EXCEL_1904_OFFSET as f64,
    }
}

/// Serial of 1582-10-15, the first day of the Gregorian calendar. LibreOffice
/// does not let `DATE` build a date before it.
const GREGORIAN_START_SERIAL: i64 = -115_858;

/// Day 0 of the IronCalc system: 1899-12-30
fn epoch() -> NaiveDate {
    #[allow(clippy::expect_used)]
    NaiveDate::from_ymd_opt(1899, 12, 30).expect("1899-12-30 is a valid date")
}

/// Days from 1899-12-30. Not range checked.
fn ironcalc_serial(date: NaiveDate) -> i64 {
    (date.num_days_from_ce() - EXCEL_DATE_BASE) as i64
}

/// Serial of a real date in the Excel 1900 system. Before 1900-03-01 Excel is
/// one day behind the IronCalc system (1900-01-01 is serial 1, not 2) because of
/// its fictional 29 February 1900. Not range checked.
fn excel_1900_serial(date: NaiveDate) -> i64 {
    let serial = ironcalc_serial(date);
    if serial >= EXCEL_1900_FIRST_SHARED_SERIAL {
        serial
    } else {
        serial - 1
    }
}

/// The year of a date as a spreadsheet reports it. chrono counts years
/// astronomically (1 BC is year 0), while LibreOffice has no year 0, so YEAR of
/// a date in 1 BC is -1.
pub fn calendar_year(year: i32) -> i32 {
    if year <= 0 {
        year - 1
    } else {
        year
    }
}

fn epoch_plus(serial: i64) -> Result<NaiveDate, DateError> {
    epoch()
        .checked_add_signed(Duration::days(serial))
        .ok_or(DateError::OutOfRange)
}

/// The calendar date of a serial number (date part only)
pub fn serial_to_date(serial: i64, system: DateSystem) -> Result<NaiveDate, DateError> {
    if serial > maximum_serial(system) {
        return Err(DateError::OutOfRange);
    }
    match system {
        DateSystem::IronCalc => epoch_plus(serial),
        DateSystem::Excel1900 => {
            if serial < 0 {
                Err(DateError::OutOfRange)
            } else if serial < EXCEL_1900_FIRST_SHARED_SERIAL {
                // Serials 0..=60 are Excel's "0 January 1900" to "29 February
                // 1900", which do not map to real dates.
                Err(DateError::NotImplemented)
            } else {
                epoch_plus(serial)
            }
        }
        DateSystem::Excel1904 => {
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            epoch_plus(serial + EXCEL_1904_OFFSET)
        }
    }
}

/// A calendar date in a date system. In the Excel 1900 system `day` may be 0
/// (serial 0 is "0 January 1900") and 1900-02-29 exists, neither of which a
/// `NaiveDate` can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ymd {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Ymd {
    fn from_date(date: NaiveDate) -> Ymd {
        Ymd {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

/// Whether `year` is a leap year in the Gregorian calendar
pub fn is_gregorian_leap_year(year: i32) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

/// Whether `year` is a leap year in the calendar the date system displays.
/// Excel's 1900 system shows a 29 February 1900, so there 1900 is a leap year.
/// Excel only uses this calendar to label serials (DAY, MONTH, YEAR, TEXT,
/// DATE, DAYS360); its month arithmetic (EDATE, EOMONTH) and year lengths
/// (YEARFRAC) follow the Gregorian calendar, see [`gregorian_days_in_month`].
pub fn is_leap_year(year: i32, system: DateSystem) -> bool {
    if year == 1900 && system == DateSystem::Excel1900 {
        return true;
    }
    is_gregorian_leap_year(year)
}

/// Days in a month of the Gregorian calendar. Month must be 1 to 12.
pub fn gregorian_days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_gregorian_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days in a month of the calendar the date system displays (February 1900
/// has 29 days in the Excel 1900 system). Month must be 1 to 12.
pub fn days_in_month(year: i32, month: u32, system: DateSystem) -> u32 {
    if month == 2 && is_leap_year(year, system) {
        return 29;
    }
    gregorian_days_in_month(year, month)
}

/// Year and month (1 to 12) of month number `month` of `year`, where `month`
/// may be out of range and rolls into the neighbouring years, the way
/// LibreOffice's DATE does.
pub fn rolled_year_month(month: i64, year: i64) -> (i64, u32) {
    if month > 0 {
        (year + (month - 1) / 12, ((month - 1) % 12 + 1) as u32)
    } else {
        (year + (month - 12) / 12, (12 - (-month) % 12) as u32)
    }
}

/// The weekday of a serial of the date system. Not range checked.
pub fn weekday_of(serial: i64, system: DateSystem) -> Weekday {
    match system {
        DateSystem::IronCalc | DateSystem::Excel1900 => weekday_of_serial(serial),
        DateSystem::Excel1904 => weekday_of_serial(serial + EXCEL_1904_OFFSET),
    }
}

/// The weekday of a serial in any 1900-based system. Serial 0 is a Saturday
/// (1899-12-30, or Excel's "0 January 1900"). Not range checked.
fn weekday_of_serial(serial: i64) -> Weekday {
    match serial.rem_euclid(7) {
        0 => Weekday::Sat,
        1 => Weekday::Sun,
        2 => Weekday::Mon,
        3 => Weekday::Tue,
        4 => Weekday::Wed,
        5 => Weekday::Thu,
        _ => Weekday::Fri,
    }
}

/// The calendar date of a serial number (date part only)
pub fn serial_to_ymd(serial: i64, system: DateSystem) -> Result<Ymd, DateError> {
    if serial > maximum_serial(system) {
        return Err(DateError::OutOfRange);
    }
    match system {
        DateSystem::IronCalc => Ok(Ymd::from_date(epoch_plus(serial)?)),
        DateSystem::Excel1900 => {
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            if serial == 0 {
                // Excel shows serial 0 as "1/0/1900"
                return Ok(Ymd {
                    year: 1900,
                    month: 1,
                    day: 0,
                });
            }
            if serial == 60 {
                return Ok(Ymd {
                    year: 1900,
                    month: 2,
                    day: 29,
                });
            }
            if serial < EXCEL_1900_FIRST_SHARED_SERIAL {
                // Before the fictional leap day Excel is one day ahead: serial
                // 1 is 1900-01-01
                return Ok(Ymd::from_date(epoch_plus(serial + 1)?));
            }
            Ok(Ymd::from_date(epoch_plus(serial)?))
        }
        DateSystem::Excel1904 => Ok(Ymd::from_date(serial_to_date(serial, system)?)),
    }
}

/// The serial of day `day` of month `month` of `year`, where the month rolls
/// into the neighbouring years and the day into the neighbouring months (day 0
/// is the last day of the previous month). The year is taken literally; this is
/// the building block of DATE, EDATE and EOMONTH.
pub fn rolled_ymd_to_serial(
    day: i64,
    month: i64,
    year: i64,
    system: DateSystem,
) -> Result<i64, DateError> {
    let first = first_of_rolled_month(month, year)?;
    let serial = match system {
        DateSystem::IronCalc => ironcalc_serial(first) + day - 1,
        DateSystem::Excel1900 => {
            // Adding the days in serial space makes the fictional 29 February
            // 1900 count as a day
            let serial = excel_1900_serial(first) + day - 1;
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            serial
        }
        DateSystem::Excel1904 => {
            let serial = ironcalc_serial(first) - EXCEL_1904_OFFSET + day - 1;
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            serial
        }
    };
    if serial > maximum_serial(system) {
        return Err(DateError::OutOfRange);
    }
    Ok(serial)
}

/// The ISO 8601 week number of a serial. The weeks of the Excel 1900 and
/// IronCalc systems contain the same serials (both have the same weekdays and
/// serial 2 starts the first ISO week of 1900), so the shared epoch serves both.
pub fn serial_to_iso_week(serial: i64, system: DateSystem) -> Result<u32, DateError> {
    serial_to_weekday(serial, system)?;
    let serial = match system {
        DateSystem::IronCalc | DateSystem::Excel1900 => serial,
        DateSystem::Excel1904 => serial + EXCEL_1904_OFFSET,
    };
    Ok(epoch_plus(serial)?.iso_week().week())
}

/// The weekday of a serial number. Every 1900-based system agrees on it, the
/// fictional 29 February 1900 included: Excel calls serial 1 a Sunday and so
/// does the IronCalc system (1899-12-31), so this is available for the Excel
/// 1900 serials below 61 that `serial_to_date` cannot convert.
pub fn serial_to_weekday(serial: i64, system: DateSystem) -> Result<Weekday, DateError> {
    if serial > maximum_serial(system) {
        return Err(DateError::OutOfRange);
    }
    match system {
        DateSystem::IronCalc => {
            epoch_plus(serial)?;
            Ok(weekday_of_serial(serial))
        }
        DateSystem::Excel1900 => {
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            Ok(weekday_of_serial(serial))
        }
        DateSystem::Excel1904 => {
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            Ok(weekday_of(serial, system))
        }
    }
}

/// The serial number of a calendar date
pub fn date_to_serial(date: NaiveDate, system: DateSystem) -> Result<i64, DateError> {
    let serial = match system {
        DateSystem::IronCalc => ironcalc_serial(date),
        DateSystem::Excel1900 => {
            let serial = ironcalc_serial(date);
            if serial < 1 {
                // Before 1899-12-31 ("0 January 1900"), which Excel rejects
                return Err(DateError::OutOfRange);
            }
            if serial < EXCEL_1900_FIRST_SHARED_SERIAL {
                // A real date in January or February 1900 reached by calendar
                // arithmetic may be off by Excel's fictional leap day; calendar
                // work in this system goes through `Ymd` instead.
                return Err(DateError::NotImplemented);
            }
            serial
        }
        DateSystem::Excel1904 => {
            let serial = ironcalc_serial(date) - EXCEL_1904_OFFSET;
            if serial < 0 {
                return Err(DateError::OutOfRange);
            }
            serial
        }
    };
    if serial > maximum_serial(system) {
        return Err(DateError::OutOfRange);
    }
    Ok(serial)
}

/// The serial number of an explicit year/month/day. The date must exist, except
/// that the Excel 1900 system accepts its fictional 29 February 1900 (serial 60).
pub fn ymd_to_serial(
    day: u32,
    month: u32,
    year: i32,
    system: DateSystem,
) -> Result<i64, DateError> {
    match system {
        DateSystem::IronCalc => {
            let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(DateError::OutOfRange)?;
            date_to_serial(date, system)
        }
        DateSystem::Excel1900 => {
            if (year, month, day) == (1900, 2, 29) {
                return Ok(60);
            }
            let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(DateError::OutOfRange)?;
            let serial = excel_1900_serial(date);
            // Excel accepts serial 0 (1899-12-31, shown as "0 January 1900")
            if !(0..=MAXIMUM_DATE_SERIAL_NUMBER as i64).contains(&serial) {
                return Err(DateError::OutOfRange);
            }
            Ok(serial)
        }
        DateSystem::Excel1904 => {
            let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(DateError::OutOfRange)?;
            date_to_serial(date, system)
        }
    }
}

/// The serial number for the `DATE` function. Month and day may be out of
/// range and roll over into the neighbouring years and months. The year is
/// interpreted the way the date system does:
///
/// * `IronCalc` follows LibreOffice. Years 0 to 29 are 2000 to 2029 and 30 to 99
///   are 1930 to 1999; any other year is taken literally. The result must be a
///   Gregorian date, on or after 1582-10-15.
/// * `Excel1904` is the same with 1904: a year from 0 to 1903 is an offset
///   from 1904, and the result may not be before 1904-01-01 (serial 0).
/// * `Excel1900` follows Excel. A year from 0 to 1899 is an offset from 1900,
///   so `DATE(100, 1, 1)` is 2000-01-01. Excel checks the range after every
///   step, so `DATE(10000, -12, 1)` is an error although the final date,
///   9998-12-01, is fine. The result may be serial 0 (1899-12-31, which Excel
///   shows as "0 January 1900").
///
/// A negative year is an error in both.
pub fn permissive_ymd_to_serial(
    day: i32,
    month: i32,
    year: i32,
    system: DateSystem,
) -> Result<i64, DateError> {
    if year < 0 {
        return Err(DateError::OutOfRange);
    }
    match system {
        DateSystem::IronCalc => permissive_ironcalc(day as i64, month as i64, year as i64),
        DateSystem::Excel1900 => permissive_excel_1900(day as i64, month as i64, year as i64),
        DateSystem::Excel1904 => permissive_excel_1904(day as i64, month as i64, year as i64),
    }
}

fn permissive_excel_1904(day: i64, month: i64, year: i64) -> Result<i64, DateError> {
    let year = if year < 1904 { year + 1904 } else { year };
    // Range checked after every step, like the 1900 system
    let system = DateSystem::Excel1904;
    rolled_ymd_to_serial(1, 1, year, system)?;
    rolled_ymd_to_serial(1, month, year, system)?;
    rolled_ymd_to_serial(day, month, year, system)
}

/// The first day of month `month` (1-based, may be out of range) of `year`,
/// rolling the excess months into the year the way LibreOffice does.
fn first_of_rolled_month(month: i64, year: i64) -> Result<NaiveDate, DateError> {
    let (year, month) = rolled_year_month(month, year);
    let year = i32::try_from(year).map_err(|_| DateError::OutOfRange)?;
    NaiveDate::from_ymd_opt(year, month, 1).ok_or(DateError::OutOfRange)
}

fn permissive_ironcalc(day: i64, month: i64, year: i64) -> Result<i64, DateError> {
    let year = match year {
        0..=29 => year + 2000,
        30..=99 => year + 1900,
        _ => year,
    };
    let serial = rolled_ymd_to_serial(day, month, year, DateSystem::IronCalc)?;
    if serial < GREGORIAN_START_SERIAL {
        return Err(DateError::OutOfRange);
    }
    Ok(serial)
}

fn permissive_excel_1900(day: i64, month: i64, year: i64) -> Result<i64, DateError> {
    let year = if year < 1900 { year + 1900 } else { year };
    // Excel checks the range after each step: year, then month, then day. A
    // year out of range is an error even if the month or day would bring the
    // date back in range.
    let system = DateSystem::Excel1900;
    rolled_ymd_to_serial(1, 1, year, system)?;
    rolled_ymd_to_serial(1, month, year, system)?;
    // Days are added in serial space, so the fictional 29 February 1900 counts
    // as a day: DATE(1900, 2, 29) is 60 and DATE(1900, 2, 30) is 61.
    rolled_ymd_to_serial(day, month, year, system)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn ymd(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn ironcalc_epoch_and_negatives() {
        let s = DateSystem::IronCalc;
        assert_eq!(serial_to_date(0, s), Ok(ymd(1899, 12, 30)));
        assert_eq!(serial_to_date(1, s), Ok(ymd(1899, 12, 31)));
        assert_eq!(serial_to_date(2, s), Ok(ymd(1900, 1, 1)));
        assert_eq!(serial_to_date(60, s), Ok(ymd(1900, 2, 28)));
        assert_eq!(serial_to_date(61, s), Ok(ymd(1900, 3, 1)));
        assert_eq!(serial_to_date(-100, s), Ok(ymd(1899, 9, 21)));
        assert_eq!(serial_to_date(-103615, s), Ok(ymd(1616, 4, 22)));
        // 9053 BC, which is the astronomical year -9052
        assert_eq!(serial_to_date(-4000000, s), Ok(ymd(-9052, 5, 14)));
        assert_eq!(calendar_year(-9052), -9053);
        assert_eq!(calendar_year(0), -1);
        assert_eq!(calendar_year(1), 1);
        assert_eq!(date_to_serial(ymd(1616, 4, 22), s), Ok(-103615));
        assert_eq!(date_to_serial(ymd(9999, 12, 31), s), Ok(2958465));
        assert_eq!(
            serial_to_date(2958466, s),
            Err(DateError::OutOfRange),
            "beyond 9999-12-31"
        );
        assert_eq!(ymd_to_serial(29, 2, 1900, s), Err(DateError::OutOfRange));
    }

    #[test]
    fn excel_1900_serials() {
        let s = DateSystem::Excel1900;
        assert_eq!(serial_to_date(61, s), Ok(ymd(1900, 3, 1)));
        assert_eq!(serial_to_date(60, s), Err(DateError::NotImplemented));
        assert_eq!(serial_to_date(0, s), Err(DateError::NotImplemented));
        assert_eq!(serial_to_date(-1, s), Err(DateError::OutOfRange));
        assert_eq!(date_to_serial(ymd(1900, 3, 1), s), Ok(61));
        assert_eq!(
            date_to_serial(ymd(1900, 2, 28), s),
            Err(DateError::NotImplemented)
        );
        assert_eq!(
            date_to_serial(ymd(1899, 12, 30), s),
            Err(DateError::OutOfRange)
        );
        // Explicit dates in the two odd months are fine
        assert_eq!(ymd_to_serial(31, 12, 1899, s), Ok(0));
        assert_eq!(ymd_to_serial(1, 1, 1900, s), Ok(1));
        assert_eq!(ymd_to_serial(28, 2, 1900, s), Ok(59));
        assert_eq!(ymd_to_serial(29, 2, 1900, s), Ok(60));
        assert_eq!(ymd_to_serial(1, 3, 1900, s), Ok(61));
        assert_eq!(ymd_to_serial(30, 12, 1899, s), Err(DateError::OutOfRange));
    }

    fn y(year: i32, month: u32, day: u32) -> Ymd {
        Ymd { year, month, day }
    }

    #[test]
    fn excel_1900_calendar() {
        let s = DateSystem::Excel1900;
        assert_eq!(serial_to_ymd(-1, s), Err(DateError::OutOfRange));
        assert_eq!(serial_to_ymd(0, s), Ok(y(1900, 1, 0)));
        assert_eq!(serial_to_ymd(1, s), Ok(y(1900, 1, 1)));
        assert_eq!(serial_to_ymd(31, s), Ok(y(1900, 1, 31)));
        assert_eq!(serial_to_ymd(32, s), Ok(y(1900, 2, 1)));
        assert_eq!(serial_to_ymd(59, s), Ok(y(1900, 2, 28)));
        assert_eq!(serial_to_ymd(60, s), Ok(y(1900, 2, 29)));
        assert_eq!(serial_to_ymd(61, s), Ok(y(1900, 3, 1)));
        assert_eq!(serial_to_ymd(366, s), Ok(y(1900, 12, 31)));
        assert_eq!(serial_to_ymd(367, s), Ok(y(1901, 1, 1)));
        assert!(is_leap_year(1900, s));
        assert!(!is_leap_year(1900, DateSystem::IronCalc));
        assert_eq!(days_in_month(1900, 2, s), 29);
        assert_eq!(days_in_month(1900, 2, DateSystem::IronCalc), 28);
        // Rolled dates: day 0 of March is 29 February
        assert_eq!(rolled_ymd_to_serial(0, 3, 1900, s), Ok(60));
        assert_eq!(rolled_ymd_to_serial(0, 1, 1900, s), Ok(0));
        assert_eq!(
            rolled_ymd_to_serial(-1, 1, 1900, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(rolled_ymd_to_serial(1, 13, 1900, s), Ok(367));
        assert_eq!(rolled_ymd_to_serial(31, 0, 1901, s), Ok(366));
        // Every serial round-trips
        for serial in 0..1000 {
            let ymd = serial_to_ymd(serial, s).unwrap();
            assert_eq!(
                rolled_ymd_to_serial(ymd.day as i64, ymd.month as i64, ymd.year as i64, s),
                Ok(serial)
            );
        }
    }

    #[test]
    fn ironcalc_calendar() {
        let s = DateSystem::IronCalc;
        assert_eq!(serial_to_ymd(0, s), Ok(y(1899, 12, 30)));
        assert_eq!(serial_to_ymd(-100, s), Ok(y(1899, 9, 21)));
        assert_eq!(serial_to_ymd(60, s), Ok(y(1900, 2, 28)));
        assert_eq!(rolled_ymd_to_serial(0, 1, 1900, s), Ok(1));
        assert_eq!(rolled_ymd_to_serial(29, 2, 1900, s), Ok(61));
        assert_eq!(rolled_ymd_to_serial(21, 9, 1899, s), Ok(-100));
        for serial in -1000..1000 {
            let ymd = serial_to_ymd(serial, s).unwrap();
            assert_eq!(
                rolled_ymd_to_serial(ymd.day as i64, ymd.month as i64, ymd.year as i64, s),
                Ok(serial)
            );
        }
    }

    #[test]
    fn iso_weeks_are_shared_by_the_1900_systems() {
        for serial in 0..=400 {
            assert_eq!(
                serial_to_iso_week(serial, DateSystem::Excel1900),
                serial_to_iso_week(serial, DateSystem::IronCalc)
            );
        }
        // Excel: Sunday 1 January 1900 is in the last week of 1899, Monday the
        // 2nd starts week 1
        assert_eq!(serial_to_iso_week(1, DateSystem::Excel1900), Ok(52));
        assert_eq!(serial_to_iso_week(2, DateSystem::Excel1900), Ok(1));
        assert_eq!(serial_to_iso_week(8, DateSystem::Excel1900), Ok(1));
        assert_eq!(serial_to_iso_week(9, DateSystem::Excel1900), Ok(2));
    }

    #[test]
    fn weekdays_below_61_are_shared_by_the_1900_systems() {
        for serial in 0..=61 {
            assert_eq!(
                serial_to_weekday(serial, DateSystem::Excel1900),
                serial_to_weekday(serial, DateSystem::IronCalc)
            );
        }
        // Excel: serial 1 is a Sunday and serial 60 (29 February 1900) a Wednesday
        assert_eq!(
            serial_to_weekday(1, DateSystem::Excel1900),
            Ok(Weekday::Sun)
        );
        assert_eq!(
            serial_to_weekday(60, DateSystem::Excel1900),
            Ok(Weekday::Wed)
        );
        assert_eq!(
            serial_to_weekday(-1, DateSystem::Excel1900),
            Err(DateError::OutOfRange)
        );
        assert_eq!(
            serial_to_weekday(-1, DateSystem::IronCalc),
            Ok(Weekday::Fri)
        );
        // 1904-01-02 is a Saturday
        assert_eq!(
            serial_to_weekday(1, DateSystem::Excel1904),
            Ok(Weekday::Sat)
        );
    }

    #[test]
    fn excel_1904_calendar() {
        let s = DateSystem::Excel1904;
        assert_eq!(serial_to_date(0, s), Ok(ymd(1904, 1, 1)));
        assert_eq!(serial_to_date(1, s), Ok(ymd(1904, 1, 2)));
        assert_eq!(serial_to_date(-1, s), Err(DateError::OutOfRange));
        assert_eq!(serial_to_ymd(144, s), Ok(y(1904, 5, 24)));
        assert_eq!(serial_to_ymd(1234, s), Ok(y(1907, 5, 19)));
        assert_eq!(date_to_serial(ymd(1904, 1, 1), s), Ok(0));
        assert_eq!(
            date_to_serial(ymd(1903, 12, 31), s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(date_to_serial(ymd(9999, 12, 31), s), Ok(maximum_serial(s)));
        assert_eq!(maximum_serial(s), 2958465 - 1462);
        assert_eq!(
            serial_to_date(maximum_serial(s) + 1, s),
            Err(DateError::OutOfRange)
        );
        // The same day is 1462 serials apart in the two Excel systems
        assert_eq!(
            ymd_to_serial(1, 1, 2020, s),
            ymd_to_serial(1, 1, 2020, DateSystem::Excel1900).map(|n| n - 1462)
        );
        assert_eq!(serial_to_weekday(0, s), Ok(Weekday::Fri)); // 1904-01-01
        assert_eq!(serial_to_weekday(-1, s), Err(DateError::OutOfRange));
        assert_eq!(
            serial_to_iso_week(0, s),
            serial_to_iso_week(1462, DateSystem::IronCalc)
        );
        // DATE: years below 1904 are offsets from 1904
        assert_eq!(permissive_ymd_to_serial(1, 1, 0, s), Ok(0));
        assert_eq!(permissive_ymd_to_serial(1, 1, 1904, s), Ok(0));
        assert_eq!(
            permissive_ymd_to_serial(0, 1, 1904, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(
            permissive_ymd_to_serial(2, 1, 108, s),
            ymd_to_serial(2, 1, 2012, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, -1, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(from_ironcalc_serial(1462.5, s), 0.5);
    }

    #[test]
    fn permissive_rollover_is_the_same_in_both_systems() {
        for s in [DateSystem::IronCalc, DateSystem::Excel1900] {
            assert_eq!(
                permissive_ymd_to_serial(42, 42, 2002, s),
                ymd_to_serial(12, 7, 2005, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(1, 42, 2002, s),
                ymd_to_serial(1, 6, 2005, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(1, 15, 2000, s),
                ymd_to_serial(1, 3, 2001, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(31, 49, 2000, s),
                ymd_to_serial(31, 1, 2004, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(256, 1, 2004, s),
                ymd_to_serial(12, 9, 2004, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(1, 0, 2004, s),
                ymd_to_serial(1, 12, 2003, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(1, -1, 2004, s),
                ymd_to_serial(1, 11, 2003, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(0, 3, 2004, s),
                ymd_to_serial(29, 2, 2004, s)
            );
            assert_eq!(
                permissive_ymd_to_serial(31, 12, 9999, s),
                Ok(MAXIMUM_DATE_SERIAL_NUMBER as i64)
            );
            assert_eq!(
                permissive_ymd_to_serial(32, 12, 9999, s),
                Err(DateError::OutOfRange)
            );
            assert_eq!(
                permissive_ymd_to_serial(1, 1, -1, s),
                Err(DateError::OutOfRange)
            );
        }
    }

    #[test]
    fn permissive_ironcalc_years() {
        let s = DateSystem::IronCalc;
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 0, s),
            ymd_to_serial(1, 1, 2000, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 29, s),
            ymd_to_serial(1, 1, 2029, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 30, s),
            ymd_to_serial(1, 1, 1930, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 99, s),
            ymd_to_serial(1, 1, 1999, s)
        );
        // Years from 100 on are literal, but DATE only builds Gregorian dates
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 100, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 1600, s),
            ymd_to_serial(1, 1, 1600, s)
        );
        assert_eq!(permissive_ymd_to_serial(30, 12, 1899, s), Ok(0));
        assert_eq!(permissive_ymd_to_serial(21, 9, 1899, s), Ok(-100));
        assert_eq!(permissive_ymd_to_serial(22, 4, 1616, s), Ok(-103615));
        // No 29 February 1900: it rolls over to 1 March
        assert_eq!(permissive_ymd_to_serial(29, 2, 1900, s), Ok(61));
        // Gregorian calendar only
        assert_eq!(
            permissive_ymd_to_serial(15, 10, 1582, s),
            Ok(GREGORIAN_START_SERIAL)
        );
        assert_eq!(
            permissive_ymd_to_serial(14, 10, 1582, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(
            permissive_ymd_to_serial(16, 1, 531, s),
            Err(DateError::OutOfRange)
        );
    }

    #[test]
    fn permissive_excel_1900_years() {
        let s = DateSystem::Excel1900;
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 0, s),
            ymd_to_serial(1, 1, 1900, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 100, s),
            ymd_to_serial(1, 1, 2000, s)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, 1, 1899, s),
            ymd_to_serial(1, 1, 3799, s)
        );
        assert_eq!(permissive_ymd_to_serial(1, 1, 1900, s), Ok(1));
        assert_eq!(permissive_ymd_to_serial(0, 1, 1900, s), Ok(0));
        assert_eq!(
            permissive_ymd_to_serial(-1, 1, 1900, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(permissive_ymd_to_serial(29, 2, 1900, s), Ok(60));
        assert_eq!(permissive_ymd_to_serial(60, 1, 1900, s), Ok(60));
        assert_eq!(permissive_ymd_to_serial(30, 2, 1900, s), Ok(61));
        assert_eq!(permissive_ymd_to_serial(1, 3, 1900, s), Ok(61));
        // The range is checked after every step
        assert_eq!(
            permissive_ymd_to_serial(1, -12, 10000, s),
            Err(DateError::OutOfRange)
        );
        assert_eq!(
            permissive_ymd_to_serial(1, -12, 10000, DateSystem::IronCalc),
            ymd_to_serial(1, 12, 9998, DateSystem::IronCalc)
        );
    }
}
