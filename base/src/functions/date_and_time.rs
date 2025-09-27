use chrono::DateTime;
use chrono::Datelike;
use chrono::Months;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;

const SECONDS_PER_DAY: i32 = 86_400;
const SECONDS_PER_DAY_F64: f64 = SECONDS_PER_DAY as f64;

// ---------------------------------------------------------------------------
// Helper macros to eliminate boilerplate in date/time component extraction
// functions (DAY, MONTH, YEAR, HOUR, MINUTE, SECOND).
// ---------------------------------------------------------------------------

// Generate DAY / MONTH / YEAR helpers – simply convert the serial number to a
// NaiveDate and return the requested component as a number.
macro_rules! date_part_fn {
    ($name:ident, $method:ident) => {
        pub(crate) fn $name(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
            if args.len() != 1 {
                return CalcResult::new_args_number_error(cell);
            }
            let serial_number = match self.get_number(&args[0], cell) {
                Ok(num) => num.floor() as i64,
                Err(e) => return e,
            };
            let date = match self.excel_date(serial_number, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            CalcResult::Number(date.$method() as f64)
        }
    };
}

// Generate HOUR / MINUTE / SECOND helpers – extract the desired component from
// a day-fraction value. We pass an extraction closure so each helper can keep
// its own formula while sharing the surrounding boilerplate.
macro_rules! time_part_fn {
    ($name:ident, $extract:expr) => {
        pub(crate) fn $name(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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
            CalcResult::Number(($extract)(value))
        }
    };
}

use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::dates::date_to_serial_number;
use crate::formatter::dates::permissive_date_to_serial_number;
use crate::formatter::dates::DATE_OUT_OF_RANGE_MESSAGE;
use crate::model::get_milliseconds_since_epoch;
use crate::number_format::to_precision;
use crate::{
    calc_result::CalcResult,
    constants::EXCEL_DATE_BASE,
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    formatter::dates::from_excel_date,
    model::Model,
};

#[derive(Debug, Clone, Copy)]
enum WeekendPattern {
    SatSun,
    SunMon,
    MonTue,
    TueWed,
    WedThu,
    ThuFri,
    FriSat,
    SunOnly,
    MonOnly,
    TueOnly,
    WedOnly,
    ThuOnly,
    FriOnly,
    SatOnly,
}

impl std::convert::TryFrom<i32> for WeekendPattern {
    type Error = ();
    fn try_from(code: i32) -> Result<Self, Self::Error> {
        Ok(match code {
            1 => Self::SatSun,
            2 => Self::SunMon,
            3 => Self::MonTue,
            4 => Self::TueWed,
            5 => Self::WedThu,
            6 => Self::ThuFri,
            7 => Self::FriSat,
            11 => Self::SunOnly,
            12 => Self::MonOnly,
            13 => Self::TueOnly,
            14 => Self::WedOnly,
            15 => Self::ThuOnly,
            16 => Self::FriOnly,
            17 => Self::SatOnly,
            _ => return Err(()),
        })
    }
}

impl WeekendPattern {
    fn to_mask(self) -> [bool; 7] {
        match self {
            Self::SatSun => [false, false, false, false, false, true, true],
            Self::SunMon => [true, false, false, false, false, false, true],
            Self::MonTue => [true, true, false, false, false, false, false],
            Self::TueWed => [false, true, true, false, false, false, false],
            Self::WedThu => [false, false, true, true, false, false, false],
            Self::ThuFri => [false, false, false, true, true, false, false],
            Self::FriSat => [false, false, false, false, true, true, false],
            Self::SunOnly => [false, false, false, false, false, false, true],
            Self::MonOnly => [true, false, false, false, false, false, false],
            Self::TueOnly => [false, true, false, false, false, false, false],
            Self::WedOnly => [false, false, true, false, false, false, false],
            Self::ThuOnly => [false, false, false, true, false, false, false],
            Self::FriOnly => [false, false, false, false, true, false, false],
            Self::SatOnly => [false, false, false, false, false, true, false],
        }
    }
}

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
                return Some(time.num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
            }
        }
    }

    // Standard patterns
    let patterns_time = ["%H:%M:%S", "%H:%M", "%I:%M %p", "%I %p", "%I:%M:%S %p"];
    for p in patterns_time {
        if let Ok(t) = NaiveTime::parse_from_str(text, p) {
            return Some(t.num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
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
            return Some(dt.time().num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
        }
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        return Some(dt.time().num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64);
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
        total_seconds = total_seconds.rem_euclid(SECONDS_PER_DAY);
    }

    // Normalize to within a day (0-86399 seconds)
    total_seconds %= SECONDS_PER_DAY;

    // Convert to fraction of a day
    total_seconds as f64 / SECONDS_PER_DAY_F64
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
        return total_seconds == SECONDS_PER_DAY; // Exactly 24:00:00
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

fn parse_day_simple(day_str: &str) -> Result<u32, String> {
    let bytes_len = day_str.len();
    if bytes_len == 0 || bytes_len > 2 {
        return Err("Not a valid day".to_string());
    }
    match day_str.parse::<u32>() {
        Ok(y) => Ok(y),
        Err(_) => Err("Not a valid day".to_string()),
    }
}

fn parse_month_simple(month_str: &str) -> Result<u32, String> {
    let bytes_len = month_str.len();
    if bytes_len == 0 {
        return Err("Not a valid month".to_string());
    }
    if bytes_len <= 2 {
        // Numeric month representation. Ensure it is within the valid range 1-12.
        return match month_str.parse::<u32>() {
            Ok(m) if (1..=12).contains(&m) => Ok(m),
            _ => Err("Not a valid month".to_string()),
        };
    }

    // Textual month representations.
    // Use standard 3-letter abbreviations (e.g. "Sep") but also accept the legacy "Sept".
    let month_names_short = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month_names_long = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    if let Some(m) = month_names_short
        .iter()
        .position(|&r| r.eq_ignore_ascii_case(month_str))
    {
        return Ok(m as u32 + 1);
    }
    // Special-case the non-standard abbreviation "Sept" so older inputs still work.
    if month_str.eq_ignore_ascii_case("Sept") {
        return Ok(9);
    }

    if let Some(m) = month_names_long
        .iter()
        .position(|&r| r.eq_ignore_ascii_case(month_str))
    {
        return Ok(m as u32 + 1);
    }
    Err("Not a valid month".to_string())
}

fn parse_year_simple(year_str: &str) -> Result<i32, String> {
    let bytes_len = year_str.len();
    if bytes_len != 2 && bytes_len != 4 {
        return Err("Not a valid year".to_string());
    }
    let y = year_str
        .parse::<i32>()
        .map_err(|_| "Not a valid year".to_string())?;
    if y < 30 && bytes_len == 2 {
        Ok(2000 + y)
    } else if y < 100 && bytes_len == 2 {
        Ok(1900 + y)
    } else {
        Ok(y)
    }
}

fn parse_datevalue_text(value: &str) -> Result<i32, String> {
    // Trim whitespace and discard any time component (e.g., "2024-02-29 06:00" -> "2024-02-29")
    let mut date_str = value.trim();
    if let Some(idx) = date_str.find('T') {
        date_str = &date_str[..idx];
    }
    if let Some(idx) = date_str.find(' ') {
        date_str = &date_str[..idx];
    }

    let separator = if date_str.contains('/') {
        '/'
    } else if date_str.contains('-') {
        '-'
    } else {
        return Err("Not a valid date".to_string());
    };

    let mut parts: Vec<&str> = date_str.split(separator).map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return Err("Not a valid date".to_string());
    }

    // Identify the year: prefer the one that is four-digit numeric, otherwise assume the third part.
    let mut year_idx: usize = 2;
    for (idx, p) in parts.iter().enumerate() {
        if p.len() == 4 && p.chars().all(char::is_numeric) {
            year_idx = idx;
            break;
        }
    }

    let year_str = parts[year_idx];
    // Remove the year from the remaining vector to process day / month.
    parts.remove(year_idx);
    let part1 = parts[0];
    let part2 = parts[1];

    // Helper closures
    let is_numeric = |s: &str| s.chars().all(char::is_numeric);

    // Determine month and day.
    let (month_str, day_str) = if !is_numeric(part1) {
        // textual month in first
        (part1, part2)
    } else if !is_numeric(part2) {
        // textual month in second
        (part2, part1)
    } else {
        // Both numeric – apply disambiguation rules
        let v1: u32 = part1.parse().unwrap_or(0);
        let v2: u32 = part2.parse().unwrap_or(0);
        match (v1 > 12, v2 > 12) {
            (true, false) => (part2, part1), // first cannot be month
            (false, true) => (part1, part2), // second cannot be month
            _ => (part1, part2),             // ambiguous -> assume MM/DD
        }
    };

    let day = parse_day_simple(day_str)?;
    let month = parse_month_simple(month_str)?;
    let year = parse_year_simple(year_str)?;

    // Excel 1900 leap-year bug: 29-Feb-1900 is treated as serial 60
    if year == 1900 && month == 2 && day == 29 {
        return Ok(60);
    }

    match date_to_serial_number(day, month, year) {
        Ok(n) => Ok(n),
        Err(_) => Err("Not a valid date".to_string()),
    }
}

impl Model {
    fn get_date_serial(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        match result {
            CalcResult::Number(f) => Ok(f.floor() as i64),
            CalcResult::String(s) => match parse_datevalue_text(&s) {
                Ok(n) => Ok(n as i64),
                Err(_) => Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid date".to_string(),
                }),
            },
            CalcResult::Boolean(b) => {
                if b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(0),
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
        }
    }

    // -----------------------------------------------------------------------
    // Auto-generated DATE part helpers (DAY / MONTH / YEAR)
    // -----------------------------------------------------------------------
    date_part_fn!(fn_day, day);
    date_part_fn!(fn_month, month);
    date_part_fn!(fn_year, year);

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
        let date = match self.excel_date(serial_number, cell) {
            Ok(d) => d,
            Err(e) => return e,
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
                        message: DATE_OUT_OF_RANGE_MESSAGE.to_string(),
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
        let date = match self.excel_date(serial_number, cell) {
            Ok(d) => d,
            Err(e) => return e,
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

    /// Walk a scalar / range / array node and invoke the provided closure with every
    /// numeric date serial that is encountered.
    fn collect_serial_numbers<F>(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        mut handle: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(i64) -> Result<(), CalcResult>,
    {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(v) => {
                let serial = v.floor() as i64;
                // Validate serial is in bounds
                self.excel_date(serial, cell)?;
                handle(serial)?;
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
                                let serial = v.floor() as i64;
                                self.excel_date(serial, cell)?;
                                handle(serial)?;
                            }
                            CalcResult::EmptyCell => {
                                // ignore empty cells
                            }
                            e @ CalcResult::Error { .. } => return Err(e),
                            _ => {
                                return Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Invalid holiday date".to_string(),
                                ))
                            }
                        }
                    }
                }
            }
            CalcResult::Array(array) => {
                for row in array {
                    for value in row {
                        match value {
                            ArrayNode::Number(num) => {
                                let serial = num.floor() as i64;
                                self.excel_date(serial, cell)?;
                                handle(serial)?;
                            }
                            ArrayNode::Error(error) => {
                                return Err(CalcResult::Error {
                                    error,
                                    origin: cell,
                                    message: "Error in array".to_string(),
                                });
                            }
                            _ => {
                                return Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Invalid holiday date".to_string(),
                                ))
                            }
                        }
                    }
                }
            }
            e @ CalcResult::Error { .. } => return Err(e),
            _ => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid holiday date".to_string(),
                ))
            }
        }
        Ok(())
    }

    fn get_array_of_dates(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i64>, CalcResult> {
        let mut values = Vec::new();
        self.collect_serial_numbers(arg, cell, |serial| {
            values.push(serial);
            Ok(())
        })?;
        Ok(values)
    }

    // Returns the current date/time as an Excel serial number in the model's configured timezone.
    // Used by TODAY() and NOW().
    fn current_excel_serial(&self) -> Option<f64> {
        let seconds = get_milliseconds_since_epoch() / 1000;
        DateTime::from_timestamp(seconds, 0).map(|dt| {
            let local_time = dt.with_timezone(&self.tz);
            let days_from_1900 = local_time.num_days_from_ce() - EXCEL_DATE_BASE;
            let fraction = (local_time.num_seconds_from_midnight() as f64) / (60.0 * 60.0 * 24.0);
            days_from_1900 as f64 + fraction
        })
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
            let date = match self.excel_date(serial, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            let weekday = date.weekday().number_from_monday();
            let is_weekend = matches!(weekday, 6 | 7);
            if !is_weekend && !holidays.contains(&serial) {
                count += 1;
            }
        }
        CalcResult::Number(count as f64 * sign)
    }

    fn excel_date(
        &self,
        serial: i64,
        cell: CellReferenceIndex,
    ) -> Result<chrono::NaiveDate, CalcResult> {
        match from_excel_date(serial) {
            Ok(date) => Ok(date),
            Err(_) => Err(CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Out of range parameters for date".to_string(),
            }),
        }
    }

    fn weekend_mask(
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
                weekend = match WeekendPattern::try_from(code) {
                    Ok(pattern) => pattern.to_mask(),
                    Err(_) => {
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

        let weekend_pattern = match self.weekend_mask(args.get(2), cell) {
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
            let date = match self.excel_date(serial, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            let weekday = date.weekday().number_from_monday() as usize - 1;
            if !weekend_pattern[weekday] && !holidays.contains(&serial) {
                count += 1;
            }
        }
        CalcResult::Number(count as f64 * sign)
    }

    pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        match self.current_excel_serial() {
            Some(serial) => CalcResult::Number(serial.floor()),
            None => CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Invalid date".to_string(),
            },
        }
    }

    pub(crate) fn fn_now(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        match self.current_excel_serial() {
            Some(serial) => CalcResult::Number(serial),
            None => CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Invalid date".to_string(),
            },
        }
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
        let total_seconds = hour.floor() * 3600.0 + minute.floor() * 60.0 + second.floor();
        if total_seconds < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid time".to_string(),
            };
        }
        let secs = total_seconds.rem_euclid(SECONDS_PER_DAY_F64);
        CalcResult::Number(secs / SECONDS_PER_DAY_F64)
    }

    // -----------------------------------------------------------------------
    // Auto-generated TIME part helpers (HOUR / MINUTE / SECOND)
    // -----------------------------------------------------------------------

    time_part_fn!(fn_hour, |v: f64| (v.rem_euclid(1.0) * 24.0).floor());
    time_part_fn!(fn_minute, |v: f64| {
        let total_seconds = (v.rem_euclid(1.0) * SECONDS_PER_DAY_F64).floor();
        ((total_seconds / 60.0) as i64 % 60) as f64
    });
    time_part_fn!(fn_second, |v: f64| {
        let total_seconds = to_precision(v.rem_euclid(1.0) * SECONDS_PER_DAY_F64, 15).floor();
        (total_seconds as i64 % 60) as f64
    });

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

    pub(crate) fn fn_datevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::String(s) => match parse_datevalue_text(&s) {
                Ok(n) => CalcResult::Number(n as f64),
                Err(_) => CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid date".to_string(),
                },
            },
            CalcResult::Number(f) => CalcResult::Number(f.floor()),
            CalcResult::Boolean(b) => {
                if b {
                    CalcResult::Number(1.0)
                } else {
                    CalcResult::Number(0.0)
                }
            }
            err @ CalcResult::Error { .. } => err,
            CalcResult::Range { .. } | CalcResult::Array(_) => CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            },
            CalcResult::EmptyCell | CalcResult::EmptyArg => CalcResult::Number(0.0),
        }
    }

    pub(crate) fn fn_datedif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let start_serial = match self.get_date_serial(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let end_serial = match self.get_date_serial(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if end_serial < start_serial {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Start date greater than end date".to_string(),
            };
        }
        let start = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let end = match self.excel_date(end_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let unit = match self.get_string(&args[2], cell) {
            Ok(s) => s.to_ascii_uppercase(),
            Err(e) => return e,
        };

        let result = match unit.as_str() {
            "Y" => {
                let mut years = end.year() - start.year();
                if (end.month(), end.day()) < (start.month(), start.day()) {
                    years -= 1;
                }
                years as f64
            }
            "M" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                months as f64
            }
            "D" => (end_serial - start_serial) as f64,
            "YM" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                (months % 12).abs() as f64
            }
            "YD" => {
                // Helper to create a date or early-return with #NUM! if impossible
                let make_date = |y: i32, m: u32, d: u32| -> Result<chrono::NaiveDate, CalcResult> {
                    match chrono::NaiveDate::from_ymd_opt(y, m, d) {
                        Some(dt) => Ok(dt),
                        None => Err(CalcResult::Error {
                            error: Error::NUM,
                            origin: cell,
                            message: "Invalid date".to_string(),
                        }),
                    }
                };

                // Compute the last valid day of a given month/year.
                let make_last_day_of_month =
                    |y: i32, m: u32| -> Result<chrono::NaiveDate, CalcResult> {
                        let (next_y, next_m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                        let first_next = make_date(next_y, next_m, 1)?;
                        let last_day = first_next - chrono::Duration::days(1);
                        make_date(y, m, last_day.day())
                    };

                // Attempt to build the anniversary date in the end year.
                let mut start_adj =
                    match chrono::NaiveDate::from_ymd_opt(end.year(), start.month(), start.day()) {
                        Some(d) => d,
                        None => match make_last_day_of_month(end.year(), start.month()) {
                            Ok(d) => d,
                            Err(e) => return e,
                        },
                    };

                // If the adjusted date is after the end date, shift one year back.
                if start_adj > end {
                    let shift_year = end.year() - 1;
                    start_adj = match chrono::NaiveDate::from_ymd_opt(
                        shift_year,
                        start.month(),
                        start.day(),
                    ) {
                        Some(d) => d,
                        None => match make_last_day_of_month(shift_year, start.month()) {
                            Ok(d) => d,
                            Err(e) => return e,
                        },
                    };
                }

                (end - start_adj).num_days() as f64
            }
            "MD" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                let start_shifted = if months >= 0 {
                    start + Months::new(months as u32)
                } else {
                    start - Months::new((-months) as u32)
                };
                (end - start_shifted).num_days() as f64
            }
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid unit".to_string(),
                };
            }
        };
        CalcResult::Number(result)
    }

    pub(crate) fn fn_days(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let end_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let start_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        match self.excel_date(end_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        CalcResult::Number((end_serial - start_serial) as f64)
    }

    pub(crate) fn fn_days360(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let end_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let method = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            false
        };
        let start_date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let end_date = match self.excel_date(end_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        fn last_day_feb(year: i32) -> u32 {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        let mut sd_day = start_date.day();
        let sd_month = start_date.month();
        let sd_year = start_date.year();
        let mut ed_day = end_date.day();
        let ed_month = end_date.month();
        let ed_year = end_date.year();

        if method {
            if sd_day == 31 {
                sd_day = 30;
            }
            if ed_day == 31 {
                ed_day = 30;
            }
        } else {
            if (sd_month == 2 && sd_day == last_day_feb(sd_year)) || sd_day == 31 {
                sd_day = 30;
            }
            if ed_month == 2 && ed_day == last_day_feb(ed_year) && sd_day == 30 {
                ed_day = 30;
            }
            if ed_day == 31 && sd_day >= 30 {
                ed_day = 30;
            }
        }

        let result = (ed_year - sd_year) * 360
            + (ed_month as i32 - sd_month as i32) * 30
            + (ed_day as i32 - sd_day as i32);
        CalcResult::Number(result as f64)
    }

    pub(crate) fn fn_weekday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match self.excel_date(serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let return_type = if args.len() == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let weekday = date.weekday();
        let num = match return_type {
            1 => weekday.num_days_from_sunday() + 1,
            2 => weekday.number_from_monday(),
            3 => (weekday.number_from_monday() - 1) % 7, // 0-based Monday start
            11..=17 => {
                let start = (return_type - 11) as u32; // 0 = Monday, 6 = Sunday
                let zero_based = weekday.number_from_monday() - 1; // 0..6, Monday = 0
                ((zero_based + 7 - start) % 7) + 1
            }
            0 => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid return_type".to_string())
            }
            _ => return CalcResult::new_error(Error::NUM, cell, "Invalid return_type".to_string()),
        };
        CalcResult::Number(num as f64)
    }

    pub(crate) fn fn_weeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match self.excel_date(serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let return_type = if args.len() == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        if return_type == 21 {
            let w = date.iso_week().week();
            return CalcResult::Number(w as f64);
        }
        let start_offset = match return_type {
            1 => chrono::Weekday::Sun,
            2 | 11 => chrono::Weekday::Mon,
            12 => chrono::Weekday::Tue,
            13 => chrono::Weekday::Wed,
            14 => chrono::Weekday::Thu,
            15 => chrono::Weekday::Fri,
            16 => chrono::Weekday::Sat,
            17 => chrono::Weekday::Sun,
            x if x <= 0 || x == 3 => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid return_type".to_string())
            }
            _ => return CalcResult::new_error(Error::NUM, cell, "Invalid return_type".to_string()),
        };
        let mut first = match chrono::NaiveDate::from_ymd_opt(date.year(), 1, 1) {
            Some(d) => d,
            None => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    DATE_OUT_OF_RANGE_MESSAGE.to_string(),
                );
            }
        };
        while first.weekday() != start_offset {
            first -= chrono::Duration::days(1);
        }
        let week = (date - first).num_days() / 7 + 1;
        CalcResult::Number(week as f64)
    }

    pub(crate) fn fn_isoweeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match self.excel_date(serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        CalcResult::Number(date.iso_week().week() as f64)
    }

    fn is_weekend(day: chrono::Weekday, weekend_mask: &[bool; 7]) -> bool {
        match day {
            chrono::Weekday::Mon => weekend_mask[0],
            chrono::Weekday::Tue => weekend_mask[1],
            chrono::Weekday::Wed => weekend_mask[2],
            chrono::Weekday::Thu => weekend_mask[3],
            chrono::Weekday::Fri => weekend_mask[4],
            chrono::Weekday::Sat => weekend_mask[5],
            chrono::Weekday::Sun => weekend_mask[6],
        }
    }

    pub(crate) fn fn_workday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let mut date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let mut days = match self.get_number(&args[1], cell) {
            Ok(f) => f as i32,
            Err(s) => return s,
        };
        let weekend = [false, false, false, false, false, true, true];
        let holiday_set = match self.get_holiday_set(args.get(2), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        while days != 0 {
            if days > 0 {
                date += chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend) && !holiday_set.contains(&date) {
                    days -= 1;
                }
            } else {
                date -= chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend) && !holiday_set.contains(&date) {
                    days += 1;
                }
            }
        }
        let serial = date.num_days_from_ce() - EXCEL_DATE_BASE;
        CalcResult::Number(serial as f64)
    }

    fn get_holiday_set(
        &mut self,
        arg_option: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<std::collections::HashSet<chrono::NaiveDate>, CalcResult> {
        let mut holiday_set = std::collections::HashSet::new();

        if let Some(arg) = arg_option {
            self.collect_serial_numbers(arg, cell, |serial| match from_excel_date(serial) {
                Ok(date) => {
                    holiday_set.insert(date);
                    Ok(())
                }
                Err(_) => Err(CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid holiday date".to_string(),
                }),
            })?;
        }

        Ok(holiday_set)
    }

    pub(crate) fn fn_workday_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let mut date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let mut days = match self.get_number(&args[1], cell) {
            Ok(f) => f as i32,
            Err(s) => return s,
        };
        let weekend_mask = match self.weekend_mask(args.get(2), cell) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let holiday_set = match self.get_holiday_set(args.get(3), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };
        while days != 0 {
            if days > 0 {
                date += chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend_mask) && !holiday_set.contains(&date)
                {
                    days -= 1;
                }
            } else {
                date -= chrono::Duration::days(1);
                if !Self::is_weekend(date.weekday(), &weekend_mask) && !holiday_set.contains(&date)
                {
                    days += 1;
                }
            }
        }
        let serial = date.num_days_from_ce() - EXCEL_DATE_BASE;
        CalcResult::Number(serial as f64)
    }

    pub(crate) fn fn_yearfrac(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let start_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let end_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let basis = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            }
        } else {
            0
        };
        let start_date = match self.excel_date(start_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let end_date = match self.excel_date(end_serial, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let days = (end_date - start_date).num_days() as f64;
        let result = match basis {
            0 => {
                let d360 = self.fn_days360(args, cell);
                if let CalcResult::Number(n) = d360 {
                    n / 360.0
                } else {
                    return d360;
                }
            }
            1 => {
                let year_days = if start_date.year() == end_date.year() {
                    if (start_date.year() % 4 == 0 && start_date.year() % 100 != 0)
                        || start_date.year() % 400 == 0
                    {
                        366.0
                    } else {
                        365.0
                    }
                } else {
                    365.0
                };
                days / year_days
            }
            2 => days / 360.0,
            3 => days / 365.0,
            4 => {
                let d360 = self.fn_days360(
                    &[args[0].clone(), args[1].clone(), Node::NumberKind(1.0)],
                    cell,
                );
                if let CalcResult::Number(n) = d360 {
                    n / 360.0
                } else {
                    return d360;
                }
            }
            _ => return CalcResult::new_error(Error::NUM, cell, "Invalid basis".to_string()),
        };
        CalcResult::Number(result)
    }
}
