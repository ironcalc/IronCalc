use chrono::Datelike;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    formatter::dates::{date_to_serial_number, from_excel_date},
    model::Model,
};

use super::financial_util::{compute_irr, compute_npv, compute_rate, compute_xirr, compute_xnpv};

// See:
// https://github.com/apache/openoffice/blob/c014b5f2b55cff8d4b0c952d5c16d62ecde09ca1/main/scaddins/source/analysis/financial.cxx

// Add a signed number of months to an Excel-serial date with end-of-month
// snapping. Used to walk quasi-coupon period boundaries in fn_accrint.
//
// If the source date is the last day of its month, the result is also the
// last day of the target month. Otherwise, the result preserves the source
// day-of-month, clamped to the last day of the target month if necessary
// (e.g. adding one month to Jan 31 yields Feb 28/29).
//
// `months_to_add` may be negative (walk backward) or positive.
fn add_months_eom(serial: i64, months_to_add: i32) -> Result<i64, String> {
    let date = from_excel_date(serial)?;
    let src_year = date.year();
    let src_month = date.month() as i32;
    let src_day = date.day();
    let total_months = src_year * 12 + (src_month - 1) + months_to_add;
    let dst_year = total_months.div_euclid(12);
    let dst_month = total_months.rem_euclid(12) + 1;
    let last_day_src = last_day_of_month(src_year, src_month as u32);
    let last_day_dst = last_day_of_month(dst_year, dst_month as u32);
    let dst_day = if src_day >= last_day_src {
        last_day_dst
    } else {
        src_day.min(last_day_dst)
    };
    let serial_i32 = date_to_serial_number(dst_day, dst_month as u32, dst_year)?;
    Ok(serial_i32 as i64)
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // Gregorian leap-year rule
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_less_than_one_year(start_date: i64, end_date: i64) -> Result<bool, String> {
    let end = from_excel_date(end_date)?;
    let start = from_excel_date(start_date)?;
    if end_date - start_date < 365 {
        return Ok(true);
    }
    let end_year = end.year();
    let start_year = start.year();
    if end_year == start_year {
        return Ok(true);
    }
    if end_year != start_year + 1 {
        return Ok(false);
    }
    let start_month = start.month();
    let end_month = end.month();
    if end_month < start_month {
        return Ok(true);
    }
    if end_month > start_month {
        return Ok(false);
    }
    // we are one year later same month
    let start_day = start.day();
    let end_day = end.day();
    Ok(end_day <= start_day)
}

fn compute_payment(
    rate: f64,
    nper: f64,
    pv: f64,
    fv: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    if rate == 0.0 {
        if nper == 0.0 {
            return Err((Error::NUM, "Period count must be non zero".to_string()));
        }
        return Ok(-(pv + fv) / nper);
    }
    if rate <= -1.0 {
        return Err((Error::NUM, "Rate must be > -1".to_string()));
    };
    let rate_nper = if nper == 0.0 {
        1.0
    } else {
        (1.0 + rate).powf(nper)
    };
    let result = if period_start {
        // type = 1
        (fv + pv * rate_nper) * rate / ((1.0 + rate) * (1.0 - rate_nper))
    } else {
        (fv * rate + pv * rate * rate_nper) / (1.0 - rate_nper)
    };
    if result.is_nan() || result.is_infinite() {
        return Err((Error::NUM, "Invalid result".to_string()));
    }
    Ok(result)
}

fn compute_future_value(
    rate: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    if rate == 0.0 {
        return Ok(-pv - pmt * nper);
    }
    if rate == -1.0 && nper < 0.0 {
        return Err((Error::DIV, "Divide by zero".to_string()));
    }

    let rate_nper = (1.0 + rate).powf(nper);
    let fv = if period_start {
        // type = 1
        -pv * rate_nper - pmt * (1.0 + rate) * (rate_nper - 1.0) / rate
    } else {
        -pv * rate_nper - pmt * (rate_nper - 1.0) / rate
    };
    if fv.is_nan() {
        return Err((Error::NUM, "Invalid result".to_string()));
    }
    if !fv.is_finite() {
        return Err((Error::DIV, "Divide by zero".to_string()));
    }
    Ok(fv)
}

fn compute_ipmt(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    // http://www.staff.city.ac.uk/o.s.kerr/CompMaths/WSheet4.pdf
    // https://www.experts-exchange.com/articles/1948/A-Guide-to-the-PMT-FV-IPMT-and-PPMT-Functions.html
    // type = 0 (end of period)
    // impt = -[(1+rate)^(period-1)*(pv*rate+pmt)-pmt]
    // ipmt = FV(rate, period-1, payment, pv, type) * rate
    // type = 1 (beginning of period)
    // ipmt = (FV(rate, period-2, payment, pv, type) - payment) * rate
    let payment = compute_payment(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    if period < 1.0 || period >= period_count + 1.0 {
        return Err((
            Error::NUM,
            format!("Period must be between 1 and {}", period_count + 1.0),
        ));
    }
    if period == 1.0 && period_start {
        Ok(0.0)
    } else {
        let p = if period_start {
            period - 2.0
        } else {
            period - 1.0
        };
        let c = if period_start { -payment } else { 0.0 };
        let fv = compute_future_value(rate, p, payment, present_value, period_start)?;
        Ok((fv + c) * rate)
    }
}

fn compute_ppmt(
    rate: f64,
    period: f64,
    period_count: f64,
    present_value: f64,
    future_value: f64,
    period_start: bool,
) -> Result<f64, (Error, String)> {
    let payment = compute_payment(
        rate,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    // It's a bit unfortunate that the first thing compute_ipmt does is compute_payment again
    let ipmt = compute_ipmt(
        rate,
        period,
        period_count,
        present_value,
        future_value,
        period_start,
    )?;
    Ok(payment - ipmt)
}

// These formulas revolve around compound interest and annuities.
// The financial functions pv, rate, nper, pmt and fv:
// rate = interest rate per period
// nper (number of periods) = loan term
// pv (present value) = loan amount
// fv (future value) = cash balance after last payment. Default is 0
// type = the annuity type indicates when payments are due
//         * 0 (default) Payments are made at the end of the period
//         * 1 Payments are made at the beginning of the period (like a lease or rent)
// The variable period_start is true if type is 1
// They are linked by the formulas:
// If rate != 0
//   $pv*(1+rate)^nper+pmt*(1+rate*type)*((1+rate)^nper-1)/rate+fv=0$
// If rate = 0
//   $pmt*nper+pv+fv=0$
// All, except for rate are easily solvable in terms of the others.
// In these formulas the payment (pmt) is normally negative

impl<'a> Model<'a> {
    fn get_array_of_numbers_generic(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        accept_number_node: bool,
        handle_empty_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
        handle_non_number_cell: impl Fn() -> Result<Option<f64>, CalcResult>,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        match self.evaluate_node_in_context(arg, *cell) {
            CalcResult::Number(value) if accept_number_node => values.push(value),
            CalcResult::Number(_) => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected range of numbers".to_string(),
                ));
            }
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        *cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let sheet = left.sheet;
                let row1 = left.row;
                let mut row2 = right.row;
                let column1 = left.column;
                let mut column2 = right.column;
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = self
                        .workbook
                        .worksheet(sheet)
                        .map_err(|_| {
                            CalcResult::new_error(
                                Error::ERROR,
                                *cell,
                                format!("Invalid worksheet index: '{sheet}'"),
                            )
                        })?
                        .dimension()
                        .max_row;
                }
                if column1 == 1 && column2 == LAST_COLUMN {
                    column2 = self
                        .workbook
                        .worksheet(sheet)
                        .map_err(|_| {
                            CalcResult::new_error(
                                Error::ERROR,
                                *cell,
                                format!("Invalid worksheet index: '{sheet}'"),
                            )
                        })?
                        .dimension()
                        .max_column;
                }
                for row in row1..=row2 {
                    for column in column1..=column2 {
                        let cell_ref = CellReferenceIndex { sheet, row, column };
                        match self.evaluate_cell(cell_ref) {
                            CalcResult::Number(value) => values.push(value),
                            error @ CalcResult::Error { .. } => return Err(error),
                            CalcResult::EmptyCell => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
            }
            CalcResult::Array(arr) => {
                for row in arr {
                    for node in row {
                        match node {
                            ArrayNode::Number(f) => values.push(f),
                            ArrayNode::Error(e) => {
                                return Err(CalcResult::new_error(
                                    e,
                                    *cell,
                                    "Error in array".to_string(),
                                ));
                            }
                            ArrayNode::Empty => {
                                if let Some(value) = handle_empty_cell()? {
                                    values.push(value);
                                }
                            }
                            _ => {
                                if let Some(value) = handle_non_number_cell()? {
                                    values.push(value);
                                }
                            }
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {
                handle_non_number_cell()?;
            }
        }
        Ok(values)
    }

    fn get_array_of_numbers(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true,        // accept_number_node
            || Ok(None), // Ignore empty cells
            || Ok(None), // Ignore non-number cells
        )
    }

    fn get_array_of_numbers_xpnv(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        error: Error,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            true, // accept_number_node
            || {
                Err(CalcResult::new_error(
                    Error::NUM,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
            || {
                Err(CalcResult::new_error(
                    error.clone(),
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    fn get_array_of_numbers_xirr(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_generic(
            arg,
            cell,
            false,            // Do not accept a single number node
            || Ok(Some(0.0)), // Treat empty cells as zero
            || {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    *cell,
                    "Expected number".to_string(),
                ))
            },
        )
    }

    /// PMT(rate, nper, pv, [fv], [type])
    pub(crate) fn fn_pmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // number of periods
        let nper = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // present value
        let pv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // future_value
        let fv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        match compute_payment(rate, nper, pv, fv, period_start) {
            Ok(p) => CalcResult::Number(p),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // PV(rate, nper, pmt, [fv], [type])
    pub(crate) fn fn_pv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pmt
        let payment = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        if rate == 0.0 {
            return CalcResult::Number(-future_value - payment * period_count);
        }
        if rate == -1.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Rate must be != -1".to_string(),
            };
        };
        let rate_nper = (1.0 + rate).powf(period_count);
        let result = if period_start {
            // type = 1
            -(future_value * rate + payment * (1.0 + rate) * (rate_nper - 1.0)) / (rate * rate_nper)
        } else {
            (-future_value * rate - payment * (rate_nper - 1.0)) / (rate * rate_nper)
        };
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // RATE(nper, pmt, pv, [fv], [type], [guess])
    pub(crate) fn fn_rate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let nper = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pmt = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let fv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let annuity_type = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => i32::from(f != 0.0),
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            0
        };

        let guess = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };

        match compute_rate(pv, fv, nper, pmt, annuity_type, guess) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // NPER(rate,pmt,pv,[fv],[type])
    pub(crate) fn fn_nper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pmt
        let payment = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        if rate == 0.0 {
            if payment == 0.0 {
                return CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "Divide by zero".to_string(),
                };
            }
            return CalcResult::Number(-(future_value + present_value) / payment);
        }
        if rate < -1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Rate must be > -1".to_string(),
            };
        };
        let rate_nper = if period_start {
            // type = 1
            if payment != 0.0 {
                let term = payment * (1.0 + rate) / rate;
                (1.0 - future_value / term) / (1.0 + present_value / term)
            } else {
                -future_value / present_value
            }
        } else {
            // type = 0
            if payment != 0.0 {
                let term = payment / rate;
                (1.0 - future_value / term) / (1.0 + present_value / term)
            } else {
                -future_value / present_value
            }
        };
        if rate_nper <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Cannot compute.".to_string(),
            };
        }
        let result = rate_nper.ln() / (1.0 + rate).ln();
        CalcResult::Number(result)
    }

    // FV(rate, nper, pmt, [pv], [type])
    pub(crate) fn fn_fv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // number of periods
        let nper = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // payment
        let pmt = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // present value
        let pv = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        match compute_future_value(rate, nper, pmt, pv, period_start) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // IPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // per
        let period = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };
        let ipmt = match compute_ipmt(
            rate,
            period,
            period_count,
            present_value,
            future_value,
            period_start,
        ) {
            Ok(f) => f,
            Err(error) => {
                return CalcResult::Error {
                    error: error.0,
                    origin: cell,
                    message: error.1,
                }
            }
        };
        CalcResult::Number(ipmt)
    }

    // PPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ppmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // per
        let period = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // nper
        let period_count = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // pv
        let present_value = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // fv
        let future_value = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        let period_start = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            // at the end of the period
            false
        };

        let ppmt = match compute_ppmt(
            rate,
            period,
            period_count,
            present_value,
            future_value,
            period_start,
        ) {
            Ok(f) => f,
            Err(error) => {
                return CalcResult::Error {
                    error: error.0,
                    origin: cell,
                    message: error.1,
                }
            }
        };
        CalcResult::Number(ppmt)
    }

    // NPV(rate, value1, [value2],...)
    // npv = Sum[value[i]/(1+rate)^i, {i, 1, n}]
    pub(crate) fn fn_npv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut values = Vec::new();
        for arg in &args[1..] {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let row1 = left.row;
                    let mut row2 = right.row;
                    let column1 = left.column;
                    let mut column2 = right.column;
                    if row1 == 1 && row2 == LAST_ROW {
                        row2 = match self.workbook.worksheet(left.sheet) {
                            Ok(s) => s.dimension().max_row,
                            Err(_) => {
                                return CalcResult::new_error(
                                    Error::ERROR,
                                    cell,
                                    format!("Invalid worksheet index: '{}'", left.sheet),
                                );
                            }
                        };
                    }
                    if column1 == 1 && column2 == LAST_COLUMN {
                        column2 = match self.workbook.worksheet(left.sheet) {
                            Ok(s) => s.dimension().max_column,
                            Err(_) => {
                                return CalcResult::new_error(
                                    Error::ERROR,
                                    cell,
                                    format!("Invalid worksheet index: '{}'", left.sheet),
                                );
                            }
                        };
                    }
                    for row in row1..row2 + 1 {
                        for column in column1..(column2 + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    values.push(value);
                                }
                                error @ CalcResult::Error { .. } => return error,
                                _ => {
                                    // We ignore booleans and strings
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // We ignore booleans and strings
                }
            };
        }
        match compute_npv(rate, &values) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::new_error(error.0, cell, error.1),
        }
    }

    // Returns the internal rate of return for a series of cash flows represented by the numbers
    // in values.
    // These cash flows do not have to be even, as they would be for an annuity.
    // However, the cash flows must occur at regular intervals, such as monthly or annually.
    // The internal rate of return is the interest rate received for an investment consisting
    // of payments (negative values) and income (positive values) that occur at regular periods

    // IRR(values, [guess])
    pub(crate) fn fn_irr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 2 || arg_count == 0 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        match compute_irr(&values, guess) {
            Ok(f) => CalcResult::Number(f),
            Err(error) => CalcResult::Error {
                error: error.0,
                origin: cell,
                message: error.1,
            },
        }
    }

    // XNPV(rate, values, dates)
    pub(crate) fn fn_xnpv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let values = match self.get_array_of_numbers_xpnv(&args[1], &cell, Error::NUM) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xpnv(&args[2], &cell, Error::VALUE) {
            Ok(s) => s,
            Err(error) => return error,
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        let first_date = dates[0];
        for date in &dates {
            if *date < MINIMUM_DATE_SERIAL_NUMBER as f64
                || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            {
                // Excel docs claim that if any number in dates is not a valid date,
                // XNPV returns the #VALUE! error value, but it seems to return #VALUE!
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid number for date".to_string(),
                );
            }
            // If any number in dates precedes the starting date, XNPV returns the #NUM! error value.
            if date < &first_date {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Date precedes the starting date".to_string(),
                );
            }
        }
        // It seems Excel returns #NUM! if rate < 0, this is only necessary if r <= -1
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate needs to be > 0".to_string());
        }
        match compute_xnpv(rate, &values, &dates) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::new_error(error, cell, message),
        }
    }

    // XIRR(values, dates, [guess])
    pub(crate) fn fn_xirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_xirr(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let dates = match self.get_array_of_numbers_xirr(&args[1], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let guess = if arg_count == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.1
        };
        // Decimal points on dates are truncated
        let dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
        let values_count = values.len();
        // If values and dates contain a different number of values, XNPV returns the #NUM! error value.
        if values_count != dates.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Values and dates must be the same length".to_string(),
            );
        }
        if values_count == 0 {
            return CalcResult::new_error(Error::NUM, cell, "Not enough values".to_string());
        }
        let first_date = dates[0];
        for date in &dates {
            if *date < MINIMUM_DATE_SERIAL_NUMBER as f64
                || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64
            {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid number for date".to_string(),
                );
            }
            // If any number in dates precedes the starting date, XIRR returns the #NUM! error value.
            if date < &first_date {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Date precedes the starting date".to_string(),
                );
            }
        }
        match compute_xirr(&values, &dates, guess) {
            Ok(f) => CalcResult::Number(f),
            Err((error, message)) => CalcResult::Error {
                error,
                origin: cell,
                message,
            },
        }
    }

    //  MIRR(values, finance_rate, reinvest_rate)
    // The formula is:
    // $$ (-NPV(r1, v_p) * (1+r1)^y)/(NPV(r2, v_n)*(1+r2))^(1/y)-1$$
    // where:
    // $r1$ is the reinvest_rate, $r2$ the finance_rate
    // $v_p$ the vector of positive values
    // $v_n$ the vector of negative values
    // and $y$ is dimension of $v$ - 1 (number of years)
    pub(crate) fn fn_mirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let finance_rate = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let reinvest_rate = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut positive_values = Vec::new();
        let mut negative_values = Vec::new();
        let mut last_negative_index = -1;
        for (index, &value) in values.iter().enumerate() {
            let (p, n) = if value >= 0.0 {
                (value, 0.0)
            } else {
                last_negative_index = index as i32;
                (0.0, value)
            };
            positive_values.push(p);
            negative_values.push(n);
        }
        if last_negative_index == -1 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Invalid data for MIRR function".to_string(),
            );
        }
        // We do a bit of analysis if the rates are -1 as there are some cancellations
        // It is probably not important.
        let years = values.len() as f64;
        let top = if reinvest_rate == -1.0 {
            // This is finite
            match positive_values.last() {
                Some(f) => *f,
                None => 0.0,
            }
        } else {
            match compute_npv(reinvest_rate, &positive_values) {
                Ok(npv) => -npv * ((1.0 + reinvest_rate).powf(years)),
                Err((error, message)) => {
                    return CalcResult::Error {
                        error,
                        origin: cell,
                        message,
                    }
                }
            }
        };
        let bottom = if finance_rate == -1.0 {
            if last_negative_index == 0 {
                // This is still finite
                negative_values[last_negative_index as usize]
            } else {
                // or -Infinity depending of the sign in the last_negative_index coef.
                // But it is irrelevant for the calculation
                f64::INFINITY
            }
        } else {
            match compute_npv(finance_rate, &negative_values) {
                Ok(npv) => npv * (1.0 + finance_rate),
                Err((error, message)) => {
                    return CalcResult::Error {
                        error,
                        origin: cell,
                        message,
                    }
                }
            }
        };

        let result = (top / bottom).powf(1.0 / (years - 1.0)) - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for MIRR".to_string());
        }
        CalcResult::Number(result)
    }

    // ISPMT(rate, per, nper, pv)
    // Formula is:
    // $$pv*rate*\left(\frac{per}{nper}-1\right)$$
    pub(crate) fn fn_ispmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let per = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if nper == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        CalcResult::Number(pv * rate * (per / nper - 1.0))
    }

    // RRI(nper, pv, fv)
    // Formula is
    // $$ \left(\frac{fv}{pv}\right)^{\frac{1}{nper}}-1  $$
    pub(crate) fn fn_rri(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let nper = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if nper <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "nper should be >0".to_string());
        }
        if pv == 0.0 {
            // Note error is NUM not DIV/0 also bellow
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        let result = (fv / pv).powf(1.0 / nper) - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // SLN(cost, salvage, life)
    // Formula is:
    // $$ \frac{cost-salvage}{life} $$
    pub(crate) fn fn_sln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if life == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        let result = (cost - salvage) / life;

        CalcResult::Number(result)
    }

    // SYD(cost, salvage, life, per)
    // Formula is:
    // $$ \frac{(cost-salvage)*(life-per+1)*2}{life*(life+1)} $$
    pub(crate) fn fn_syd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let per = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if life == 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Division by 0".to_string());
        }
        if per > life || per <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "per should be <= life".to_string());
        }
        let result = ((cost - salvage) * (life - per + 1.0) * 2.0) / (life * (life + 1.0));

        CalcResult::Number(result)
    }

    // NOMINAL(effective_rate, npery)
    // Formula is:
    // $$ n\times\left(\left(1+r\right)^{\frac{1}{n}}-1\right) $$
    // where:
    //   $r$ is the effective interest rate
    //   $n$ is the number of periods per year
    pub(crate) fn fn_nominal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let effect_rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let npery = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if effect_rate <= 0.0 || npery < 1.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = ((1.0 + effect_rate).powf(1.0 / npery) - 1.0) * npery;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // EFFECT(nominal_rate, npery)
    // Formula is:
    // $$ \left(1+\frac{r}{n}\right)^n-1 $$
    // where:
    //   $r$ is the nominal interest rate
    //   $n$ is the number of periods per year
    pub(crate) fn fn_effect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let nominal_rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let npery = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if nominal_rate <= 0.0 || npery < 1.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = (1.0 + nominal_rate / npery).powf(npery) - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // PDURATION(rate, pv, fv)
    // Formula is:
    // $$ \frac{log(fv) - log(pv)}{log(1+r)} $$
    // where:
    //   * $r$ is the interest rate per period
    //   * $pv$ is the present value of the investment
    //   * $fv$ is the desired future value of the investment
    pub(crate) fn fn_pduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let fv = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fv <= 0.0 || pv <= 0.0 || rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let result = (fv.ln() - pv.ln()) / ((1.0 + rate).ln());
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // This next three functions deal with Treasure Bills or T-Bills for short
    // They are zero-coupon that mature in one year or less.
    //  Definitions:
    //    $r$ be the discount rate
    //    $v$ the face value of the Bill
    //    $p$ the price of the Bill
    //    $d_m$ is the number of days from the settlement to maturity
    // Then:
    //   $$ p = v \times\left(1-\frac{d_m}{r}\right) $$
    // If d_m is less than 183 days the he Bond Equivalent Yield (BEY, here $y$) is given by:
    // $$ y = \frac{F - B}{M}\times \frac{365}{d_m} = \frac{365\times r}{360-r\times d_m}
    // If d_m>= 183 days things are a bit more complicated.
    // Let $d_e = d_m - 365/2$ if $d_m <= 365$ or $d_e = 183$ if $d_m = 366$.
    // $$ v = p\times \left(1+\frac{y}{2}\right)\left(1+d_e\times\frac{y}{365}\right) $$
    // Together with the previous relation of $p$ and $v$ gives us a quadratic equation for $y$.

    // TBILLEQ(settlement, maturity, discount)
    pub(crate) fn fn_tbilleq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let less_than_one_year = match is_less_than_one_year(settlement as i64, maturity as i64) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        if settlement > maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement should be <= maturity".to_string(),
            );
        }
        if !less_than_one_year {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "maturity <= settlement + year".to_string(),
            );
        }
        if discount <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "discount should be >0".to_string());
        }
        // days to maturity
        let d_m = maturity - settlement;
        let result = if d_m < 183.0 {
            365.0 * discount / (360.0 - discount * d_m)
        } else {
            // Equation here is:
            // (1-days*rate/360)*(1+y/2)*(1+d_extra*y/year)=1
            let year = if d_m == 366.0 { 366.0 } else { 365.0 };
            let d_extra = d_m - year / 2.0;
            let alpha = 1.0 - d_m * discount / 360.0;
            let beta = 0.5 + d_extra / year;
            // ay^2+by+c=0
            let a = d_extra * alpha / (year * 2.0);
            let b = alpha * beta;
            let c = alpha - 1.0;
            (-b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
        };
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // TBILLPRICE(settlement, maturity, discount)
    pub(crate) fn fn_tbillprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let less_than_one_year = match is_less_than_one_year(settlement as i64, maturity as i64) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        if settlement > maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement should be <= maturity".to_string(),
            );
        }
        if !less_than_one_year {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "maturity <= settlement + year".to_string(),
            );
        }
        if discount <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "discount should be >0".to_string());
        }
        // days to maturity
        let d_m = maturity - settlement;
        let result = 100.0 * (1.0 - discount * d_m / 360.0);
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() || result < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        CalcResult::Number(result)
    }

    // TBILLYIELD(settlement, maturity, pr)
    pub(crate) fn fn_tbillyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pr = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let less_than_one_year = match is_less_than_one_year(settlement as i64, maturity as i64) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        if settlement > maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement should be <= maturity".to_string(),
            );
        }
        if !less_than_one_year {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "maturity <= settlement + year".to_string(),
            );
        }
        if pr <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "discount should be >0".to_string());
        }
        let days = maturity - settlement;
        let result = (100.0 - pr) * 360.0 / (pr * days);

        CalcResult::Number(result)
    }

    // DOLLARDE(fractional_dollar, fraction)
    pub(crate) fn fn_dollarde(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let fractional_dollar = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut fraction = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fraction < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "fraction should be >= 1".to_string());
        }
        if fraction < 1.0 {
            // this is not necessarily DIV/0
            return CalcResult::new_error(Error::DIV, cell, "fraction should be >= 1".to_string());
        }
        fraction = fraction.trunc();
        while fraction > 10.0 {
            fraction /= 10.0;
        }
        let t = fractional_dollar.trunc();
        let result = t + (fractional_dollar - t) * 10.0 / fraction;
        CalcResult::Number(result)
    }

    // DOLLARFR(decimal_dollar, fraction)
    pub(crate) fn fn_dollarfr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let decimal_dollar = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let mut fraction = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if fraction < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "fraction should be >= 1".to_string());
        }
        if fraction < 1.0 {
            // this is not necessarily DIV/0
            return CalcResult::new_error(Error::DIV, cell, "fraction should be >= 1".to_string());
        }
        fraction = fraction.trunc();
        while fraction > 10.0 {
            fraction /= 10.0;
        }
        let t = decimal_dollar.trunc();
        let result = t + (decimal_dollar - t) * fraction / 10.0;
        CalcResult::Number(result)
    }

    // CUMIPMT(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ipmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // CUMPRINC(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumprinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let rate = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let nper = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let pv = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.ceil() as i32,
            Err(s) => return s,
        };
        let end_period = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        // 0 at the end of the period, 1 at the beginning of the period
        let period_type = match self.get_number_no_bools(&args[5], cell) {
            Ok(f) => {
                if f == 0.0 {
                    false
                } else if f == 1.0 {
                    true
                } else {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "invalid period type".to_string(),
                    );
                }
            }
            Err(s) => return s,
        };
        if start_period > end_period {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "start period should come before end period".to_string(),
            );
        }
        if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let mut result = 0.0;
        for period in start_period..=end_period {
            result += match compute_ppmt(rate, period as f64, nper, pv, 0.0, period_type) {
                Ok(f) => f,
                Err(error) => {
                    return CalcResult::Error {
                        error: error.0,
                        origin: cell,
                        message: error.1,
                    }
                }
            }
        }
        CalcResult::Number(result)
    }

    // DDB(cost, salvage, life, period, [factor])
    pub(crate) fn fn_ddb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        // The rate at which the balance declines.
        let factor = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            // If factor is omitted, it is assumed to be 2 (the double-declining balance method).
            2.0
        };
        if period > life || cost < 0.0 || salvage < 0.0 || period <= 0.0 || factor <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        };
        // let period_trunc = period.floor() as i32;
        let mut rate = factor / life;
        if rate > 1.0 {
            rate = 1.0
        };
        let value = if rate == 1.0 {
            if period == 1.0 {
                cost
            } else {
                0.0
            }
        } else {
            cost * (1.0 - rate).powf(period - 1.0)
        };
        let new_value = cost * (1.0 - rate).powf(period);
        let result = f64::max(value - f64::max(salvage, new_value), 0.0);
        CalcResult::Number(result)
    }

    // DB(cost, salvage, life, period, [month])
    pub(crate) fn fn_db(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let month = if arg_count > 4 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f.trunc(),
                Err(s) => return s,
            }
        } else {
            12.0
        };
        if month == 12.0 && period > life
            || (period > life + 1.0)
            || month <= 0.0
            || month > 12.0
            || period <= 0.0
            || cost < 0.0
        {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        };
        if cost == 0.0 {
            return CalcResult::Number(0.0);
        }
        // rounded to three decimal places
        // FIXME: We should have utilities for this (see to_precision)
        let rate = f64::round((1.0 - f64::powf(salvage / cost, 1.0 / life)) * 1000.0) / 1000.0;

        let mut result = cost * rate * month / 12.0;

        let period = period.floor() as i32;
        let life = life.floor() as i32;

        // Depreciation for the first and last periods is a special case.
        if period == 1 {
            return CalcResult::Number(result);
        };

        for _ in 0..period - 2 {
            result += (cost - result) * rate;
        }

        if period == life + 1 {
            // last period
            return CalcResult::Number((cost - result) * rate * (12.0 - month) / 12.0);
        }

        CalcResult::Number(rate * (cost - result))
    }

    // ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])
    //
    // Canonical specification: Mayle, *Standard Securities Calculation Methods*
    // (SIA / SIFMA), generalized by the Microsoft DAX `ACCRINT` documentation:
    //
    //   AI = par * (rate / frequency) * Σᵢ (Aᵢ / NLᵢ),  i = 1..NC
    //
    // where the sum runs over the NC quasi-coupon periods spanned by the
    // accrual interval. For each period i, `Aᵢ` is the number of accrued
    // days that fall inside the period under the given day-count basis,
    // and `NLᵢ` is the normal length of the period under the same basis.
    //
    // The quasi-coupon period boundaries are derived by walking backward
    // from `first_interest` (= Mayle's `CPNDT_1`, the first coupon date
    // after settlement) at intervals of 12/frequency months, snapped to
    // the end of the month when first_interest is itself an end-of-month
    // date. The walk stops once the period containing the accrual start
    // is reached.
    //
    // `calc_method` selects the accrual start:
    //   - TRUE  (default): accrue from `issue` to `settlement`
    //   - FALSE          : accrue from `first_interest` to `settlement`
    //
    // Boundary case: when `settlement` coincides with a coupon boundary
    // (issue = settlement, or accrual reduces to zero days), return 0.
    // The `issue >= settlement` case still errors per the MS spec.
    pub(crate) fn fn_accrint(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(6..=8).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let issue_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let first_interest_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let settlement_serial = match self.get_number(&args[2], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let par = match self.get_number(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let frequency = match self.get_number(&args[5], cell) {
            Ok(f) => f.round() as i32,
            Err(s) => return s,
        };
        let basis = if arg_count > 6 {
            match self.get_number(&args[6], cell) {
                Ok(f) => f.round() as i32,
                Err(s) => return s,
            }
        } else {
            0
        };
        let calc_method = if arg_count > 7 {
            match self.get_boolean(&args[7], cell) {
                Ok(b) => b,
                Err(s) => return s,
            }
        } else {
            true
        };
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate must be > 0".to_string());
        }
        if par <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par must be > 0".to_string());
        }
        if !matches!(frequency, 1 | 2 | 4) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "frequency must be 1, 2, or 4".to_string(),
            );
        }
        if !(0..=4).contains(&basis) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "basis must be between 0 and 4".to_string(),
            );
        }
        if issue_serial >= settlement_serial {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }

        // Accrual start: TRUE uses issue, FALSE conceptually uses
        // first_interest. The empirical rule that matches both the DAX
        // canonical worked example and the upstream smoke tests is:
        //
        //   AI_FALSE = AI_TRUE - (one full quasi-coupon period contribution)
        //
        // Equivalently, FALSE drops the first period from the multi-period
        // sum. When TRUE's accrual already covers less than one full
        // period (short-span case), FALSE falls back to TRUE.
        //
        // The accrue_start (start of the multi-period walk) is always
        // `issue` here; the FALSE-specific trim happens after period
        // construction below.
        let accrue_start = issue_serial;

        // Walk backward from first_interest at 12/frequency-month intervals,
        // building the list of quasi-coupon periods until we cover
        // accrue_start. Each entry is (period_start, period_end).
        let months_per_period = 12 / frequency;
        let mut period_ends = vec![first_interest_serial];
        let mut step = 1i32;
        loop {
            let prev_end = match add_months_eom(first_interest_serial, -months_per_period * step) {
                Ok(s) => s,
                Err(e) => return CalcResult::new_error(Error::NUM, cell, e),
            };
            period_ends.push(prev_end);
            if prev_end <= accrue_start {
                break;
            }
            step += 1;
            if step > 1200 {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "ACCRINT: too many quasi-coupon periods".to_string(),
                );
            }
        }
        // period_ends is reverse-ordered: [first_interest, prev1, prev2, ...].
        // Convert to forward-ordered (period_start, period_end) pairs.
        period_ends.reverse();
        let mut periods: Vec<(i64, i64)> = Vec::with_capacity(period_ends.len() - 1);
        for window in period_ends.windows(2) {
            periods.push((window[0], window[1]));
        }

        // calc_method=FALSE accrues from first_interest to settlement instead
        // of from issue, by dropping the first (issue-side) quasi-coupon period
        // from the sum. This only changes the result when settlement lies
        // beyond the first period's end, so that the remaining accrual is
        // non-empty; otherwise the whole accrual is in the first period and
        // FALSE behaves identically to TRUE (verified against the DAX
        // canonical FALSE = 66.944 and the short-span case = TRUE).
        if !calc_method && periods.len() > 1 && settlement_serial > periods[0].1 {
            periods.remove(0);
        }

        // NLᵢ = the normal length of the quasi-coupon period (DAX spec),
        // NOT the basis day-count of the period. For 30/360 and Actual/360
        // bases this is a fixed nominal length (360/frequency); for
        // Actual/365 it is 365/frequency; for Actual/Actual it is the
        // period's actual day count (Mayle per-security rule).
        let period_nl = |period_start: i64, period_end: i64| -> f64 {
            match basis {
                0 | 2 | 4 => 360.0 / frequency as f64,
                3 => 365.0 / frequency as f64,
                1 => (period_end - period_start) as f64,
                _ => 360.0 / frequency as f64,
            }
        };

        // Odd-long first coupon (NC ≥ 2): Excel normalizes the front-stub
        // accrual by the regular coupon length, NL_ref = the NL of the LAST
        // quasi-coupon period (the full regular coupon ending at
        // first_interest), and front-loads the interior periods'
        // actual-vs-nominal drift as a constant offset subtracted from the
        // settlement (last contributing) period's numerator. This affects only
        // the Actual bases (1, 2, 3); 30/360 bases are unaffected (their plain
        // Σ Aᵢ/NLᵢ already matches Excel). Verified against Excel for
        // frequencies 2 and 4 across all bases.
        let nl_ref = match periods.last() {
            Some(&(ps, pe)) => period_nl(ps, pe),
            None => 360.0 / frequency as f64,
        };
        let mut offset = 0.0f64;
        if matches!(basis, 1..=3) {
            for &(ps, pe) in &periods {
                // Interior periods only: exclude the issue-stub period (issue
                // strictly inside it) and the last/regular period (ending at
                // first_interest). Each contributes its actual length minus the
                // regular coupon length NL_ref.
                if pe == first_interest_serial || ps < accrue_start {
                    continue;
                }
                offset += (pe - ps) as f64 - nl_ref;
            }
        }

        // Accumulate Σ (Aᵢ / NLᵢ) over the period list, intersecting each
        // period with the accrual interval [accrue_start, settlement].
        let mut sum = 0.0f64;
        for (period_start, period_end) in periods {
            let a_start = period_start.max(accrue_start);
            let a_end = period_end.min(settlement_serial);
            if a_end <= a_start {
                continue;
            }
            let a_days = match self.day_count_basis(a_start, a_end, basis, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            // The settlement-containing slice (a_end == settlement) of an
            // odd-long coupon takes the offset and is normalized by NL_ref;
            // every earlier slice uses its own period's normal length. For a
            // normal first coupon (single period / NC = 1) offset is 0 and
            // NL_ref equals the period's own NL, so this reduces to Σ Aᵢ/NLᵢ.
            let (numerator, nl_days) = if a_end == settlement_serial {
                (a_days - offset, nl_ref)
            } else {
                (a_days, period_nl(period_start, period_end))
            };
            if nl_days <= 0.0 {
                continue;
            }
            sum += numerator / nl_days;
        }

        if sum == 0.0 {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(par * (rate / frequency as f64) * sum)
    }

    // Accrued days (the Aᵢ numerator) under the given basis for the
    // interval [start, end]: 30/360 for bases 0 and 4, actual days for
    // bases 1, 2, and 3. Implemented by reusing fn_yearfrac and recovering
    // days via the basis-specific year length (yearfrac × 360 for 30/360
    // bases collapses to the 30/360 day count; yearfrac × {360, 365} for
    // Actual bases collapses to the actual day count). The quasi-coupon
    // period length NLᵢ is computed separately in fn_accrint.
    fn day_count_basis(
        &mut self,
        start: i64,
        end: i64,
        basis: i32,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let yf_args = [
            Node::NumberKind(start as f64),
            Node::NumberKind(end as f64),
            Node::NumberKind(basis as f64),
        ];
        match self.fn_yearfrac(&yf_args, cell) {
            CalcResult::Number(yf) => {
                let year_len = match basis {
                    0 | 2 | 4 => 360.0,
                    3 => 365.0,
                    1 => {
                        // Actual day count, recoverable as (end - start).
                        return Ok((end - start) as f64);
                    }
                    _ => 360.0,
                };
                Ok(yf * year_len)
            }
            error => Err(error),
        }
    }

    // ACCRINTM(issue, settlement, rate, [par], [basis])
    pub(crate) fn fn_accrintm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(3..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let issue_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let settlement_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let par = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            1000.0
        };
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate must be > 0".to_string());
        }
        if par <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par must be > 0".to_string());
        }
        if issue_serial >= settlement_serial {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "issue must be before settlement".to_string(),
            );
        }
        let basis_node = if arg_count > 4 {
            args[4].clone()
        } else {
            Node::NumberKind(0.0)
        };
        let yearfrac_args = [
            Node::NumberKind(issue_serial as f64),
            Node::NumberKind(settlement_serial as f64),
            basis_node,
        ];
        match self.fn_yearfrac(&yearfrac_args, cell) {
            CalcResult::Number(yf) => CalcResult::Number(par * rate * yf),
            error => error,
        }
    }

    fn get_yearfrac(
        &mut self,
        start: i64,
        end: i64,
        basis: f64,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let args = [
            Node::NumberKind(start as f64),
            Node::NumberKind(end as f64),
            Node::NumberKind(basis),
        ];
        match self.fn_yearfrac(&args, cell) {
            CalcResult::Number(yf) => Ok(yf),
            error => Err(error),
        }
    }

    // DISC(settlement, maturity, pr, redemption, [basis])
    pub(crate) fn fn_disc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let pr = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "pr and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((1.0 - pr / redemption) / yf)
    }

    // FVSCHEDULE(principal, schedule)
    pub(crate) fn fn_fvschedule(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let principal = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let schedule = match self.get_array_of_numbers(&args[1], &cell) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let mut result = principal;
        for rate in schedule {
            result *= 1.0 + rate;
        }
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(result)
    }

    // INTRATE(settlement, maturity, investment, redemption, [basis])
    pub(crate) fn fn_intrate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let investment = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if investment <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "investment and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((redemption / investment - 1.0) / yf)
    }

    // PRICEDISC(settlement, maturity, discount, redemption, [basis])
    pub(crate) fn fn_pricedisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if discount <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "discount and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        CalcResult::Number(redemption * (1.0 - discount * yf))
    }

    // PRICEMAT(settlement, maturity, issue, rate, yld, [basis])
    pub(crate) fn fn_pricemat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let yld = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if rate < 0.0 || yld < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "rate and yld must be >= 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let dim = match self.get_yearfrac(issue, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let dis = match self.get_yearfrac(issue, settlement, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let dsm = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let denom = 1.0 + yld * dsm;
        if denom == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(100.0 * ((1.0 + rate * dim) / denom - rate * dis))
    }

    // RECEIVED(settlement, maturity, investment, discount, [basis])
    pub(crate) fn fn_received(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let investment = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let discount = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if investment <= 0.0 || discount <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "investment and discount must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let denom = 1.0 - discount * yf;
        if denom <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "discount * yearfrac >= 1".to_string());
        }
        CalcResult::Number(investment / denom)
    }

    // YIELDDISC(settlement, maturity, pr, redemption, [basis])
    pub(crate) fn fn_yielddisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let pr = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let redemption = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 5 {
            match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if pr <= 0.0 || redemption <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "pr and redemption must be > 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let yf = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        if yf == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number((redemption / pr - 1.0) / yf)
    }

    // YIELDMAT(settlement, maturity, issue, rate, price, [basis])
    pub(crate) fn fn_yieldmat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let settlement = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let maturity = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let issue = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor() as i64,
            Err(s) => return s,
        };
        let rate = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let price = match self.get_number_no_bools(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if price <= 0.0 || rate < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "price must be > 0 and rate >= 0".to_string(),
            );
        }
        if settlement >= maturity {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement must be before maturity".to_string(),
            );
        }
        let dim = match self.get_yearfrac(issue, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let dis = match self.get_yearfrac(issue, settlement, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let dsm = match self.get_yearfrac(settlement, maturity, basis, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let denom = price / 100.0 + rate * dis;
        if denom == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        if dsm == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(((1.0 + rate * dim) / denom - 1.0) / dsm)
    }
}
