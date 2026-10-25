//! Displacing a parsed formula: what inserting or deleting rows or columns, or
//! moving cells, does to the references in it.
//!
//! The formula of a cell is stored relative to the cell. When rows are
//! inserted above a formula that reads `A5`, both the formula and `A5` move
//! down, and the stored formula does not change at all. The same formula in a
//! cell that stays put has to change, `A5` becoming `A6`. [`displaced_node`]
//! works that out from where the cell was, where it is now and what moved,
//! and says when nothing changes, which in a large sheet is almost always.
//!
//! The result is the same as writing the formula out with
//! [`to_string_displaced`](super::stringify::to_string_displaced) and parsing
//! it back in the cell's new place, without the writing and the parsing.

use super::stringify::{displace_coordinates, DisplaceData};
use super::Node;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;

/// The displacement of one formula: what moved in the workbook, and where the
/// cell that holds the formula was and is. `from` and `to` are the same when
/// the cell itself did not move.
pub(crate) struct Displacement<'a> {
    pub data: &'a DisplaceData,
    /// `(row, column)` of the cell when the formula was written
    pub from: (i32, i32),
    /// `(row, column)` of the cell now
    pub to: (i32, i32),
}

/// A reference that has been displaced: it is either somewhere, or gone
enum Displaced {
    At {
        row: i32,
        column: i32,
    },
    /// The cell it pointed to has been deleted, or it fell off the sheet
    Gone,
}

impl Displaced {
    fn is_gone(&self) -> bool {
        matches!(self, Displaced::Gone)
    }
}

impl Displacement<'_> {
    /// Displaces one reference. The coordinates given and returned are
    /// relative to the cell where they are not absolute. `None` when the
    /// displacement leaves the reference as it is.
    #[allow(clippy::too_many_arguments)]
    fn reference(
        &self,
        sheet_index: Option<u32>,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
        full_row: bool,
        full_column: bool,
    ) -> Option<Displaced> {
        let absolute = (
            if absolute_row { row } else { row + self.from.0 },
            if absolute_column {
                column
            } else {
                column + self.from.1
            },
        );
        let moved = match sheet_index {
            Some(sheet_index) => displace_coordinates(
                sheet_index,
                absolute.0,
                absolute.1,
                full_row,
                full_column,
                self.data,
            ),
            // A reference to a sheet that does not exist is not displaced
            None => Some(absolute),
        };
        let Some((mut new_row, mut new_column)) = moved else {
            return Some(Displaced::Gone);
        };
        // The rows of `A:A` and the columns of `3:3` are not written out, so
        // nothing that happens to them shows: they stay what they are.
        if full_row {
            new_row = absolute.0;
        }
        if full_column {
            new_column = absolute.1;
        }
        if !(1..=LAST_ROW).contains(&new_row) || !(1..=LAST_COLUMN).contains(&new_column) {
            return Some(Displaced::Gone);
        }
        let new_row = if absolute_row {
            new_row
        } else {
            new_row - self.to.0
        };
        let new_column = if absolute_column {
            new_column
        } else {
            new_column - self.to.1
        };
        if new_row == row && new_column == column {
            None
        } else {
            Some(Displaced::At {
                row: new_row,
                column: new_column,
            })
        }
    }
}

impl Displacement<'_> {
    /// The two corners of a range in the order the parser keeps them: the
    /// first corner is the top left one. Displacing one end of a range can
    /// put it past the other, `A1:B2` becoming `C1:B2`, and parsing that back
    /// gives `B1:C2`. The coordinates are relative to the cell where they are
    /// not absolute, so they are compared in absolute terms.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn ordered(
        &self,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    ) -> ((bool, bool, i32, i32), (bool, bool, i32, i32)) {
        let absolute = |is_absolute: bool, value: i32, origin: i32| {
            if is_absolute {
                value
            } else {
                value + origin
            }
        };
        let mut first = (absolute_row1, absolute_column1, row1, column1);
        let mut second = (absolute_row2, absolute_column2, row2, column2);
        if absolute(absolute_row1, row1, self.to.0) > absolute(absolute_row2, row2, self.to.0) {
            (first.0, first.2, second.0, second.2) = (second.0, second.2, first.0, first.2);
        }
        if absolute(absolute_column1, column1, self.to.1)
            > absolute(absolute_column2, column2, self.to.1)
        {
            (first.1, first.3, second.1, second.3) = (second.1, second.3, first.1, first.3);
        }
        (first, second)
    }
}

