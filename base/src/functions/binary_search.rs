use std::cmp::Ordering;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::types::CellReferenceIndex,
    model::Model,
};

use super::util::compare_values;

// NOTE: We don't know how Excel exactly implements binary search internally.
// This means that if the values on the lookup range are not in order our results and Excel's will differ

// Assumes values are in ascending order, returns matching index or the largest value smaller than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_or_smaller<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    // We apply binary search leftmost for value in the range
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        if &array[m] < target {
            l = m + 1;
        } else {
            r = m;
        }
    }
    if l == n {
        return Some((l - 1) as i32);
    }
    // Now l points to the leftmost element
    if &array[l] == target {
        return Some(l as i32);
    }
    // If target is less than the minimum return None
    if l == 0 {
        return None;
    }
    Some((l - 1) as i32)
}

// Assumes values are in ascending order, returns matching index or the smaller value larger than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_or_greater<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let mut l = 0;
    let mut r = array.len();
    while l < r {
        let mut m = (l + r) / 2;
        match &array[m].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while m > 1 {
                    if &array[m - 1] == target {
                        m -= 1;
                    } else {
                        break;
                    }
                }
                return Some(m as i32);
            }
        }
    }
    // If target is larger than the maximum return None
    if r == array.len() {
        return None;
    }
    // Now r points to the rightmost element
    Some(r as i32)
}

// Assumes values are in descending order
pub(crate) fn binary_search_descending_or_smaller<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        let mut index = n - m - 1;
        match &array[index].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while index < n - 1 {
                    if &array[index + 1] == target {
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(index as i32);
            }
        }
    }
    if l == 0 {
        return None;
    }
    Some((n - l) as i32)
}

// Assumes values are in descending order, returns matching index or the smaller value larger than target.
// Returns None if target is smaller than the smaller value.
pub(crate) fn binary_search_descending_or_greater<T: Ord>(target: &T, array: &[T]) -> Option<i32> {
    let n = array.len();
    let mut l = 0;
    let mut r = n;
    while l < r {
        let m = (l + r) / 2;
        let mut index = n - m - 1;
        match &array[index].cmp(target) {
            Ordering::Less => {
                l = m + 1;
            }
            Ordering::Greater => {
                r = m;
            }
            Ordering::Equal => {
                while index < n - 1 {
                    if &array[index + 1] == target {
                        index += 1;
                    } else {
                        break;
                    }
                }
                return Some(index as i32);
            }
        }
    }
    if r == n {
        return None;
    }
    Some((n - r - 1) as i32)
}

impl<'a> Model<'a> {
    /// Returns an array with the list of cell values in the range
    pub(crate) fn prepare_array(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> Vec<CalcResult> {
        // A full-column or full-row reference spans the whole worksheet. Clamp
        // the scan to the last stored cell in the lookup vector so that MATCH
        // and vector LOOKUP do not walk 1,048,576 rows (or 16,384 columns) and
        // return a position past the data. See issue #1414.
        let (mut end_row, mut end_column) = (right.row, right.column);
        if is_row_vector && right.row == LAST_ROW {
            end_row = self.last_stored_row(left.sheet, left.row, left.column);
        } else if !is_row_vector && right.column == LAST_COLUMN {
            end_column = self.last_stored_column(left.sheet, left.row, left.column);
        }
        let n = if is_row_vector {
            end_row - left.row
        } else {
            end_column - left.column
        } + 1;
        let mut result = vec![];
        for index in 0..n {
            let row;
            let column;
            if is_row_vector {
                row = left.row + index;
                column = left.column;
            } else {
                column = left.column + index;
                row = left.row;
            }
            let value = self.evaluate_cell(CellReferenceIndex {
                sheet: left.sheet,
                row,
                column,
            });
            result.push(value);
        }
        result
    }

    /// Returns the last row in `column` that has a stored cell, starting the
    /// search at `start_row`. Returns `start_row` when the column has no stored
    /// cells at or below `start_row`.
    fn last_stored_row(&self, sheet: u32, start_row: i32, column: i32) -> i32 {
        let mut last = start_row;
        if let Ok(worksheet) = self.workbook.worksheet(sheet) {
            for (row, col, _) in worksheet.sheet_data.cells() {
                if col == column && row >= start_row && row > last {
                    last = row;
                }
            }
        }
        last
    }

    /// Returns the last column in `row` that has a stored cell, starting the
    /// search at `start_column`. Returns `start_column` when the row has no
    /// stored cells at or to the right of `start_column`.
    fn last_stored_column(&self, sheet: u32, row: i32, start_column: i32) -> i32 {
        let mut last = start_column;
        if let Ok(worksheet) = self.workbook.worksheet(sheet) {
            for (r, col, _) in worksheet.sheet_data.cells() {
                if r == row && col >= start_column && col > last {
                    last = col;
                }
            }
        }
        last
    }
}

/// Old style binary search over an already materialized vector of values.
/// Returns the index of the matching element or the largest element smaller
/// than `target`. Returns `-2` if `target` is smaller than every element.
pub(crate) fn binary_search_on_array(target: &CalcResult, array: &[CalcResult]) -> i32 {
    // We apply binary search leftmost for value in the array
    let mut l = 0;
    let mut r = array.len();
    while l < r {
        let m = (l + r) / 2;
        match compare_values(&array[m], target) {
            -1 => {
                l = m + 1;
            }
            1 => {
                r = m;
            }
            _ => {
                return m as i32;
            }
        }
    }
    // If target is less than the minimum return #N/A
    if l == 0 {
        return -2;
    }
    // Now l points to the leftmost element
    (l - 1) as i32
}
