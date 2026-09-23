use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
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
        // lookup_array: one row or one column, from cells or a list of values
        let (lookup, lookup_is_column) = match self.xlookup_vector(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let n = lookup.len();
        // return_array: as many rows (or columns) as lookup_array; the whole
        // matching row (or column) is returned, so it may spill
        let ret = match self.evaluate_node_in_context(&args[2], cell) {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return CalcResult::new_error(Error::VALUE, cell, "3D ranges".to_string());
                }
                let mut right = right;
                // A whole column or row stops at the last used cell, same as lookup_array
                if let Ok(ws) = self.workbook.worksheet(left.sheet) {
                    let dimension = ws.dimension();
                    if left.row == 1 && right.row == LAST_ROW {
                        right.row = dimension.max_row.max(1);
                    }
                    if left.column == 1 && right.column == LAST_COLUMN {
                        right.column = dimension.max_column.max(1);
                    }
                }
                let size = if lookup_is_column {
                    right.row - left.row + 1
                } else {
                    right.column - left.column + 1
                };
                if size != n as i32 {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Arrays must be of the same size".to_string(),
                    );
                }
                XlookupReturn::Cells(left, right)
            }
            CalcResult::Array(a) => {
                let size = if lookup_is_column {
                    a.len()
                } else {
                    a.first().map_or(0, |r| r.len())
                };
                if size != n {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Arrays must be of the same size".to_string(),
                    );
                }
                XlookupReturn::Values(a)
            }
            error @ CalcResult::Error { .. } => return error,
            other => XlookupReturn::Values(vec![vec![calc_result_to_array_node(other)]]),
        };
        if let XlookupReturn::Values(a) = &ret {
            if n > 1 && a.len() == 1 && a[0].len() == 1 {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Arrays must be of the same size".to_string(),
                );
            }
        }

        let find = |value: &CalcResult| -> Option<usize> {
            match search_mode {
                SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => linear_search(
                    value,
                    &lookup,
                    if search_mode == SearchMode::StartAtFirstItem {
                        SearchMode::StartAtFirstItem
                    } else {
                        SearchMode::StartAtLastItem
                    },
                    match match_mode {
                        MatchMode::ExactMatch => MatchMode::ExactMatch,
                        MatchMode::ExactMatchSmaller => MatchMode::ExactMatchSmaller,
                        MatchMode::ExactMatchLarger => MatchMode::ExactMatchLarger,
                        MatchMode::WildcardMatch => MatchMode::WildcardMatch,
                    },
                ),
                SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending => {
                    let ascending = search_mode == SearchMode::BinarySearchAscending;
                    let index = match (&match_mode, ascending) {
                        (MatchMode::ExactMatchLarger, true) => {
                            binary_search_or_greater(value, &lookup)
                        }
                        (MatchMode::ExactMatchLarger, false) => {
                            binary_search_descending_or_greater(value, &lookup)
                        }
                        (_, true) => binary_search_or_smaller(value, &lookup),
                        (_, false) => binary_search_descending_or_smaller(value, &lookup),
                    }?;
                    let index = usize::try_from(index).ok()?;
                    match match_mode {
                        MatchMode::ExactMatch => {
                            (compare_values(lookup.get(index)?, value) == 0).then_some(index)
                        }
                        MatchMode::WildcardMatch => None,
                        _ => Some(index),
                    }
                }
            }
        };
        if match_mode == MatchMode::WildcardMatch
            && matches!(
                search_mode,
                SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending
            )
        {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Cannot use wildcard in binary search".to_string(),
            );
        }

        // Several values looked up at once give one answer each
        let many = match &lookup_value {
            CalcResult::Array(a) => Some(a.clone()),
            CalcResult::Range { left, right } if left != right => {
                Some(self.evaluate_range(*left, *right))
            }
            _ => None,
        };
        if let Some(values) = many {
            let result = values
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|v| {
                            let v = array_node_to_result(v, cell);
                            match find(&v) {
                                Some(index) => {
                                    let answer = self.xlookup_pick(&ret, index, lookup_is_column);
                                    first_value(answer, self)
                                }
                                None => calc_result_to_array_node(if_not_found.clone()),
                            }
                        })
                        .collect()
                })
                .collect();
            return CalcResult::Array(result);
        }
        let lookup_value = match lookup_value {
            CalcResult::Range { left, .. } => self.evaluate_cell(left),
            v => v,
        };
        match find(&lookup_value) {
            Some(index) => self.xlookup_pick(&ret, index, lookup_is_column),
            None => if_not_found,
        }
    }

    /// The lookup_array of XLOOKUP as a list of values, and whether it is a
    /// column (true) or a row.
    fn xlookup_vector(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<CalcResult>, bool), CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => {
                let is_column = left.column == right.column;
                if !is_column && left.row != right.row {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Second argument must be a vector".to_string(),
                    ));
                }
                let mut right = right;
                // A whole column or row stops at the last used cell
                if let Ok(ws) = self.workbook.worksheet(left.sheet) {
                    let dimension = ws.dimension();
                    if is_column && left.row == 1 && right.row == LAST_ROW {
                        right.row = dimension.max_row.max(1);
                    }
                    if !is_column && left.column == 1 && right.column == LAST_COLUMN {
                        right.column = dimension.max_column.max(1);
                    }
                }
                Ok((self.prepare_array(&left, &right, is_column), is_column))
            }
            CalcResult::Array(a) => {
                let is_column = a.first().map_or(0, |r| r.len()) == 1;
                if !is_column && a.len() != 1 {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Second argument must be a vector".to_string(),
                    ));
                }
                let values = a
                    .iter()
                    .flat_map(|row| row.iter().map(|v| array_node_to_result(v, cell)))
                    .collect();
                Ok((values, is_column))
            }
            error @ CalcResult::Error { .. } => Err(error),
            other => Ok((vec![other], true)),
        }
    }

    /// Row (or column) `index` of XLOOKUP's return_array: a single value, or
    /// the cells of that row to spill.
    fn xlookup_pick(
        &mut self,
        ret: &XlookupReturn,
        index: usize,
        lookup_is_column: bool,
    ) -> CalcResult {
        let index_i32 = index as i32;
        match ret {
            XlookupReturn::Cells(left, right) => {
                let (l, r) = if lookup_is_column {
                    (
                        CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row + index_i32,
                            column: left.column,
                        },
                        CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row + index_i32,
                            column: right.column,
                        },
                    )
                } else {
                    (
                        CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row,
                            column: left.column + index_i32,
                        },
                        CellReferenceIndex {
                            sheet: left.sheet,
                            row: right.row,
                            column: left.column + index_i32,
                        },
                    )
                };
                if l == r {
                    self.evaluate_cell(l)
                } else {
                    CalcResult::Range { left: l, right: r }
                }
            }
            XlookupReturn::Values(a) => {
                let slice: Vec<Vec<ArrayNode>> = if a.len() == 1 && a[0].len() == 1 {
                    vec![vec![a[0][0].clone()]]
                } else if lookup_is_column {
                    a.get(index).cloned().into_iter().collect()
                } else {
                    a.iter()
                        .filter_map(|row| row.get(index).cloned().map(|v| vec![v]))
                        .collect()
                };
                if slice.len() == 1 && slice[0].len() == 1 {
                    array_node_to_result(
                        &slice[0][0],
                        CellReferenceIndex {
                            sheet: 0,
                            row: 1,
                            column: 1,
                        },
                    )
                } else {
                    CalcResult::Array(slice)
                }
            }
        }
    }
}

enum XlookupReturn {
    Cells(CellReferenceIndex, CellReferenceIndex),
    Values(Vec<Vec<ArrayNode>>),
}

fn array_node_to_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(f) => CalcResult::Number(*f),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::new_error(e.clone(), cell, String::new()),
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// The first value of an answer, for one element of a spilled lookup
fn first_value(answer: CalcResult, model: &mut Model) -> ArrayNode {
    match answer {
        CalcResult::Range { left, .. } => calc_result_to_array_node(model.evaluate_cell(left)),
        CalcResult::Array(a) => a
            .first()
            .and_then(|row| row.first())
            .cloned()
            .unwrap_or(ArrayNode::Empty),
        other => calc_result_to_array_node(other),
    }
}
