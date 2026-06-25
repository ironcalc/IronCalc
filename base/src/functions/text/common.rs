use crate::{
    arithmetic::bcast_idx,
    calc_result::CalcResult,
    cast::{array_node_to_string, calc_result_to_array_node},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    formatter::format::{format_number, parse_formatted_number},
    functions::{
        text::util::{substitute, text_after, text_before, Case},
        util::from_wildcard_to_regex,
    },
    model::Model,
    number_format::to_precision,
};

/// A LEFT/RIGHT/MID argument: a single scalar value or a 2-D array of values.
/// When any argument is an array (e.g. a SEQUENCE or a multi-cell range) the
/// scalar arguments are broadcast across it and the result is an array.
enum TextArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
}

impl TextArg {
    /// The (rows, columns) shape of this argument.
    fn dims(&self) -> (usize, usize) {
        match self {
            TextArg::Scalar(_) => (1, 1),
            TextArg::Array(a) => (a.len(), a.first().map(|r| r.len()).unwrap_or(0)),
        }
    }

    /// Returns the element at (`i`, `j`), broadcasting scalars and length-1 dimensions.
    /// A `None` result means the dimensions are incompatible at that position.
    fn elem(&self, i: usize, j: usize) -> Option<&ArrayNode> {
        match self {
            TextArg::Scalar(n) => Some(n),
            TextArg::Array(a) => bcast_idx(a.len(), i)
                .and_then(|ri| a.get(ri))
                .and_then(|row| bcast_idx(row.len(), j).and_then(|cj| row.get(cj))),
        }
    }
}

/// Coerces a single array element to a number for the start/length arguments of
/// LEFT/RIGHT/MID. Booleans and strings are rejected with `#VALUE!`, matching the
/// scalar behaviour of these functions; empties are treated as 0 and errors
/// propagate.
fn text_num_arg(node: &ArrayNode) -> Result<f64, Error> {
    match node {
        ArrayNode::Number(v) => Ok(*v),
        ArrayNode::Empty => Ok(0.0),
        ArrayNode::Error(e) => Err(e.clone()),
        ArrayNode::Boolean(_) | ArrayNode::String(_) => Err(Error::VALUE),
    }
}

/// Computes a single LEFT result from one element of each argument.
fn left_element(text: Option<&ArrayNode>, num: Option<&ArrayNode>) -> Result<ArrayNode, Error> {
    let (text, num) = match (text, num) {
        (Some(t), Some(n)) => (t, n),
        // A broadcast hole (incompatible dimensions) yields #N/A, as in Excel.
        _ => return Err(Error::NA),
    };
    let s = array_node_to_string(text)?;
    let num = text_num_arg(num)?;
    if num < 0.0 {
        return Err(Error::VALUE);
    }
    let num_chars = num.floor() as usize;
    Ok(ArrayNode::String(s.chars().take(num_chars).collect()))
}

/// Computes a single RIGHT result from one element of each argument.
fn right_element(text: Option<&ArrayNode>, num: Option<&ArrayNode>) -> Result<ArrayNode, Error> {
    let (text, num) = match (text, num) {
        (Some(t), Some(n)) => (t, n),
        _ => return Err(Error::NA),
    };
    let s = array_node_to_string(text)?;
    let num = text_num_arg(num)?;
    if num < 0.0 {
        return Err(Error::VALUE);
    }
    let num_chars = num.floor() as usize;
    let skip = s.chars().count().saturating_sub(num_chars);
    Ok(ArrayNode::String(s.chars().skip(skip).collect()))
}

