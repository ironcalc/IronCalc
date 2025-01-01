use chrono::DateTime;
use chrono::Datelike;
use chrono::Months;
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
}