/// The formula after the displacement, or `None` if it is unchanged. Only
/// references change: the rest of the formula is copied as it is.
///
/// A reference to a deleted cell becomes `#REF!`; a range with a deleted end
/// becomes the range operator on what is left, `#REF!:B4`, as it is when the
/// formula is written out and read back.
pub(crate) fn displaced_node(node: &Node, displacement: &Displacement) -> Option<Node> {
    use Node::*;
    match node {
        ReferenceKind {
            sheet_name,
            sheet_index,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            let displaced = displacement.reference(
                Some(*sheet_index),
                *absolute_row,
                *absolute_column,
                *row,
                *column,
                false,
                false,
            )?;
            Some(match displaced {
                Displaced::Gone => ErrorKind(Error::REF),
                Displaced::At { row, column } => ReferenceKind {
                    sheet_name: sheet_name.clone(),
                    sheet_index: *sheet_index,
                    absolute_row: *absolute_row,
                    absolute_column: *absolute_column,
                    row,
                    column,
                },
            })
        }
        WrongReferenceKind {
            sheet_name,
            absolute_row,
            absolute_column,
            row,
            column,
        } => {
            let displaced = displacement.reference(
                None,
                *absolute_row,
                *absolute_column,
                *row,
                *column,
                false,
                false,
            )?;
            Some(match displaced {
                Displaced::Gone => ErrorKind(Error::REF),
                Displaced::At { row, column } => WrongReferenceKind {
                    sheet_name: sheet_name.clone(),
                    absolute_row: *absolute_row,
                    absolute_column: *absolute_column,
                    row,
                    column,
                },
            })
        }
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
            // See `stringify`: `A:A` is stored as `R1C[0]:R1048576C[0]`, and
            // such a range is not shortened or lengthened by rows coming or going.
            let full_row = *absolute_row1 && *absolute_row2 && *row1 == 1 && *row2 == LAST_ROW;
            let full_column =
                *absolute_column1 && *absolute_column2 && *column1 == 1 && *column2 == LAST_COLUMN;
            let first = displacement.reference(
                Some(*sheet_index),
                *absolute_row1,
                *absolute_column1,
                *row1,
                *column1,
                full_row,
                full_column,
            );
            let second = displacement.reference(
                Some(*sheet_index),
                *absolute_row2,
                *absolute_column2,
                *row2,
                *column2,
                full_row,
                full_column,
            );
            if first.is_none() && second.is_none() {
                return None;
            }
            let (row1, column1) = match first {
                Some(Displaced::At { row, column }) => (row, column),
                _ => (*row1, *column1),
            };
            let (row2, column2) = match second {
                Some(Displaced::At { row, column }) => (row, column),
                _ => (*row2, *column2),
            };
            let first_gone = first.as_ref().is_some_and(Displaced::is_gone);
            let second_gone = second.as_ref().is_some_and(Displaced::is_gone);
            if !first_gone && !second_gone {
                let (
                    (absolute_row1, absolute_column1, row1, column1),
                    (absolute_row2, absolute_column2, row2, column2),
                ) = displacement.ordered(
                    *absolute_row1,
                    *absolute_column1,
                    row1,
                    column1,
                    *absolute_row2,
                    *absolute_column2,
                    row2,
                    column2,
                );
                return Some(RangeKind {
                    sheet_name: sheet_name.clone(),
                    sheet_index: *sheet_index,
                    absolute_row1,
                    absolute_column1,
                    row1,
                    column1,
                    absolute_row2,
                    absolute_column2,
                    row2,
                    column2,
                });
            }
            let end = |gone: bool, absolute_row: bool, absolute_column: bool, row, column| {
                if gone {
                    ErrorKind(Error::REF)
                } else {
                    ReferenceKind {
                        sheet_name: sheet_name.clone(),
                        sheet_index: *sheet_index,
                        absolute_row,
                        absolute_column,
                        row,
                        column,
                    }
                }
            };
            Some(OpRangeKind {
                left: Box::new(end(
                    first_gone,
                    *absolute_row1,
                    *absolute_column1,
                    row1,
                    column1,
                )),
                right: Box::new(end(
                    second_gone,
                    *absolute_row2,
                    *absolute_column2,
                    row2,
                    column2,
                )),
            })
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
            let full_row = *absolute_row1 && *absolute_row2 && *row1 == 1 && *row2 == LAST_ROW;
            let full_column =
                *absolute_column1 && *absolute_column2 && *column1 == 1 && *column2 == LAST_COLUMN;
            let first = displacement.reference(
                None,
                *absolute_row1,
                *absolute_column1,
                *row1,
                *column1,
                full_row,
                full_column,
            );
            let second = displacement.reference(
                None,
                *absolute_row2,
                *absolute_column2,
                *row2,
                *column2,
                full_row,
                full_column,
            );
            if first.is_none() && second.is_none() {
                return None;
            }
            let (row1, column1) = match first {
                Some(Displaced::At { row, column }) => (row, column),
                _ => (*row1, *column1),
            };
            let (row2, column2) = match second {
                Some(Displaced::At { row, column }) => (row, column),
                _ => (*row2, *column2),
            };
            let first_gone = first.as_ref().is_some_and(Displaced::is_gone);
            let second_gone = second.as_ref().is_some_and(Displaced::is_gone);
            if !first_gone && !second_gone {
                let (
                    (absolute_row1, absolute_column1, row1, column1),
                    (absolute_row2, absolute_column2, row2, column2),
                ) = displacement.ordered(
                    *absolute_row1,
                    *absolute_column1,
                    row1,
                    column1,
                    *absolute_row2,
                    *absolute_column2,
                    row2,
                    column2,
                );
                return Some(WrongRangeKind {
                    sheet_name: sheet_name.clone(),
                    absolute_row1,
                    absolute_column1,
                    row1,
                    column1,
                    absolute_row2,
                    absolute_column2,
                    row2,
                    column2,
                });
            }
            let end = |gone: bool, absolute_row: bool, absolute_column: bool, row, column| {
                if gone {
                    ErrorKind(Error::REF)
                } else {
                    WrongReferenceKind {
                        sheet_name: sheet_name.clone(),
                        absolute_row,
                        absolute_column,
                        row,
                        column,
                    }
                }
            };
            Some(OpRangeKind {
                left: Box::new(end(
                    first_gone,
                    *absolute_row1,
                    *absolute_column1,
                    row1,
                    column1,
                )),
                right: Box::new(end(
                    second_gone,
                    *absolute_row2,
                    *absolute_column2,
                    row2,
                    column2,
                )),
            })
        }
        OpRangeKind { left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(OpRangeKind { left, right })
        }
        OpConcatenateKind { left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(OpConcatenateKind { left, right })
        }
        OpSumKind { kind, left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(OpSumKind {
                kind: kind.clone(),
                left,
                right,
            })
        }
        OpProductKind { kind, left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(OpProductKind {
                kind: kind.clone(),
                left,
                right,
            })
        }
        OpPowerKind { left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(OpPowerKind { left, right })
        }
        CompareKind { kind, left, right } => {
            let (left, right) = displaced_pair(left, right, displacement)?;
            Some(CompareKind {
                kind: kind.clone(),
                left,
                right,
            })
        }
        UnaryKind { kind, right } => Some(UnaryKind {
            kind: kind.clone(),
            right: Box::new(displaced_node(right, displacement)?),
        }),
        ImplicitIntersection { automatic, child } => Some(ImplicitIntersection {
            automatic: *automatic,
            child: Box::new(displaced_node(child, displacement)?),
        }),
        SpillRangeOperator { child } => Some(SpillRangeOperator {
            child: Box::new(displaced_node(child, displacement)?),
        }),
        FunctionKind { kind, args } => Some(FunctionKind {
            kind: kind.clone(),
            args: displaced_list(args, displacement)?,
        }),
        NamedFunctionKind { id, name, args } => Some(NamedFunctionKind {
            id: *id,
            name: name.clone(),
            args: displaced_list(args, displacement)?,
        }),
        LambdaDefKind { parameters, body } => Some(LambdaDefKind {
            parameters: parameters.clone(),
            body: Box::new(displaced_node(body, displacement)?),
        }),
        LambdaCallKind { lambda, args } => {
            let new_lambda = displaced_node(lambda, displacement);
            let new_args = displaced_list(args, displacement);
            if new_lambda.is_none() && new_args.is_none() {
                return None;
            }
            Some(LambdaCallKind {
                lambda: Box::new(new_lambda.unwrap_or_else(|| (**lambda).clone())),
                args: new_args.unwrap_or_else(|| args.clone()),
            })
        }
        // Nothing in these points anywhere
        BooleanKind(_)
        | NumberKind(_)
        | StringKind(_)
        | ArrayKind(_)
        | DefinedNameKind { .. }
        | TableNameKind(_)
        | NamedVariableKind { .. }
        | ErrorKind(_)
        | ParseErrorKind { .. }
        | EmptyArgKind => None,
    }
}

/// Both children of a node, displaced, if either of them changes
fn displaced_pair(
    left: &Node,
    right: &Node,
    displacement: &Displacement,
) -> Option<(Box<Node>, Box<Node>)> {
    let new_left = displaced_node(left, displacement);
    let new_right = displaced_node(right, displacement);
    if new_left.is_none() && new_right.is_none() {
        return None;
    }
    Some((
        Box::new(new_left.unwrap_or_else(|| left.clone())),
        Box::new(new_right.unwrap_or_else(|| right.clone())),
    ))
}

/// The arguments of a function, displaced, if any of them changes. Nothing is
/// copied until one does.
fn displaced_list(nodes: &[Node], displacement: &Displacement) -> Option<Vec<Node>> {
    let mut displaced: Option<Vec<Node>> = None;
    for (index, node) in nodes.iter().enumerate() {
        match displaced_node(node, displacement) {
            Some(new_node) => {
                displaced
                    .get_or_insert_with(|| nodes[..index].to_vec())
                    .push(new_node);
            }
            None => {
                if let Some(list) = &mut displaced {
                    list.push(node.clone());
                }
            }
        }
    }
    displaced
}
