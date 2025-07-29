use chrono::DateTime;
use chrono::Datelike;
use chrono::Months;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;

use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::dates::date_to_serial_number;
use crate::formatter::dates::permissive_date_to_serial_number;
use crate::model::get_milliseconds_since_epoch;
use crate::{
    calc_result::CalcResult, constants::EXCEL_DATE_BASE, expressions::parser::Node,
    expressions::token::Error, formatter::dates::from_excel_date, model::Model,
};

fn parse_time_string(text: &str) -> Option<f64> {
    let text = text.trim();

    // First, try custom parsing for edge cases like "24:00:00", "23:60:00", "23:59:60"
    // that need normalization to match Excel behavior
    if let Some(time_fraction) = parse_time_with_normalization(text) {
        return Some(time_fraction);
    }

    // First, try manual parsing for simple "N PM" / "N AM" format
    if let Some((hour_str, is_pm)) = parse_simple_am_pm(text) {
        if let Ok(hour) = hour_str.parse::<u32>() {
            if (1..=12).contains(&hour) {
                let hour_24 = if is_pm {
                    if hour == 12 {
                        12
                    } else {
                        hour + 12
                    }
                } else if hour == 12 {
                    0
                } else {
                    hour
                };
                let time = NaiveTime::from_hms_opt(hour_24, 0, 0)?;
                return Some(time.num_seconds_from_midnight() as f64 / 86_400.0);
            }
        }
    }

    // Standard patterns
    let patterns_time = ["%H:%M:%S", "%H:%M", "%I:%M %p", "%I %p", "%I:%M:%S %p"];
    for p in patterns_time {
        if let Ok(t) = NaiveTime::parse_from_str(text, p) {
            return Some(t.num_seconds_from_midnight() as f64 / 86_400.0);
        }
    }

    let patterns_dt = [
        // ISO formats
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        // Excel-style date formats with AM/PM
        "%d-%b-%Y %I:%M:%S %p", // "22-Aug-2011 6:35:00 AM"
        "%d-%b-%Y %I:%M %p",    // "22-Aug-2011 6:35 AM"
        "%d-%b-%Y %H:%M:%S",    // "22-Aug-2011 06:35:00"
        "%d-%b-%Y %H:%M",       // "22-Aug-2011 06:35"
        // US date formats with AM/PM
        "%m/%d/%Y %I:%M:%S %p", // "8/22/2011 6:35:00 AM"
        "%m/%d/%Y %I:%M %p",    // "8/22/2011 6:35 AM"
        "%m/%d/%Y %H:%M:%S",    // "8/22/2011 06:35:00"
        "%m/%d/%Y %H:%M",       // "8/22/2011 06:35"
        // European date formats with AM/PM
        "%d/%m/%Y %I:%M:%S %p", // "22/8/2011 6:35:00 AM"
        "%d/%m/%Y %I:%M %p",    // "22/8/2011 6:35 AM"
        "%d/%m/%Y %H:%M:%S",    // "22/8/2011 06:35:00"
        "%d/%m/%Y %H:%M",       // "22/8/2011 06:35"
    ];
    for p in patterns_dt {
        if let Ok(dt) = NaiveDateTime::parse_from_str(text, p) {
            return Some(dt.time().num_seconds_from_midnight() as f64 / 86_400.0);
        }
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.time().num_seconds_from_midnight() as f64 / 86_400.0);
    }
    None
}

// Custom parser that handles time normalization like Excel does
fn parse_time_with_normalization(text: &str) -> Option<f64> {
    // Try to parse H:M:S format with potential overflow values
    let parts: Vec<&str> = text.split(':').collect();

    if parts.len() == 3 {
        // H:M:S format
        if let (Ok(h), Ok(m), Ok(s)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<i32>(),
            parts[2].parse::<i32>(),
        ) {
            // Only normalize specific edge cases that Excel handles
            // Don't normalize arbitrary large values like 25:00:00
            if should_normalize_time_components(h, m, s) {
                return Some(normalize_time_components(h, m, s));
            }
        }
    } else if parts.len() == 2 {
        // H:M format (assume seconds = 0)
        if let (Ok(h), Ok(m)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
            // Only normalize specific edge cases
            if should_normalize_time_components(h, m, 0) {
                return Some(normalize_time_components(h, m, 0));
            }
        }
    }

    None
}

