use chrono::Datelike;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    model::Model,
};

use super::financial_util::{compute_irr, compute_npv, compute_rate, compute_xirr, compute_xnpv};

// Financial calculation constants
const DAYS_IN_YEAR_360: i32 = 360;
const DAYS_ACTUAL: i32 = 365;
const DAYS_LEAP_YEAR: i32 = 366;
const DAYS_IN_MONTH_360: i32 = 30;
const TBILL_MATURITY_THRESHOLD: f64 = 183.0;

// See:
// https://github.com/apache/openoffice/blob/c014b5f2b55cff8d4b0c952d5c16d62ecde09ca1/main/scaddins/source/analysis/financial.cxx

fn is_less_than_one_year(start_date: i64, end_date: i64) -> Result<bool, String> {
    let end = from_excel_date(end_date)?;
    let start = from_excel_date(start_date)?;
    if end_date - start_date < DAYS_ACTUAL as i64 {
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

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

fn is_last_day_of_february(date: chrono::NaiveDate) -> bool {
    date.month() == 2 && date.day() == if is_leap_year(date.year()) { 29 } else { 28 }
}

fn days360_us(start: chrono::NaiveDate, end: chrono::NaiveDate) -> i32 {
    let mut d1 = start.day() as i32;
    let mut d2 = end.day() as i32;
    let m1 = start.month() as i32;
    let m2 = end.month() as i32;
    let y1 = start.year();
    let y2 = end.year();

    // US (NASD) 30/360 method - implementing official specification

    // Rule 1: If both date A and B fall on the last day of February, then date B will be changed to the 30th
    if is_last_day_of_february(start) && is_last_day_of_february(end) {
        d2 = DAYS_IN_MONTH_360;
    }

    // Rule 2: If date A falls on the 31st of a month or last day of February, then date A will be changed to the 30th
    if d1 == 31 || is_last_day_of_february(start) {
        d1 = DAYS_IN_MONTH_360;
    }

    // Rule 3: If date A falls on the 30th after applying rule 2 and date B falls on the 31st, then date B will be changed to the 30th
    if d1 == DAYS_IN_MONTH_360 && d2 == 31 {
        d2 = DAYS_IN_MONTH_360;
    }

    DAYS_IN_YEAR_360 * (y2 - y1) + DAYS_IN_MONTH_360 * (m2 - m1) + (d2 - d1)
}

fn days360_eu(start: chrono::NaiveDate, end: chrono::NaiveDate) -> i32 {
    let mut d1 = start.day() as i32;
    let mut d2 = end.day() as i32;
    let m1 = start.month() as i32;
    let m2 = end.month() as i32;
    let y1 = start.year();
    let y2 = end.year();

    if d1 == 31 {
        d1 = DAYS_IN_MONTH_360;
    }
    if d2 == 31 {
        d2 = DAYS_IN_MONTH_360;
    }

    d2 + m2 * DAYS_IN_MONTH_360 + y2 * DAYS_IN_YEAR_360
        - d1
        - m1 * DAYS_IN_MONTH_360
        - y1 * DAYS_IN_YEAR_360
}

fn days_in_year(date: chrono::NaiveDate, basis: i32) -> Result<i32, String> {
    Ok(match basis {
        0 | 2 | 4 => DAYS_IN_YEAR_360,
        1 => {
            if is_leap_year(date.year()) {
                DAYS_LEAP_YEAR
            } else {
                DAYS_ACTUAL
            }
        }
        3 => DAYS_ACTUAL,
        _ => return Err("invalid basis".to_string()),
    })
}

/// Returns days in year for financial calculations (simplified version without leap year checking)
fn days_in_year_simple(basis: i32) -> f64 {
    match basis {
        0 | 2 | 4 => DAYS_IN_YEAR_360 as f64,
        1 | 3 => DAYS_ACTUAL as f64,
        _ => DAYS_IN_YEAR_360 as f64,
    }
}

/// Macro to reduce duplication in financial functions that follow the pattern:
/// 1. Parse settlement/maturity and two parameters with validation
/// 2. Calculate year fraction
/// 3. Apply formula and return result
macro_rules! financial_function_with_year_frac {
    (
        $args:ident, $self:ident, $cell:ident,
        param1_name: $param1_name:literal,
        param2_name: $param2_name:literal,
        validator: $validator:expr,
        formula: |$settlement:ident, $maturity:ident, $param1:ident, $param2:ident, $basis:ident, $year_frac:ident| $formula:expr
    ) => {{
        // Parse and validate arguments
        let arg_count = $args.len();
        if !(4..=5).contains(&arg_count) {
            return CalcResult::new_args_number_error($cell);
        }

        let ($settlement, $maturity) =
            match parse_and_validate_settlement_maturity($args, $self, $cell, true) {
                Ok(sm) => sm,
                Err(err) => return err,
            };
        let $param1 = match $self.get_number_no_bools(&$args[2], $cell) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let $param2 = match $self.get_number_no_bools(&$args[3], $cell) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let $basis = match parse_optional_basis($args, 4, arg_count, $self, $cell) {
            Ok(b) => b,
            Err(err) => return err,
        };

        // Apply custom validation
        if let Err(msg) = ($validator)($param1, $param2) {
            return CalcResult::new_error(
                Error::NUM,
                $cell,
                format!("{} and {}: {}", $param1_name, $param2_name, msg),
            );
        }

        let $year_frac = match year_frac($settlement as i64, $maturity as i64, $basis) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, $cell, "Invalid date".to_string()),
        };

        let result = $formula;
        CalcResult::Number(result)
    }};
}

/// Helper function to convert date serial number to chrono date with error handling
fn convert_date_serial(
    date_serial: f64,
    cell: CellReferenceIndex,
) -> Result<chrono::NaiveDate, CalcResult> {
    match from_excel_date(date_serial as i64) {
        Ok(date) => Ok(date),
        Err(_) => Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "Invalid date".to_string(),
        )),
    }
}

