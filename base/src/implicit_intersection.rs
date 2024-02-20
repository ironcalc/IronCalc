use crate::{calc_result::Range, expressions::types::CellReferenceIndex};

/// It returns the closest cell from cell_reference to range in the same column/row
/// Examples
///  * i_i(B5, A2:A9) -> B5
///  * i_i(B5, A7:A9) -> None
///  * i_i(B5, A2:D2) -> B2
pub(crate) fn implicit_intersection(
    cell_reference: &CellReferenceIndex,
    range: &Range,
) -> Option<CellReferenceIndex> {
    let left = &range.left;
    let right = &range.right;
    let sheet = cell_reference.sheet;
    // If they are not all in the same sheet there is no intersection
    if sheet != left.sheet && sheet != right.sheet {
        return None;
    }
    let row = cell_reference.row;
    let column = cell_reference.column;
    if row >= left.row && row <= right.row {
        if left.column != right.column {
            return None;
        }
        return Some(CellReferenceIndex {
            sheet,
            row,
            column: left.column,
        });
    } else if column >= left.column && column <= right.column {
        if left.row != right.row {
            return None;
        }
        return Some(CellReferenceIndex {
            sheet,
            row: left.row,
            column,
        });
    } else if left.row == right.row && left.column == right.column {
        // If the range is a single cell, then return it.
        return Some(CellReferenceIndex {
            sheet,
            row: left.row,
            column: right.column,
        });
    }
    None
}
