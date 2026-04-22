use crate::{
    calc_result::CalcResult,
    cast::{NumberOrArray, ValueOrArray},
    expressions::{
        parser::{ArrayNode, Node},
        token::{Error, OpCompare},
        types::CellReferenceIndex,
    },
    functions::util::compare_values,
    model::Model,
};

/// Unify how we map booleans/strings to f64
fn to_f64(value: &ArrayNode) -> Result<f64, Error> {
    match value {
        ArrayNode::Number(f) => Ok(*f),
        ArrayNode::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        ArrayNode::String(s) => match s.parse::<f64>() {
            Ok(f) => Ok(f),
            Err(_) => Err(Error::VALUE),
        },
        ArrayNode::Error(err) => Err(err.clone()),
        ArrayNode::Empty => Ok(0.0),
    }
}

impl<'a> Model<'a> {
    /// Applies `op` element‐wise for arrays/numbers.
    pub(crate) fn handle_arithmetic(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
        op: &dyn Fn(f64, f64) -> Result<f64, Error>,
    ) -> CalcResult {
        let l = match self.get_number_or_array(left, cell) {
            Ok(f) => f,
            Err(s) => {
                return s;
            }
        };
        let r = match self.get_number_or_array(right, cell) {
            Ok(f) => f,
            Err(s) => {
                return s;
            }
        };
        match (l, r) {
            // -----------------------------------------------------
            // Case 1: Both are numbers
            // -----------------------------------------------------
            (NumberOrArray::Number(f1), NumberOrArray::Number(f2)) => match op(f1, f2) {
                Ok(x) => CalcResult::Number(x),
                Err(Error::DIV) => CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "Divide by 0".to_string(),
                },
                Err(Error::VALUE) => CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid number".to_string(),
                },
                Err(e) => CalcResult::Error {
                    error: e,
                    origin: cell,
                    message: "Unknown error".to_string(),
                },
            },

