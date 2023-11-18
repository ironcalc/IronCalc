use super::{move_formula::ref_is_in_area, Node};

use crate::expressions::types::{Area, CellReferenceIndex};

pub(crate) fn forward_references(
    node: &mut Node,
    context: &CellReferenceIndex,
    source_area: &Area,
    target_sheet: u32,
    target_sheet_name: &str,
    target_row: i32,
    target_column: i32,
) {
    match node {
        Node::ReferenceKind {
            sheet_name,
            sheet_index: reference_sheet,
            absolute_row,
            absolute_column,
            row: reference_row,
            column: reference_column,
        } => {
            let reference_row_absolute = if *absolute_row {
                *reference_row
            } else {
                *reference_row + context.row
            };
            let reference_column_absolute = if *absolute_column {
                *reference_column
            } else {
                *reference_column + context.column
            };
            if ref_is_in_area(
                *reference_sheet,
                reference_row_absolute,
                reference_column_absolute,
                source_area,
            ) {
                if *reference_sheet != target_sheet {
                    *sheet_name = Some(target_sheet_name.to_string());
                    *reference_sheet = target_sheet;
                }
                *reference_row = target_row + *reference_row - source_area.row;
                *reference_column = target_column + *reference_column - source_area.column;
            }
        }
        Node::RangeKind {
            sheet_name,
            sheet_index,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => {
            let reference_row1 = if *absolute_row1 {
                *row1
            } else {
                *row1 + context.row
            };
            let reference_column1 = if *absolute_column1 {
                *column1
            } else {
                *column1 + context.column
            };

            let reference_row2 = if *absolute_row2 {
                *row2
            } else {
                *row2 + context.row
            };
            let reference_column2 = if *absolute_column2 {
                *column2
            } else {
                *column2 + context.column
            };
            if ref_is_in_area(*sheet_index, reference_row1, reference_column1, source_area)
                && ref_is_in_area(*sheet_index, reference_row2, reference_column2, source_area)
            {
                if *sheet_index != target_sheet {
                    *sheet_index = target_sheet;
                    *sheet_name = Some(target_sheet_name.to_string());
                }
                *row1 = target_row + *row1 - source_area.row;
                *column1 = target_column + *column1 - source_area.column;
                *row2 = target_row + *row2 - source_area.row;
                *column2 = target_column + *column2 - source_area.column;
            }
        }
        // Recurse
        Node::OpRangeKind { left, right } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::OpConcatenateKind { left, right } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::OpSumKind {
            kind: _,
            left,
            right,
        } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::OpProductKind {
            kind: _,
            left,
            right,
        } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::OpPowerKind { left, right } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::FunctionKind { kind: _, args } => {
            for arg in args {
                forward_references(
                    arg,
                    context,
                    source_area,
                    target_sheet,
                    target_sheet_name,
                    target_row,
                    target_column,
                );
            }
        }
        Node::InvalidFunctionKind { name: _, args } => {
            for arg in args {
                forward_references(
                    arg,
                    context,
                    source_area,
                    target_sheet,
                    target_sheet_name,
                    target_row,
                    target_column,
                );
            }
        }
        Node::CompareKind {
            kind: _,
            left,
            right,
        } => {
            forward_references(
                left,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        Node::UnaryKind { kind: _, right } => {
            forward_references(
                right,
                context,
                source_area,
                target_sheet,
                target_sheet_name,
                target_row,
                target_column,
            );
        }
        // TODO: Not implemented
        Node::ArrayKind(_) => {}
        // Do nothing. Note: we could do a blanket _ => {}
        Node::VariableKind(_) => {}
        Node::ErrorKind(_) => {}
        Node::ParseErrorKind { .. } => {}
        Node::EmptyArgKind => {}
        Node::BooleanKind(_) => {}
        Node::NumberKind(_) => {}
        Node::StringKind(_) => {}
        Node::WrongReferenceKind { .. } => {}
        Node::WrongRangeKind { .. } => {}
    }
}
