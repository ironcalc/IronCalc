use crate::cast::calc_result_to_array_node;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

use super::{
    binary_search::{
        binary_search_descending_or_greater, binary_search_descending_or_smaller,
        binary_search_or_greater, binary_search_or_smaller,
    },
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
    /// The `index`-th item of `return_array` along the lookup axis: a single value, or,
    /// when `return_array` is wider (taller) than a vector, the whole row (column) as an
    /// array that spills.
    fn xlookup_pick(
        &mut self,
        result_left: CellReferenceIndex,
        result_right: CellReferenceIndex,
        is_row_vector: bool,
        index: i32,
    ) -> CalcResult {
        let (first_row, first_column) = if is_row_vector {
            (result_left.row + index, result_left.column)
        } else {
            (result_left.row, result_left.column + index)
        };
        let (last_row, last_column) = if is_row_vector {
            (first_row, result_right.column)
        } else {
            (result_right.row, first_column)
        };
        if first_row == last_row && first_column == last_column {
            return self.evaluate_cell(CellReferenceIndex {
                sheet: result_left.sheet,
                row: first_row,
                column: first_column,
            });
        }
        let mut output = Vec::new();
        for row in first_row..=last_row {
            let mut line = Vec::new();
            for column in first_column..=last_column {
                line.push(calc_result_to_array_node(self.evaluate_cell(
                    CellReferenceIndex {
                        sheet: result_left.sheet,
                        row,
                        column,
                    },
                )));
            }
            output.push(line);
        }
        CalcResult::Array(output)
    }

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
        // lookup_array
        match self.evaluate_node_in_context(&args[1], cell) {
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
                // return array
                match self.evaluate_node_in_context(&args[2], cell) {
                    CalcResult::Range {
                        left: result_left,
                        right: result_right,
                    } => {
                        // return_array must span the lookup_array along its own axis. Across
                        // it, it can be wider: the whole matching row (or column) is returned
                        // and spills.
                        let same_size = if is_row_vector {
                            result_right.row - result_left.row == right.row - left.row
                        } else {
                            result_right.column - result_left.column == right.column - left.column
                        };
                        if !same_size {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Arrays must be of the same size".to_string(),
                            };
                        }
                        let mut row2 = right.row;
                        let row1 = left.row;
                        let mut column2 = right.column;
                        let column1 = left.column;

                        match self.clip_to_used_area(left.sheet, row1, column1, row2, column2) {
                            Ok((r, c)) => {
                                row2 = r;
                                column2 = c;
                            }
                            Err(message) => {
                                return CalcResult::new_error(Error::ERROR, cell, message)
                            }
                        }
                        let left = CellReferenceIndex {
                            sheet: left.sheet,
                            column: column1,
                            row: row1,
                        };
                        let right = CellReferenceIndex {
                            sheet: left.sheet,
                            column: column2,
                            row: row2,
                        };
                        match search_mode {
                            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                                let array = &self.prepare_array(&left, &right, is_row_vector);
                                match linear_search(&lookup_value, array, search_mode, match_mode) {
                                    Some(index) => self.xlookup_pick(
                                        result_left,
                                        result_right,
                                        is_row_vector,
                                        index as i32,
                                    ),
                                    None => if_not_found,
                                }
                            }
                            SearchMode::BinarySearchAscending
                            | SearchMode::BinarySearchDescending => {
                                let index = if match_mode == MatchMode::ExactMatchLarger {
                                    if search_mode == SearchMode::BinarySearchAscending {
                                        binary_search_or_greater(
                                            &lookup_value,
                                            &self.prepare_array(&left, &right, is_row_vector),
                                        )
                                    } else {
                                        binary_search_descending_or_greater(
                                            &lookup_value,
                                            &self.prepare_array(&left, &right, is_row_vector),
                                        )
                                    }
                                } else if search_mode == SearchMode::BinarySearchAscending {
                                    binary_search_or_smaller(
                                        &lookup_value,
                                        &self.prepare_array(&left, &right, is_row_vector),
                                    )
                                } else {
                                    binary_search_descending_or_smaller(
                                        &lookup_value,
                                        &self.prepare_array(&left, &right, is_row_vector),
                                    )
                                };
                                match index {
                                    None => if_not_found,
                                    Some(l) => {
                                        if match_mode == MatchMode::ExactMatch {
                                            let value = self.evaluate_cell(CellReferenceIndex {
                                                sheet: left.sheet,
                                                row: left.row + if is_row_vector { l } else { 0 },
                                                column: left.column
                                                    + if is_row_vector { 0 } else { l },
                                            });
                                            if compare_values(&value, &lookup_value) == 0 {
                                                self.xlookup_pick(
                                                    result_left,
                                                    result_right,
                                                    is_row_vector,
                                                    l,
                                                )
                                            } else {
                                                if_not_found
                                            }
                                        } else if match_mode == MatchMode::ExactMatchSmaller
                                            || match_mode == MatchMode::ExactMatchLarger
                                        {
                                            self.xlookup_pick(
                                                result_left,
                                                result_right,
                                                is_row_vector,
                                                l,
                                            )
                                        } else {
                                            CalcResult::Error {
                                                error: Error::VALUE,
                                                origin: cell,
                                                message: "Cannot use wildcard in binary search"
                                                    .to_string(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    error @ CalcResult::Error { .. } => error,
                    _ => CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Range expected".to_string(),
                    },
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
}
