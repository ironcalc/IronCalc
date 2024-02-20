use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::PI;

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
                        row2 = self
                            .workbook
                            .worksheet(left.sheet)
                            .expect("Sheet expected during evaluation.")
                            .dimension()
                            .max_row;
                    }
                    if column1 == 1 && column2 == LAST_COLUMN {
                        column2 = self
                            .workbook
                            .worksheet(left.sheet)
                            .expect("Sheet expected during evaluation.")
                            .dimension()
                            .max_column;
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
                        row2 = self
                            .workbook
                            .worksheet(left.sheet)
                            .expect("Sheet expected during evaluation.")
                            .dimension()
                            .max_row;
                    }
                    if column1 == 1 && column2 == LAST_COLUMN {
                        column2 = self
                            .workbook
                            .worksheet(left.sheet)
                            .expect("Sheet expected during evaluation.")
                            .dimension()
                            .max_column;
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

    pub(crate) fn fn_round(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            // Incorrect number of arguments
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let number_of_digits = match self.get_number(&args[1], cell) {
            Ok(f) => {
                if f > 0.0 {
                    f.floor()
                } else {
                    f.ceil()
                }
            }
            Err(s) => return s,
        };
        let scale = 10.0_f64.powf(number_of_digits);
        CalcResult::Number((value * scale).round() / scale)
    }
    pub(crate) fn fn_roundup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let number_of_digits = match self.get_number(&args[1], cell) {
            Ok(f) => {
                if f > 0.0 {
                    f.floor()
                } else {
                    f.ceil()
                }
            }
            Err(s) => return s,
        };
        let scale = 10.0_f64.powf(number_of_digits);
        if value > 0.0 {
            CalcResult::Number((value * scale).ceil() / scale)
        } else {
            CalcResult::Number((value * scale).floor() / scale)
        }
    }
    pub(crate) fn fn_rounddown(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let number_of_digits = match self.get_number(&args[1], cell) {
            Ok(f) => {
                if f > 0.0 {
                    f.floor()
                } else {
                    f.ceil()
                }
            }
            Err(s) => return s,
        };
        let scale = 10.0_f64.powf(number_of_digits);
        if value > 0.0 {
            CalcResult::Number((value * scale).floor() / scale)
        } else {
            CalcResult::Number((value * scale).ceil() / scale)
        }
    }

    pub(crate) fn fn_sin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.sin();
        CalcResult::Number(result)
    }
    pub(crate) fn fn_cos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.cos();
        CalcResult::Number(result)
    }

    pub(crate) fn fn_tan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.tan();
        CalcResult::Number(result)
    }

    pub(crate) fn fn_sinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.sinh();
        CalcResult::Number(result)
    }
    pub(crate) fn fn_cosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.cosh();
        CalcResult::Number(result)
    }

    pub(crate) fn fn_tanh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.tanh();
        CalcResult::Number(result)
    }

    pub(crate) fn fn_asin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.asin();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for ASIN".to_string(),
            };
        }
        CalcResult::Number(result)
    }
    pub(crate) fn fn_acos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.acos();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for COS".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_atan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.atan();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for ATAN".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_asinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.asinh();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for ASINH".to_string(),
            };
        }
        CalcResult::Number(result)
    }
    pub(crate) fn fn_acosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.acosh();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for ACOSH".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_atanh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = value.atanh();
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid argument for ATANH".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_pi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(PI)
    }

    pub(crate) fn fn_abs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        CalcResult::Number(value.abs())
    }

    pub(crate) fn fn_sqrtpi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if value < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Argument of SQRTPI should be >= 0".to_string(),
            };
        }
        CalcResult::Number((value * PI).sqrt())
    }

    pub(crate) fn fn_sqrt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if value < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Argument of SQRT should be >= 0".to_string(),
            };
        }
        CalcResult::Number(value.sqrt())
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