/// Computes a single MID result from one element of each argument.
fn mid_element(
    text: Option<&ArrayNode>,
    start: Option<&ArrayNode>,
    length: Option<&ArrayNode>,
) -> Result<ArrayNode, Error> {
    let (text, start, length) = match (text, start, length) {
        (Some(t), Some(s), Some(l)) => (t, s, l),
        _ => return Err(Error::NA),
    };
    let s = array_node_to_string(text)?;
    let start = text_num_arg(start)?;
    let length = text_num_arg(length)?;
    if start < 1.0 {
        return Err(Error::VALUE);
    }
    if length < 0.0 {
        return Err(Error::VALUE);
    }
    let start_num = start.floor() as usize;
    let num_chars = length.floor() as usize;
    let mut result = String::new();
    let mut count: usize = 0;
    for (index, ch) in s.chars().enumerate() {
        if count >= num_chars {
            break;
        }
        if index + 1 >= start_num {
            result.push(ch);
            count += 1;
        }
    }
    Ok(ArrayNode::String(result))
}

/// Broadcasts `compute` over the (possibly array) `operands`. When every operand
/// is a scalar the result is a single value; otherwise it is an array whose shape
/// is the element-wise maximum of the operand shapes.
fn broadcast_text(
    cell: CellReferenceIndex,
    operands: &[&TextArg],
    compute: impl Fn(usize, usize) -> Result<ArrayNode, Error>,
) -> CalcResult {
    let mut rows = 1;
    let mut cols = 1;
    for operand in operands {
        let (r, c) = operand.dims();
        rows = rows.max(r);
        cols = cols.max(c);
    }

    // All scalars: return a single value.
    if rows <= 1 && cols <= 1 {
        return match compute(0, 0) {
            Ok(ArrayNode::String(s)) => CalcResult::String(s),
            Ok(ArrayNode::Number(n)) => CalcResult::Number(n),
            Ok(ArrayNode::Boolean(b)) => CalcResult::Boolean(b),
            Ok(ArrayNode::Empty) => CalcResult::String(String::new()),
            Ok(ArrayNode::Error(e)) | Err(e) => CalcResult::new_error(e, cell, String::new()),
        };
    }

    let mut result = Vec::with_capacity(rows);
    for i in 0..rows {
        let mut data_row = Vec::with_capacity(cols);
        for j in 0..cols {
            data_row.push(compute(i, j).unwrap_or_else(ArrayNode::Error));
        }
        result.push(data_row);
    }
    CalcResult::Array(result)
}

/// Finds the first instance of 'search_for' in text starting at char index start
fn find(search_for: &str, text: &str, start: usize) -> Option<i32> {
    let ch = text.chars();
    let mut byte_index = 0;
    for (char_index, c) in ch.enumerate() {
        if char_index + 1 >= start && text[byte_index..].starts_with(search_for) {
            return Some((char_index + 1) as i32);
        }
        byte_index += c.len_utf8();
    }
    None
}

/// You can use the wildcard characters — the question mark (?) and asterisk (*) — in the find_text argument.
/// * A question mark matches any single character.
/// * An asterisk matches any sequence of characters.
/// * If you want to find an actual question mark or asterisk, type a tilde (~) before the character.
fn search(search_for: &str, text: &str, start: usize) -> Option<i32> {
    let re = match from_wildcard_to_regex(search_for, false) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let ch = text.chars();
    let mut byte_index = 0;
    for (char_index, c) in ch.enumerate() {
        if char_index + 1 >= start {
            if let Some(m) = re.find(&text[byte_index..]) {
                return Some((text[0..(m.start() + byte_index)].chars().count() as i32) + 1);
            } else {
                return None;
            }
        }
        byte_index += c.len_utf8();
    }
    None
}

