mod and_or_xor_not;
mod bycol_byrow;
mod lambda;
mod r#let;
mod map_reduce;
mod scan;
mod switch;

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

// ── IF array helpers ──────────────────────────────────────────────────────────

/// A branch argument to IF: either a scalar (broadcast to all positions) or a
/// 2-D array (indexed by position; out-of-bounds → #N/A).
enum IfArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
}

fn if_arg_dims(arg: &IfArg) -> (usize, usize) {
    match arg {
        IfArg::Scalar(_) => (1, 1),
        IfArg::Array(arr) => (arr.len(), arr.first().map_or(0, |r| r.len())),
    }
}

fn if_arg_at(arg: &IfArg, row: usize, col: usize) -> ArrayNode {
    match arg {
        IfArg::Scalar(node) => node.clone(),
        IfArg::Array(arr) => match arr.get(row).and_then(|r| r.get(col)) {
            Some(n) => n.clone(),
            None => ArrayNode::Error(Error::NA),
        },
    }
}

fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        CalcResult::Lambda(_) | CalcResult::Range { .. } | CalcResult::Array(_) => {
            ArrayNode::Error(Error::VALUE)
        }
    }
}

/// Cast one element of a cond array to bool. Returns Err with the error node to
/// place in the output when the element is itself an error or unparseable string.
fn array_node_to_bool(node: &ArrayNode) -> Result<bool, ArrayNode> {
    match node {
        ArrayNode::Boolean(b) => Ok(*b),
        ArrayNode::Number(n) => Ok(*n != 0.0),
        ArrayNode::String(s) => {
            let lower = s.to_lowercase();
            if lower == "true" {
                Ok(true)
            } else if lower == "false" {
                Ok(false)
            } else {
                Err(ArrayNode::Error(Error::VALUE))
            }
        }
        ArrayNode::Empty => Ok(false),
        ArrayNode::Error(e) => Err(ArrayNode::Error(e.clone())),
    }
}

// ─────────────────────────────────────────────────────────────────────────────

impl<'a> Model<'a> {
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
        if args.len() != 2 && args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let has_else = args.len() == 3;
        let cond_result = self.evaluate_node_in_context(&args[0], cell);

        // Normalize cond to a 2-D array, or take the scalar early-return path.
        let cond_array: Vec<Vec<ArrayNode>> = match cond_result {
            error @ CalcResult::Error { .. } => return error,
            CalcResult::Range { left, right } => self.evaluate_range(left, right),
            CalcResult::Array(a) => a,
            other => {
                // Scalar cond: cast to bool and evaluate the chosen branch directly.
                // The branch result may itself be an array (it will spill).
                let b = match self.cast_to_bool(other, cell) {
                    Ok(b) => b,
                    Err(e) => return e,
                };
                return if b {
                    self.evaluate_node_in_context(&args[1], cell)
                } else if has_else {
                    self.evaluate_node_in_context(&args[2], cell)
                } else {
                    CalcResult::Boolean(false)
                };
            }
        };

        // Evaluate branches once and normalise each to a scalar or 2-D array.
        let true_result = self.evaluate_node_in_context(&args[1], cell);
        let true_arg = match true_result {
            CalcResult::Range { left, right } => IfArg::Array(self.evaluate_range(left, right)),
            CalcResult::Array(a) => IfArg::Array(a),
            other => IfArg::Scalar(calc_result_to_array_node(other)),
        };

        let false_arg: Option<IfArg> = if has_else {
            let r = self.evaluate_node_in_context(&args[2], cell);
            Some(match r {
                CalcResult::Range { left, right } => IfArg::Array(self.evaluate_range(left, right)),
                CalcResult::Array(a) => IfArg::Array(a),
                other => IfArg::Scalar(calc_result_to_array_node(other)),
            })
        } else {
            None
        };

        // Output size = largest extent across all array arguments.
        let cond_rows = cond_array.len();
        let cond_cols = cond_array.first().map_or(0, |r| r.len());
        let (true_rows, true_cols) = if_arg_dims(&true_arg);
        let (false_rows, false_cols) = false_arg.as_ref().map_or((1, 1), if_arg_dims);
        let max_rows = cond_rows.max(true_rows).max(false_rows);
        let max_cols = cond_cols.max(true_cols).max(false_cols);

        let mut output: Vec<Vec<ArrayNode>> = Vec::with_capacity(max_rows);
        for r in 0..max_rows {
            let mut row: Vec<ArrayNode> = Vec::with_capacity(max_cols);
            for c in 0..max_cols {
                let node = match cond_array.get(r).and_then(|row| row.get(c)) {
                    None => ArrayNode::Error(Error::NA),
                    Some(cond_node) => match array_node_to_bool(cond_node) {
                        Err(err_node) => err_node,
                        Ok(true) => if_arg_at(&true_arg, r, c),
                        Ok(false) => match &false_arg {
                            Some(fa) => if_arg_at(fa, r, c),
                            None => ArrayNode::Boolean(false),
                        },
                    },
                };
                row.push(node);
            }
            output.push(row);
        }

        CalcResult::Array(output)
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

    /// =IFS(condition1, value, [condition, value]*)
    pub(crate) fn fn_ifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        if !args_count.is_multiple_of(2) {
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
