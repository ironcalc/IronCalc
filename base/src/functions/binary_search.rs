use std::cmp::Ordering;

use crate::{calc_result::CalcResult, expressions::types::CellReferenceIndex, model::Model};

use super::util::compare_values;

static EMPTY_CELL: CalcResult = CalcResult::EmptyCell;

/// The row or column a lookup function searches. A range that runs to the end
/// of the sheet, a whole column or a whole row, is read only as far as the
/// sheet's used area (`Model::clip_to_used_area`): the cells past it are empty
/// and are not read one by one. They still take part in the search. `len` is
/// the length of the whole vector and `get` gives `EmptyCell` past what was
/// read, so a search finds what it would have found had every cell been read.
pub(crate) struct LookupVector {
    values: Vec<CalcResult>,
    len: usize,
}

impl From<Vec<CalcResult>> for LookupVector {
    fn from(values: Vec<CalcResult>) -> Self {
        let len = values.len();
        LookupVector { values, len }
    }
}

impl LookupVector {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn get(&self, index: usize) -> &CalcResult {
        self.values.get(index).unwrap_or(&EMPTY_CELL)
    }

    /// The first position whose value satisfies `predicate`. The cells that
    /// were not read are all empty: the first of them is the answer if no
    /// cell read is and an empty cell is.
    pub(crate) fn position(&self, predicate: impl Fn(&CalcResult) -> bool) -> Option<usize> {
        if let Some(index) = self.values.iter().position(&predicate) {
            return Some(index);
        }
        if self.values.len() < self.len && predicate(&EMPTY_CELL) {
            return Some(self.values.len());
        }
        None
    }
}

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
        let n = if is_row_vector {
            right.row - left.row
        } else {
            right.column - left.column
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

    /// The vector from `left` to `right` for a lookup function, read only as
    /// far as the sheet's used area (see `LookupVector`). `is_row_vector` is
    /// as for `prepare_array`.
    pub(crate) fn lookup_vector(
        &mut self,
        left: &CellReferenceIndex,
        right: &CellReferenceIndex,
        is_row_vector: bool,
    ) -> Result<LookupVector, String> {
        let len = if is_row_vector {
            right.row - left.row
        } else {
            right.column - left.column
        } + 1;
        let (row, column) =
            self.clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)?;
        let right = CellReferenceIndex {
            sheet: left.sheet,
            row,
            column,
        };
        Ok(LookupVector {
            values: self.prepare_array(left, &right, is_row_vector),
            len: len as usize,
        })
    }
}

/// Old style binary search over a vector of values.
/// Returns the index of the matching element or the largest element smaller
/// than `target`. Returns `-2` if `target` is smaller than every element.
pub(crate) fn binary_search_on_array(target: &CalcResult, array: &LookupVector) -> i32 {
    // We apply binary search leftmost for value in the array
    let mut l = 0;
    let mut r = array.len();
    while l < r {
        let m = (l + r) / 2;
        match compare_values(array.get(m), target) {
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
