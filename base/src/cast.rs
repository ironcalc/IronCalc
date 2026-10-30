use crate::functions::date_and_time::parse_time_of_day;
use crate::number_format::{number_to_text, parse_finite_number};
use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    formatter::format::parse_formatted_number,
    model::Model,
};

pub(crate) enum NumberOrArray {
    Number(f64),
    Array(Vec<Vec<ArrayNode>>),
}

/// A scalar CalcResult or a 2-D array, used for element-wise comparison.
pub(crate) enum ValueOrArray {
    Value(CalcResult),
    Array(Vec<Vec<ArrayNode>>),
}

/// A scalar string or a 2-D array, used for element-wise text operations.
pub(crate) enum StringOrArray {
    String(String),
    Array(Vec<Vec<ArrayNode>>),
}

/// Extracts the payload of an [`ArrayNode`] produced by a unary broadcast, so the
/// scalar and array paths can share one element-wise core.
pub(crate) trait FromArrayNode: Sized {
    fn from_array_node(node: ArrayNode) -> Option<Self>;
}

impl FromArrayNode for f64 {
    fn from_array_node(node: ArrayNode) -> Option<Self> {
        match node {
            ArrayNode::Number(f) => Some(f),
            _ => None,
        }
    }
}

impl FromArrayNode for String {
    fn from_array_node(node: ArrayNode) -> Option<Self> {
        match node {
            ArrayNode::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Collapses a scalar [`CalcResult`] into a single array element.
/// Ranges, arrays and lambdas cannot be a single element, so they become `#VALUE!`.
pub(crate) fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        _ => ArrayNode::Error(Error::VALUE),
    }
}

/// Converts a single array element to a string using the same rules as
/// [`Model::cast_to_string`]. Errors propagate.
pub(crate) fn array_node_to_string(node: &ArrayNode) -> Result<String, Error> {
    match node {
        ArrayNode::Number(f) => Ok(number_to_text(*f)),
        ArrayNode::String(s) => Ok(s.clone()),
        ArrayNode::Boolean(b) => Ok(if *b {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        }),
        ArrayNode::Empty => Ok("".to_string()),
        ArrayNode::Error(e) => Err(e.clone()),
    }
}

impl<'a> Model<'a> {
    pub(crate) fn cast_number(&self, s: &str) -> Option<f64> {
        match parse_finite_number(s.trim()) {
            Some(f) => Some(f),
            None => {
                let currency = &self.locale.currency.symbol;
                let mut currencies = vec!["$", "€"];
                if !currencies.iter().any(|e| *e == currency) {
                    currencies.push(currency);
                }
                // Try to parse as a formatted number (e.g., dates, currencies, percentages)
                if let Ok((v, _number_format)) = parse_formatted_number(s, &currencies, self.locale)
                {
                    return Some(v);
                }
                // Or as a time of the day ("12:00", "3:30 PM")
                parse_time_of_day(s)
            }
        }
    }
    pub(crate) fn get_number_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<NumberOrArray, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(f) => Ok(NumberOrArray::Number(f)),
            CalcResult::String(s) => match self.cast_number(&s) {
                Some(f) => Ok(NumberOrArray::Number(f)),
                None => Err(CalcResult::new_error(
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
                                CalcResult::EmptyCell => ArrayNode::Empty,
                                CalcResult::EmptyArg => ArrayNode::Empty,
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    // if we do things right this can never happen.
                                    // the evaluation of a cell should never return an array or lambda
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
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "LAMBDA cannot be used as a number".to_string(),
            )),
        }
    }

