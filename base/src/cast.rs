use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

pub(crate) enum NumberOrArray {
    Number(f64),
    Array(Vec<Vec<ArrayNode>>),
}

impl Model {
    pub(crate) fn get_number_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<NumberOrArray, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(f) => Ok(NumberOrArray::Number(f)),
            CalcResult::String(s) => match s.parse::<f64>() {
                Ok(f) => Ok(NumberOrArray::Number(f)),
                _ => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expecting number".to_string(),
                )),
            },
            CalcResult::Boolean(f) => {
                if f {
                    Ok(NumberOrArray::Number(1.0))
                } else {
                    Ok(NumberOrArray::Number(0.0))
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(NumberOrArray::Number(0.0)),
            CalcResult::Range { left, right } => {
                let sheet = left.sheet;
                if sheet != right.sheet {
                    return Err(CalcResult::Error {
                        error: Error::ERROR,
                        origin: cell,
                        message: "3D ranges are not allowed".to_string(),
                    });
                }
                // we need to convert the range into an array
                let mut array = Vec::new();
                for row in left.row..=right.row {
                    let mut row_data = Vec::new();
                    for column in left.column..=right.column {
                        let value =
                            match self.evaluate_cell(CellReferenceIndex { sheet, column, row }) {
                                CalcResult::String(s) => ArrayNode::String(s),
                                CalcResult::Number(f) => ArrayNode::Number(f),
                                CalcResult::Boolean(b) => ArrayNode::Boolean(b),
                                CalcResult::Error { error, .. } => ArrayNode::Error(error),
                                CalcResult::Range { .. } => {
                                    // if we do things right this can never happen.
                                    // the evaluation of a cell should never return a range
                                    ArrayNode::Number(0.0)
                                }
                                CalcResult::EmptyCell => ArrayNode::Number(0.0),
                                CalcResult::EmptyArg => ArrayNode::Number(0.0),
                                CalcResult::Array(_) => {
                                    // if we do things right this can never happen.
                                    // the evaluation of a cell should never return an array
                                    ArrayNode::Number(0.0)
                                }
                            };
                        row_data.push(value);
                    }
                    array.push(row_data);
                }
                Ok(NumberOrArray::Array(array))
            }
            CalcResult::Array(s) => Ok(NumberOrArray::Array(s)),
            error @ CalcResult::Error { .. } => Err(error),
        }
    }
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
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
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
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
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
            CalcResult::Range { .. } => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
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
