use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

use super::util::compare_values;

impl Model {
    pub(crate) fn fn_true(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            CalcResult::Boolean(true)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    pub(crate) fn fn_false(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            CalcResult::Boolean(false)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

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
        self.logical_nary(
            args,
            cell,
            |acc, value| acc.unwrap_or(true) && value,
            Some(false),
        )
    }

    pub(crate) fn fn_or(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.logical_nary(
            args,
            cell,
            |acc, value| acc.unwrap_or(false) || value,
            Some(true),
        )
    }

    pub(crate) fn fn_xor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.logical_nary(args, cell, |acc, value| acc.unwrap_or(false) ^ value, None)
    }

    /// Base function for AND, OR, XOR. These are all n-ary functions that perform a boolean operation on a series of
    /// boolean values. These boolean values are sourced from `args`. Note that there is not a 1-1 relationship between
    /// arguments and boolean values evaluated (see how Ranges are handled for example).
    ///
    /// Each argument in `args` is evaluated and the resulting value is interpreted as a boolean as follows:
    /// - Boolean: The value is used directly.
    /// - Number: 0 is FALSE, all other values are TRUE.
    /// - Range: Each cell in the range is evaluated as if they were individual arguments with some caveats
    /// - Empty arg: FALSE
    /// - Empty cell & String: Ignored, behaves exactly like the argument wasn't passed in at all
    /// - Error: Propagated
    ///
    /// If no arguments are provided, or all arguments are ignored, the function returns a #VALUE! error
    ///
    /// **`fold_fn`:** The function that combines the running result with the next value boolean value. The running result
    /// starts as `None`.
    ///
    /// **`short_circuit_value`:** If the running result reaches `short_circuit_value`, the function returns early.
    fn logical_nary(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        fold_fn: fn(Option<bool>, bool) -> bool,
        short_circuit_value: Option<bool>,
    ) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = None;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Boolean(value) => result = Some(fold_fn(result, value)),
                CalcResult::Number(value) => result = Some(fold_fn(result, value != 0.0)),
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
                                CalcResult::Boolean(value) => result = Some(fold_fn(result, value)),
                                CalcResult::Number(value) => {
                                    result = Some(fold_fn(result, value != 0.0))
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::EmptyArg => {} // unreachable
                                CalcResult::Range { .. }
                                | CalcResult::String { .. }
                                | CalcResult::EmptyCell => {}
                                CalcResult::Array(_) => {
                                    return CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    }
                                }
                            }
                            if let (Some(current_result), Some(short_circuit_value)) =
                                (result, short_circuit_value)
                            {
                                if current_result == short_circuit_value {
                                    return CalcResult::Boolean(current_result);
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::EmptyArg => result = Some(result.unwrap_or(false)),
                // Strings are ignored unless they are "TRUE" or "FALSE" (case insensitive). EXCEPT if the string value
                // comes from a reference, in which case it is always ignored regardless of its value.
                CalcResult::String(..) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        if let Ok(f) = self.get_boolean(arg, cell) {
                            result = Some(fold_fn(result, f));
                        }
                    }
                }
                // References to empty cells are ignored. If all args are ignored the result is #VALUE!
                CalcResult::EmptyCell => {}
                CalcResult::Array(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            }

            if let (Some(current_result), Some(short_circuit_value)) = (result, short_circuit_value)
            {
                if current_result == short_circuit_value {
                    return CalcResult::Boolean(current_result);
                }
            }
        }

        if let Some(result) = result {
            CalcResult::Boolean(result)
        } else {
            CalcResult::new_error(
                Error::VALUE,
                cell,
                "No logical values in argument list".to_string(),
            )
        }
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
