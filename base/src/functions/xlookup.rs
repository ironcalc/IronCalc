use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult,
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

use super::{
    binary_search::{
        binary_search_descending_or_greater, binary_search_descending_or_smaller,
        binary_search_or_greater, binary_search_or_smaller,
    },
    util::{compare_values, from_wildcard_to_regex, result_matches_regex},
};

#[derive(PartialEq, Clone, Copy)]
enum SearchMode {
    StartAtFirstItem = 1,
    StartAtLastItem = -1,
    BinarySearchDescending = -2,
    BinarySearchAscending = 2,
}

#[derive(PartialEq, Clone, Copy)]
enum MatchMode {
    ExactMatchSmaller = -1,
    ExactMatch = 0,
    ExactMatchLarger = 1,
    WildcardMatch = 2,
}

// lookup_value in array, match_mode search_mode
fn linear_search(
    lookup_value: &CalcResult,
    array: &[CalcResult],
    search_mode: SearchMode,
    match_mode: MatchMode,
) -> Option<usize> {
    let length = array.len();

    match match_mode {
        MatchMode::ExactMatch => {
            // exact match
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };

                let value = &array[index];
                if compare_values(value, lookup_value) == 0 {
                    return Some(index);
                }
            }
            return None;
        }
        MatchMode::ExactMatchSmaller | MatchMode::ExactMatchLarger => {
            // exact match, if none found return the next smaller/larger item
            let mut found_index = 0;
            let mut approx = None;
            let m_mode = match_mode as i32;
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };

                let value = &array[index];
                let c = compare_values(value, lookup_value);
                if c == 0 {
                    return Some(index);
                } else if c == m_mode {
                    match approx {
                        None => {
                            approx = Some(value.clone());
                            found_index = index;
                        }
                        Some(ref p) => {
                            if compare_values(p, value) == m_mode {
                                approx = Some(value.clone());
                                found_index = index;
                            }
                        }
                    }
                }
            }
            if approx.is_none() {
                return None;
            } else {
                return Some(found_index);
            }
        }
        MatchMode::WildcardMatch => {
            let result_matches: Box<dyn Fn(&CalcResult) -> bool> =
                if let CalcResult::String(s) = &lookup_value {
                    if let Ok(reg) = from_wildcard_to_regex(&s.to_lowercase(), true) {
                        Box::new(move |x| result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else {
                    Box::new(move |x| compare_values(x, lookup_value) == 0)
                };
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };
                let value = &array[index];
                if result_matches(value) {
                    return Some(index);
                }
            }
        }
    }
    None
}