/// Helper function to parse optional basis parameter (defaults to 0)
fn parse_optional_basis(
    args: &[Node],
    basis_arg_index: usize,
    arg_count: usize,
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<i32, CalcResult> {
    if arg_count > basis_arg_index {
        match model.get_number_no_bools(&args[basis_arg_index], cell) {
            Ok(f) => Ok(f.trunc() as i32),
            Err(s) => Err(s),
        }
    } else {
        Ok(0)
    }
}

/// Enhanced helper function to parse, validate settlement/maturity with optional date range validation
fn parse_and_validate_settlement_maturity(
    args: &[Node],
    model: &mut Model,
    cell: CellReferenceIndex,
    check_date_range: bool,
) -> Result<(f64, f64), CalcResult> {
    // Parse settlement and maturity
    let settlement = model.get_number_no_bools(&args[0], cell)?;
    let maturity = model.get_number_no_bools(&args[1], cell)?;

    // Validate settlement < maturity
    if settlement >= maturity {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "settlement should be < maturity".to_string(),
        ));
    }

    // Optionally validate date ranges
    if check_date_range {
        if settlement < MINIMUM_DATE_SERIAL_NUMBER as f64
            || settlement > MAXIMUM_DATE_SERIAL_NUMBER as f64
        {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid number for date".to_string(),
            ));
        }
        if maturity < MINIMUM_DATE_SERIAL_NUMBER as f64
            || maturity > MAXIMUM_DATE_SERIAL_NUMBER as f64
        {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid number for date".to_string(),
            ));
        }
    }

    Ok((settlement, maturity))
}

/// Helper function to parse multiple required numeric parameters efficiently
/// Returns a vector of parsed values in the same order as the indices
fn parse_required_params(
    args: &[Node],
    indices: &[usize],
    model: &mut Model,
    cell: CellReferenceIndex,
    use_no_bools: bool,
) -> Result<Vec<f64>, CalcResult> {
    let mut params = Vec::with_capacity(indices.len());
    for &index in indices {
        let value = if use_no_bools {
            model.get_number_no_bools(&args[index], cell)?
        } else {
            model.get_number(&args[index], cell)?
        };
        params.push(value);
    }
    Ok(params)
}

/// Helper function to validate argument count and return early if invalid
fn validate_arg_count_or_return(
    arg_count: usize,
    min: usize,
    max: usize,
    cell: CellReferenceIndex,
) -> Option<CalcResult> {
    if !(min..=max).contains(&arg_count) {
        Some(CalcResult::new_args_number_error(cell))
    } else {
        None
    }
}

/// Helper function to convert date to serial number with consistent error handling
fn date_to_serial_with_validation(date: chrono::NaiveDate, cell: CellReferenceIndex) -> CalcResult {
    match crate::formatter::dates::date_to_serial_number(date.day(), date.month(), date.year()) {
        Ok(n) => {
            if !(MINIMUM_DATE_SERIAL_NUMBER..=MAXIMUM_DATE_SERIAL_NUMBER).contains(&n) {
                CalcResult::new_error(Error::NUM, cell, "date out of range".to_string())
            } else {
                CalcResult::Number(n as f64)
            }
        }
        Err(msg) => CalcResult::new_error(Error::NUM, cell, msg),
    }
}

/// Helper struct for common financial function optional parameters
struct FinancialOptionalParams {
    pub optional_value: f64,
    pub period_start: bool,
}

/// Helper function to parse common optional financial parameters: [optional_value], [type]
/// optional_value defaults to 0.0, type defaults to false (end of period)
fn parse_financial_optional_params(
    args: &[Node],
    arg_count: usize,
    optional_value_index: usize,
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<FinancialOptionalParams, CalcResult> {
    let optional_value = if arg_count > optional_value_index {
        model.get_number(&args[optional_value_index], cell)?
    } else {
        0.0
    };

    let period_start = if arg_count > optional_value_index + 1 {
        model.get_number(&args[optional_value_index + 1], cell)? != 0.0
    } else {
        false // at the end of the period
    };

    Ok(FinancialOptionalParams {
        optional_value,
        period_start,
    })
}

/// Helper struct for validated coupon function parameters
struct ValidatedCouponParams {
    pub settlement_date: chrono::NaiveDate,
    pub maturity_date: chrono::NaiveDate,
    pub frequency: i32,
    pub basis: i32,
}

/// Helper function to validate T-Bill calculation results
fn validate_tbill_result(result: f64, cell: CellReferenceIndex) -> CalcResult {
    if result.is_infinite() {
        CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string())
    } else if result.is_nan() {
        CalcResult::new_error(
            Error::NUM,
            cell,
            "Invalid data for T-Bill calculation".to_string(),
        )
    } else {
        CalcResult::Number(result)
    }
}

/// Helper function to validate and normalize fraction parameter for DOLLARDE/DOLLARFR functions
fn validate_and_normalize_fraction(
    fraction: f64,
    cell: CellReferenceIndex,
) -> Result<f64, CalcResult> {
    if fraction < 0.0 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "fraction should be >= 1".to_string(),
        ));
    }
    if fraction < 1.0 {
        return Err(CalcResult::new_error(
            Error::DIV,
            cell,
            "fraction should be >= 1".to_string(),
        ));
    }

    let mut normalized_fraction = fraction.trunc();
    while normalized_fraction > 10.0 {
        normalized_fraction /= 10.0;
    }

    Ok(normalized_fraction)
}

/// Helper function to handle compute function errors consistently
fn handle_compute_error<T>(
    result: Result<T, (Error, String)>,
    cell: CellReferenceIndex,
) -> Result<T, CalcResult> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(CalcResult::Error {
            error: error.0,
            origin: cell,
            message: error.1,
        }),
    }
}

