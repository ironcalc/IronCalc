//! Calendar arithmetic for the financial functions, in the date system of the
//! workbook.
//!
//! The bond, accrued-interest and depreciation functions step through coupon
//! schedules and count days between dates. They follow the model of Excel's
//! EDATE, EOMONTH and YEARFRAC in every date system:
//!
//! * a serial is labelled with the year, month and day the system shows for it
//!   (in the Excel 1900 system serial 60 is 29 February 1900);
//! * month arithmetic uses the Gregorian month lengths, so a day past the end
//!   of the target month is clamped to the Gregorian month end;
//! * whether a day is the last of its month follows the calendar the system
//!   displays (the next serial is the first of a month): in the Excel 1900
//!   system 28 February 1900 is not a month end but serial 60, its fictional
//!   29 February, is;
//! * the days between two dates are the difference of their serials.
//!
//! A coupon schedule may step back to a date before the first day of the
//! system (before 1900 in the Excel 1900 system). Excel keeps counting with
//! such dates in COUPDAYS, reports them as serial 0 in COUPPCD and COUPDAYBS,
//! and fails with `#NUM!` everywhere else. The `unchecked` functions give those
//! dates a negative serial, continuing the numbering of the system backwards;
//! the checked ones fail on them.

use std::cmp::Ordering;

use chrono::{Datelike, Duration, NaiveDate};

use crate::constants::EXCEL_DATE_BASE;
use crate::formatter::dates::{
    days_in_month, gregorian_days_in_month, maximum_serial, rolled_year_month, serial_to_ymd,
    DATE_OUT_OF_RANGE_MESSAGE, EXCEL_1904_OFFSET,
};
use crate::types::DateSystem;

/// A day of the date system: its serial and the calendar date shown for it.
/// Days compare by serial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Day {
    pub serial: i64,
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl PartialOrd for Day {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Day {
    fn cmp(&self, other: &Self) -> Ordering {
        self.serial.cmp(&other.serial)
    }
}

fn out_of_range() -> String {
    DATE_OUT_OF_RANGE_MESSAGE.to_string()
}

/// The first serial of the system, if it has one. The IronCalc system has a
/// date for every serial.
pub(super) fn first_serial(system: DateSystem) -> Option<i64> {
    match system {
        DateSystem::IronCalc => None,
        DateSystem::Excel1900 | DateSystem::Excel1904 => Some(0),
    }
}

/// Whether `serial` is before the first day of the system
pub(super) fn is_before_first(serial: i64, system: DateSystem) -> bool {
    first_serial(system).is_some_and(|first| serial < first)
}

/// `serial`, or the first serial of the system if it is before it
pub(super) fn clamp_to_first(serial: i64, system: DateSystem) -> i64 {
    match first_serial(system) {
        Some(first) => serial.max(first),
        None => serial,
    }
}

/// Serial 0 of the IronCalc system, 1899-12-30
fn epoch() -> NaiveDate {
    #[allow(clippy::expect_used)]
    NaiveDate::from_ymd_opt(1899, 12, 30).expect("1899-12-30 is a valid date")
}

/// Serial of 1900-03-01, from which the Excel 1900 and IronCalc systems agree
const EXCEL_1900_FIRST_SHARED_SERIAL: i64 = 61;

/// The serial of a Gregorian date, continuing the numbering of the system
/// before its first day
fn unchecked_serial(date: NaiveDate, system: DateSystem) -> i64 {
    let ironcalc = (date.num_days_from_ce() - EXCEL_DATE_BASE) as i64;
    match system {
        DateSystem::IronCalc => ironcalc,
        DateSystem::Excel1900 => {
            if ironcalc >= EXCEL_1900_FIRST_SHARED_SERIAL {
                ironcalc
            } else {
                // Before its fictional 29 February 1900 Excel is one day behind
                ironcalc - 1
            }
        }
        DateSystem::Excel1904 => ironcalc - EXCEL_1904_OFFSET,
    }
}

/// The day of a serial number, which may be before the first day of the system
pub(super) fn unchecked_day_of(serial: i64, system: DateSystem) -> Result<Day, String> {
    if serial > maximum_serial(system) {
        return Err(out_of_range());
    }
    if !is_before_first(serial, system) {
        return day_of(serial, system);
    }
    // A Gregorian date before the first day of the system
    let days = match system {
        DateSystem::IronCalc => unreachable!("every serial is a day of the IronCalc system"),
        DateSystem::Excel1900 => serial + 1,
        DateSystem::Excel1904 => serial + EXCEL_1904_OFFSET,
    };
    let date = epoch()
        .checked_add_signed(Duration::days(days))
        .ok_or_else(out_of_range)?;
    Ok(Day {
        serial,
        year: date.year(),
        month: date.month(),
        day: date.day(),
    })
}

/// The day of a serial number
pub(super) fn day_of(serial: i64, system: DateSystem) -> Result<Day, String> {
    let ymd = serial_to_ymd(serial, system).map_err(|e| e.to_string())?;
    Ok(Day {
        serial,
        year: ymd.year,
        month: ymd.month,
        day: ymd.day,
    })
}

/// The day of a Gregorian calendar date, which may be before the first day of
/// the system. The day of the month may be 0 (the last day of the previous
/// month) or run past the end of the month.
pub(super) fn unchecked_day_from_ymd(
    year: i32,
    month: u32,
    day: u32,
    system: DateSystem,
) -> Result<Day, String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(out_of_range)?;
    let date = first
        .checked_add_signed(Duration::days(day as i64 - 1))
        .ok_or_else(out_of_range)?;
    unchecked_day_of(unchecked_serial(date, system), system)
}