// Normalize time components with overflow handling (like Excel)
fn normalize_time_components(hour: i32, minute: i32, second: i32) -> f64 {
    // Convert everything to total seconds
    let mut total_seconds = hour * 3600 + minute * 60 + second;

    // Handle negative values by wrapping around
    if total_seconds < 0 {
        total_seconds = total_seconds.rem_euclid(86400);
    }

    // Normalize to within a day (0-86399 seconds)
    total_seconds %= 86400;

    // Convert to fraction of a day
    total_seconds as f64 / 86400.0
}

// Check if time components should be normalized (only specific Excel edge cases)
fn should_normalize_time_components(hour: i32, minute: i32, second: i32) -> bool {
    // Only normalize these specific cases that Excel handles:
    // 1. Hour 24 with valid minutes/seconds
    // 2. Hour 23 with minute 60 (becomes 24:00)
    // 3. Any time with second 60 that normalizes to exactly 24:00

    if hour == 24 && (0..=59).contains(&minute) && (0..=59).contains(&second) {
        return true; // 24:MM:SS -> normalize to next day
    }

    if hour == 23 && minute == 60 && (0..=59).contains(&second) {
        return true; // 23:60:SS -> normalize to 24:00:SS
    }

    if (0..=23).contains(&hour) && (0..=59).contains(&minute) && second == 60 {
        // Check if this normalizes to exactly 24:00:00
        let total_seconds = hour * 3600 + minute * 60 + second;
        return total_seconds == 86400; // Exactly 24:00:00
    }

    false
}

// Helper function to parse simple "N PM" / "N AM" formats
fn parse_simple_am_pm(text: &str) -> Option<(&str, bool)> {
    if let Some(hour_part) = text.strip_suffix(" PM") {
        if hour_part.chars().all(|c| c.is_ascii_digit()) {
            return Some((hour_part, true));
        }
    } else if let Some(hour_part) = text.strip_suffix(" AM") {
        if hour_part.chars().all(|c| c.is_ascii_digit()) {
            return Some((hour_part, false));
        }
    }
    None
}

impl Model {
    pub(crate) fn fn_day(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let day = date.day() as f64;
        CalcResult::Number(day)
    }

    pub(crate) fn fn_month(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let month = date.month() as f64;
        CalcResult::Number(month)
    }

