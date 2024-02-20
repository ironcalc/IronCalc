use crate::{
    calc_result::{CalcResult, Range},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    implicit_intersection::implicit_intersection,
    model::Model,
};

impl Model {
    pub(crate) fn get_number(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_number(result, cell)
    }

    fn cast_to_number(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        match result {
            CalcResult::Number(f) => Ok(f),
            CalcResult::String(s) => match s.parse::<f64>() {
                Ok(f) => Ok(f),
                _ => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expecting number".to_string(),
                )),
            },
            CalcResult::Boolean(f) => {
                if f {
                    Ok(1.0)
                } else {
                    Ok(0.0)
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(0.0),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                match implicit_intersection(&cell, &Range { left, right }) {
                    Some(cell_reference) => {
                        let result = self.evaluate_cell(cell_reference);
                        self.cast_to_number(result, cell_reference)
                    }
                    None => Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid reference (number)".to_string(),
                    }),
                }
            }
        }
    }

    pub(crate) fn get_number_no_bools(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        if matches!(result, CalcResult::Boolean(_)) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting number".to_string(),
            ));
        }
        self.cast_to_number(result, cell)
    }

    pub(crate) fn get_string(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<String, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_string(result, cell)
    }

    pub(crate) fn cast_to_string(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<String, CalcResult> {
        // FIXME: I think when casting a number we should convert it to_precision(x, 15)
        // See function Exact
        match result {
            CalcResult::Number(f) => Ok(format!("{}", f)),
            CalcResult::String(s) => Ok(s),
            CalcResult::Boolean(f) => {
                if f {
                    Ok("TRUE".to_string())
                } else {
                    Ok("FALSE".to_string())
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok("".to_string()),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                match implicit_intersection(&cell, &Range { left, right }) {
                    Some(cell_reference) => {
                        let result = self.evaluate_cell(cell_reference);
                        self.cast_to_string(result, cell_reference)
                    }
                    None => Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid reference (string)".to_string(),
                    }),
                }
            }
        }
    }

    pub(crate) fn get_boolean(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.cast_to_bool(result, cell)
    }

    fn cast_to_bool(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        match result {
            CalcResult::Number(f) => {
                if f == 0.0 {
                    return Ok(false);
                }
                Ok(true)
            }
            CalcResult::String(s) => {
                if s.to_lowercase() == *"true" {
                    return Ok(true);
                } else if s.to_lowercase() == *"false" {
                    return Ok(false);
                }
                Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Expected boolean".to_string(),
                })
            }
            CalcResult::Boolean(b) => Ok(b),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(false),
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                match implicit_intersection(&cell, &Range { left, right }) {
                    Some(cell_reference) => {
                        let result = self.evaluate_cell(cell_reference);
                        self.cast_to_bool(result, cell_reference)
                    }
                    None => Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid reference (bool)".to_string(),
                    }),
                }
            }
        }
    }

    // tries to return a reference. That is either a reference or a formula that evaluates to a range/reference
    pub(crate) fn get_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        match node {
            Node::ReferenceKind {
                column,
                absolute_column,
                row,
                absolute_row,
                sheet_index,
                sheet_name: _,
            } => {
                let left = CellReferenceIndex {
                    sheet: *sheet_index,
                    row: if *absolute_row { *row } else { *row + cell.row },
                    column: if *absolute_column {
                        *column
                    } else {
                        *column + cell.column
                    },
                };

                Ok(Range { left, right: left })
            }
            _ => {
                let value = self.evaluate_node_in_context(node, cell);
                if value.is_error() {
                    return Err(value);
                }
                if let CalcResult::Range { left, right } = value {
                    Ok(Range { left, right })
                } else {
                    Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Expected reference".to_string(),
                    })
                }
            }
        }
    }
}