/// Helper function to validate values and dates arrays for XNPV/XIRR functions
fn validate_values_dates_arrays(
    values: &[f64],
    dates: &[f64],
    cell: CellReferenceIndex,
) -> Result<Vec<f64>, CalcResult> {
    // Decimal points on dates are truncated
    let normalized_dates: Vec<f64> = dates.iter().map(|s| s.floor()).collect();
    let values_count = values.len();

    // If values and dates contain a different number of values, return error
    if values_count != dates.len() {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "Values and dates must be the same length".to_string(),
        ));
    }

    if values_count == 0 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "Not enough values".to_string(),
        ));
    }

    let first_date = normalized_dates[0];
    for date in &normalized_dates {
        // Validate date range
        if *date < MINIMUM_DATE_SERIAL_NUMBER as f64 || *date > MAXIMUM_DATE_SERIAL_NUMBER as f64 {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid number for date".to_string(),
            ));
        }

        // If any date precedes the starting date, return error
        if date < &first_date {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Date precedes the starting date".to_string(),
            ));
        }
    }

    Ok(normalized_dates)
}

/// Parse and validate T-Bill parameters, returning (days_to_maturity, param_value)
fn parse_tbill_params(
    args: &[Node],
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<(f64, f64), CalcResult> {
    // Parse settlement, maturity, and third parameter
    let settlement = model.get_number_no_bools(&args[0], cell)?;
    let maturity = model.get_number_no_bools(&args[1], cell)?;
    let param_value = model.get_number_no_bools(&args[2], cell)?;

    // Validate settlement <= maturity
    if settlement > maturity {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "settlement should be <= maturity".to_string(),
        ));
    }

    // Validate less than one year
    let less_than_one_year = match is_less_than_one_year(settlement as i64, maturity as i64) {
        Ok(f) => f,
        Err(_) => {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid date".to_string(),
            ))
        }
    };
    if !less_than_one_year {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "maturity <= settlement + year".to_string(),
        ));
    }

    // Validate parameter > 0
    if param_value <= 0.0 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "parameter should be >0".to_string(),
        ));
    }

    let days_to_maturity = maturity - settlement;
    Ok((days_to_maturity, param_value))
}

/// Helper struct for validated bond pricing function parameters  
struct BondPricingParams {
    pub third_param: f64, // yld for PRICE, price for YIELD
    pub redemption: f64,
    pub frequency: i32,
    pub periods: f64,
    pub coupon: f64,
}

/// Helper function to parse and validate common bond pricing function parameters
/// Used by PRICE and YIELD functions
fn parse_and_validate_bond_pricing_params(
    args: &[Node],
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<BondPricingParams, CalcResult> {
    if !(6..=7).contains(&args.len()) {
        return Err(CalcResult::new_args_number_error(cell));
    }

    let settlement = model.get_number_no_bools(&args[0], cell)?;
    let maturity = model.get_number_no_bools(&args[1], cell)?;
    let rate = model.get_number_no_bools(&args[2], cell)?;
    let third_param = model.get_number_no_bools(&args[3], cell)?;
    let redemption = model.get_number_no_bools(&args[4], cell)?;

    let frequency = match model.get_number_no_bools(&args[5], cell) {
        Ok(f) => f.trunc() as i32,
        Err(s) => return Err(s),
    };

    if frequency != 1 && frequency != 2 && frequency != 4 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "frequency should be 1, 2 or 4".to_string(),
        ));
    }
    if settlement >= maturity {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "settlement should be < maturity".to_string(),
        ));
    }

    let basis = if args.len() == 7 {
        match model.get_number_no_bools(&args[6], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return Err(s),
        }
    } else {
        0
    };

    let days_in_year = days_in_year_simple(basis);
    let days = maturity - settlement;
    let periods = ((days * frequency as f64) / days_in_year).round();
    if periods <= 0.0 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "invalid dates".to_string(),
        ));
    }
    let coupon = redemption * rate / frequency as f64;

    Ok(BondPricingParams {
        third_param,
        redemption,
        frequency,
        periods,
        coupon,
    })
}

/// Helper struct for validated cumulative payment function parameters
struct CumulativePaymentParams {
    pub rate: f64,
    pub nper: f64,
    pub pv: f64,
    pub start_period: i32,
    pub end_period: i32,
    pub period_type: bool,
}

/// Helper function to parse and validate cumulative payment function parameters
fn parse_and_validate_cumulative_payment_params(
    args: &[Node],
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<CumulativePaymentParams, CalcResult> {
    // Check argument count
    if args.len() != 6 {
        return Err(CalcResult::new_args_number_error(cell));
    }

    // Parse rate, nper, pv
    let rate = model.get_number_no_bools(&args[0], cell)?;
    let nper = model.get_number_no_bools(&args[1], cell)?;
    let pv = model.get_number_no_bools(&args[2], cell)?;

    // Parse periods with appropriate rounding
    let start_period = model.get_number_no_bools(&args[3], cell)?.ceil() as i32;
    let end_period = model.get_number_no_bools(&args[4], cell)?.trunc() as i32;

    // Parse and validate period type (0 = end of period, 1 = beginning of period)
    let period_type = match model.get_number_no_bools(&args[5], cell)? {
        0.0 => false,
        1.0 => true,
        _ => {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "invalid period type".to_string(),
            ))
        }
    };

    // Validate period order
    if start_period > end_period {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "start period should come before end period".to_string(),
        ));
    }

    // Validate positive parameters
    if rate <= 0.0 || nper <= 0.0 || pv <= 0.0 || start_period < 1 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "invalid parameters".to_string(),
        ));
    }

    Ok(CumulativePaymentParams {
        rate,
        nper,
        pv,
        start_period,
        end_period,
        period_type,
    })
}