impl<'a> Model<'a> {
    pub(crate) fn fn_concat(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut result = "".to_string();
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::String(value) => result = format!("{result}{value}"),
                CalcResult::Number(value) => result = format!("{result}{value}"),
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                CalcResult::Boolean(value) => {
                    if value {
                        result = format!("{result}TRUE");
                    } else {
                        result = format!("{result}FALSE");
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    for row in left.row..(right.row + 1) {
                        for column in left.column..(right.column + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::String(value) => {
                                    result = format!("{result}{value}");
                                }
                                CalcResult::Number(value) => result = format!("{result}{value}"),
                                CalcResult::Boolean(value) => {
                                    if value {
                                        result = format!("{result}TRUE");
                                    } else {
                                        result = format!("{result}FALSE");
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::Range { .. } => {}
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    return CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    }
                                }
                            }
                        }
                    }
                }
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        CalcResult::String(result)
    }
    pub(crate) fn fn_text(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        use crate::expressions::parser::ArrayNode;
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value_result = self.evaluate_node_in_context(&args[0], cell);
        // Normalise a range to an array so both broadcast (spill) the same way.
        let value_result = match value_result {
            CalcResult::Range { left, right } => {
                CalcResult::Array(self.evaluate_range(left, right))
            }
            other => other,
        };
        match value_result {
            CalcResult::Array(arr) => {
                let format_code = match self.get_string(&args[1], cell) {
                    Ok(s) => s,
                    Err(e) => return e,
                };
                let locale = self.locale;
                let mut output = Vec::with_capacity(arr.len());
                for row in arr {
                    let mut data_row = Vec::with_capacity(row.len());
                    for node in row {
                        let out = match node {
                            ArrayNode::Number(f) => {
                                let d = format_number(f, &format_code, locale);
                                if d.error.is_some() {
                                    ArrayNode::Error(Error::VALUE)
                                } else {
                                    ArrayNode::String(d.text)
                                }
                            }
                            ArrayNode::Empty => {
                                let d = format_number(0.0, &format_code, locale);
                                if d.error.is_some() {
                                    ArrayNode::Error(Error::VALUE)
                                } else {
                                    ArrayNode::String(d.text)
                                }
                            }
                            ArrayNode::Boolean(b) => ArrayNode::Boolean(b),
                            ArrayNode::String(s) => ArrayNode::String(s),
                            e @ ArrayNode::Error(_) => e,
                        };
                        data_row.push(out);
                    }
                    output.push(data_row);
                }
                CalcResult::Array(output)
            }
            other => {
                let value = match other {
                    CalcResult::Number(f) => f,
                    CalcResult::String(s) => return CalcResult::String(s),
                    CalcResult::Boolean(b) => return CalcResult::Boolean(b),
                    error @ CalcResult::Error { .. } => return error,
                    CalcResult::Range { .. } => {
                        return CalcResult::Error {
                            error: Error::NIMPL,
                            origin: cell,
                            message: "Implicit Intersection not implemented".to_string(),
                        };
                    }
                    CalcResult::EmptyCell | CalcResult::EmptyArg => 0.0,
                    CalcResult::Array(_) | CalcResult::Lambda(_) => unreachable!(),
                };
                let format_code = match self.get_string(&args[1], cell) {
                    Ok(s) => s,
                    Err(s) => return s,
                };
                let d = format_number(value, &format_code, self.locale);
                if let Some(_e) = d.error {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Invalid format code".to_string(),
                    };
                }
                CalcResult::String(d.text)
            }
        }
    }

    /// FIND(find_text, within_text, [start_num])
    ///  * FIND and FINDB are case sensitive and don't allow wildcard characters.
    ///  * If find_text is "" (empty text), FIND matches the first character in the search string (that is, the character numbered start_num or 1).
    ///  * Find_text cannot contain any wildcard characters.
    ///  * If find_text does not appear in within_text, FIND and FINDB return the #VALUE! error value.
    ///  * If start_num is not greater than zero, FIND and FINDB return the #VALUE! error value.
    ///  * If start_num is greater than the length of within_text, FIND and FINDB return the #VALUE! error value.
    ///    NB: FINDB is not implemented. It is the same as FIND function unless locale is a DBCS (Double Byte Character Set)
    pub(crate) fn fn_find(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let find_text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let within_text = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let start_num = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(s) => s.floor(),
                Err(s) => return s,
            }
        } else {
            1.0
        };

        if start_num < 1.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Start num must be >= 1".to_string(),
            };
        }
        let start_num = start_num as usize;

        if start_num > within_text.len() {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Start num greater than length".to_string(),
            };
        }
        if let Some(s) = find(&find_text, &within_text, start_num) {
            CalcResult::Number(s as f64)
        } else {
            CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Text not found".to_string(),
            }
        }
    }

    /// Same API as FIND but:
    ///  * Allows wildcards
    ///  * It is case insensitive
    ///    SEARCH(find_text, within_text, [start_num])
    pub(crate) fn fn_search(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let find_text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let within_text = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let start_num = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(s) => s.floor(),
                Err(s) => return s,
            }
        } else {
            1.0
        };

        if start_num < 1.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Start num must be >= 1".to_string(),
            };
        }
        let start_num = start_num as usize;

        if start_num > within_text.len() {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Start num greater than length".to_string(),
            };
        }
        // SEARCH is case insensitive
        if let Some(s) = search(
            &find_text.to_lowercase(),
            &within_text.to_lowercase(),
            start_num,
        ) {
            CalcResult::Number(s as f64)
        } else {
            CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Text not found".to_string(),
            }
        }
    }

    // LEN, LEFT, RIGHT, MID, LOWER, UPPER, TRIM
    pub(crate) fn fn_len(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 1 {
            let s = match self.evaluate_node_in_context(&args[0], cell) {
                CalcResult::Number(v) => format!("{v}"),
                CalcResult::String(v) => v,
                CalcResult::Boolean(b) => {
                    if b {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { .. } => {
                    // Implicit Intersection not implemented
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Implicit Intersection not implemented".to_string(),
                    };
                }
                CalcResult::EmptyCell | CalcResult::EmptyArg => "".to_string(),
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
            return CalcResult::Number(s.chars().count() as f64);
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_trim(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 1 {
            let s = match self.evaluate_node_in_context(&args[0], cell) {
                CalcResult::Number(v) => format!("{v}"),
                CalcResult::String(v) => v,
                CalcResult::Boolean(b) => {
                    if b {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { .. } => {
                    // Implicit Intersection not implemented
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Implicit Intersection not implemented".to_string(),
                    };
                }
                CalcResult::EmptyCell | CalcResult::EmptyArg => "".to_string(),
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
            return CalcResult::String(s.trim().to_owned());
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_lower(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, |s| s.to_lowercase())
    }

    /// Applies a single-argument string transform element-wise.
    /// A scalar argument yields a single String; a range or array argument yields
    /// an array (which spills, or is consumed element-wise in array contexts such
    /// as SUMPRODUCT). Used by text functions like UPPER and LOWER.
    pub(crate) fn apply_text_unary(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        f: impl Fn(&str) -> String,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.get_string_or_array(&args[0], cell) {
            Ok(crate::cast::StringOrArray::String(s)) => CalcResult::String(f(&s)),
            Ok(crate::cast::StringOrArray::Array(arr)) => CalcResult::Array(
                arr.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| match crate::cast::array_node_to_string(n) {
                                Ok(s) => crate::expressions::parser::ArrayNode::String(f(&s)),
                                Err(e) => crate::expressions::parser::ArrayNode::Error(e),
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Err(e) => e,
        }
    }

    pub(crate) fn fn_unicode(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 1 {
            let s = match self.evaluate_node_in_context(&args[0], cell) {
                CalcResult::Number(v) => format!("{v}"),
                CalcResult::String(v) => v,
                CalcResult::Boolean(b) => {
                    if b {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Range { .. } => {
                    // Implicit Intersection not implemented
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Implicit Intersection not implemented".to_string(),
                    };
                }
                CalcResult::EmptyCell | CalcResult::EmptyArg => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Empty cell".to_string(),
                    }
                }
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };

            match s.chars().next() {
                Some(c) => {
                    let unicode_number = c as u32;
                    return CalcResult::Number(unicode_number as f64);
                }
                None => {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Empty cell".to_string(),
                    };
                }
            }
        }
        CalcResult::new_args_number_error(cell)
    }

    pub(crate) fn fn_upper(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, |s| s.to_uppercase())
    }

    /// Evaluates a LEFT/RIGHT/MID argument into a scalar value or a 2-D array of
    /// values. References to a single cell collapse to a scalar; multi-cell ranges
    /// and array literals/spills become arrays that the result is broadcast over.
    fn text_arg(&mut self, node: &Node, cell: CellReferenceIndex) -> Result<TextArg, CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            err @ CalcResult::Error { .. } => Err(err),
            CalcResult::Range { left, right } => {
                Ok(TextArg::Array(self.evaluate_range(left, right)))
            }
            CalcResult::Array(a) => Ok(TextArg::Array(a)),
            CalcResult::Lambda(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting a value".to_string(),
            )),
            other => Ok(TextArg::Scalar(calc_result_to_array_node(other))),
        }
    }

    pub(crate) fn fn_left(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let num = if args.len() == 2 {
            match self.text_arg(&args[1], cell) {
                Ok(o) => o,
                Err(e) => return e,
            }
        } else {
            TextArg::Scalar(ArrayNode::Number(1.0))
        };
        broadcast_text(cell, &[&text, &num], |i, j| {
            left_element(text.elem(i, j), num.elem(i, j))
        })
    }

    pub(crate) fn fn_right(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 2 || args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let num = if args.len() == 2 {
            match self.text_arg(&args[1], cell) {
                Ok(o) => o,
                Err(e) => return e,
            }
        } else {
            TextArg::Scalar(ArrayNode::Number(1.0))
        };
        broadcast_text(cell, &[&text, &num], |i, j| {
            right_element(text.elem(i, j), num.elem(i, j))
        })
    }

    pub(crate) fn fn_mid(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.text_arg(&args[0], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let start = match self.text_arg(&args[1], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        let length = match self.text_arg(&args[2], cell) {
            Ok(o) => o,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text, &start, &length], |i, j| {
            mid_element(text.elem(i, j), start.elem(i, j), length.elem(i, j))
        })
    }

    // REPT(text, number_times)
    pub(crate) fn fn_rept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let number_times = match self.get_number(&args[1], cell) {
            Ok(f) => f.floor() as i32,
            Err(s) => return s,
        };
        let text_len = text.len() as i32;

        // We normally don't follow Excel's sometimes archaic size's restrictions
        // But this might be a security issue
        if text_len * number_times > 32767 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "number times too high".to_string(),
            };
        }
        if number_times < 0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "number times too high".to_string(),
            };
        }
        if number_times == 0 {
            return CalcResult::String("".to_string());
        }
        CalcResult::String(text.repeat(number_times as usize))
    }

    // TEXTAFTER(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])
    pub(crate) fn fn_textafter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let delimiter = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let instance_num = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f.floor() as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let match_mode = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => {
                    if f == 0.0 {
                        Case::Sensitive
                    } else {
                        Case::Insensitive
                    }
                }
                Err(s) => return s,
            }
        } else {
            Case::Sensitive
        };

        let match_end = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            // disabled by default
            // the delimiter is specified in the formula
            0.0
        };
        if instance_num == 0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "instance_num must be <> 0".to_string(),
            };
        }
        if delimiter.len() > text.len() {
            // so this is fun(!)
            // if the function was provided with two arguments is a #VALUE!
            // if it had more is a #N/A (irrespective of their values)
            if arg_count > 2 {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "The delimiter is longer than the text is trying to match".to_string(),
                };
            } else {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "The delimiter is longer than the text is trying to match".to_string(),
                };
            }
        }
        if match_end != 0.0 && match_end != 1.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "argument must be 0 or 1".to_string(),
            };
        };
        match text_after(&text, &delimiter, instance_num, match_mode) {
            Some(s) => CalcResult::String(s),
            None => {
                if match_end == 1.0 {
                    if instance_num == 1 {
                        return CalcResult::String("".to_string());
                    } else if instance_num == -1 {
                        return CalcResult::String(text);
                    }
                }
                if arg_count == 6 {
                    // An empty cell is converted to empty string (not 0)
                    match self.evaluate_node_in_context(&args[5], cell) {
                        CalcResult::EmptyCell => CalcResult::String("".to_string()),
                        result => result,
                    }
                } else {
                    CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Value not found".to_string(),
                    }
                }
            }
        }
    }

    pub(crate) fn fn_textbefore(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let delimiter = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let instance_num = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f.floor() as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let match_mode = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => {
                    if f == 0.0 {
                        Case::Sensitive
                    } else {
                        Case::Insensitive
                    }
                }
                Err(s) => return s,
            }
        } else {
            Case::Sensitive
        };

        let match_end = if arg_count > 4 {
            match self.get_number(&args[4], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            // disabled by default
            // the delimiter is specified in the formula
            0.0
        };
        if instance_num == 0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "instance_num must be <> 0".to_string(),
            };
        }
        if delimiter.len() > text.len() {
            // so this is fun(!)
            // if the function was provided with two arguments is a #VALUE!
            // if it had more is a #N/A (irrespective of their values)
            if arg_count > 2 {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "The delimiter is longer than the text is trying to match".to_string(),
                };
            } else {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "The delimiter is longer than the text is trying to match".to_string(),
                };
            }
        }
        if match_end != 0.0 && match_end != 1.0 {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "argument must be 0 or 1".to_string(),
            };
        };
        match text_before(&text, &delimiter, instance_num, match_mode) {
            Some(s) => CalcResult::String(s),
            None => {
                if match_end == 1.0 {
                    if instance_num == -1 {
                        return CalcResult::String("".to_string());
                    } else if instance_num == 1 {
                        return CalcResult::String(text);
                    }
                }
                if arg_count == 6 {
                    // An empty cell is converted to empty string (not 0)
                    match self.evaluate_node_in_context(&args[5], cell) {
                        CalcResult::EmptyCell => CalcResult::String("".to_string()),
                        result => result,
                    }
                } else {
                    CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Value not found".to_string(),
                    }
                }
            }
        }
    }

    // TEXTJOIN(delimiter, ignore_empty, text1, [text2], …)
    pub(crate) fn fn_textjoin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let delimiter = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let ignore_empty = match self.get_boolean(&args[1], cell) {
            Ok(b) => b,
            Err(error) => return error,
        };
        let mut values = Vec::new();
        for arg in &args[2..] {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(format!("{value}")),
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        );
                    }
                    let row1 = left.row;
                    let mut row2 = right.row;
                    let column1 = left.column;
                    let mut column2 = right.column;
                    if row1 == 1 && row2 == LAST_ROW {
                        row2 = match self.workbook.worksheet(left.sheet) {
                            Ok(s) => s.dimension().max_row,
                            Err(_) => {
                                return CalcResult::new_error(
                                    Error::ERROR,
                                    cell,
                                    format!("Invalid worksheet index: '{}'", left.sheet),
                                );
                            }
                        };
                    }
                    if column1 == 1 && column2 == LAST_COLUMN {
                        column2 = match self.workbook.worksheet(left.sheet) {
                            Ok(s) => s.dimension().max_column,
                            Err(_) => {
                                return CalcResult::new_error(
                                    Error::ERROR,
                                    cell,
                                    format!("Invalid worksheet index: '{}'", left.sheet),
                                );
                            }
                        };
                    }
                    for row in row1..row2 + 1 {
                        for column in column1..(column2 + 1) {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(value) => {
                                    values.push(format!("{value}"));
                                }
                                CalcResult::String(value) => values.push(value),
                                CalcResult::Boolean(value) => {
                                    if value {
                                        values.push("TRUE".to_string())
                                    } else {
                                        values.push("FALSE".to_string())
                                    }
                                }
                                CalcResult::EmptyCell => {
                                    if !ignore_empty {
                                        values.push("".to_string())
                                    }
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::EmptyArg | CalcResult::Range { .. } => {}
                                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                                    return CalcResult::Error {
                                        error: Error::NIMPL,
                                        origin: cell,
                                        message: "Arrays not supported yet".to_string(),
                                    }
                                }
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(value) => values.push(value),
                CalcResult::Boolean(value) => {
                    if value {
                        values.push("TRUE".to_string())
                    } else {
                        values.push("FALSE".to_string())
                    }
                }
                CalcResult::EmptyCell => {
                    if !ignore_empty {
                        values.push("".to_string())
                    }
                }
                CalcResult::EmptyArg => {}
                CalcResult::Array(_) | CalcResult::Lambda(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        let result = values.join(&delimiter);
        CalcResult::String(result)
    }

    // SUBSTITUTE(text, old_text, new_text, [instance_num])
    pub(crate) fn fn_substitute(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=4).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let old_text = match self.get_string(&args[1], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let new_text = match self.get_string(&args[2], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let instance_num = if arg_count > 3 {
            match self.get_number(&args[3], cell) {
                Ok(f) => Some(f.floor() as i32),
                Err(s) => return s,
            }
        } else {
            // means every instance is replaced
            None
        };
        if let Some(num) = instance_num {
            if num < 1 {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid value".to_string(),
                };
            }
            if old_text.is_empty() {
                return CalcResult::String(text);
            }
            CalcResult::String(substitute(&text, &old_text, &new_text, num))
        } else {
            if old_text.is_empty() {
                return CalcResult::String(text);
            }
            CalcResult::String(text.replace(&old_text, &new_text))
        }
    }
    pub(crate) fn fn_concatenate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if arg_count == 0 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut text_array = Vec::new();
        for arg in args {
            let text = match self.get_string(arg, cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            text_array.push(text)
        }
        CalcResult::String(text_array.join(""))
    }

    pub(crate) fn fn_exact(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let result1 = &self.evaluate_node_in_context(&args[0], cell);
        let result2 = &self.evaluate_node_in_context(&args[1], cell);
        // FIXME: Implicit intersection
        if let (CalcResult::Number(number1), CalcResult::Number(number2)) = (result1, result2) {
            // In Excel two numbers are the same if they are the same up to 15 digits.
            CalcResult::Boolean(to_precision(*number1, 15) == to_precision(*number2, 15))
        } else {
            let string1 = match self.cast_to_string(result1.clone(), cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            let string2 = match self.cast_to_string(result2.clone(), cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            CalcResult::Boolean(string1 == string2)
        }
    }
    // VALUE(text)
    pub(crate) fn fn_value(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::String(text) => {
                let currencies = vec!["$", "€"];
                if let Ok((value, _)) = parse_formatted_number(&text, &currencies, self.locale) {
                    return CalcResult::Number(value);
                };
                CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid number".to_string(),
                }
            }
            CalcResult::Number(f) => CalcResult::Number(f),
            CalcResult::Boolean(_) => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid number".to_string(),
            },
            error @ CalcResult::Error { .. } => error,
            CalcResult::Range { .. } => {
                // TODO Implicit Intersection
                CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid number".to_string(),
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => CalcResult::Number(0.0),
            CalcResult::Array(_) | CalcResult::Lambda(_) => CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            },
        }
    }

    pub(crate) fn fn_t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // FIXME: Implicit intersection
        let result = self.evaluate_node_in_context(&args[0], cell);
        match result {
            CalcResult::String(_) => result,
            error @ CalcResult::Error { .. } => error,
            _ => CalcResult::String("".to_string()),
        }
    }

    // VALUETOTEXT(value)
    pub(crate) fn fn_valuetotext(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let text = match self.get_string(&args[0], cell) {
            Ok(s) => s,
            Err(error) => match error {
                CalcResult::Error { error, .. } => error.to_string(),
                _ => "".to_string(),
            },
        };
        CalcResult::String(text)
    }
}
