use crate::cast::NumberOrArray;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::math_util::{from_roman, to_roman_with_form};
use crate::number_format::to_precision;
use crate::single_number_fn;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::PI;

#[cfg(not(target_arch = "wasm32"))]
pub fn random() -> f64 {
    rand::random()
}

// Euclidean gcd for i64 (non-negative inputs expected)
fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

// lcm(a, b) = a / gcd(a, b) * b
// we do it in i128 to reduce overflow risk, then back to i64/f64
fn lcm_i64(a: i64, b: i64) -> Option<i64> {
    if a == 0 || b == 0 {
        return Some(0);
    }
    let g = gcd_i64(a, b);
    let a_div_g = (a / g) as i128;
    let prod = a_div_g * (b as i128);
    if prod > i64::MAX as i128 {
        None
    } else {
        Some(prod as i64)
    }
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

    pub(crate) fn fn_base(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }

        // number to convert
        let mut value = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i64,
            Err(s) => return s,
        };
        // radix
        let radix = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc() as i64,
            Err(s) => return s,
        };
        // optional min_length
        let min_length = if arg_count == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => {
                    if f < 0.0 {
                        return CalcResult::Error {
                            error: Error::NUM,
                            origin: cell,
                            message: "Minimum length must be non-negative".to_string(),
                        };
                    }
                    f.trunc() as usize
                }
                Err(s) => return s,
            }
        } else {
            0
        };

        if !(2..=36).contains(&radix) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Radix must be between 2 and 36".to_string(),
            };
        }

        // number must be >= 0
        if value < 0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Number must be non-negative".to_string(),
            };
        }

        let mut buf = String::new();
        if value == 0 {
            buf.push('0');
        } else {
            while value > 0 {
                let digit = (value % radix) as u8;
                let ch = match digit {
                    0..=9 => (b'0' + digit) as char,
                    10..=35 => (b'A' + (digit - 10)) as char,
                    _ => unreachable!(),
                };
                buf.push(ch);
                value /= radix;
            }
            // we built it in reverse
            buf = buf.chars().rev().collect();
        }

        // pad with leading zeros if needed
        if buf.len() < min_length {
            let mut padded = String::with_capacity(min_length);
            for _ in 0..(min_length - buf.len()) {
                padded.push('0');
            }
            padded.push_str(&buf);
            buf = padded;
        }

        CalcResult::String(buf)
    }

    pub(crate) fn fn_decimal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let radix = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        if !(2..=36).contains(&radix) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Radix must be between 2 and 36".to_string(),
            };
        }
        match i64::from_str_radix(&text, radix as u32) {
            Ok(n) => CalcResult::Number(n as f64),
            Err(_) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: format!("'{}' is not a valid number in base {}", text, radix),
            },
        }
    }

    pub(crate) fn fn_gcd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut acc: Option<i64> = None;
        let mut saw_number = false;
        let mut has_range = false;

        // Returns Some(CalcResult) if an error occurred
        let mut handle_number = |value: f64| -> Option<CalcResult> {
            if !value.is_finite() {
                return Some(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Non-finite number in GCD".to_string(),
                ));
            }
            let n = value.trunc() as i64;
            if n < 0 {
                return Some(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "GCD only accepts non-negative integers".to_string(),
                ));
            }
            saw_number = true;
            acc = Some(match acc {
                Some(cur) => gcd_i64(cur, n),
                None => n,
            });
            None
        };

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    if let Some(res) = handle_number(value) {
                        return res;
                    }
                }
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    has_range = true;
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

                    for row in row1..=row2 {
                        for column in column1..=column2 {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    if let Some(res) = handle_number(value) {
                                        return res;
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                _ => {
                                    // ignore strings / booleans
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
                                    if let Some(res) = handle_number(value) {
                                        return res;
                                    }
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
                                _ => {
                                    // ignore strings / booleans
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // ignore strings / booleans
                }
            }
        }

        if !saw_number && !has_range {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No valid numbers found".to_string(),
            };
        }

        CalcResult::Number(acc.unwrap_or(0) as f64)
    }

    pub(crate) fn fn_lcm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut acc: Option<i64> = None;
        let mut saw_number = false;
        let mut has_range = false;

        // Returns Some(CalcResult) if an error occurred
        let mut handle_number = |value: f64| -> Option<CalcResult> {
            if !value.is_finite() {
                return Some(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Non-finite number in LCM".to_string(),
                ));
            }
            let n = value.trunc() as i64;
            if n < 0 {
                return Some(CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "LCM only accepts non-negative integers".to_string(),
                ));
            }
            saw_number = true;
            acc = Some(match acc {
                Some(cur) => match lcm_i64(cur, n) {
                    Some(v) => v,
                    None => {
                        return Some(CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "LCM result too large".to_string(),
                        ));
                    }
                },
                None => n,
            });
            None
        };

        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    if let Some(res) = handle_number(value) {
                        return res;
                    }
                }
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    has_range = true;
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

                    for row in row1..=row2 {
                        for column in column1..=column2 {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    if let Some(res) = handle_number(value) {
                                        return res;
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                _ => {
                                    // ignore strings / booleans
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
                                    if let Some(res) = handle_number(value) {
                                        return res;
                                    }
                                }
                                ArrayNode::Error(error) => {
                                    return CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    }
                                }
                                _ => {
                                    // ignore strings / booleans
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    // ignore strings / booleans
                }
            }
        }

        if !saw_number && !has_range {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No valid numbers found".to_string(),
            };
        }

        CalcResult::Number(acc.unwrap_or(0) as f64)
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

    pub(crate) fn fn_sumsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => result += value * value,
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
                                    result += value * value;
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
                                    result += value * value;
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

    pub(crate) fn fn_round(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            // Incorrect number of arguments
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => to_precision(f, 15),
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
            Ok(f) => to_precision(f, 15),
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
            Ok(f) => to_precision(f, 15),
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

    // (number, divisor)
    pub(crate) fn fn_mod(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let divisor = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if divisor == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }
        let result = value - divisor * (value / divisor).floor();
        CalcResult::Number(result)
    }

    pub(crate) fn fn_quotient(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let divisor = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if divisor == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }

        let result = value / divisor;
        CalcResult::Number(result.signum() * result.abs().floor())
    }

    pub(crate) fn fn_floor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if significance == 0.0 {
            if value == 0.0 {
                return CalcResult::Number(0.0);
            }
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }
        if significance < 0.0 && value > 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Significance must be positive when value is positive".to_string(),
            };
        }

        let result = f64::floor(value / significance) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if significance == 0.0 {
            // This behaviour is different from FLOOR where division by zero returns an error
            return CalcResult::Number(0.0);
        }
        if significance < 0.0 && value > 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Significance must be positive when value is positive".to_string(),
            };
        }

        let result = f64::ceil(value / significance) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_ceiling_math(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        let mode = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        if value < 0.0 && mode != 0.0 {
            let result = f64::floor(value / significance) * significance;
            CalcResult::Number(result)
        } else {
            let result = f64::ceil(value / significance) * significance;
            CalcResult::Number(result)
        }
    }

    pub(crate) fn fn_ceiling_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }

        let result = f64::ceil(value / significance) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_iso_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // ISO.CEILING is equivalent to CEILING.PRECISE
        self.fn_ceiling_precise(args, cell)
    }

    pub(crate) fn fn_floor_math(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            1.0
        };
        let mode = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        let significance = significance.abs();
        if value < 0.0 && mode != 0.0 {
            let result = f64::ceil(value / significance) * significance;
            CalcResult::Number(result)
        } else {
            let result = f64::floor(value / significance) * significance;
            CalcResult::Number(result)
        }
    }

    pub(crate) fn fn_floor_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if arg_count > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }

        let result = f64::floor(value / significance) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_mround(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // MROUND(number, multiple)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let multiple = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if multiple == 0.0 {
            return CalcResult::Number(0.0);
        }
        if (value > 0.0 && multiple < 0.0) || (value < 0.0 && multiple > 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "number and multiple must have the same sign".to_string(),
            };
        }
        let result = (value / multiple).round() * multiple;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_trunc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let num_digits = if args.len() == 2 {
            match self.get_number(&args[1], cell) {
                Ok(f) => {
                    if f > 0.0 {
                        f.floor()
                    } else {
                        f.ceil()
                    }
                }
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if !(-15.0..=15.0).contains(&num_digits) {
            return CalcResult::Number(value);
        }
        CalcResult::Number(if value >= 0.0 {
            f64::floor(value * 10f64.powf(num_digits)) / 10f64.powf(num_digits)
        } else {
            f64::ceil(value * 10f64.powf(num_digits)) / 10f64.powf(num_digits)
        })
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
    single_number_fn!(fn_acot, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(f64::atan(1.0 / f))
    });
    single_number_fn!(fn_acoth, |f: f64| if f.abs() == 1.0 {
        Err(Error::DIV)
    } else {
        Ok(0.5 * (f64::ln((f + 1.0) / (f - 1.0))))
    });
    single_number_fn!(fn_cot, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(f64::cos(f) / f64::sin(f))
    });
    single_number_fn!(fn_coth, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(f64::cosh(f) / f64::sinh(f))
    });
    single_number_fn!(fn_csc, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / f64::sin(f))
    });
    single_number_fn!(fn_csch, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / f64::sinh(f))
    });
    single_number_fn!(fn_sec, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / f64::cos(f))
    });
    single_number_fn!(fn_sech, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / f64::cosh(f))
    });
    single_number_fn!(fn_exp, |f: f64| Ok(f64::exp(f)));
    single_number_fn!(fn_fact, |x: f64| {
        let x = x.floor();
        if x < 0.0 {
            return Err(Error::NUM);
        }
        let mut acc = 1.0;
        let mut k = 2.0;
        while k <= x {
            acc *= k;
            k += 1.0;
        }
        Ok(acc)
    });
    single_number_fn!(fn_factdouble, |x: f64| {
        let x = x.floor();
        if x < -1.0 {
            return Err(Error::NUM);
        }
        if x < 0.0 {
            return Ok(1.0);
        }
        let mut acc = 1.0;
        let mut k = if x % 2.0 == 0.0 { 2.0 } else { 1.0 };
        while k <= x {
            acc *= k;
            k += 2.0;
        }
        Ok(acc)
    });
    single_number_fn!(fn_sign, |f| Ok(f64::signum(f)));
    single_number_fn!(fn_degrees, |f| Ok(f * (180.0 / PI)));
    single_number_fn!(fn_radians, |f| Ok(f * (PI / 180.0)));
    single_number_fn!(fn_odd, |f| {
        let sign = f64::signum(f);
        Ok(sign * (f64::ceil((f64::abs(f) - 1.0) / 2.0) * 2.0 + 1.0))
    });
    single_number_fn!(fn_even, |f| Ok(f64::signum(f)
        * f64::ceil(f64::abs(f) / 2.0)
        * 2.0));
    single_number_fn!(fn_int, |f| Ok(f64::floor(f)));

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

    pub(crate) fn fn_roman(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() || args.len() > 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };

        if number == 0.0 {
            return CalcResult::String(String::new());
        }
        if !(0.0..=3999.0).contains(&number) {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Number must be between 0 and 3999".to_string(),
            };
        }
        let form = if args.len() == 2 {
            let mut t = match self.get_number(&args[1], cell) {
                Ok(f) => f as i32,
                Err(s) => return s,
            };
            // If the value is a boolean TRUE/FALSE, convert to 0/4
            if t == 0 || t == 1 {
                if let CalcResult::Boolean(b) = self.evaluate_node_in_context(&args[1], cell) {
                    if b {
                        // classic form
                        t = 0;
                    } else {
                        // simplified form
                        t = 4;
                    }
                }
            }
            t
        } else {
            0
        };
        if !(0..=4).contains(&form) {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Form must be between 0 and 4".to_string(),
            };
        }
        let roman_numeral = match to_roman_with_form(number as u32, form) {
            Ok(s) => s,
            Err(e) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: format!("Could not convert to Roman numeral: {e}"),
                }
            }
        };
        CalcResult::String(roman_numeral)
    }

    pub(crate) fn fn_arabic(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let roman_numeral = match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::String(s) => s,
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Argument must be a text string".to_string(),
                }
            }
        };
        if roman_numeral.is_empty() {
            return CalcResult::Number(0.0);
        }
        match from_roman(&roman_numeral) {
            Ok(value) => CalcResult::Number(value as f64),
            Err(e) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: format!("Invalid Roman numeral: {e}"),
            },
        }
    }

    pub(crate) fn fn_combin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let n = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if n < 0.0 || k < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments must be non-negative integers".to_string(),
            };
        }
        if k > n {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "k cannot be greater than n".to_string(),
            };
        }
        let k = k as usize;
        let mut result = 1.0;
        for i in 0..k {
            let t = i as f64;
            result *= (n - t) / (t + 1.0);
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_combina(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let n = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if n < 0.0 || k < 0.0 || (n == 0.0 && k > 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments must be non-negative integers".to_string(),
            };
        }
        let k = k as usize;
        let mut result = 1.0;
        for i in 0..k {
            let t = i as f64;
            result *= (n + t) / (t + 1.0);
        }
        CalcResult::Number(result)
    }
}
