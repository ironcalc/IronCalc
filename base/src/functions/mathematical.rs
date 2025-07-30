use crate::cast::NumberOrArray;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::single_number_fn;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::PI;

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

impl Model {
    pub(crate) fn fn_min(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = f64::NAN;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => result = value.min(result),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    for row in left.row..(right.row + 1) {
                        for column in left.column..(right.column + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    result = value.min(result);
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
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_max(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = f64::NAN;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => result = value.max(result),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    for row in left.row..(right.row + 1) {
                        for column in left.column..(right.column + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    result = value.max(result);
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
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => result += value,
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    // TODO: We should do this for all functions that run through ranges
                    // Running cargo test for the ironcalc takes around .8 seconds with this speedup
                    // and ~ 3.5 seconds without it. Note that once properly in place sheet.dimension should be almost a noop
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
                                    result += value;
                                }
                                error @ CalcResult::Error { .. } => return error,
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
                                    result += value;
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
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
        CalcResult::Number(result)
    }

    pub(crate) fn fn_product(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 1.0;
        let mut seen_value = false;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    seen_value = true;
                    result *= value;
                }
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
                                    seen_value = true;
                                    result *= value;
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
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let y = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
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
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let y = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if x == 0.0 && y == 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments can't be both zero".to_string(),
            };
        }
        if y == 0.0 {
            return CalcResult::Number(1.0);
        }
        let result = x.powf(y);
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
}