    /// Like `get_number_or_array` but preserves the full CalcResult type (no coercion to f64).
    /// Used for element-wise comparison operators.
    pub(crate) fn get_value_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<ValueOrArray, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                Ok(ValueOrArray::Array(self.evaluate_range(left, right)))
            }
            CalcResult::Array(arr) => Ok(ValueOrArray::Array(arr)),
            other => Ok(ValueOrArray::Value(other)),
        }
    }

    /// Like `get_number_or_array` but for text: scalars are coerced to a `String`,
    /// ranges and arrays are returned as a 2-D array (to be processed element-wise).
    pub(crate) fn get_string_or_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<StringOrArray, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            error @ CalcResult::Error { .. } => Err(error),
            CalcResult::Range { left, right } => {
                Ok(StringOrArray::Array(self.evaluate_range(left, right)))
            }
            CalcResult::Array(arr) => Ok(StringOrArray::Array(arr)),
            other => Ok(StringOrArray::String(self.cast_to_string(other, cell)?)),
        }
    }

    /// Applies a scalar numeric transform element-wise over a node that may evaluate
    /// to a number, a range, or an array. Mirrors the `single_number_fn!` /
    /// `date_part_fn!` macros but accepts a closure, so callers can capture extra
    /// scalar arguments (e.g. WEEKDAY's `return_type`) or use non-arithmetic
    /// transforms (e.g. date helpers).
    ///
    /// Element coercion matches the rest of the array machinery: booleans → 0/1,
    /// empty → 0, strings are parsed with `cast_number` (`#VALUE!` if they don't
    /// parse), and errors propagate.
    pub(crate) fn apply_number_unary(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        match self.get_number_or_array(node, cell) {
            Ok(value) => self.apply_number_unary_value(value, cell, op),
            Err(e) => e,
        }
    }

    /// [`Self::apply_number_unary`] over an already evaluated argument, so a caller
    /// that had to inspect the value (e.g. WEEKDAY checks the date before reading
    /// `return_type`) does not pay for evaluating the node twice.
    pub(crate) fn apply_number_unary_value(
        &mut self,
        value: NumberOrArray,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        self.apply_number_unary_node(
            value,
            cell,
            |f| op(f).map(ArrayNode::Number),
            CalcResult::Number,
        )
    }

    /// Like [`Self::apply_number_unary`] for transforms from a number to a text,
    /// such as CHAR or UNICHAR. A scalar argument yields a `String`; a range or
    /// array argument yields an array of strings (with `#VALUE!` where an
    /// element does not coerce to a number).
    pub(crate) fn apply_number_to_text_unary(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<String, Error>,
    ) -> CalcResult {
        match self.get_number_or_array(node, cell) {
            Ok(value) => self.apply_number_unary_node(
                value,
                cell,
                |f| op(f).map(ArrayNode::String),
                CalcResult::String,
            ),
            Err(e) => e,
        }
    }

    /// Shared core of the numeric unary broadcasts: `op` maps each coerced number
    /// to an array element, `scalar` lifts that element's payload to a
    /// `CalcResult` when the argument was a single number.
    fn apply_number_unary_node<T>(
        &mut self,
        value: NumberOrArray,
        cell: CellReferenceIndex,
        op: impl Fn(f64) -> Result<ArrayNode, Error>,
        scalar: impl Fn(T) -> CalcResult,
    ) -> CalcResult
    where
        T: FromArrayNode,
    {
        match value {
            NumberOrArray::Number(f) => match op(f) {
                Ok(node) => match T::from_array_node(node) {
                    Some(x) => scalar(x),
                    None => CalcResult::new_error(Error::VALUE, cell, String::new()),
                },
                Err(e) => CalcResult::new_error(e, cell, String::new()),
            },
            NumberOrArray::Array(a) => {
                let mut array = Vec::with_capacity(a.len());
                for row in a {
                    let mut data_row = Vec::with_capacity(row.len());
                    for value in row {
                        let n = match value {
                            ArrayNode::Number(n) => Some(n),
                            ArrayNode::Boolean(b) => Some(if b { 1.0 } else { 0.0 }),
                            ArrayNode::Empty => Some(0.0),
                            ArrayNode::String(s) => self.cast_number(&s),
                            ArrayNode::Error(e) => {
                                data_row.push(ArrayNode::Error(e));
                                continue;
                            }
                        };
                        let out = match n {
                            Some(f) => match op(f) {
                                Ok(x) => x,
                                Err(e) => ArrayNode::Error(e),
                            },
                            None => ArrayNode::Error(Error::VALUE),
                        };
                        data_row.push(out);
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }
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

    /// A reference where a value is expected (typically returned by INDIRECT,
    /// OFFSET or INDEX inside a scalar argument) is dereferenced: a single cell
    /// gives its value, a larger range is intersected with the row or column of
    /// `cell`, as Excel does in a scalar context. Literal ranges never get here:
    /// static analysis wraps them in `@` at parse time.
    ///
    /// A 1x1 array (the result of arithmetic on such a reference, for instance
    /// `INDIRECT("C3")*1`) is a scalar wrapped in an array and unwraps likewise.
    fn dereference(&mut self, result: CalcResult, cell: CellReferenceIndex) -> CalcResult {
        match result {
            CalcResult::Range { .. } => self.implicit_intersection_to_value(result, cell),
            CalcResult::Array(ref array) if array.len() == 1 && array[0].len() == 1 => {
                match &array[0][0] {
                    ArrayNode::Number(n) => CalcResult::Number(*n),
                    ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
                    ArrayNode::String(s) => CalcResult::String(s.clone()),
                    ArrayNode::Error(error) => CalcResult::new_error(
                        error.clone(),
                        cell,
                        error.to_localized_error_string(self.language),
                    ),
                    ArrayNode::Empty => CalcResult::EmptyCell,
                }
            }
            other => other,
        }
    }

    pub(crate) fn cast_to_number(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let result = self.dereference(result, cell);
        match result {
            CalcResult::Number(f) => Ok(f),
            CalcResult::String(s) => match self.cast_number(&s) {
                Some(f) => Ok(f),
                None => Err(CalcResult::new_error(
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
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
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
        let result = self.dereference(result, cell);
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
        let result = self.dereference(result, cell);
        match result {
            CalcResult::Number(f) => Ok(number_to_text(f)),
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
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
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

    pub(crate) fn cast_to_bool(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<bool, CalcResult> {
        let result = self.dereference(result, cell);
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
            CalcResult::Array(_) | CalcResult::Lambda(_) => Err(CalcResult::Error {
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
        // Reference context: references stay references and an `@` node yields
        // the intersected 1x1 reference (e.g. COLUMNS(@A1:C1) is 1).
        let value = self.evaluate_node_with_reference(node, cell);
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
