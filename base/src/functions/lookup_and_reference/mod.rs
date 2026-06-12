use crate::calc_result::Range;
use crate::cast::{calc_result_to_array_node, NumberOrArray};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::implicit_intersection::implicit_intersection;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
    utils::ParsedReference,
};

use super::util::{compare_values, from_wildcard_to_regex, result_matches_regex, values_are_equal};

mod address_areas;
mod choosecols_chooserows;
mod drop_take;
mod expand;
mod hstack_vstack;
mod tocol_torow;
mod transpose;
mod trimrange;
mod wrapcols_wraprows;
mod xmatch;

/// Converts a single array element into a scalar [`CalcResult`].
fn array_node_to_calc_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(n) => CalcResult::Number(*n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::Error {
            error: e.clone(),
            origin: cell,
            message: "".to_string(),
        },
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// Resolves the effective (row, column) indices for INDEX, where `0` means "the
/// whole row/column" (used to spill an entire row, column or array).
///
/// In the two-argument form `INDEX(array, num)` the single index is interpreted
/// based on the shape of the source: for a single-row source it is a column
/// index (all rows), otherwise it is a row index (all columns — i.e. a whole row
/// for a 2-D source, or a single element for a column vector).
fn index_effective_indices(
    num_rows: usize,
    row_num: usize,
    col_num: usize,
    two_arg: bool,
) -> (usize, usize) {
    if two_arg {
        if num_rows == 1 {
            (0, row_num)
        } else {
            (row_num, 0)
        }
    } else {
        (row_num, col_num)
    }
}

/// Extracts the requested cell, row, column or whole array from `arr`.
/// `row_eff`/`col_eff` of `0` select every row/column respectively. A selection
/// that resolves to a single cell is returned as a scalar; otherwise an array.
fn index_from_array(
    arr: &[Vec<ArrayNode>],
    row_eff: usize,
    col_eff: usize,
    cell: CellReferenceIndex,
) -> CalcResult {
    let num_rows = arr.len();
    let num_cols = arr[0].len();
    if row_eff > num_rows || col_eff > num_cols {
        return CalcResult::Error {
            error: Error::REF,
            origin: cell,
            message: "Wrong reference".to_string(),
        };
    }
    let (row_start, row_end) = if row_eff == 0 {
        (1, num_rows)
    } else {
        (row_eff, row_eff)
    };
    let (col_start, col_end) = if col_eff == 0 {
        (1, num_cols)
    } else {
        (col_eff, col_eff)
    };
    if row_start == row_end && col_start == col_end {
        return array_node_to_calc_result(&arr[row_start - 1][col_start - 1], cell);
    }
    let mut result = Vec::with_capacity(row_end - row_start + 1);
    for r in row_start..=row_end {
        let mut data_row = Vec::with_capacity(col_end - col_start + 1);
        for c in col_start..=col_end {
            data_row.push(arr[r - 1][c - 1].clone());
        }
        result.push(data_row);
    }
    CalcResult::Array(result)
}

impl<'a> Model<'a> {
    // INDEX(array, row_num, [column_num])
    // INDEX(range, row_num, [column_num], [area_num])
    // At the moment IronCalc does not support references with multiple areas,
    // so area_num = 1 (or missing).
    pub(crate) fn fn_index(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        // area_num (4th argument). We only support references with a single area,
        // so anything other than 1 is out of range.
        if args.len() == 4 {
            let area_num = match self.get_number(&args[3], cell) {
                Ok(f) => f.floor() as i64,
                Err(s) => return s,
            };
            if area_num < 1 {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Argument must be >= 1".to_string(),
                };
            }
            if area_num != 1 {
                return CalcResult::Error {
                    error: Error::REF,
                    origin: cell,
                    message: "Area out of range".to_string(),
                };
            }
        }
        // A missing or empty argument evaluates to 0, which means "the whole
        // row/column". A negative index is an error.
        let row_num = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if row_num < 0.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Argument must be >= 0".to_string(),
            };
        }
        let two_arg = args.len() == 2;
        let col_num = if two_arg {
            0.0
        } else {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        };
        if col_num < 0.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Argument must be >= 0".to_string(),
            };
        }
        let row_num = row_num.floor() as usize;
        let col_num = col_num.floor() as usize;

        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Range { left, right } => {
                let num_rows = (right.row - left.row + 1) as usize;
                let (row_eff, col_eff) =
                    index_effective_indices(num_rows, row_num, col_num, two_arg);
                let num_cols = (right.column - left.column + 1) as usize;
                if row_eff > num_rows || col_eff > num_cols {
                    return CalcResult::Error {
                        error: Error::REF,
                        origin: cell,
                        message: "Wrong reference".to_string(),
                    };
                }
                let (row_start, row_end) = if row_eff == 0 {
                    (left.row, right.row)
                } else {
                    let r = left.row + row_eff as i32 - 1;
                    (r, r)
                };
                let (col_start, col_end) = if col_eff == 0 {
                    (left.column, right.column)
                } else {
                    let c = left.column + col_eff as i32 - 1;
                    (c, c)
                };
                if row_start == row_end && col_start == col_end {
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row: row_start,
                        column: col_start,
                    })
                } else {
                    CalcResult::Range {
                        left: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row_start,
                            column: col_start,
                        },
                        right: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row_end,
                            column: col_end,
                        },
                    }
                }
            }
            CalcResult::Array(arr) => {
                if arr.is_empty() || arr[0].is_empty() {
                    return CalcResult::Error {
                        error: Error::REF,
                        origin: cell,
                        message: "Empty array".to_string(),
                    };
                }
                let (row_eff, col_eff) =
                    index_effective_indices(arr.len(), row_num, col_num, two_arg);
                index_from_array(&arr, row_eff, col_eff, cell)
            }
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Expecting a Range".to_string(),
            },
        }
    }

    //     MATCH(lookup_value, lookup_array, [match_type])
    // The MATCH function syntax has the following arguments:
    //   * lookup_value    Required. The value that you want to match in lookup_array.
    //                     The lookup_value argument can be a value (number, text, or logical value)
    //                     or a cell reference to a number, text, or logical value.
    //   * lookup_array    Required. The range of cells being searched.
    //   * match_type      Optional. The number -1, 0, or 1.
    //                     The match_type argument specifies how Excel matches lookup_value
    //                     with values in lookup_array. The default value for this argument is 1.
    // NOTE: Please read the caveat above in binary search
    pub(crate) fn fn_match(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.evaluate_node_in_context(&args[0], cell);
        if target.is_error() {
            return target;
        }
        if matches!(target, CalcResult::EmptyCell) {
            return CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Cannot match empty cell".to_string(),
            };
        }
        let match_type = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(v) => v as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let match_range = self.evaluate_node_in_context(&args[1], cell);

        match match_range {
            CalcResult::Range { left, right } => {
                match match_type {
                    -1 => {
                        // We apply binary search leftmost for value in the range
                        let is_row_vector;
                        if left.row == right.row {
                            is_row_vector = false;
                        } else if left.column == right.column {
                            is_row_vector = true;
                        } else {
                            // second argument must be a vector
                            return CalcResult::Error {
                                error: Error::ERROR,
                                origin: cell,
                                message: "Argument must be a vector".to_string(),
                            };
                        }
                        let n = if is_row_vector {
                            right.row - left.row
                        } else {
                            right.column - left.column
                        } + 1;
                        let mut l = 0;
                        let mut r = n;
                        while l < r {
                            let m = (l + r) / 2;
                            let row;
                            let column;
                            if is_row_vector {
                                row = left.row + m;
                                column = left.column;
                            } else {
                                column = left.column + m;
                                row = left.row;
                            }
                            let value = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            });

                            if compare_values(&value, &target) >= 0 {
                                l = m + 1;
                            } else {
                                r = m;
                            }
                        }
                        // r is the number of elements less than target in the vector
                        // If target is less than the minimum return #N/A
                        if l == 0 {
                            return CalcResult::Error {
                                error: Error::NA,
                                origin: cell,
                                message: "Not found".to_string(),
                            };
                        }
                        // Now l points to the leftmost element
                        CalcResult::Number(l as f64)
                    }
                    0 => {
                        // We apply linear search
                        let is_row_vector;
                        if left.row == right.row {
                            is_row_vector = false;
                        } else if left.column == right.column {
                            is_row_vector = true;
                        } else {
                            // second argument must be a vector
                            return CalcResult::Error {
                                error: Error::ERROR,
                                origin: cell,
                                message: "Argument must be a vector".to_string(),
                            };
                        }
                        let n = if is_row_vector {
                            right.row - left.row
                        } else {
                            right.column - left.column
                        } + 1;
                        let result_matches: Box<dyn Fn(&CalcResult) -> bool> =
                            if let CalcResult::String(s) = &target {
                                if let Ok(reg) = from_wildcard_to_regex(&s.to_lowercase(), true) {
                                    Box::new(move |x| result_matches_regex(x, &reg))
                                } else {
                                    Box::new(move |_| false)
                                }
                            } else {
                                Box::new(move |x| values_are_equal(x, &target))
                            };
                        for l in 0..n {
                            let row;
                            let column;
                            if is_row_vector {
                                row = left.row + l;
                                column = left.column;
                            } else {
                                column = left.column + l;
                                row = left.row;
                            }
                            let value = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            });
                            if result_matches(&value) {
                                return CalcResult::Number(l as f64 + 1.0);
                            }
                        }
                        CalcResult::Error {
                            error: Error::NA,
                            origin: cell,
                            message: "Not found".to_string(),
                        }
                    }
                    _ => {
                        // l is the number of elements less than target in the vector
                        let is_row_vector;
                        if left.row == right.row {
                            is_row_vector = false;
                        } else if left.column == right.column {
                            is_row_vector = true;
                        } else {
                            // second argument must be a vector
                            return CalcResult::Error {
                                error: Error::ERROR,
                                origin: cell,
                                message: "Argument must be a vector".to_string(),
                            };
                        }
                        let l = self.binary_search(&target, &left, &right, is_row_vector);
                        if l == -2 {
                            return CalcResult::Error {
                                error: Error::NA,
                                origin: cell,
                                message: "Not found".to_string(),
                            };
                        }

                        CalcResult::Number(l as f64 + 1.0)
                    }
                }
            }
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Invalid".to_string(),
            },
        }
    }

    /// HLOOKUP(lookup_value, table_array, row_index, [is_sorted])
    /// We look for `lookup_value` in the first row of table array
    /// We return the value in row `row_index` of the same column in `table_array`
    /// `is_sorted` is true by default and assumes that values in first row are ordered
    pub(crate) fn fn_hlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        let row_index = match self.get_number(&args[2], cell) {
            Ok(v) => v.floor() as i32,
            Err(s) => return s,
        };
        let is_sorted = if args.len() == 4 {
            match self.get_boolean(&args[3], cell) {
                Ok(v) => v,
                Err(s) => return s,
            }
        } else {
            true
        };
        let range = self.evaluate_node_in_context(&args[1], cell);
        match range {
            CalcResult::Range { left, right } => {
                if is_sorted {
                    // This assumes the values in row are in order
                    let l = self.binary_search(&lookup_value, &left, &right, false);
                    if l == -2 {
                        return CalcResult::Error {
                            error: Error::NA,
                            origin: cell,
                            message: "Not found".to_string(),
                        };
                    }
                    let row = left.row + row_index - 1;
                    let column = left.column + l;
                    if row > right.row {
                        return CalcResult::Error {
                            error: Error::REF,
                            origin: cell,
                            message: "Invalid reference".to_string(),
                        };
                    }
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    })
                } else {
                    // Linear search for exact match
                    let n = right.column - left.column + 1;
                    let row = left.row + row_index - 1;
                    if row > right.row {
                        return CalcResult::Error {
                            error: Error::REF,
                            origin: cell,
                            message: "Invalid reference".to_string(),
                        };
                    }
                    let result_matches: Box<dyn Fn(&CalcResult) -> bool> =
                        if let CalcResult::String(s) = &lookup_value {
                            if let Ok(reg) = from_wildcard_to_regex(&s.to_lowercase(), true) {
                                Box::new(move |x| result_matches_regex(x, &reg))
                            } else {
                                Box::new(move |_| false)
                            }
                        } else {
                            Box::new(move |x| compare_values(x, &lookup_value) == 0)
                        };
                    for l in 0..n {
                        let value = self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row,
                            column: left.column + l,
                        });
                        if result_matches(&value) {
                            return self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column: left.column + l,
                            });
                        }
                    }
                    CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    }
                }
            }
            error @ CalcResult::Error { .. } => error,
            CalcResult::String(_) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Range expected".to_string(),
            },
            _ => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            },
        }
    }

    /// VLOOKUP(lookup_value, table_array, row_index, [is_sorted])
    /// We look for `lookup_value` in the first column of table array
    /// We return the value in column `column_index` of the same row in `table_array`
    /// `is_sorted` is true by default and assumes that values in first column are ordered
    pub(crate) fn fn_vlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 4 || args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        let column_index = match self.get_number(&args[2], cell) {
            Ok(v) => v.floor() as i32,
            Err(s) => return s,
        };
        let is_sorted = if args.len() == 4 {
            match self.get_boolean(&args[3], cell) {
                Ok(v) => v,
                Err(s) => return s,
            }
        } else {
            true
        };
        let range = self.evaluate_node_in_context(&args[1], cell);
        match range {
            CalcResult::Range { left, right } => {
                if is_sorted {
                    // This assumes the values in column are in order
                    let l = self.binary_search(&lookup_value, &left, &right, true);
                    if l == -2 {
                        return CalcResult::Error {
                            error: Error::NA,
                            origin: cell,
                            message: "Not found".to_string(),
                        };
                    }
                    let row = left.row + l;
                    let column = left.column + column_index - 1;
                    if column > right.column {
                        return CalcResult::Error {
                            error: Error::REF,
                            origin: cell,
                            message: "Invalid reference".to_string(),
                        };
                    }
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    })
                } else {
                    // Linear search for exact match
                    let n = right.row - left.row + 1;
                    let column = left.column + column_index - 1;
                    if column > right.column {
                        return CalcResult::Error {
                            error: Error::REF,
                            origin: cell,
                            message: "Invalid reference".to_string(),
                        };
                    }
                    let result_matches: Box<dyn Fn(&CalcResult) -> bool> =
                        if let CalcResult::String(s) = &lookup_value {
                            if let Ok(reg) = from_wildcard_to_regex(&s.to_lowercase(), true) {
                                Box::new(move |x| result_matches_regex(x, &reg))
                            } else {
                                Box::new(move |_| false)
                            }
                        } else {
                            Box::new(move |x| compare_values(x, &lookup_value) == 0)
                        };
                    for l in 0..n {
                        let value = self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row + l,
                            column: left.column,
                        });
                        if result_matches(&value) {
                            return self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row: left.row + l,
                                column,
                            });
                        }
                    }
                    CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    }
                }
            }
            error @ CalcResult::Error { .. } => error,
            CalcResult::String(_) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Range expected".to_string(),
            },
            _ => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            },
        }
    }

    // LOOKUP(lookup_value, lookup_vector, [result_vector])
    // Important: The values in lookup_vector must be placed in ascending order:
    // ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
    // otherwise, LOOKUP might not return the correct value.
    // Uppercase and lowercase text are equivalent.
    // TODO: Implement the other form of INDEX:
    // INDEX(reference, row_num, [column_num], [area_num])
    // NOTE: Please read the caveat above in binary search
    pub(crate) fn fn_lookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 3 || args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let target = self.evaluate_node_in_context(&args[0], cell);
        if target.is_error() {
            return target;
        }
        let value = self.evaluate_node_in_context(&args[1], cell);
        match value {
            CalcResult::Range { left, right } => {
                let is_row_vector;
                if left.row == right.row {
                    is_row_vector = false;
                } else if left.column == right.column {
                    is_row_vector = true;
                } else {
                    // second argument must be a vector
                    return CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                }
                let l = self.binary_search(&target, &left, &right, is_row_vector);
                if l == -2 {
                    return CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    };
                }

                if args.len() == 3 {
                    let target_range = self.evaluate_node_in_context(&args[2], cell);
                    match target_range {
                        CalcResult::Range {
                            left: l1,
                            right: _r1,
                        } => {
                            let row;
                            let column;
                            if is_row_vector {
                                row = l1.row + l;
                                column = l1.column;
                            } else {
                                column = l1.column + l;
                                row = l1.row;
                            }
                            self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            })
                        }
                        error @ CalcResult::Error { .. } => error,
                        _ => CalcResult::Error {
                            error: Error::NA,
                            origin: cell,
                            message: "Range expected".to_string(),
                        },
                    }
                } else {
                    let row;
                    let column;
                    if is_row_vector {
                        row = left.row + l;
                        column = left.column;
                    } else {
                        column = left.column + l;
                        row = left.row;
                    }
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    })
                }
            }
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            },
        }
    }

    // ROW([reference])
    // If reference is not present returns the row of the present cell.
    // Otherwise returns the row number of reference
    pub(crate) fn fn_row(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if args.is_empty() {
            return CalcResult::Number(cell.row as f64);
        }
        match self.get_reference(&args[0], cell) {
            Ok(c) => CalcResult::Number(c.left.row as f64),
            Err(s) => s,
        }
    }

    // ROWS(range)
    // Returns the number of rows in range
    pub(crate) fn fn_rows(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_reference(&args[0], cell) {
            Ok(c) => CalcResult::Number((c.right.row - c.left.row + 1) as f64),
            Err(s) => s,
        }
    }

    // COLUMN([reference])
    // If reference is not present returns the column of the present cell.
    // Otherwise returns the column number of reference
    pub(crate) fn fn_column(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if args.is_empty() {
            return CalcResult::Number(cell.column as f64);
        }

        match self.get_reference(&args[0], cell) {
            Ok(range) => CalcResult::Number(range.left.column as f64),
            Err(s) => s,
        }
    }

    /// CHOOSE(index_num, value1, [value2], ...)
    /// Uses index_num to return a value from the list of value arguments.
    pub(crate) fn fn_choose(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // The index argument may be a single number or an array/range. In the array
        // case we broadcast: each index element selects the matching value argument,
        // collapsed to a single scalar (the result has the shape of the index array).
        match self.get_number_or_array(&args[0], cell) {
            Ok(NumberOrArray::Number(index_num)) => {
                let index_num = index_num as usize;
                if index_num < 1 || index_num > (args.len() - 1) {
                    return CalcResult::new_error(Error::VALUE, cell, "Invalid index".to_string());
                }
                self.evaluate_node_with_reference(&args[index_num], cell)
            }
            Ok(NumberOrArray::Array(index_array)) => {
                let result = index_array
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|node| self.choose_element(args, node, cell))
                            .collect()
                    })
                    .collect();
                CalcResult::Array(result)
            }
            Err(calc_err) => calc_err,
        }
    }

    /// Selects the CHOOSE value argument for a single index element and collapses
    /// it to one array cell. A chosen range is reduced by implicit intersection.
    fn choose_element(
        &mut self,
        args: &[Node],
        index_node: &ArrayNode,
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        let index_num = match index_node {
            ArrayNode::Number(n) => *n,
            ArrayNode::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            ArrayNode::Empty => 0.0,
            ArrayNode::String(s) => match self.cast_number(s) {
                Some(f) => f,
                None => return ArrayNode::Error(Error::VALUE),
            },
            ArrayNode::Error(e) => return ArrayNode::Error(e.clone()),
        } as usize;

        if index_num < 1 || index_num > (args.len() - 1) {
            return ArrayNode::Error(Error::VALUE);
        }

        let value = match self.evaluate_node_with_reference(&args[index_num], cell) {
            CalcResult::Range { left, right } => {
                match implicit_intersection(&cell, &Range { left, right }) {
                    Some(reference) => self.evaluate_cell(reference),
                    None => CalcResult::new_error(Error::VALUE, cell, "Invalid range".to_string()),
                }
            }
            other => other,
        };
        calc_result_to_array_node(value)
    }

    // COLUMNS(range)
    // Returns the number of columns in range
    pub(crate) fn fn_columns(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_reference(&args[0], cell) {
            Ok(c) => CalcResult::Number((c.right.column - c.left.column + 1) as f64),
            Err(s) => s,
        }
    }

    // INDIRECT(ref_tex)
    // Returns the reference specified by 'ref_text'
    pub(crate) fn fn_indirect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let value = self.get_string(&args[0], cell);
        match value {
            Ok(s) => {
                if args.len() == 2 {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Not implemented".to_string(),
                    };
                }

                let parsed_reference = ParsedReference::parse_reference_formula(
                    Some(cell.sheet),
                    &s,
                    self.locale,
                    |name| self.get_sheet_index_by_name(name),
                );

                let parsed_reference = match parsed_reference {
                    Ok(reference) => reference,
                    Err(message) => {
                        return CalcResult::Error {
                            error: Error::REF,
                            origin: cell,
                            message,
                        };
                    }
                };

                match parsed_reference {
                    ParsedReference::CellReference(reference) => CalcResult::Range {
                        left: reference,
                        right: reference,
                    },
                    ParsedReference::Range(left, right) => CalcResult::Range { left, right },
                }
            }
            Err(v) => v,
        }
    }

    // OFFSET(reference, rows, cols, [height], [width])
    // Returns a reference to a range that is a specified number of rows and columns from a cell or range of cells.
    // The reference that is returned can be a single cell or a range of cells.
    // You can specify the number of rows and the number of columns to be returned.
    pub(crate) fn fn_offset(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let l = args.len();
        if !(3..=5).contains(&l) {
            return CalcResult::new_args_number_error(cell);
        }
        let reference = match self.get_reference(&args[0], cell) {
            Ok(c) => c,
            Err(s) => return s,
        };
        let rows = match self.get_number(&args[1], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        let cols = match self.get_number(&args[2], cell) {
            Ok(c) => {
                if c < 0.0 {
                    c.ceil() as i32
                } else {
                    c.floor() as i32
                }
            }
            Err(s) => return s,
        };
        let row_start = reference.left.row + rows;
        let column_start = reference.left.column + cols;
        let width;
        let height;
        if l == 4 {
            height = match self.get_number(&args[3], cell) {
                Ok(c) => {
                    if c < 1.0 {
                        c.ceil() as i32 - 1
                    } else {
                        c.floor() as i32 - 1
                    }
                }
                Err(s) => return s,
            };
            width = reference.right.column - reference.left.column;
        } else if l == 5 {
            height = match self.get_number(&args[3], cell) {
                Ok(c) => {
                    if c < 1.0 {
                        c.ceil() as i32 - 1
                    } else {
                        c.floor() as i32 - 1
                    }
                }
                Err(s) => return s,
            };
            width = match self.get_number(&args[4], cell) {
                Ok(c) => {
                    if c < 1.0 {
                        c.ceil() as i32 - 1
                    } else {
                        c.floor() as i32 - 1
                    }
                }
                Err(s) => return s,
            };
        } else {
            width = reference.right.column - reference.left.column;
            height = reference.right.row - reference.left.row;
        }
        // This is what Excel does
        if width == -1 || height == -1 {
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        }
        // NB: Excel documentation says that negative values of width and height are not valid
        // but in practice they are valid. We follow the documentation and not Excel
        if width < -1 || height < -1 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "width and height cannot be negative".to_string(),
            };
        }

        let column_end = column_start + width;
        let row_end = row_start + height;
        if row_start < 1 || row_end > LAST_ROW || column_start < 1 || column_end > LAST_COLUMN {
            return CalcResult::Error {
                error: Error::REF,
                origin: cell,
                message: "Invalid reference".to_string(),
            };
        }
        let left = CellReferenceIndex {
            sheet: reference.left.sheet,
            row: row_start,
            column: column_start,
        };
        let right = CellReferenceIndex {
            sheet: reference.right.sheet,
            row: row_end,
            column: column_end,
        };
        CalcResult::Range { left, right }
    }

    // FORMULATEXT(reference)
    // Returns a formula as a string. Two differences with Excel:
    // - It returns the formula in English
    // - It formats the formula without spaces between elements
    pub(crate) fn fn_formulatext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        if let CalcResult::Range { left, right } = self.evaluate_node_with_reference(&args[0], cell)
        {
            if left.sheet != right.sheet {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "3D ranges not supported".to_string(),
                };
            }
            if left.row != right.row || left.column != right.column {
                // FIXME: Implicit intersection or dynamic arrays
                return CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "argument must be a reference to a single cell".to_string(),
                };
            }
            if let Ok(Some(f)) = self.get_english_cell_formula(left.sheet, left.row, left.column) {
                CalcResult::String(f)
            } else {
                CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Reference does not have a formula".to_string(),
                }
            }
        } else {
            CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Argument must be a reference".to_string(),
            }
        }
    }
}
