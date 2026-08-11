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

/// The `lookup_array` and `return_array` arguments of XLOOKUP are
/// one-dimensional vectors that can come either from a stored range or from a
/// computed array (an array literal, or any expression that evaluates to one).
///
/// A range keeps its coordinates so that the return side is read lazily: only
/// the element the search lands on is evaluated.
enum Vector {
    Range {
        /// Top-left cell of the vector.
        start: CellReferenceIndex,
        len: i32,
        is_column: bool,
    },
    Array {
        values: Vec<CalcResult>,
        is_column: bool,
    },
}

impl Vector {
    fn len(&self) -> i32 {
        match self {
            Vector::Range { len, .. } => *len,
            Vector::Array { values, .. } => values.len() as i32,
        }
    }

    fn is_column(&self) -> bool {
        match self {
            Vector::Range { is_column, .. } | Vector::Array { is_column, .. } => *is_column,
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
        // lookup_array: a stored range or a computed array
        let lookup_array = match self.xlookup_vector(&args[1], cell, Error::NA, Error::ERROR) {
            Ok(v) => v,
            Err(e) => return e,
        };
        // return_array: a stored range or a computed array
        let return_array = match self.xlookup_vector(&args[2], cell, Error::VALUE, Error::VALUE) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if lookup_array.len() != return_array.len()
            || lookup_array.is_column() != return_array.is_column()
        {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Arrays must be of the same size".to_string(),
            };
        }
        let array = self.xlookup_vector_values(&lookup_array);
        let index = match search_mode {
            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                match linear_search(&lookup_value, &array, search_mode, match_mode) {
                    Some(index) => index as i32,
                    None => return if_not_found,
                }
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
                    None => return if_not_found,
                    Some(l) if l >= 0 => {
                        if match_mode == MatchMode::ExactMatch {
                            match array.get(l as usize) {
                                Some(value) if compare_values(value, &lookup_value) == 0 => l,
                                _ => return if_not_found,
                            }
                        } else if match_mode == MatchMode::ExactMatchSmaller
                            || match_mode == MatchMode::ExactMatchLarger
                        {
                            l
                        } else {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Cannot use wildcard in binary search".to_string(),
                            };
                        }
                    }
                    Some(_) => return if_not_found,
                }
            }
        };
        self.xlookup_vector_element(&return_array, index)
    }

    /// Evaluates an XLOOKUP array argument into a [`Vector`].
    ///
    /// `type_error` is returned when the argument is neither a range nor an
    /// array (XLOOKUP historically reports `#N/A` for `lookup_array` and
    /// `#VALUE!` for `return_array`); `shape_error` when it is two-dimensional.
    fn xlookup_vector(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        type_error: Error,
        shape_error: Error,
    ) -> Result<Vector, CalcResult> {
        match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Range { left, right } => {
                let is_column = if left.row == right.row {
                    false
                } else if left.column == right.column {
                    true
                } else {
                    return Err(CalcResult::Error {
                        error: shape_error,
                        origin: cell,
                        message: "Argument must be a vector".to_string(),
                    });
                };
                // A whole-column or whole-row reference is clamped to the used area
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
                            ));
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
                            ));
                        }
                    };
                }
                let len = if is_column {
                    row2 - left.row + 1
                } else {
                    column2 - left.column + 1
                };
                Ok(Vector::Range {
                    start: left,
                    len,
                    is_column,
                })
            }
            CalcResult::Array(rows) => {
                let is_column = if rows.len() == 1 {
                    false
                } else if rows.iter().all(|row| row.len() == 1) {
                    true
                } else {
                    return Err(CalcResult::Error {
                        error: shape_error,
                        origin: cell,
                        message: "Argument must be a vector".to_string(),
                    });
                };
                let values = rows
                    .iter()
                    .flatten()
                    .map(|node| array_node_to_calc_result(node, cell))
                    .collect();
                Ok(Vector::Array { values, is_column })
            }
            error @ CalcResult::Error { .. } => Err(error),
            _ => Err(CalcResult::Error {
                error: type_error,
                origin: cell,
                message: "Range expected".to_string(),
            }),
        }
    }

    /// Materializes every element of a vector; used for the search side.
    fn xlookup_vector_values(&mut self, vector: &Vector) -> Vec<CalcResult> {
        match vector {
            Vector::Range {
                start,
                len,
                is_column,
            } => {
                let mut result = Vec::new();
                for index in 0..*len {
                    let (row, column) = if *is_column {
                        (start.row + index, start.column)
                    } else {
                        (start.row, start.column + index)
                    };
                    result.push(self.evaluate_cell(CellReferenceIndex {
                        sheet: start.sheet,
                        row,
                        column,
                    }));
                }
                result
            }
            Vector::Array { values, .. } => values.clone(),
        }
    }

    /// Reads a single element of a vector, so that a whole-column return range
    /// is never materialized.
    fn xlookup_vector_element(&mut self, vector: &Vector, index: i32) -> CalcResult {
        match vector {
            Vector::Range {
                start, is_column, ..
            } => {
                let (row, column) = if *is_column {
                    (start.row + index, start.column)
                } else {
                    (start.row, start.column + index)
                };
                self.evaluate_cell(CellReferenceIndex {
                    sheet: start.sheet,
                    row,
                    column,
                })
            }
            Vector::Array { values, .. } => values
                .get(index as usize)
                .cloned()
                .unwrap_or(CalcResult::EmptyCell),
        }
    }
}