/// Helper function to validate and parse common coupon function parameters
fn parse_and_validate_coupon_params(
    args: &[Node],
    arg_count: usize,
    model: &mut Model,
    cell: CellReferenceIndex,
) -> Result<ValidatedCouponParams, CalcResult> {
    // Check argument count
    if !(3..=4).contains(&arg_count) {
        return Err(CalcResult::new_args_number_error(cell));
    }

    // Parse parameters
    let settlement = match model.get_number_no_bools(&args[0], cell) {
        Ok(f) => f.trunc() as i64,
        Err(s) => return Err(s),
    };
    let maturity = match model.get_number_no_bools(&args[1], cell) {
        Ok(f) => f.trunc() as i64,
        Err(s) => return Err(s),
    };
    let frequency = match model.get_number_no_bools(&args[2], cell) {
        Ok(f) => f.trunc() as i32,
        Err(s) => return Err(s),
    };
    let basis = if arg_count > 3 {
        match model.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return Err(s),
        }
    } else {
        0
    };

    // Validate frequency and basis
    if ![1, 2, 4].contains(&frequency) || !(0..=4).contains(&basis) {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "invalid arguments".to_string(),
        ));
    }

    // Validate settlement < maturity
    if settlement as f64 >= maturity as f64 {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "settlement should be < maturity".to_string(),
        ));
    }

    // Convert to dates
    let settlement_date = convert_date_serial(settlement as f64, cell)?;
    let maturity_date = convert_date_serial(maturity as f64, cell)?;

    Ok(ValidatedCouponParams {
        settlement_date,
        maturity_date,
        frequency,
        basis,
    })
}

fn year_frac(start: i64, end: i64, basis: i32) -> Result<f64, String> {
    let start_date = from_excel_date(start)?;
    let end_date = from_excel_date(end)?;
    let days = match basis {
        0 => days360_us(start_date, end_date),
        1..=3 => (end - start) as i32,
        4 => days360_eu(start_date, end_date),
        _ => return Err("invalid basis".to_string()),
    } as f64;
    let year_days = days_in_year(start_date, basis)? as f64;
    Ok(days / year_days)
}

fn year_fraction(
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
    basis: i32,
) -> Result<f64, String> {
    let days = match basis {
        0 => days360_us(start, end) as f64 / DAYS_IN_YEAR_360 as f64,
        1 => (end - start).num_days() as f64 / DAYS_ACTUAL as f64,
        2 => (end - start).num_days() as f64 / DAYS_IN_YEAR_360 as f64,
        3 => (end - start).num_days() as f64 / DAYS_ACTUAL as f64,
        4 => days360_eu(start, end) as f64 / DAYS_IN_YEAR_360 as f64,
        _ => return Err("Invalid basis".to_string()),
    };
    Ok(days)
}

fn days_between_dates(start: chrono::NaiveDate, end: chrono::NaiveDate, basis: i32) -> i32 {
    match basis {
        0 => days360_us(start, end),
        1 | 2 => (end - start).num_days() as i32,
        3 => (end - start).num_days() as i32,
        4 => days360_eu(start, end),
        _ => (end - start).num_days() as i32,
    }
}