/// The day of a Gregorian calendar date, see [`unchecked_day_from_ymd`]
pub(super) fn day_from_ymd(
    year: i32,
    month: u32,
    day: u32,
    system: DateSystem,
) -> Result<Day, String> {
    let d = unchecked_day_from_ymd(year, month, day, system)?;
    if is_before_first(d.serial, system) {
        return Err(out_of_range());
    }
    Ok(d)
}

/// Days in a month of the Gregorian calendar
pub(super) fn last_day_of_month(year: i32, month: u32) -> u32 {
    gregorian_days_in_month(year, month)
}

/// Whether `d` is the last day of its month in the calendar the system shows.
/// Excel's COUPDAYS(30, 425, 2, 1) in the 1900 system is 184: it steps back
/// from 28 February 1901 to 28 February 1900 (serial 59) and, as that is not a
/// month end there, on to 28 August 1899 rather than 31 August 1899.
pub(super) fn is_last_day_of_month(d: Day, system: DateSystem) -> bool {
    d.day == days_in_month(d.year, d.month, system)
}

/// `d` moved by `months` months, the day of the month clamped to the length of
/// the target month (as EDATE does). With `return_last` the result is the last
/// day of the target month: the end-of-month coupon rule, where a bond maturing
/// on a month end pays every coupon on a month end. The result may be before
/// the first day of the system.
pub(super) fn unchecked_add_months(
    d: Day,
    months: i32,
    return_last: bool,
    system: DateSystem,
) -> Result<Day, String> {
    let (year, month) = rolled_year_month(d.month as i64 + months as i64, d.year as i64);
    let year = i32::try_from(year).map_err(|_| out_of_range())?;
    let last = last_day_of_month(year, month);
    let day = if return_last { last } else { d.day.min(last) };
    unchecked_day_from_ymd(year, month, day, system)
}

/// `d` moved by `months` months, see [`unchecked_add_months`]. A result before
/// the first day of the system is an error.
pub(super) fn add_months(
    d: Day,
    months: i32,
    return_last: bool,
    system: DateSystem,
) -> Result<Day, String> {
    let moved = unchecked_add_months(d, months, return_last, system)?;
    if is_before_first(moved.serial, system) {
        return Err(out_of_range());
    }
    Ok(moved)
}

/// The number of days from `d1` to `d2`
pub(super) fn actual_days(d1: Day, d2: Day) -> f64 {
    (d2.serial - d1.serial) as f64
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn excel_1900_numbering_continues_before_1900() {
        let s = DateSystem::Excel1900;
        let d = unchecked_day_of(-1, s).unwrap();
        assert_eq!((d.year, d.month, d.day), (1899, 12, 30));
        let d = unchecked_day_from_ymd(1899, 8, 7, s).unwrap();
        assert_eq!(d.serial, -146);
        assert!(day_from_ymd(1899, 8, 7, s).is_err());
        // 1 January 1900 is serial 1 and the fictional leap day is counted
        assert_eq!(day_from_ymd(1900, 1, 1, s).unwrap().serial, 1);
        assert_eq!(day_from_ymd(1900, 3, 1, s).unwrap().serial, 61);
        assert_eq!(day_from_ymd(1899, 12, 31, s).unwrap().serial, 0);
        let feb7 = day_of(38, s).unwrap();
        let aug7 = unchecked_add_months(feb7, -6, false, s).unwrap();
        assert_eq!(aug7.serial, -146);
        assert_eq!(actual_days(aug7, feb7), 184.0);
        assert!(add_months(feb7, -6, false, s).is_err());
    }

    #[test]
    fn month_ends_are_gregorian() {
        let s = DateSystem::Excel1900;
        let feb28_1901 = day_of(425, s).unwrap();
        assert!(is_last_day_of_month(feb28_1901, s));
        assert!(!is_last_day_of_month(day_of(59, s).unwrap(), s));
        assert!(is_last_day_of_month(day_of(60, s).unwrap(), s));
        // 28 February 1900 is serial 60 in the IronCalc system, and a month end
        assert!(is_last_day_of_month(
            day_of(60, DateSystem::IronCalc).unwrap(),
            DateSystem::IronCalc
        ));
        // EOMONTH-like: the last day of February 1900 is the 28th, serial 59
        assert_eq!(add_months(feb28_1901, -12, true, s).unwrap().serial, 59);
        let feb29_1900 = day_of(60, s).unwrap();
        assert_eq!((feb29_1900.month, feb29_1900.day), (2, 29));
        assert_eq!(add_months(feb29_1900, 0, false, s).unwrap().serial, 59);
    }

    #[test]
    fn ironcalc_system_has_no_first_day() {
        let s = DateSystem::IronCalc;
        assert_eq!(first_serial(s), None);
        assert_eq!(clamp_to_first(-1000, s), -1000);
        assert_eq!(unchecked_day_from_ymd(1899, 8, 7, s).unwrap().serial, -145);
        assert_eq!(day_from_ymd(1899, 8, 7, s).unwrap().serial, -145);
    }

    #[test]
    fn excel_1904_numbering_continues_before_1904() {
        let s = DateSystem::Excel1904;
        assert_eq!(unchecked_day_from_ymd(1903, 12, 31, s).unwrap().serial, -1);
        assert!(day_from_ymd(1903, 12, 31, s).is_err());
        assert_eq!(clamp_to_first(-5, s), 0);
    }
}