            // -----------------------------------------------------
            // Case 2: left is Number, right is Array
            // -----------------------------------------------------
            (NumberOrArray::Number(f1), NumberOrArray::Array(a2)) => {
                let mut array = Vec::new();
                for row in a2 {
                    let mut data_row = Vec::new();
                    for node in row {
                        match to_f64(&node) {
                            Ok(f2) => match op(f1, f2) {
                                Ok(x) => data_row.push(ArrayNode::Number(x)),
                                Err(Error::DIV) => data_row.push(ArrayNode::Error(Error::DIV)),
                                Err(Error::VALUE) => data_row.push(ArrayNode::Error(Error::VALUE)),
                                Err(e) => data_row.push(ArrayNode::Error(e)),
                            },
                            Err(err) => data_row.push(ArrayNode::Error(err)),
                        }
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }

            // -----------------------------------------------------
            // Case 3: left is Array, right is Number
            // -----------------------------------------------------
            (NumberOrArray::Array(a1), NumberOrArray::Number(f2)) => {
                let mut array = Vec::new();
                for row in a1 {
                    let mut data_row = Vec::new();
                    for node in row {
                        match to_f64(&node) {
                            Ok(f1) => match op(f1, f2) {
                                Ok(x) => data_row.push(ArrayNode::Number(x)),
                                Err(Error::DIV) => data_row.push(ArrayNode::Error(Error::DIV)),
                                Err(Error::VALUE) => data_row.push(ArrayNode::Error(Error::VALUE)),
                                Err(e) => data_row.push(ArrayNode::Error(e)),
                            },
                            Err(err) => data_row.push(ArrayNode::Error(err)),
                        }
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }

            // -----------------------------------------------------
            // Case 4: Both are arrays
            // -----------------------------------------------------
            (NumberOrArray::Array(a1), NumberOrArray::Array(a2)) => {
                let n1 = a1.len();
                let m1 = a1.first().map(|r| r.len()).unwrap_or(0);
                let n2 = a2.len();
                let m2 = a2.first().map(|r| r.len()).unwrap_or(0);
                let n = n1.max(n2);
                let m = m1.max(m2);

                let mut array = Vec::new();
                for i in 0..n {
                    let row1 = a1.get(i);
                    let row2 = a2.get(i);

                    let mut data_row = Vec::new();
                    for j in 0..m {
                        let val1 = row1.and_then(|r| r.get(j));
                        let val2 = row2.and_then(|r| r.get(j));

                        match (val1, val2) {
                            (Some(v1), Some(v2)) => match (to_f64(v1), to_f64(v2)) {
                                (Ok(f1), Ok(f2)) => match op(f1, f2) {
                                    Ok(x) => data_row.push(ArrayNode::Number(x)),
                                    Err(Error::DIV) => data_row.push(ArrayNode::Error(Error::DIV)),
                                    Err(Error::VALUE) => {
                                        data_row.push(ArrayNode::Error(Error::VALUE))
                                    }
                                    Err(e) => data_row.push(ArrayNode::Error(e)),
                                },
                                (Err(e), _) | (_, Err(e)) => data_row.push(ArrayNode::Error(e)),
                            },
                            // Mismatched dimensions => #VALUE!
                            _ => data_row.push(ArrayNode::Error(Error::VALUE)),
                        }
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }
        }
    }

    /// Applies a comparison operator element-wise.
    /// When either operand is a range or array the result is an array of booleans;
    /// when both are scalars the result is a single Boolean.
    pub(crate) fn handle_comparison(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
        kind: &OpCompare,
    ) -> CalcResult {
        let l = match self.get_value_or_array(left, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let r = match self.get_value_or_array(right, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let apply = |lv: &CalcResult, rv: &CalcResult| -> bool {
            let cmp = compare_values(lv, rv);
            match kind {
                OpCompare::Equal => cmp == 0,
                OpCompare::LessThan => cmp == -1,
                OpCompare::GreaterThan => cmp == 1,
                OpCompare::LessOrEqualThan => cmp < 1,
                OpCompare::GreaterOrEqualThan => cmp > -1,
                OpCompare::NonEqual => cmp != 0,
            }
        };

        let node_to_calc = |node: &ArrayNode| -> CalcResult {
            match node {
                ArrayNode::Number(n) => CalcResult::Number(*n),
                ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
                ArrayNode::String(s) => CalcResult::String(s.clone()),
                ArrayNode::Error(e) => CalcResult::Error {
                    error: e.clone(),
                    origin: cell,
                    message: String::new(),
                },
                ArrayNode::Empty => CalcResult::EmptyCell,
            }
        };

        match (l, r) {
            (ValueOrArray::Value(lv), ValueOrArray::Value(rv)) => {
                CalcResult::Boolean(apply(&lv, &rv))
            }
            (ValueOrArray::Array(la), ValueOrArray::Value(rv)) => CalcResult::Array(
                la.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| ArrayNode::Boolean(apply(&node_to_calc(n), &rv)))
                            .collect()
                    })
                    .collect(),
            ),
            (ValueOrArray::Value(lv), ValueOrArray::Array(ra)) => CalcResult::Array(
                ra.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| ArrayNode::Boolean(apply(&lv, &node_to_calc(n))))
                            .collect()
                    })
                    .collect(),
            ),
            (ValueOrArray::Array(la), ValueOrArray::Array(ra)) => {
                let rows = la.len().max(ra.len());
                let cols = la
                    .first()
                    .map(|r| r.len())
                    .unwrap_or(0)
                    .max(ra.first().map(|r| r.len()).unwrap_or(0));
                let mut array = Vec::with_capacity(rows);
                for ri in 0..rows {
                    let mut data_row = Vec::with_capacity(cols);
                    for ci in 0..cols {
                        let lv = la.get(ri).and_then(|r| r.get(ci)).map(node_to_calc);
                        let rv = ra.get(ri).and_then(|r| r.get(ci)).map(node_to_calc);
                        let node = match (lv, rv) {
                            (Some(lv), Some(rv)) => ArrayNode::Boolean(apply(&lv, &rv)),
                            _ => ArrayNode::Error(Error::VALUE),
                        };
                        data_row.push(node);
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }
        }
    }
}
