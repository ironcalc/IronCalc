use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::{
    binary_search::{
        binary_search_descending_or_greater, binary_search_descending_or_smaller,
        binary_search_or_greater, binary_search_or_smaller,
    },
    lookup_and_reference::array_node_to_calc_result,
    util::{compare_values, from_wildcard_to_regex, result_matches_regex},
};

#[derive(PartialEq)]
enum SearchMode {
    StartAtFirstItem = 1,
    StartAtLastItem = -1,
    BinarySearchDescending = -2,
    BinarySearchAscending = 2,
}

#[derive(PartialEq)]
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
    /// lookup_array must be a column or row, a range or a computed array. return_array must be as
    /// long as lookup_array; if it has several columns (or rows) the whole row (or column) of the
    /// match is returned. Otherwise #VALUE! is returned
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
        // lookup_array: a range or an array with a single row or a single column.
        // `is_row_vector` is true when the values run down a column (so we move along rows).
        let (array, length, is_row_vector) = match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Range { left, right } => {
                let is_row_vector = if left.row == right.row {
                    false
                } else if left.column == right.column {
                    true
                } else {
                    // second argument must be a vector
                    return CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                };
                let length = if is_row_vector {
                    right.row - left.row + 1
                } else {
                    right.column - left.column + 1
                };
                let right = match self.clip_to_used_area(
                    left.sheet,
                    left.row,
                    left.column,
                    right.row,
                    right.column,
                ) {
                    Ok((row, column)) => CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    },
                    Err(message) => return CalcResult::new_error(Error::ERROR, cell, message),
                };
                (
                    self.prepare_array(&left, &right, is_row_vector),
                    length,
                    is_row_vector,
                )
            }
            CalcResult::Array(rows) => {
                let row_count = rows.len();
                let column_count = rows.first().map_or(0, |row| row.len());
                if row_count == 0 || column_count == 0 || (row_count != 1 && column_count != 1) {
                    return CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                }
                let array: Vec<CalcResult> = rows
                    .iter()
                    .flatten()
                    .map(|node| array_node_to_calc_result(node, cell))
                    .collect();
                let length = array.len() as i32;
                (array, length, row_count != 1)
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };

        // return_array: a range or an array as long as lookup_array. It can have several
        // columns (or rows), and then the whole row (or column) of the match is returned.
        let return_array = self.evaluate_node_in_context(&args[2], cell);
        let (return_rows, return_columns) = match &return_array {
            CalcResult::Range { left, right } => {
                (right.row - left.row + 1, right.column - left.column + 1)
            }
            CalcResult::Array(rows) => (
                rows.len() as i32,
                rows.first().map_or(0, |row| row.len()) as i32,
            ),
            CalcResult::Error { .. } => return return_array,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        let return_length = if is_row_vector {
            return_rows
        } else {
            return_columns
        };
        // A single-cell lookup_array has no direction, so it needs a single-cell return_array.
        let single_cell_mismatch = length == 1 && (return_rows != 1 || return_columns != 1);
        if return_length != length || single_cell_mismatch {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Arrays must be of the same size".to_string(),
            };
        }

        let index = match search_mode {
            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                linear_search(&lookup_value, &array, search_mode, match_mode)
            }
            SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending => {
                let index = if match_mode == MatchMode::ExactMatchLarger {
                    if search_mode == SearchMode::BinarySearchAscending {
                        binary_search_or_greater(&lookup_value, &array)
                    } else {
                        binary_search_descending_or_greater(&lookup_value, &array)
                    }
                } else if search_mode == SearchMode::BinarySearchAscending {
                    binary_search_or_smaller(&lookup_value, &array)
                } else {
                    binary_search_descending_or_smaller(&lookup_value, &array)
                };
                match index {
                    Some(l) => {
                        let l = l as usize;
                        if match_mode == MatchMode::WildcardMatch {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Cannot use wildcard in binary search".to_string(),
                            };
                        }
                        if match_mode == MatchMode::ExactMatch
                            && compare_values(&array[l], &lookup_value) != 0
                        {
                            None
                        } else {
                            Some(l)
                        }
                    }
                    None => None,
                }
            }
        };
        let Some(index) = index else {
            return if_not_found;
        };

        let index = index as i32;
        match return_array {
            CalcResult::Range { left, right } => {
                let (row1, column1, row2, column2) = if is_row_vector {
                    (
                        left.row + index,
                        left.column,
                        left.row + index,
                        right.column,
                    )
                } else {
                    (
                        left.row,
                        left.column + index,
                        right.row,
                        left.column + index,
                    )
                };
                if row1 == row2 && column1 == column2 {
                    self.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row: row1,
                        column: column1,
                    })
                } else {
                    CalcResult::Range {
                        left: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row1,
                            column: column1,
                        },
                        right: CellReferenceIndex {
                            sheet: left.sheet,
                            row: row2,
                            column: column2,
                        },
                    }
                }
            }
            CalcResult::Array(rows) => {
                let index = index as usize;
                let result: Vec<Vec<ArrayNode>> = if is_row_vector {
                    vec![rows[index].clone()]
                } else {
                    rows.iter().map(|row| vec![row[index].clone()]).collect()
                };
                if result.len() == 1 && result[0].len() == 1 {
                    array_node_to_calc_result(&result[0][0], cell)
                } else {
                    CalcResult::Array(result)
                }
            }
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Range expected".to_string(),
            },
        }
    }
}