impl<'a> Model<'a> {
    /// The XLOOKUP function searches a range or an array, and then returns the item corresponding
    /// to the first match it finds. If no match exists, then XLOOKUP can return the closest (approximate) match.
    /// =XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])
    ///
    /// lookup_array and return_array must be column or row arrays and of the same dimension.
    /// Otherwise #VALUE! is returned
    /// [if_not_found]
    /// Where a valid match is not found, return the [if_not_found] text you supply.
    /// If a valid match is not found, and [if_not_found] is missing, #N/A is returned.
    ///
    /// [match_mode]
    /// Specify the match type:
    ///   *  0 - Exact match. If none found, return #N/A. This is the default.
    ///   * -1 - Exact match. If none found, return the next smaller item.
    ///   *  1 - Exact match. If none found, return the next larger item.
    ///   *  2 - A wildcard match where *, ?, and ~ have special meaning.
    ///
    /// [search_mode]
    /// Specify the search mode to use:
    ///   *  1 - Perform a search starting at the first item. This is the default.
    ///   * -1 - Perform a reverse search starting at the last item.
    ///   *  2 - Perform a binary search that relies on lookup_array being sorted
    ///      in ascending order. If not sorted, invalid results will be returned.
    ///   * -2 - Perform a binary search that relies on lookup_array being sorted
    ///     in descending order. If not sorted, invalid results will be returned.
    pub(crate) fn fn_xlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 3 || args.len() > 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        // Get optional arguments
        let if_not_found = if args.len() >= 4 {
            let v = self.evaluate_node_in_context(&args[3], cell);
            match v {
                CalcResult::EmptyArg => CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Not found".to_string(),
                },
                _ => v,
            }
        } else {
            // default
            CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            }
        };
        let match_mode = if args.len() >= 5 {
            match self.get_number(&args[4], cell) {
                Ok(c) => match c.floor() as i32 {
                    -1 => MatchMode::ExactMatchSmaller,
                    1 => MatchMode::ExactMatchLarger,
                    0 => MatchMode::ExactMatch,
                    2 => MatchMode::WildcardMatch,
                    _ => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            MatchMode::ExactMatch
        };
        let search_mode = if args.len() == 6 {
            match self.get_number(&args[5], cell) {
                Ok(c) => match c.floor() as i32 {
                    1 => SearchMode::StartAtFirstItem,
                    -1 => SearchMode::StartAtLastItem,
                    -2 => SearchMode::BinarySearchDescending,
                    2 => SearchMode::BinarySearchAscending,
                    _ => {
                        return CalcResult::Error {
                            error: Error::ERROR,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            SearchMode::StartAtFirstItem
        };
        // Materialise lookup_array (args[1]) and return_array (args[2]) into flat
        // vectors so a range reference or an in-formula array constant (e.g.
        // `A1:A2&"|"&B1:B2`) is handled the same way. See issue #1338.
        let lookup_array = match self.xlookup_vector(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let return_array = match self.xlookup_vector(&args[2], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if lookup_array.len() != return_array.len() {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Arrays must be of the same size".to_string(),
            };
        }
        match search_mode {
            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                match linear_search(&lookup_value, &lookup_array, search_mode, match_mode) {
                    Some(index) => return_array[index].clone(),
                    None => if_not_found,
                }
            }
            SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending => {
                let index = if match_mode == MatchMode::ExactMatchLarger {
                    if search_mode == SearchMode::BinarySearchAscending {
                        binary_search_or_greater(&lookup_value, &lookup_array)
                    } else {
                        binary_search_descending_or_greater(&lookup_value, &lookup_array)
                    }
                } else if search_mode == SearchMode::BinarySearchAscending {
                    binary_search_or_smaller(&lookup_value, &lookup_array)
                } else {
                    binary_search_descending_or_smaller(&lookup_value, &lookup_array)
                };
                match index {
                    None => if_not_found,
                    Some(l) => {
                        let l = l as usize;
                        if match_mode == MatchMode::ExactMatch {
                            if compare_values(&lookup_array[l], &lookup_value) == 0 {
                                return_array[l].clone()
                            } else {
                                if_not_found
                            }
                        } else if match_mode == MatchMode::ExactMatchSmaller
                            || match_mode == MatchMode::ExactMatchLarger
                        {
                            return_array[l].clone()
                        } else {
                            CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Cannot use wildcard in binary search".to_string(),
                            }
                        }
                    }
                }
            }
        }
    }

    /// Evaluates `node` and materialises it into a flat vector of values for the
    /// XLOOKUP family. The argument must be a single-row or single-column range
    /// reference, or an in-formula array constant of the same shape (issue #1338).
    fn xlookup_vector(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<CalcResult>, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => {
                let is_row_vector = if left.row == right.row {
                    false
                } else if left.column == right.column {
                    true
                } else {
                    return Err(CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    });
                };
                let mut row2 = right.row;
                let mut column2 = right.column;
                if left.row == 1 && row2 == LAST_ROW {
                    row2 = match self.workbook.worksheet(left.sheet) {
                        Ok(s) => s.dimension().max_row,
                        Err(_) => {
                            return Err(CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            ))
                        }
                    };
                }
                if left.column == 1 && column2 == LAST_COLUMN {
                    column2 = match self.workbook.worksheet(left.sheet) {
                        Ok(s) => s.dimension().max_column,
                        Err(_) => {
                            return Err(CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            ))
                        }
                    };
                }
                let right = CellReferenceIndex {
                    sheet: left.sheet,
                    row: row2,
                    column: column2,
                };
                Ok(self.prepare_array(&left, &right, is_row_vector))
            }
            CalcResult::Array(rows) => {
                let n_rows = rows.len();
                let n_cols = rows.first().map(|r| r.len()).unwrap_or(0);
                let is_row_vec = n_rows == 1;
                let is_col_vec = n_cols == 1;
                if n_rows == 0 || n_cols == 0 || (!is_row_vec && !is_col_vec) {
                    return Err(CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    });
                }
                Ok(rows
                    .iter()
                    .flatten()
                    .map(|array_node| array_node_to_calc_result(array_node, cell))
                    .collect())
            }
            error @ CalcResult::Error { .. } => Err(error),
            _ => Err(CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Range expected".to_string(),
            }),
        }
    }
}

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
