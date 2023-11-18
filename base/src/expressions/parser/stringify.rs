use super::{super::utils::quote_name, Node, Reference};
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::OpUnary;
use crate::{expressions::types::CellReferenceRC, number_format::to_excel_precision_str};

pub enum DisplaceData {
    Column {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    Row {
        sheet: u32,
        row: i32,
        delta: i32,
    },
    CellHorizontal {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    CellVertical {
        sheet: u32,
        row: i32,
        column: i32,
        delta: i32,
    },
    ColumnMove {
        sheet: u32,
        column: i32,
        delta: i32,
    },
    None,
}

pub fn to_rc_format(node: &Node) -> String {
    stringify(node, None, &DisplaceData::None, false)
}

pub fn to_string_displaced(
    node: &Node,
    context: &CellReferenceRC,
    displace_data: &DisplaceData,
) -> String {
    stringify(node, Some(context), displace_data, false)
}

pub fn to_string(node: &Node, context: &CellReferenceRC) -> String {
    stringify(node, Some(context), &DisplaceData::None, false)
}

pub fn to_excel_string(node: &Node, context: &CellReferenceRC) -> String {
    stringify(node, Some(context), &DisplaceData::None, true)
}

/// Converts a local reference to a string applying some displacement if needed.
/// It uses A1 style if context is not None. If context is None it uses R1C1 style
/// If full_row is true then the row details will be omitted in the A1 case
/// If full_colum is true then column details will be omitted.
pub(crate) fn stringify_reference(
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    reference: &Reference,
    full_row: bool,
    full_column: bool,
) -> String {
    let sheet_name = reference.sheet_name;
    let sheet_index = reference.sheet_index;
    let absolute_row = reference.absolute_row;
    let absolute_column = reference.absolute_column;
    let row = reference.row;
    let column = reference.column;
    match context {
        Some(context) => {
            let mut row = if absolute_row { row } else { row + context.row };
            let mut column = if absolute_column {
                column
            } else {
                column + context.column
            };
            match displace_data {
                DisplaceData::Row {
                    sheet,
                    row: displace_row,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_row {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::Column {
                    sheet,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && !full_column {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellHorizontal {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_row == &row {
                        if *delta < 0 {
                            if &column >= displace_column {
                                if column < displace_column - *delta {
                                    return "#REF!".to_string();
                                }
                                column += *delta;
                            }
                        } else if &column >= displace_column {
                            column += *delta;
                        }
                    }
                }
                DisplaceData::CellVertical {
                    sheet,
                    row: displace_row,
                    column: displace_column,
                    delta,
                } => {
                    if sheet_index == *sheet && displace_column == &column {
                        if *delta < 0 {
                            if &row >= displace_row {
                                if row < displace_row - *delta {
                                    return "#REF!".to_string();
                                }
                                row += *delta;
                            }
                        } else if &row >= displace_row {
                            row += *delta;
                        }
                    }
                }
                DisplaceData::ColumnMove {
                    sheet,
                    column: move_column,
                    delta,
                } => {
                    if sheet_index == *sheet {
                        if column == *move_column {
                            column += *delta;
                        } else if (*delta > 0
                            && column > *move_column
                            && column <= *move_column + *delta)
                            || (*delta < 0
                                && column < *move_column
                                && column >= *move_column + *delta)
                        {
                            column -= *delta;
                        }
                    }
                }
                DisplaceData::None => {}
            }
            if row < 1 {
                return "#REF!".to_string();
            }
            let mut row_abs = if absolute_row {
                format!("${}", row)
            } else {
                format!("{}", row)
            };
            let column = match crate::expressions::utils::number_to_column(column) {
                Some(s) => s,
                None => return "#REF!".to_string(),
            };
            let mut col_abs = if absolute_column {
                format!("${}", column)
            } else {
                column
            };
            if full_row {
                row_abs = "".to_string()
            }
            if full_column {
                col_abs = "".to_string()
            }
            match &sheet_name {
                Some(name) => {
                    format!("{}!{}{}", quote_name(name), col_abs, row_abs)
                }
                None => {
                    format!("{}{}", col_abs, row_abs)
                }
            }
        }
        None => {
            let row_abs = if absolute_row {
                format!("R{}", row)
            } else {
                format!("R[{}]", row)
            };
            let col_abs = if absolute_column {
                format!("C{}", column)
            } else {
                format!("C[{}]", column)
            };
            match &sheet_name {
                Some(name) => {
                    format!("{}!{}{}", quote_name(name), row_abs, col_abs)
                }
                None => {
                    format!("{}{}", row_abs, col_abs)
                }
            }
        }
    }
}

fn format_function(
    name: &str,
    args: &Vec<Node>,
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    use_original_name: bool,
) -> String {
    let mut first = true;
    let mut arguments = "".to_string();
    for el in args {
        if !first {
            arguments = format!(
                "{},{}",
                arguments,
                stringify(el, context, displace_data, use_original_name)
            );
        } else {
            first = false;
            arguments = stringify(el, context, displace_data, use_original_name);
        }
    }
    format!("{}({})", name, arguments)
}

fn stringify(
    node: &Node,
    context: Option<&CellReferenceRC>,
    displace_data: &DisplaceData,
    use_original_name: bool,
) -> String {
    use self::Node::*;
    match node {
        BooleanKind(value) => format!("{}", value).to_ascii_uppercase(),
        NumberKind(number) => to_excel_precision_str(*number),
        StringKind(value) => format!("\"{}\"", value),
        WrongReferenceKind {
            sheet_name,
            column,
            row,
            absolute_row,
            absolute_column,
        } => stringify_reference(
            context,
            &DisplaceData::None,
            &Reference {
                sheet_name,
                sheet_index: 0,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            },
            false,
            false,
        ),
        ReferenceKind {
            sheet_name,
            sheet_index,
            column,
            row,
            absolute_row,
            absolute_column,
        } => stringify_reference(
            context,
            displace_data,
            &Reference {
                sheet_name,
                sheet_index: *sheet_index,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            },
            false,
            false,
        ),
        RangeKind {
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
            // Note that open ranges SUM(A:A) or SUM(1:1) will be treated as normal ranges in the R1C1 (internal) representation
            // A:A will be R1C[0]:R1048576C[0]
            // So when we are forming the A1 range we need to strip the irrelevant information
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);
            let s1 = stringify_reference(
                context,
                displace_data,
                &Reference {
                    sheet_name,
                    sheet_index: *sheet_index,
                    row: *row1,
                    column: *column1,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                context,
                displace_data,
                &Reference {
                    sheet_name: &None,
                    sheet_index: *sheet_index,
                    row: *row2,
                    column: *column2,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                },
                full_row,
                full_column,
            );
            format!("{}:{}", s1, s2)
        }
        WrongRangeKind {
            sheet_name,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
        } => {
            // Note that open ranges SUM(A:A) or SUM(1:1) will be treated as normal ranges in the R1C1 (internal) representation
            // A:A will be R1C[0]:R1048576C[0]
            // So when we are forming the A1 range we need to strip the irrelevant information
            let full_row = *absolute_row1 && *absolute_row2 && (*row1 == 1) && (*row2 == LAST_ROW);
            let full_column = *absolute_column1
                && *absolute_column2
                && (*column1 == 1)
                && (*column2 == LAST_COLUMN);
            let s1 = stringify_reference(
                context,
                &DisplaceData::None,
                &Reference {
                    sheet_name,
                    sheet_index: 0, // HACK
                    row: *row1,
                    column: *column1,
                    absolute_row: *absolute_row1,
                    absolute_column: *absolute_column1,
                },
                full_row,
                full_column,
            );
            let s2 = stringify_reference(
                context,
                &DisplaceData::None,
                &Reference {
                    sheet_name: &None,
                    sheet_index: 0, // HACK
                    row: *row2,
                    column: *column2,
                    absolute_row: *absolute_row2,
                    absolute_column: *absolute_column2,
                },
                full_row,
                full_column,
            );
            format!("{}:{}", s1, s2)
        }
        OpRangeKind { left, right } => format!(
            "{}:{}",
            stringify(left, context, displace_data, use_original_name),
            stringify(right, context, displace_data, use_original_name)
        ),
        OpConcatenateKind { left, right } => format!(
            "{}&{}",
            stringify(left, context, displace_data, use_original_name),
            stringify(right, context, displace_data, use_original_name)
        ),
        CompareKind { kind, left, right } => format!(
            "{}{}{}",
            stringify(left, context, displace_data, use_original_name),
            kind,
            stringify(right, context, displace_data, use_original_name)
        ),
        OpSumKind { kind, left, right } => format!(
            "{}{}{}",
            stringify(left, context, displace_data, use_original_name),
            kind,
            stringify(right, context, displace_data, use_original_name)
        ),
        OpProductKind { kind, left, right } => {
            let x = match **left {
                OpSumKind { .. } => format!(
                    "({})",
                    stringify(left, context, displace_data, use_original_name)
                ),
                CompareKind { .. } => format!(
                    "({})",
                    stringify(left, context, displace_data, use_original_name)
                ),
                _ => stringify(left, context, displace_data, use_original_name),
            };
            let y = match **right {
                OpSumKind { .. } => format!(
                    "({})",
                    stringify(right, context, displace_data, use_original_name)
                ),
                CompareKind { .. } => format!(
                    "({})",
                    stringify(right, context, displace_data, use_original_name)
                ),
                OpProductKind { .. } => format!(
                    "({})",
                    stringify(right, context, displace_data, use_original_name)
                ),
                _ => stringify(right, context, displace_data, use_original_name),
            };
            format!("{}{}{}", x, kind, y)
        }
        OpPowerKind { left, right } => format!(
            "{}^{}",
            stringify(left, context, displace_data, use_original_name),
            stringify(right, context, displace_data, use_original_name)
        ),
        InvalidFunctionKind { name, args } => {
            format_function(name, args, context, displace_data, use_original_name)
        }
        FunctionKind { kind, args } => {
            let name = if use_original_name {
                kind.to_xlsx_string()
            } else {
                kind.to_string()
            };
            format_function(&name, args, context, displace_data, use_original_name)
        }
        ArrayKind(args) => {
            let mut first = true;
            let mut arguments = "".to_string();
            for el in args {
                if !first {
                    arguments = format!(
                        "{},{}",
                        arguments,
                        stringify(el, context, displace_data, use_original_name)
                    );
                } else {
                    first = false;
                    arguments = stringify(el, context, displace_data, use_original_name);
                }
            }
            format!("{{{}}}", arguments)
        }
        VariableKind(value) => value.to_string(),
        UnaryKind { kind, right } => match kind {
            OpUnary::Minus => {
                format!(
                    "-{}",
                    stringify(right, context, displace_data, use_original_name)
                )
            }
            OpUnary::Percentage => {
                format!(
                    "{}%",
                    stringify(right, context, displace_data, use_original_name)
                )
            }
        },
        ErrorKind(kind) => format!("{}", kind),
        ParseErrorKind {
            formula,
            position: _,
            message: _,
        } => formula.to_string(),
        EmptyArgKind => "".to_string(),
    }
}

pub(crate) fn rename_sheet_in_node(node: &mut Node, sheet_index: u32, new_name: &str) {
    match node {
        // Rename
        Node::ReferenceKind {
            sheet_name,
            sheet_index: index,
            ..
        } => {
            if *index == sheet_index && sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
        Node::RangeKind {
            sheet_name,
            sheet_index: index,
            ..
        } => {
            if *index == sheet_index && sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }
        Node::WrongReferenceKind { sheet_name, .. } => {
            if let Some(name) = sheet_name {
                if name.to_uppercase() == new_name.to_uppercase() {
                    *sheet_name = Some(name.to_owned())
                }
            }
        }
        Node::WrongRangeKind { sheet_name, .. } => {
            if sheet_name.is_some() {
                *sheet_name = Some(new_name.to_owned());
            }
        }

        // Go next level
        Node::OpRangeKind { left, right } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::OpConcatenateKind { left, right } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::OpSumKind {
            kind: _,
            left,
            right,
        } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::OpProductKind {
            kind: _,
            left,
            right,
        } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::OpPowerKind { left, right } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::FunctionKind { kind: _, args } => {
            for arg in args {
                rename_sheet_in_node(arg, sheet_index, new_name);
            }
        }
        Node::InvalidFunctionKind { name: _, args } => {
            for arg in args {
                rename_sheet_in_node(arg, sheet_index, new_name);
            }
        }
        Node::CompareKind {
            kind: _,
            left,
            right,
        } => {
            rename_sheet_in_node(left, sheet_index, new_name);
            rename_sheet_in_node(right, sheet_index, new_name);
        }
        Node::UnaryKind { kind: _, right } => {
            rename_sheet_in_node(right, sheet_index, new_name);
        }

        // Do nothing
        Node::BooleanKind(_) => {}
        Node::NumberKind(_) => {}
        Node::StringKind(_) => {}
        Node::ErrorKind(_) => {}
        Node::ParseErrorKind { .. } => {}
        Node::ArrayKind(_) => {}
        Node::VariableKind(_) => {}
        Node::EmptyArgKind => {}
    }
}