fn coupon_dates(
    settlement: chrono::NaiveDate,
    maturity: chrono::NaiveDate,
    freq: i32,
) -> (chrono::NaiveDate, chrono::NaiveDate) {
    let months = 12 / freq;
    let step = chrono::Months::new(months as u32);
    let mut next_coupon_date = maturity;
    while let Some(prev) = next_coupon_date.checked_sub_months(step) {
        if settlement >= prev {
            return (prev, next_coupon_date);
        }
        next_coupon_date = prev;
    }
    // Fallback if we somehow exit the loop (shouldn't happen in practice)
    (settlement, maturity)
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

/// Enum to define different array processing behaviors
enum ArrayProcessingMode {
    Standard,               // Accept single numbers, ignore empty/non-number cells
    StrictWithError(Error), // Accept single numbers, error on empty/non-number with specified error type
    RangeOnlyWithZeros,     // Don't accept single numbers, treat empty as 0.0, error on non-number
}

impl Model {
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
            error @ CalcResult::Error { .. } => return Err(error),
            _ => {
                handle_non_number_cell()?;
            }
        }
        Ok(values)
    }

    fn get_array_of_numbers_with_mode(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        mode: ArrayProcessingMode,
    ) -> Result<Vec<f64>, CalcResult> {
        match mode {
            ArrayProcessingMode::Standard => {
                self.get_array_of_numbers_generic(
                    arg,
                    cell,
                    true,        // accept_number_node
                    || Ok(None), // Ignore empty cells
                    || Ok(None), // Ignore non-number cells
                )
            }
            ArrayProcessingMode::StrictWithError(error_type) => {
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
                            error_type.clone(),
                            *cell,
                            "Expected number".to_string(),
                        ))
                    },
                )
            }
            ArrayProcessingMode::RangeOnlyWithZeros => {
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
        }
    }

    fn get_array_of_numbers(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_with_mode(arg, cell, ArrayProcessingMode::Standard)
    }

    fn get_array_of_numbers_xpnv(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
        error: Error,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_with_mode(arg, cell, ArrayProcessingMode::StrictWithError(error))
    }

    fn get_array_of_numbers_xirr(
        &mut self,
        arg: &Node,
        cell: &CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.get_array_of_numbers_with_mode(arg, cell, ArrayProcessingMode::RangeOnlyWithZeros)
    }

    /// PMT(rate, nper, pv, [fv], [type])
    pub(crate) fn fn_pmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 3, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, nper, pv) = (params[0], params[1], params[2]);

        let optional_params = match parse_financial_optional_params(args, arg_count, 3, self, cell)
        {
            Ok(params) => params,
            Err(err) => return err,
        };

        match handle_compute_error(
            compute_payment(
                rate,
                nper,
                pv,
                optional_params.optional_value,
                optional_params.period_start,
            ),
            cell,
        ) {
            Ok(p) => CalcResult::Number(p),
            Err(err) => err,
        }
    }

    // PV(rate, nper, pmt, [fv], [type])
    pub(crate) fn fn_pv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 3, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, period_count, payment) = (params[0], params[1], params[2]);

        let optional_params = match parse_financial_optional_params(args, arg_count, 3, self, cell)
        {
            Ok(params) => params,
            Err(err) => return err,
        };
        if rate == 0.0 {
            return CalcResult::Number(-optional_params.optional_value - payment * period_count);
        }
        if rate == -1.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Rate must be != -1".to_string(),
            };
        };
        let rate_nper = (1.0 + rate).powf(period_count);
        let result = if optional_params.period_start {
            // type = 1
            -(optional_params.optional_value * rate + payment * (1.0 + rate) * (rate_nper - 1.0))
                / (rate * rate_nper)
        } else {
            (-optional_params.optional_value * rate - payment * (rate_nper - 1.0))
                / (rate * rate_nper)
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

    // ACCRINT(issue, first_interest, settlement, rate, par, freq, [basis], [calc])
    pub(crate) fn fn_accrint(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 6, 8, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3, 4, 5], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (issue, first, settlement, rate, par, freq) = (
            params[0],
            params[1],
            params[2],
            params[3],
            params[4],
            params[5] as i32,
        );
        let basis = match parse_optional_basis(args, 6, arg_count, self, cell) {
            Ok(b) => b,
            Err(err) => return err,
        };
        let calc = if arg_count > 7 {
            match self.get_number(&args[7], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            true
        };

        if !(freq == 1 || freq == 2 || freq == 4) {
            return CalcResult::new_error(Error::NUM, cell, "invalid frequency".to_string());
        }
        if !(0..=4).contains(&basis) {
            return CalcResult::new_error(Error::NUM, cell, "invalid basis".to_string());
        }
        if par < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par cannot be negative".to_string());
        }
        if rate < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate cannot be negative".to_string());
        }

        let issue_d = match convert_date_serial(issue, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };
        let first_d = match from_excel_date(first as i64) {
            Ok(d) => d,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        let settle_d = match convert_date_serial(settlement, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };

        if settle_d < issue_d {
            return CalcResult::new_error(Error::NUM, cell, "settlement < issue".to_string());
        }
        if first_d < issue_d {
            return CalcResult::new_error(Error::NUM, cell, "first_interest < issue".to_string());
        }
        if settle_d < first_d {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "settlement < first_interest".to_string(),
            );
        }

        let months = 12 / freq;
        let mut prev = first_d;
        if settle_d <= first_d {
            prev = issue_d;
        } else {
            while prev <= settle_d {
                let next = prev + chrono::Months::new(months as u32);
                if next > settle_d {
                    break;
                }
                prev = next;
            }
        }
        let next_coupon = prev + chrono::Months::new(months as u32);

        let mut result = 0.0;
        if calc {
            let mut next = first_d;
            while next < prev {
                result += rate * par / freq as f64;
                next = next + chrono::Months::new(months as u32);
            }
        }

        let days_in_period = match year_fraction(prev, next_coupon, basis) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "invalid basis".to_string()),
        };
        let days_elapsed = match year_fraction(prev, settle_d, basis) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "invalid basis".to_string()),
        };

        result += rate * par / freq as f64
            * if days_in_period == 0.0 {
                0.0
            } else {
                days_elapsed / days_in_period
            };
        CalcResult::Number(result)
    }

    // ACCRINTM(issue, settlement, rate, par, [basis])
    pub(crate) fn fn_accrintm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 4, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (issue, settlement, rate, par) = (params[0], params[1], params[2], params[3]);
        let basis = match parse_optional_basis(args, 4, arg_count, self, cell) {
            Ok(b) => b,
            Err(err) => return err,
        };

        if !(0..=4).contains(&basis) {
            return CalcResult::new_error(Error::NUM, cell, "invalid basis".to_string());
        }
        if par < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "par cannot be negative".to_string());
        }
        if rate < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate cannot be negative".to_string());
        }

        let issue_d = match convert_date_serial(issue, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };
        let settle_d = match convert_date_serial(settlement, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };

        if settle_d < issue_d {
            return CalcResult::new_error(Error::NUM, cell, "settlement < issue".to_string());
        }

        let frac = match year_fraction(issue_d, settle_d, basis) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "invalid basis".to_string()),
        };

        CalcResult::Number(par * rate * frac)
    }

    // RATE(nper, pmt, pv, [fv], [type], [guess])
    pub(crate) fn fn_rate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 3, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (nper, pmt, pv) = (params[0], params[1], params[2]);
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

        match handle_compute_error(compute_rate(pv, fv, nper, pmt, annuity_type, guess), cell) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
        }
    }

    // NPER(rate,pmt,pv,[fv],[type])
    pub(crate) fn fn_nper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 3, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, payment, present_value) = (params[0], params[1], params[2]);
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
        if let Some(err) = validate_arg_count_or_return(arg_count, 3, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, nper, pmt) = (params[0], params[1], params[2]);

        let optional_params = match parse_financial_optional_params(args, arg_count, 3, self, cell)
        {
            Ok(params) => params,
            Err(err) => return err,
        };

        match handle_compute_error(
            compute_future_value(
                rate,
                nper,
                pmt,
                optional_params.optional_value,
                optional_params.period_start,
            ),
            cell,
        ) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
        }
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
            Err(err) => return err,
        };
        let mut result = principal;
        for rate in schedule {
            if rate <= -1.0 {
                return CalcResult::new_error(Error::NUM, cell, "Rate must be > -1".to_string());
            }
            result *= 1.0 + rate;
        }
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid result".to_string());
        }
        CalcResult::Number(result)
    }

    // IPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 4, 6, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, period, period_count, present_value) =
            (params[0], params[1], params[2], params[3]);
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
        let ipmt = match handle_compute_error(
            compute_ipmt(
                rate,
                period,
                period_count,
                present_value,
                future_value,
                period_start,
            ),
            cell,
        ) {
            Ok(f) => f,
            Err(err) => return err,
        };
        CalcResult::Number(ipmt)
    }

    // PPMT(rate, per, nper, pv, [fv], [type])
    pub(crate) fn fn_ppmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 4, 6, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, period, period_count, present_value) =
            (params[0], params[1], params[2], params[3]);
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

        let ppmt = match handle_compute_error(
            compute_ppmt(
                rate,
                period,
                period_count,
                present_value,
                future_value,
                period_start,
            ),
            cell,
        ) {
            Ok(f) => f,
            Err(err) => return err,
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
            match self.get_array_of_numbers(arg, &cell) {
                Ok(mut arg_values) => values.append(&mut arg_values),
                Err(err) => return err,
            }
        }
        match handle_compute_error(compute_npv(rate, &values), cell) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
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
        match handle_compute_error(compute_irr(&values, guess), cell) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
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
        let dates = match validate_values_dates_arrays(&values, &dates, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };
        // It seems Excel returns #NUM! if rate < 0, this is only necessary if r <= -1
        if rate <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "rate needs to be > 0".to_string());
        }
        match handle_compute_error(compute_xnpv(rate, &values, &dates), cell) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
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
        let dates = match validate_values_dates_arrays(&values, &dates, cell) {
            Ok(d) => d,
            Err(err) => return err,
        };
        match handle_compute_error(compute_xirr(&values, &dates, guess), cell) {
            Ok(f) => CalcResult::Number(f),
            Err(err) => err,
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 3, 3, cell) {
            return err;
        }

        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let params = match parse_required_params(args, &[1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (finance_rate, reinvest_rate) = (params[0], params[1]);
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
            match handle_compute_error(compute_npv(reinvest_rate, &positive_values), cell) {
                Ok(npv) => -npv * ((1.0 + reinvest_rate).powf(years)),
                Err(err) => return err,
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
            match handle_compute_error(compute_npv(finance_rate, &negative_values), cell) {
                Ok(npv) => npv * (1.0 + finance_rate),
                Err(err) => return err,
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 4, 4, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, per, nper, pv) = (params[0], params[1], params[2], params[3]);
        if nper == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        CalcResult::Number(pv * rate * (per / nper - 1.0))
    }

    // RRI(nper, pv, fv)
    // Formula is
    // $$ \left(\frac{fv}{pv}\right)^{\frac{1}{nper}}-1  $$
    pub(crate) fn fn_rri(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if let Some(err) = validate_arg_count_or_return(args.len(), 3, 3, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (nper, pv, fv) = (params[0], params[1], params[2]);
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 3, 3, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (cost, salvage, life) = (params[0], params[1], params[2]);
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 4, 4, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (cost, salvage, life, per) = (params[0], params[1], params[2], params[3]);
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 2, 2, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (effect_rate, npery) = (params[0], params[1].floor());
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 2, 2, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (nominal_rate, npery) = (params[0], params[1].floor());
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
        if let Some(err) = validate_arg_count_or_return(args.len(), 3, 3, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (rate, pv, fv) = (params[0], params[1], params[2]);
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

    // DURATION(settlement, maturity, coupon, yld, freq, [basis])
    pub(crate) fn fn_duration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 5, 6, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3, 4], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (settlement, maturity, coupon, yld, freq) = (
            params[0],
            params[1],
            params[2],
            params[3],
            params[4].trunc() as i32,
        );
        let basis = match parse_optional_basis(args, 5, arg_count, self, cell) {
            Ok(b) => b,
            Err(err) => return err,
        };
        if settlement >= maturity || coupon < 0.0 || yld < 0.0 || !matches!(freq, 1 | 2 | 4) {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }

        let days_in_year = days_in_year_simple(basis);
        let diff_days = maturity - settlement;
        if diff_days <= 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid arguments".to_string());
        }
        let yearfrac = diff_days / days_in_year;
        let mut num_coupons = (yearfrac * freq as f64).ceil();
        if num_coupons < 1.0 {
            num_coupons = 1.0;
        }

        let cf = coupon * 100.0 / freq as f64;
        let y = 1.0 + yld / freq as f64;
        let ndiff = yearfrac * freq as f64 - num_coupons;
        let mut dur = 0.0;
        for t in 1..(num_coupons as i32) {
            let tt = t as f64 + ndiff;
            dur += tt * cf / y.powf(tt);
        }
        let last_t = num_coupons + ndiff;
        dur += last_t * (cf + 100.0) / y.powf(last_t);

        let mut price = 0.0;
        for t in 1..(num_coupons as i32) {
            let tt = t as f64 + ndiff;
            price += cf / y.powf(tt);
        }
        price += (cf + 100.0) / y.powf(last_t);

        if price == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }

        let result = (dur / price) / freq as f64;
        CalcResult::Number(result)
    }

    // MDURATION(settlement, maturity, coupon, yld, freq, [basis])
    pub(crate) fn fn_mduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut res = self.fn_duration(args, cell);
        if let CalcResult::Number(ref mut d) = res {
            let yld = match self.get_number_no_bools(&args[3], cell) {
                Ok(f) => f,
                Err(_) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid arguments".to_string(),
                    )
                }
            };
            let freq = match self.get_number_no_bools(&args[4], cell) {
                Ok(f) => f.trunc(),
                Err(_) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid arguments".to_string(),
                    )
                }
            };
            *d /= 1.0 + yld / freq;
        }
        res
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

        let (days_to_maturity, discount) = match parse_tbill_params(args, self, cell) {
            Ok(params) => params,
            Err(err) => return err,
        };

        let result = if days_to_maturity < TBILL_MATURITY_THRESHOLD {
            DAYS_ACTUAL as f64 * discount / (DAYS_IN_YEAR_360 as f64 - discount * days_to_maturity)
        } else {
            // Equation here is:
            // (1-days*rate/360)*(1+y/2)*(1+d_extra*y/year)=1
            let year = if days_to_maturity == DAYS_LEAP_YEAR as f64 {
                DAYS_LEAP_YEAR as f64
            } else {
                DAYS_ACTUAL as f64
            };
            let d_extra = days_to_maturity - year / 2.0;
            let alpha = 1.0 - days_to_maturity * discount / DAYS_IN_YEAR_360 as f64;
            let beta = 0.5 + d_extra / year;
            // ay^2+by+c=0
            let a = d_extra * alpha / (year * 2.0);
            let b = alpha * beta;
            let c = alpha - 1.0;
            (-b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
        };

        validate_tbill_result(result, cell)
    }

    // TBILLPRICE(settlement, maturity, discount)
    pub(crate) fn fn_tbillprice(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (days_to_maturity, discount) = match parse_tbill_params(args, self, cell) {
            Ok(params) => params,
            Err(err) => return err,
        };

        let result = 100.0 * (1.0 - discount * days_to_maturity / DAYS_IN_YEAR_360 as f64);

        // TBILLPRICE specifically checks for negative results (prices can't be negative)
        if result < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for RRI".to_string());
        }

        validate_tbill_result(result, cell)
    }

    // TBILLYIELD(settlement, maturity, pr)
    pub(crate) fn fn_tbillyield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let (days, price) = match parse_tbill_params(args, self, cell) {
            Ok(params) => params,
            Err(err) => return err,
        };

        let result = (100.0 - price) * DAYS_IN_YEAR_360 as f64 / (price * days);

        validate_tbill_result(result, cell)
    }

    pub(crate) fn fn_price(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_bond_pricing_params(args, self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let r = params.third_param / params.frequency as f64; // yld / frequency
        let mut price = 0.0;
        for i in 1..=(params.periods as i32) {
            price += params.coupon / (1.0 + r).powf(i as f64);
        }
        price += params.redemption / (1.0 + r).powf(params.periods);
        if price.is_nan() || price.is_infinite() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data".to_string());
        }
        CalcResult::Number(price)
    }

    pub(crate) fn fn_pricedisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        financial_function_with_year_frac!(
            args, self, cell,
            param1_name: "discount rate",
            param2_name: "redemption value",
            validator: |discount_rate, redemption_value| {
                if discount_rate <= 0.0 || redemption_value <= 0.0 {
                    Err("values must be positive".to_string())
                } else {
                    Ok(())
                }
            },
            formula: |_settlement, _maturity, discount_rate, redemption_value, _basis, year_frac| {
                redemption_value * (1.0 - discount_rate * year_frac)
            }
        )
    }

    pub(crate) fn fn_pricemat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 5, 6, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3, 4], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (settlement, maturity, issue, rate, yld) =
            (params[0], params[1], params[2], params[3], params[4]);
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if rate < 0.0 || yld < 0.0 || settlement >= maturity {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        if settlement < MINIMUM_DATE_SERIAL_NUMBER as f64
            || maturity > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || settlement > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || maturity < MINIMUM_DATE_SERIAL_NUMBER as f64
            || issue < MINIMUM_DATE_SERIAL_NUMBER as f64
            || issue > MAXIMUM_DATE_SERIAL_NUMBER as f64
        {
            return CalcResult::new_error(Error::NUM, cell, "Invalid number for date".to_string());
        }
        let issue_to_maturity_frac = match year_frac(issue as i64, maturity as i64, basis as i32) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        let issue_to_settlement_frac =
            match year_frac(issue as i64, settlement as i64, basis as i32) {
                Ok(f) => f,
                Err(_) => {
                    return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string())
                }
            };
        let settlement_to_maturity_frac =
            match year_frac(settlement as i64, maturity as i64, basis as i32) {
                Ok(f) => f,
                Err(_) => {
                    return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string())
                }
            };
        let mut result = 1.0 + issue_to_maturity_frac * rate;
        result /= 1.0 + settlement_to_maturity_frac * yld;
        result -= issue_to_settlement_frac * rate;
        result *= 100.0;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_yield(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_bond_pricing_params(args, self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        match handle_compute_error(
            compute_rate(
                -params.third_param,
                params.redemption,
                params.periods,
                params.coupon,
                0,
                0.1,
            ),
            cell,
        ) {
            Ok(r) => CalcResult::Number(r * params.frequency as f64),
            Err(err) => err,
        }
    }

    pub(crate) fn fn_yielddisc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        financial_function_with_year_frac!(
            args, self, cell,
            param1_name: "price",
            param2_name: "redemption value",
            validator: |price, redemption_value| {
                if price <= 0.0 || redemption_value <= 0.0 {
                    Err("values must be positive".to_string())
                } else {
                    Ok(())
                }
            },
            formula: |_settlement, _maturity, price, redemption_value, _basis, year_frac| {
                (redemption_value / price - 1.0) / year_frac
            }
        )
    }

    pub(crate) fn fn_yieldmat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 5, 6, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3, 4], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (settlement, maturity, issue, rate, price) =
            (params[0], params[1], params[2], params[3], params[4]);
        let basis = if arg_count == 6 {
            match self.get_number_no_bools(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if price <= 0.0 || rate < 0.0 || settlement >= maturity || settlement < issue {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        if settlement < MINIMUM_DATE_SERIAL_NUMBER as f64
            || maturity > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || settlement > MAXIMUM_DATE_SERIAL_NUMBER as f64
            || maturity < MINIMUM_DATE_SERIAL_NUMBER as f64
            || issue < MINIMUM_DATE_SERIAL_NUMBER as f64
            || issue > MAXIMUM_DATE_SERIAL_NUMBER as f64
        {
            return CalcResult::new_error(Error::NUM, cell, "Invalid number for date".to_string());
        }
        let issue_to_maturity_frac = match year_frac(issue as i64, maturity as i64, basis as i32) {
            Ok(f) => f,
            Err(_) => return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string()),
        };
        let issue_to_settlement_frac =
            match year_frac(issue as i64, settlement as i64, basis as i32) {
                Ok(f) => f,
                Err(_) => {
                    return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string())
                }
            };
        let settlement_to_maturity_frac =
            match year_frac(settlement as i64, maturity as i64, basis as i32) {
                Ok(f) => f,
                Err(_) => {
                    return CalcResult::new_error(Error::NUM, cell, "Invalid date".to_string())
                }
            };
        let mut y = 1.0 + issue_to_maturity_frac * rate;
        y /= price / 100.0 + issue_to_settlement_frac * rate;
        y -= 1.0;
        y /= settlement_to_maturity_frac;
        CalcResult::Number(y)
    }

    // DISC(settlement, maturity, pr, redemption, [basis])
    pub(crate) fn fn_disc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        financial_function_with_year_frac!(
            args, self, cell,
            param1_name: "price",
            param2_name: "redemption value",
            validator: |price, redemption_value| {
                if price <= 0.0 || redemption_value <= 0.0 {
                    Err("values must be positive".to_string())
                } else {
                    Ok(())
                }
            },
            formula: |_settlement, _maturity, price, redemption_value, _basis, year_frac| {
                (1.0 - price / redemption_value) / year_frac
            }
        )
    }

    // RECEIVED(settlement, maturity, investment, discount, [basis])
    pub(crate) fn fn_received(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        financial_function_with_year_frac!(
            args, self, cell,
            param1_name: "investment",
            param2_name: "discount rate",
            validator: |investment, discount_rate| {
                if investment <= 0.0 || discount_rate <= 0.0 {
                    Err("values must be positive".to_string())
                } else {
                    Ok(())
                }
            },
            formula: |_settlement, _maturity, investment, discount_rate, _basis, year_frac| {
                investment / (1.0 - discount_rate * year_frac)
            }
        )
    }

    // INTRATE(settlement, maturity, investment, redemption, [basis])
    pub(crate) fn fn_intrate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        financial_function_with_year_frac!(
            args, self, cell,
            param1_name: "investment",
            param2_name: "redemption value",
            validator: |investment, redemption_value| {
                if investment <= 0.0 || redemption_value <= 0.0 {
                    Err("values must be positive".to_string())
                } else {
                    Ok(())
                }
            },
            formula: |_settlement, _maturity, investment, redemption_value, _basis, year_frac| {
                ((redemption_value / investment) - 1.0) / year_frac
            }
        )
    }

    // COUPDAYBS(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_coupdaybs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let (prev_coupon_date, _) = coupon_dates(
            params.settlement_date,
            params.maturity_date,
            params.frequency,
        );
        let days = days_between_dates(prev_coupon_date, params.settlement_date, params.basis);
        CalcResult::Number(days as f64)
    }

    // COUPDAYS(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_coupdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let (prev_coupon_date, next_coupon_date) = coupon_dates(
            params.settlement_date,
            params.maturity_date,
            params.frequency,
        );
        let days = match params.basis {
            0 | 4 => DAYS_IN_YEAR_360 / params.frequency, // 30/360 conventions
            _ => days_between_dates(prev_coupon_date, next_coupon_date, params.basis), // Actual day counts
        };
        CalcResult::Number(days as f64)
    }

    // COUPDAYSNC(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_coupdaysnc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let (_, next_coupon_date) = coupon_dates(
            params.settlement_date,
            params.maturity_date,
            params.frequency,
        );
        let days = days_between_dates(params.settlement_date, next_coupon_date, params.basis);
        CalcResult::Number(days as f64)
    }

    // COUPNCD(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_coupncd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let (_, next_coupon_date) = coupon_dates(
            params.settlement_date,
            params.maturity_date,
            params.frequency,
        );
        date_to_serial_with_validation(next_coupon_date, cell)
    }

    // COUPNUM(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_coupnum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let months = 12 / params.frequency;
        let step = chrono::Months::new(months as u32);
        let mut date = params.maturity_date;
        let mut count = 0;
        while params.settlement_date < date {
            count += 1;
            date = match date.checked_sub_months(step) {
                Some(new_date) => new_date,
                None => break, // Safety check to avoid infinite loop
            };
        }
        CalcResult::Number(count as f64)
    }

    // COUPPCD(settlement, maturity, frequency, [basis])
    pub(crate) fn fn_couppcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_coupon_params(args, args.len(), self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };

        let (prev_coupon_date, _) = coupon_dates(
            params.settlement_date,
            params.maturity_date,
            params.frequency,
        );
        date_to_serial_with_validation(prev_coupon_date, cell)
    }

    // DOLLARDE(fractional_dollar, fraction)
    pub(crate) fn fn_dollarde(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if let Some(err) = validate_arg_count_or_return(args.len(), 2, 2, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (fractional_dollar, raw_fraction) = (params[0], params[1]);
        let fraction = match validate_and_normalize_fraction(raw_fraction, cell) {
            Ok(f) => f,
            Err(err) => return err,
        };

        let t = fractional_dollar.trunc();
        let result = t + (fractional_dollar - t) * 10.0 / fraction;
        CalcResult::Number(result)
    }

    // DOLLARFR(decimal_dollar, fraction)
    pub(crate) fn fn_dollarfr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if let Some(err) = validate_arg_count_or_return(args.len(), 2, 2, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1], self, cell, true) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (decimal_dollar, raw_fraction) = (params[0], params[1]);
        let fraction = match validate_and_normalize_fraction(raw_fraction, cell) {
            Ok(f) => f,
            Err(err) => return err,
        };

        let t = decimal_dollar.trunc();
        let result = t + (decimal_dollar - t) * fraction / 10.0;
        CalcResult::Number(result)
    }

    // CUMIPMT(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumipmt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_cumulative_payment_params(args, self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let mut result = 0.0;
        for period in params.start_period..=params.end_period {
            result += match handle_compute_error(
                compute_ipmt(
                    params.rate,
                    period as f64,
                    params.nper,
                    params.pv,
                    0.0,
                    params.period_type,
                ),
                cell,
            ) {
                Ok(f) => f,
                Err(err) => return err,
            }
        }
        CalcResult::Number(result)
    }

    // CUMPRINC(rate, nper, pv, start_period, end_period, type)
    pub(crate) fn fn_cumprinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let params = match parse_and_validate_cumulative_payment_params(args, self, cell) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let mut result = 0.0;
        for period in params.start_period..=params.end_period {
            result += match handle_compute_error(
                compute_ppmt(
                    params.rate,
                    period as f64,
                    params.nper,
                    params.pv,
                    0.0,
                    params.period_type,
                ),
                cell,
            ) {
                Ok(f) => f,
                Err(err) => return err,
            }
        }
        CalcResult::Number(result)
    }

    // DDB(cost, salvage, life, period, [factor])
    pub(crate) fn fn_ddb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if let Some(err) = validate_arg_count_or_return(arg_count, 4, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (cost, salvage, life, period) = (params[0], params[1], params[2], params[3]);
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
        if let Some(err) = validate_arg_count_or_return(arg_count, 4, 5, cell) {
            return err;
        }

        let params = match parse_required_params(args, &[0, 1, 2, 3], self, cell, false) {
            Ok(p) => p,
            Err(err) => return err,
        };
        let (cost, salvage, life, period) = (params[0], params[1], params[2], params[3]);
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
}
