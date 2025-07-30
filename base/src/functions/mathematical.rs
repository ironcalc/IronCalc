use crate::cast::NumberOrArray;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::single_number_fn;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::PI;

/// Shared GCD (Greatest Common Divisor) implementation
/// Used by both GCD and LCM functions
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Specifies which rounding behaviour to apply when calling `round_to_multiple`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoundKind {
    Ceiling,
    Floor,
}

/// Rounding mode used by the classic ROUND family (ROUND, ROUNDUP, ROUNDDOWN).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoundDecimalKind {
    Nearest, // ROUND
    Up,      // ROUNDUP
    Down,    // ROUNDDOWN
}

#[cfg(not(target_arch = "wasm32"))]
pub fn random() -> f64 {
    rand::random()
}

#[cfg(target_arch = "wasm32")]
pub fn random() -> f64 {
    use js_sys::Math;
    Math::random()
}

/// Utility struct to hold optimized range bounds
struct RangeBounds {
    row_start: i32,
    row_end: i32,
    col_start: i32,
    col_end: i32,
}

impl Model {
    /// Resolves worksheet bounds by replacing LAST_ROW/LAST_COLUMN with actual sheet dimensions
    /// Provides significant performance improvement for large range operations
    fn resolve_worksheet_bounds(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
    ) -> Result<RangeBounds, CalcResult> {
        let row_start = left.row;
        let mut row_end = right.row;
        let col_start = left.column;
        let mut col_end = right.column;

        if row_start == 1 && row_end == LAST_ROW {
            row_end = match self.workbook.worksheet(left.sheet) {
                Ok(s) => s.dimension().max_row,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", left.sheet),
                    ));
                }
            };
        }
        if col_start == 1 && col_end == LAST_COLUMN {
            col_end = match self.workbook.worksheet(left.sheet) {
                Ok(s) => s.dimension().max_column,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", left.sheet),
                    ));
                }
            };
        }

        Ok(RangeBounds {
            row_start,
            row_end,
            col_start,
            col_end,
        })
    }

    /// Extracts exactly two numbers from function arguments with validation
    /// Used by ATAN2, MOD, QUOTIENT, POWER, etc.
    fn extract_two_numbers(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        if args.len() != 2 {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let first = self.get_number(&args[0], cell)?;
        let second = self.get_number(&args[1], cell)?;
        Ok((first, second))
    }

    /// Applies a closure to all numeric values in function arguments (ranges, arrays, numbers)
    /// Returns early on errors. Used by aggregate functions like GCD, LCM.
    fn process_numeric_args<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        processor: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64) -> Result<(), CalcResult>,
    {
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(v) => {
                    processor(v)?;
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
                                    processor(v)?;
                                }
                                error @ CalcResult::Error { .. } => return Err(error),
                                _ => {}
                            }
                        }
                    }
                }
                CalcResult::Array(arr) => {
                    for row in arr {
                        for value in row {
                            match value {
                                ArrayNode::Number(v) => {
                                    processor(v)?;
                                }
                                ArrayNode::Error(err) => {
                                    return Err(CalcResult::Error {
                                        error: err,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return Err(error),
                _ => {}
            }
        }
        Ok(())
    }

    /// Applies a closure to all numeric values in ranges with bounds optimization
    /// Used by functions like SUM, PRODUCT that benefit from the optimization
    fn process_numeric_args_with_range_bounds<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        processor: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64) -> Result<(), CalcResult>,
    {
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    processor(value)?;
                }
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        ));
                    }
                    let bounds = self.resolve_worksheet_bounds(left, right, cell)?;

                    for row in bounds.row_start..=bounds.row_end {
                        for column in bounds.col_start..=bounds.col_end {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    processor(value)?;
                                }
                                error @ CalcResult::Error { .. } => return Err(error),
                                _ => {
                                    // We ignore booleans and strings
                                }
                            }
                        }
                    }
                }
                CalcResult::Array(array) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    processor(value)?;
                                }
                                ArrayNode::Error(error) => {
                                    return Err(CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    });
                                }
                                _ => {
                                    // We ignore booleans and strings
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return Err(error),
                _ => {
                    // We ignore booleans and strings
                }
            }
        }
        Ok(())
    }

    pub(crate) fn fn_min(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = f64::NAN;
        let mut min_processor = |value: f64| -> Result<(), CalcResult> {
            result = value.min(result);
            Ok(())
        };

        // Use the optimized utility function for range processing
        if let Err(e) = self.process_numeric_args_with_range_bounds(args, cell, &mut min_processor)
        {
            return e;
        }

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_max(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = f64::NAN;
        let mut max_processor = |value: f64| -> Result<(), CalcResult> {
            result = value.max(result);
            Ok(())
        };

        // Use the optimized utility function for range processing
        if let Err(e) = self.process_numeric_args_with_range_bounds(args, cell, &mut max_processor)
        {
            return e;
        }

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_sum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = 0.0;
        let mut sum_processor = |value: f64| -> Result<(), CalcResult> {
            result += value;
            Ok(())
        };

        // Use the new utility function with optimization for range bounds
        if let Err(e) = self.process_numeric_args_with_range_bounds(args, cell, &mut sum_processor)
        {
            return e;
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_product(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = 1.0;
        let mut seen_value = false;
        let mut product_processor = |value: f64| -> Result<(), CalcResult> {
            seen_value = true;
            result *= value;
            Ok(())
        };

        // Use the new utility function with optimization for range bounds
        if let Err(e) =
            self.process_numeric_args_with_range_bounds(args, cell, &mut product_processor)
        {
            return e;
        }

        if !seen_value {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    /// SUMIF(criteria_range, criteria, [sum_range])
    /// if sum_rage is missing then criteria_range will be used
    pub(crate) fn fn_sumif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let arguments = vec![args[0].clone(), args[0].clone(), args[1].clone()];
            self.fn_sumifs(&arguments, cell)
        } else if args.len() == 3 {
            let arguments = vec![args[2].clone(), args[0].clone(), args[1].clone()];
            self.fn_sumifs(&arguments, cell)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    /// SUMIFS(sum_range, criteria_range1, criteria1, [criteria_range2, criteria2], ...)
    pub(crate) fn fn_sumifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut total = 0.0;
        let sum = |value| total += value;
        if let Err(e) = self.apply_ifs(args, cell, sum) {
            return e;
        }
        CalcResult::Number(total)
    }

    /// Shared implementation for Excel's ROUND / ROUNDUP / ROUNDDOWN functions
    /// that round a scalar to a specified number of decimal digits.
    fn round_decimal_fn(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: RoundDecimalKind,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // Extract value and number_of_digits, propagating errors.
        let value = match self.get_number(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let digits_raw = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        // Excel truncates non-integer digit counts toward zero.
        let digits = if digits_raw > 0.0 {
            digits_raw.floor()
        } else {
            digits_raw.ceil()
        };

        let scale = 10.0_f64.powf(digits);

        let rounded = match mode {
            RoundDecimalKind::Nearest => (value * scale).round() / scale,
            RoundDecimalKind::Up => {
                if value > 0.0 {
                    (value * scale).ceil() / scale
                } else {
                    (value * scale).floor() / scale
                }
            }
            RoundDecimalKind::Down => {
                if value > 0.0 {
                    (value * scale).floor() / scale
                } else {
                    (value * scale).ceil() / scale
                }
            }
        };

        CalcResult::Number(rounded)
    }

    /// Helper used by CEILING and FLOOR to round a value to the nearest multiple of
    /// `significance`, taking into account the Excel sign rule.
    fn round_to_multiple(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        kind: RoundKind,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let value = match self.get_number(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        if significance == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }
        if value.signum() * significance.signum() < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid sign".to_string(),
            };
        }

        let quotient = value / significance;
        let use_ceil = (significance > 0.0) == matches!(kind, RoundKind::Ceiling);
        let rounded_multiple = if use_ceil {
            quotient.ceil() * significance
        } else {
            quotient.floor() * significance
        };

        CalcResult::Number(rounded_multiple)
    }

    pub(crate) fn fn_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_to_multiple(args, cell, RoundKind::Ceiling)
    }

    pub(crate) fn fn_floor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_to_multiple(args, cell, RoundKind::Floor)
    }

    pub(crate) fn fn_round(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_decimal_fn(args, cell, RoundDecimalKind::Nearest)
    }

    pub(crate) fn fn_roundup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_decimal_fn(args, cell, RoundDecimalKind::Up)
    }

    pub(crate) fn fn_rounddown(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_decimal_fn(args, cell, RoundDecimalKind::Down)
    }

    single_number_fn!(fn_log10, |f| if f <= 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::log10(f))
    });
    single_number_fn!(fn_ln, |f| if f <= 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::ln(f))
    });
    single_number_fn!(fn_sin, |f| Ok(f64::sin(f)));
    single_number_fn!(fn_cos, |f| Ok(f64::cos(f)));
    single_number_fn!(fn_tan, |f| Ok(f64::tan(f)));
    single_number_fn!(fn_sinh, |f| Ok(f64::sinh(f)));
    single_number_fn!(fn_cosh, |f| Ok(f64::cosh(f)));
    single_number_fn!(fn_tanh, |f| Ok(f64::tanh(f)));
    single_number_fn!(fn_asin, |f| Ok(f64::asin(f)));
    single_number_fn!(fn_acos, |f| Ok(f64::acos(f)));
    single_number_fn!(fn_atan, |f| Ok(f64::atan(f)));
    single_number_fn!(fn_asinh, |f| Ok(f64::asinh(f)));
    single_number_fn!(fn_acosh, |f| Ok(f64::acosh(f)));
    single_number_fn!(fn_atanh, |f| Ok(f64::atanh(f)));
    single_number_fn!(fn_degrees, |f: f64| Ok(f.to_degrees()));
    single_number_fn!(fn_radians, |f: f64| Ok(f.to_radians()));
    single_number_fn!(fn_abs, |f| Ok(f64::abs(f)));
    single_number_fn!(fn_sqrt, |f| if f < 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::sqrt(f))
    });
    single_number_fn!(fn_sqrtpi, |f: f64| if f < 0.0 {
        Err(Error::NUM)
    } else {
        Ok((f * PI).sqrt())
    });

    pub(crate) fn fn_pi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(PI)
    }

    pub(crate) fn fn_atan2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (x, y) = match self.extract_two_numbers(args, cell) {
            Ok((x, y)) => (x, y),
            Err(e) => return e,
        };
        if x == 0.0 && y == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Arguments can't be both zero".to_string(),
            };
        }
        CalcResult::Number(f64::atan2(y, x))
    }

    pub(crate) fn fn_log(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let n_args = args.len();
        if !(1..=2).contains(&n_args) {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let y = if n_args == 1 {
            10.0
        } else {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        };
        if x <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Number must be positive".to_string(),
            };
        }
        if y == 1.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Logarithm base cannot be 1".to_string(),
            };
        }
        if y <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Logarithm base must be positive".to_string(),
            };
        }
        CalcResult::Number(f64::log(x, y))
    }

    pub(crate) fn fn_power(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (base, exp) = match self.extract_two_numbers(args, cell) {
            Ok((base, exp)) => (base, exp),
            Err(e) => return e,
        };
        if base == 0.0 && exp == 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments can't be both zero".to_string(),
            };
        }
        if exp == 0.0 {
            return CalcResult::Number(1.0);
        }
        let result = base.powf(exp);
        if result.is_infinite() {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "POWER returned infinity".to_string(),
            };
        }
        if result.is_nan() {
            // This might happen for some combinations of negative base and exponent
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid arguments for POWER".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_mod(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (number, divisor) = match self.extract_two_numbers(args, cell) {
            Ok((num, div)) => (num, div),
            Err(e) => return e,
        };
        if divisor == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        }
        CalcResult::Number(number - divisor * (number / divisor).floor())
    }

    pub(crate) fn fn_quotient(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (numerator, denominator) = match self.extract_two_numbers(args, cell) {
            Ok((num, den)) => (num, den),
            Err(e) => return e,
        };
        if denominator == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Divide by 0".to_string());
        }
        CalcResult::Number((numerator / denominator).trunc())
    }

    pub(crate) fn fn_gcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result: Option<u128> = None;
        let mut update = |value: f64| -> Result<(), CalcResult> {
            if value < 0.0 {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "numbers must be positive".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "value must be finite".to_string(),
                ));
            }
            let truncated = value.trunc();
            if truncated > u128::MAX as f64 {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "value too large".to_string(),
                ));
            }
            let v = truncated as u128;
            result = Some(match result {
                None => v,
                Some(r) => gcd(r, v),
            });
            Ok(())
        };

        // Use the new utility function to process all numeric arguments
        if let Err(e) = self.process_numeric_args(args, cell, &mut update) {
            return e;
        }

        CalcResult::Number(result.unwrap_or(0) as f64)
    }

    pub(crate) fn fn_lcm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result: Option<u128> = None;
        let mut update = |value: f64| -> Result<(), CalcResult> {
            if value < 0.0 {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "numbers must be positive".to_string(),
                ));
            }
            if !value.is_finite() {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "value must be finite".to_string(),
                ));
            }
            let truncated = value.trunc();
            if truncated > u128::MAX as f64 {
                return Err(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "value too large".to_string(),
                ));
            }
            let v = truncated as u128;
            result = Some(match result {
                None => v,
                Some(r) => {
                    if r == 0 || v == 0 {
                        0
                    } else {
                        r / gcd(r, v) * v
                    }
                }
            });
            Ok(())
        };

        // Use the new utility function to process all numeric arguments
        if let Err(e) = self.process_numeric_args(args, cell, &mut update) {
            return e;
        }

        CalcResult::Number(result.unwrap_or(0) as f64)
    }

    pub(crate) fn fn_rand(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(random())
    }

    // TODO: Add tests for RANDBETWEEN
    pub(crate) fn fn_randbetween(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        let y = match self.get_number(&args[1], cell) {
            Ok(f) => f.ceil() + 1.0,
            Err(s) => return s,
        };
        if x > y {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: format!("{x}>{y}"),
            };
        }
        CalcResult::Number((x + random() * (y - x)).floor())
    }

    single_number_fn!(fn_int, |f: f64| Ok(f.floor()));

    pub(crate) fn fn_mround(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        // Special case: zero number always returns 0 regardless of significance sign
        if number == 0.0 {
            return CalcResult::Number(0.0);
        }
        // Excel requires number and significance to have the same sign for non-zero numbers
        if number.signum() != significance.signum() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "number and significance must have the same sign".to_string(),
            };
        }

        // MROUND rounds ties away from zero (unlike Rust's round() which uses banker's rounding)
        let quotient = number / significance;

        // Add f64::EPSILON to handle floating-point precision at exact 0.5 boundaries
        // e.g., when 0.15/0.1 gives 1.4999999999999998 instead of exactly 1.5
        let rounded_quotient = if quotient >= 0.0 {
            (quotient + 0.5 + f64::EPSILON).floor()
        } else {
            (quotient - 0.5 - f64::EPSILON).ceil()
        };

        CalcResult::Number(rounded_quotient * significance)
    }
}