    pub(crate) fn fn_eomonth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Function EOMONTH parameter 1 value is negative. It should be positive or zero.".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        if serial_number > MAXIMUM_DATE_SERIAL_NUMBER as i64 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Function DAY parameter 1 value is too large.".to_string(),
            };
        }

        let months = match self.get_number_no_bools(&args[1], cell) {
            Ok(c) => {
                let t = c.trunc();
                t as i32
            }
            Err(s) => return s,
        };

        let months_abs = months.unsigned_abs();

        let native_date = if months > 0 {
            date + Months::new(months_abs)
        } else {
            date - Months::new(months_abs)
        };

        // Instead of calculating the end of month we compute the first day of the following month
        // and take one day.
        let mut month = native_date.month() + 1;
        let mut year = native_date.year();
        if month == 13 {
            month = 1;
            year += 1;
        }
        match date_to_serial_number(1, month, year) {
            Ok(serial_number) => CalcResult::Number(serial_number as f64 - 1.0),
            Err(message) => CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message,
            },
        }
    }

    // year, month, day
    pub(crate) fn fn_date(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let year = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i32;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let month = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.floor();
                t as i32
            }
            Err(s) => return s,
        };
        let day = match self.get_number(&args[2], cell) {
            Ok(c) => {
                let t = c.floor();
                t as i32
            }
            Err(s) => return s,
        };
        match permissive_date_to_serial_number(day, month, year) {
            Ok(serial_number) => CalcResult::Number(serial_number as f64),
            Err(message) => CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message,
            },
        }
    }

    pub(crate) fn fn_year(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let year = date.year() as f64;
        CalcResult::Number(year)
    }

    // date, months
    pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };

        let months = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.trunc();
                t as i32
            }
            Err(s) => return s,
        };

        let months_abs = months.unsigned_abs();

        let native_date = if months > 0 {
            date + Months::new(months_abs)
        } else {
            date - Months::new(months_abs)
        };

        let serial_number = native_date.num_days_from_ce() - EXCEL_DATE_BASE;
        if serial_number < MINIMUM_DATE_SERIAL_NUMBER {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "EDATE out of bounds".to_string(),
            };
        }
        CalcResult::Number(serial_number as f64)
    }

    fn get_array_of_dates(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i64>, CalcResult> {
        let mut values = Vec::new();
        match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Number(v) => {
                let date_serial = v.floor() as i64;
                if from_excel_date(date_serial).is_err() {
                    return Err(CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    });
                }
                values.push(date_serial);
            }
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                for row in left.row..=right.row {
                    for column in left.column..=right.column {
                        match self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }) {
                            CalcResult::Number(v) => {
                                let date_serial = v.floor() as i64;
                                if from_excel_date(date_serial).is_err() {
                                    return Err(CalcResult::Error {
                                        error: Error::NUM,
                                        origin: cell,
                                        message: "Out of range parameters for date".to_string(),
                                    });
                                }
                                values.push(date_serial);
                            }
                            CalcResult::EmptyCell => {
                                // Empty cells are ignored in holiday lists
                            }
                            e @ CalcResult::Error { .. } => return Err(e),
                            _ => {
                                // Non-numeric values in holiday lists should cause VALUE error
                                return Err(CalcResult::Error {
                                    error: Error::VALUE,
                                    origin: cell,
                                    message: "Invalid holiday date".to_string(),
                                });
                            }
                        }
                    }
                }
            }
            CalcResult::String(_) => {
                // String holidays should cause VALUE error
                return Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid holiday date".to_string(),
                });
            }
            e @ CalcResult::Error { .. } => return Err(e),
            _ => {
                // Other non-numeric types should cause VALUE error
                return Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid holiday date".to_string(),
                });
            }
        }
        Ok(values)
    }

    pub(crate) fn fn_networkdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(n) => n.floor() as i64,
            Err(e) => return e,
        };
        let end_serial = match self.get_number(&args[1], cell) {
            Ok(n) => n.floor() as i64,
            Err(e) => return e,
        };
        let mut holidays: std::collections::HashSet<i64> = std::collections::HashSet::new();
        if args.len() == 3 {
            let values = match self.get_array_of_dates(&args[2], cell) {
                Ok(v) => v,
                Err(e) => return e,
            };
            for v in values {
                holidays.insert(v);
            }
        }

        let (from, to, sign) = if start_serial <= end_serial {
            (start_serial, end_serial, 1.0)
        } else {
            (end_serial, start_serial, -1.0)
        };
        let mut count = 0i64;
        for serial in from..=to {
            let date = match from_excel_date(serial) {
                Ok(d) => d,
                Err(_) => {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    }
                }
            };
            let weekday = date.weekday().number_from_monday();
            let is_weekend = matches!(weekday, 6 | 7);
            if !is_weekend && !holidays.contains(&serial) {
                count += 1;
            }
        }
        CalcResult::Number(count as f64 * sign)
    }

    fn parse_weekend_pattern(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<[bool; 7], CalcResult> {
        // Default: Saturday-Sunday weekend (pattern 1)
        let mut weekend = [false, false, false, false, false, true, true];
        if node.is_none() {
            return Ok(weekend);
        }
        let node_ref = match node {
            Some(n) => n,
            None => return Ok(weekend),
        };

        match self.evaluate_node_in_context(node_ref, cell) {
            CalcResult::Number(n) => {
                let code = n.trunc() as i32;
                if (n - n.trunc()).abs() > f64::EPSILON {
                    return Err(CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                weekend = match code {
                    1 | 0 => [false, false, false, false, false, true, true], // Saturday-Sunday
                    2 => [true, false, false, false, false, false, true],     // Sunday-Monday
                    3 => [true, true, false, false, false, false, false],     // Monday-Tuesday
                    4 => [false, true, true, false, false, false, false],     // Tuesday-Wednesday
                    5 => [false, false, true, true, false, false, false],     // Wednesday-Thursday
                    6 => [false, false, false, true, true, false, false],     // Thursday-Friday
                    7 => [false, false, false, false, true, true, false],     // Friday-Saturday
                    11 => [false, false, false, false, false, false, true],   // Sunday only
                    12 => [true, false, false, false, false, false, false],   // Monday only
                    13 => [false, true, false, false, false, false, false],   // Tuesday only
                    14 => [false, false, true, false, false, false, false],   // Wednesday only
                    15 => [false, false, false, true, false, false, false],   // Thursday only
                    16 => [false, false, false, false, true, false, false],   // Friday only
                    17 => [false, false, false, false, false, true, false],   // Saturday only
                    _ => {
                        return Err(CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "Invalid weekend".to_string(),
                        ))
                    }
                };
                Ok(weekend)
            }
            CalcResult::String(s) => {
                if s.len() != 7 {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                if !s.chars().all(|c| c == '0' || c == '1') {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                weekend = [false; 7];
                for (i, ch) in s.chars().enumerate() {
                    weekend[i] = ch == '1';
                }
                Ok(weekend)
            }
            CalcResult::Boolean(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid weekend".to_string(),
            )),
            e @ CalcResult::Error { .. } => Err(e),
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid weekend".to_string(),
            }),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(weekend),
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid weekend".to_string(),
            }),
        }
    }

    pub(crate) fn fn_networkdays_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(n) => n.floor() as i64,
            Err(e) => return e,
        };
        let end_serial = match self.get_number(&args[1], cell) {
            Ok(n) => n.floor() as i64,
            Err(e) => return e,
        };

        let weekend_pattern = match self.parse_weekend_pattern(args.get(2), cell) {
            Ok(p) => p,
            Err(e) => return e,
        };

        let mut holidays: std::collections::HashSet<i64> = std::collections::HashSet::new();
        if args.len() == 4 {
            let values = match self.get_array_of_dates(&args[3], cell) {
                Ok(v) => v,
                Err(e) => return e,
            };
            for v in values {
                holidays.insert(v);
            }
        }

        let (from, to, sign) = if start_serial <= end_serial {
            (start_serial, end_serial, 1.0)
        } else {
            (end_serial, start_serial, -1.0)
        };
        let mut count = 0i64;
        for serial in from..=to {
            let date = match from_excel_date(serial) {
                Ok(d) => d,
                Err(_) => {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    }
                }
            };
            let weekday = date.weekday().number_from_monday() as usize - 1;
            if !weekend_pattern[weekday] && !holidays.contains(&serial) {
                count += 1;
            }
        }
        CalcResult::Number(count as f64 * sign)
    }

    pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 0 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        // milliseconds since January 1, 1970 00:00:00 UTC.
        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let local_time = match DateTime::from_timestamp(seconds, 0) {
            Some(dt) => dt.with_timezone(&self.tz),
            None => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Invalid date".to_string(),
                }
            }
        };
        // 693_594 is computed as:
        // NaiveDate::from_ymd(1900, 1, 1).num_days_from_ce() - 2
        // The 2 days offset is because of Excel 1900 bug
        let days_from_1900 = local_time.num_days_from_ce() - 693_594;

        CalcResult::Number(days_from_1900 as f64)
    }

    pub(crate) fn fn_now(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 0 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        // milliseconds since January 1, 1970 00:00:00 UTC.
        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let local_time = match DateTime::from_timestamp(seconds, 0) {
            Some(dt) => dt.with_timezone(&self.tz),
            None => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Invalid date".to_string(),
                }
            }
        };
        // 693_594 is computed as:
        // NaiveDate::from_ymd(1900, 1, 1).num_days_from_ce() - 2
        // The 2 days offset is because of Excel 1900 bug
        let days_from_1900 = local_time.num_days_from_ce() - 693_594;
        let days = (local_time.num_seconds_from_midnight() as f64) / (60.0 * 60.0 * 24.0);

        CalcResult::Number(days_from_1900 as f64 + days.fract())
    }

    pub(crate) fn fn_time(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let hour = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let minute = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let second = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if hour < 0.0 || minute < 0.0 || second < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid time".to_string(),
            };
        }
        let total_seconds = hour.floor() * 3600.0 + minute.floor() * 60.0 + second.floor();
        let day_seconds = 24.0 * 3600.0;
        let secs = total_seconds.rem_euclid(day_seconds);
        CalcResult::Number(secs / day_seconds)
    }

    pub(crate) fn fn_timevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        match parse_time_string(&text) {
            Some(value) => CalcResult::Number(value),
            None => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid time".to_string(),
            },
        }
    }

    pub(crate) fn fn_hour(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if value < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid time".to_string(),
            };
        }
        let hours = (value.rem_euclid(1.0) * 24.0).floor();
        CalcResult::Number(hours)
    }

    pub(crate) fn fn_minute(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if value < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid time".to_string(),
            };
        }
        let total_seconds = (value.rem_euclid(1.0) * 86400.0).floor();
        let minutes = ((total_seconds / 60.0) as i64 % 60) as f64;
        CalcResult::Number(minutes)
    }

    pub(crate) fn fn_second(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if value < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid time".to_string(),
            };
        }
        let total_seconds = (value.rem_euclid(1.0) * 86400.0).floor();
        let seconds = (total_seconds as i64 % 60) as f64;
        CalcResult::Number(seconds)
    }
}
