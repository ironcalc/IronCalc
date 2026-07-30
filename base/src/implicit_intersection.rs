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
    // A single-cell "range" intersects to itself, on its own sheet. This holds
    // even when that sheet differs from the calling cell's — e.g. `@Sheet2!B3`,
    // or an `@`-decorated `OFFSET(INDIRECT("Sheet2!B3"),..)` that resolves to a
    // single cross-sheet cell. Handle it before the same-sheet guard below, which
    // would otherwise reject the cross-sheet case and yield #VALUE!.
    if left.sheet == right.sheet && left.row == right.row && left.column == right.column {
        return Some(*left);
    }
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
    }
    None
}
