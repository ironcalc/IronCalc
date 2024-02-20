use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

use super::util::compare_values;

impl Model {
    pub(crate) fn fn_if(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 || args.len() == 3 {
            let cond_result = self.get_boolean(&args[0], cell);
            let cond = match cond_result {
                Ok(f) => f,
                Err(s) => {
                    return s;
                }
            };
            if cond {
                return self.evaluate_node_in_context(&args[1], cell);
            } else if args.len() == 3 {
                return self.evaluate_node_in_context(&args[2], cell);
            } else {
                return CalcResult::Boolean(false);
            }
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_iferror(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let value = self.evaluate_node_in_context(&args[0], cell);
            match value {
                CalcResult::Error { .. } => {
                    return self.evaluate_node_in_context(&args[1], cell);
                }
                _ => return value,
            }
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_ifna(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let value = self.evaluate_node_in_context(&args[0], cell);
            if let CalcResult::Error { error, .. } = &value {
                if error == &Error::NA {
                    return self.evaluate_node_in_context(&args[1], cell);
                }
            }
            return value;
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_not(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 1 {
            match self.get_boolean(&args[0], cell) {
                Ok(f) => return CalcResult::Boolean(!f),
                Err(s) => {
                    return s;
                }
            };
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_and(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut true_count = 0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Boolean(b) => {
                    if !b {
                        return CalcResult::Boolean(false);
                    }
                    true_count += 1;
                }
                CalcResult::Number(value) => {
                    if value == 0.0 {
                        return CalcResult::Boolean(false);
                    }
                    true_count += 1;
                }
                CalcResult::String(_value) => {
                    true_count += 1;
                }
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
                                CalcResult::Boolean(b) => {
                                    if !b {
                                        return CalcResult::Boolean(false);
                                    }
                                    true_count += 1;
                                }
                                CalcResult::Number(value) => {
                                    if value == 0.0 {
                                        return CalcResult::Boolean(false);
                                    }
                                    true_count += 1;
                                }
                                CalcResult::String(_value) => {
                                    true_count += 1;
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::Range { .. } => {}
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
            };
        }
        if true_count == 0 {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Boolean values not found".to_string(),
            );
        }
        CalcResult::Boolean(true)
    }

    pub(crate) fn fn_or(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = false;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Boolean(value) => result = value || result,
                CalcResult::Number(value) => {
                    if value != 0.0 {
                        return CalcResult::Boolean(true);
                    }
                }
                CalcResult::String(_value) => {
                    return CalcResult::Boolean(true);
                }
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
                                CalcResult::Boolean(value) => {
                                    result = value || result;
                                }
                                CalcResult::Number(value) => {
                                    if value != 0.0 {
                                        return CalcResult::Boolean(true);
                                    }
                                }
                                CalcResult::String(_value) => {
                                    return CalcResult::Boolean(true);
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::Range { .. } => {}
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
            };
        }
        CalcResult::Boolean(result)
    }

    /// XOR(logical1, [logical]*,...)
    /// Logical1 is required, subsequent logical values are optional. Can be logical values, arrays, or references.
    /// The result of XOR is TRUE when the number of TRUE inputs is odd and FALSE when the number of TRUE inputs is even.
    pub(crate) fn fn_xor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut true_count = 0;
        let mut false_count = 0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Boolean(b) => {
                    if b {
                        true_count += 1;
                    } else {
                        false_count += 1;
                    }
                }
                CalcResult::Number(value) => {
                    if value != 0.0 {
                        true_count += 1;
                    } else {
                        false_count += 1;
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
                    for row in left.row..(right.row + 1) {
                        for column in left.column..(right.column + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Boolean(b) => {
                                    if b {
                                        true_count += 1;
                                    } else {
                                        false_count += 1;
                                    }
                                }
                                CalcResult::Number(value) => {
                                    if value != 0.0 {
                                        true_count += 1;
                                    } else {
                                        false_count += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            };
        }
        if true_count == 0 && false_count == 0 {
            return CalcResult::new_error(Error::VALUE, cell, "No booleans found".to_string());
        }
        CalcResult::Boolean(true_count % 2 == 1)
    }

    /// =SWITCH(expression, case1, value1, [case, value]*, [default])
    pub(crate) fn fn_switch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        // TODO add implicit intersection
        let expr = self.evaluate_node_in_context(&args[0], cell);
        if expr.is_error() {
            return expr;
        }

        // How many cases we have?
        // 3, 4 args -> 1 case
        let case_count = (args_count - 1) / 2;
        for case_index in 0..case_count {
            let case = self.evaluate_node_in_context(&args[2 * case_index + 1], cell);
            if case.is_error() {
                return case;
            }
            if compare_values(&expr, &case) == 0 {
                return self.evaluate_node_in_context(&args[2 * case_index + 2], cell);
            }
        }
        // None of the cases matched so we return the default
        // If there is an even number of args is the last one otherwise is #N/A
        if args_count % 2 == 0 {
            return self.evaluate_node_in_context(&args[args_count - 1], cell);
        }
        CalcResult::Error {
            error: Error::NA,
            origin: cell,
            message: "Did not find a match".to_string(),
        }
    }

    /// =IFS(condition1, value, [condition, value]*)
    pub(crate) fn fn_ifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        if args_count % 2 != 0 {
            // Missing value for last condition
            return CalcResult::new_args_number_error(cell);
        }
        let case_count = args_count / 2;
        for case_index in 0..case_count {
            let value = self.get_boolean(&args[2 * case_index], cell);
            match value {
                Ok(b) => {
                    if b {
                        return self.evaluate_node_in_context(&args[2 * case_index + 1], cell);
                    }
                }
                Err(s) => return s,
            }
        }
        CalcResult::Error {
            error: Error::NA,
            origin: cell,
            message: "Did not find a match".to_string(),
        }
    }
}
