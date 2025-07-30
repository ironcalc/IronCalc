#[cfg(feature = "use_regex_lite")]
use regex_lite as regex;

use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{
        parser::{ArrayNode, Node},
        token::{is_english_error_string, Error},
        types::CellReferenceIndex,
    },
    model::Model,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct CollectOpts {
    /// When true booleans that come from *cell references* are converted to 1/0 and counted.
    /// When false they are ignored (Excel behaviour for most statistical functions).
    pub include_bool_refs: bool,
    /// How to handle strings coming from *cell references* that are not parsable as numbers.
    /// * false – propagate #VALUE! (default Excel statistical functions behaviour)
    /// * true  – treat them as 0 (behaviour of the "…A" family – STDEVA, VARPA, …)
    pub string_ref_as_zero: bool,
}

/// This test for exact match (modulo case).
///   * strings are not cast into bools or numbers
///   * empty cell is not cast into empty string or zero
pub(crate) fn values_are_equal(left: &CalcResult, right: &CalcResult) -> bool {
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            if (value2 - value1).abs() < f64::EPSILON {
                return true;
            }
            false
        }
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            let value1 = value1.to_uppercase();
            let value2 = value2.to_uppercase();
            value1 == value2
        }
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => value1 == value2,
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => true,
        // NOTE: Errors and Ranges are not covered
        (_, _) => false,
    }
}

// In Excel there are two ways of comparing cell values.
// The old school comparison valid in formulas like D3 < D4 or HLOOKUP,... cast empty cells into empty strings or 0
// For the new formulas like XLOOKUP or SORT an empty cell is always larger than anything else.

// ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE;
pub(crate) fn compare_values(left: &CalcResult, right: &CalcResult) -> i32 {
    match (left, right) {
        (CalcResult::Number(value1), CalcResult::Number(value2)) => {
            if (value2 - value1).abs() < f64::EPSILON {
                return 0;
            }
            if value1 < value2 {
                return -1;
            }
            1
        }
        (CalcResult::Number(_value1), CalcResult::String(_value2)) => -1,
        (CalcResult::Number(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::String(value1), CalcResult::String(value2)) => {
            let value1 = value1.to_uppercase();
            let value2 = value2.to_uppercase();
            match value1.cmp(&value2) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }
        }
        (CalcResult::String(_value1), CalcResult::Boolean(_value2)) => -1,
        (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => {
            if value1 == value2 {
                return 0;
            }
            if *value1 {
                return 1;
            }
            -1
        }
        (CalcResult::EmptyCell, CalcResult::String(_value2)) => {
            compare_values(&CalcResult::String("".to_string()), right)
        }
        (CalcResult::String(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::String("".to_string()))
        }
        (CalcResult::EmptyCell, CalcResult::Number(_value2)) => {
            compare_values(&CalcResult::Number(0.0), right)
        }
        (CalcResult::Number(_value1), CalcResult::EmptyCell) => {
            compare_values(left, &CalcResult::Number(0.0))
        }
        (CalcResult::EmptyCell, CalcResult::EmptyCell) => 0,
        // NOTE: Errors and Ranges are not covered
        (_, _) => 1,
    }
}

/// We convert an Excel wildcard into a Rust (Perl family) regex
pub(crate) fn from_wildcard_to_regex(
    wildcard: &str,
    exact: bool,
) -> Result<regex::Regex, regex::Error> {
    // 1. Escape all
    let reg = &regex::escape(wildcard);

    // 2. We convert the escaped '?' into '.' (matches a single character)
    let reg = &reg.replace("\\?", ".");
    // 3. We convert the escaped '*' into '.*' (matches anything)
    let reg = &reg.replace("\\*", ".*");

    // 4. We send '\\~\\~' to '??' that is an unescaped regular expression, therefore cannot be in reg
    let reg = &reg.replace("\\~\\~", "??");

    // 5. If the escaped and converted '*' is preceded by '~' then it's a raw '*'
    let reg = &reg.replace("\\~.*", "\\*");
    // 6. If the escaped and converted '.' is preceded by '~' then it's a raw '?'
    let reg = &reg.replace("\\~.", "\\?");
    // '~' is used in Excel to escape any other character.
    //    So ~x goes to x (whatever x is)
    // 7. Remove all the others '\\~d' --> 'd'
    let reg = &reg.replace("\\~", "");
    // 8. Put back the '\\~\\~'  as '\\~'
    let reg = &reg.replace("??", "\\~");

    // And we have a valid Perl regex! (As Kim Kardashian said before me: "I know, right?")
    if exact {
        return regex::Regex::new(&format!("^{reg}$"));
    }
    regex::Regex::new(reg)
}

// NUMBERS ///
//*********///

// It could be either the number or a string representation of the number
// In the rest of the cases calc_result needs to be a number (cannot be the string "23", for instance)
fn result_is_equal_to_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => {
            if (f - target).abs() < f64::EPSILON {
                return true;
            }
            false
        }
        CalcResult::String(s) => {
            if let Ok(f) = s.parse::<f64>() {
                if (f - target).abs() < f64::EPSILON {
                    return true;
                }
                return false;
            }
            false
        }
        _ => false,
    }
}

fn result_is_less_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f < target,
        _ => false,
    }
}

fn result_is_less_or_equal_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f <= target,
        _ => false,
    }
}

fn result_is_greater_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f > target,
        _ => false,
    }
}

fn result_is_greater_or_equal_than_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => *f >= target,
        _ => false,
    }
}

fn result_is_not_equal_to_number(calc_result: &CalcResult, target: f64) -> bool {
    match calc_result {
        CalcResult::Number(f) => {
            if (f - target).abs() > f64::EPSILON {
                return true;
            }
            false
        }
        _ => true,
    }
}

// BOOLEANS ///
//**********///

// Booleans have to be "exactly" equal
fn result_is_equal_to_bool(calc_result: &CalcResult, target: bool) -> bool {
    match calc_result {
        CalcResult::Boolean(f) => target == *f,
        _ => false,
    }
}

fn result_is_not_equal_to_bool(calc_result: &CalcResult, target: bool) -> bool {
    match calc_result {
        CalcResult::Boolean(f) => target != *f,
        _ => true,
    }
}

// STRINGS ///
//*********///

// Note that strings are case insensitive. `target` must always be lower case.

pub(crate) fn result_matches_regex(calc_result: &CalcResult, reg: &regex::Regex) -> bool {
    match calc_result {
        CalcResult::String(s) => reg.is_match(&s.to_lowercase()),
        _ => false,
    }
}

fn result_is_equal_to_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            if target == s.to_lowercase() {
                return true;
            }
            false
        }
        CalcResult::EmptyCell => target.is_empty(),
        _ => false,
    }
}

fn result_is_not_equal_to_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            if target != s.to_lowercase() {
                return true;
            }
            false
        }
        _ => false,
    }
}

fn result_is_less_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => target.cmp(&s.to_lowercase()) == std::cmp::Ordering::Greater,
        _ => false,
    }
}

fn result_is_less_or_equal_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            let lower_case = &s.to_lowercase();
            target.cmp(lower_case) == std::cmp::Ordering::Less || lower_case == target
        }
        _ => false,
    }
}

fn result_is_greater_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => target.cmp(&s.to_lowercase()) == std::cmp::Ordering::Less,
        _ => false,
    }
}

fn result_is_greater_or_equal_than_string(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::String(s) => {
            let lower_case = &s.to_lowercase();
            target.cmp(lower_case) == std::cmp::Ordering::Greater || lower_case == target
        }
        _ => false,
    }
}

// ERRORS ///
//********///

fn result_is_equal_to_error(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::Error { error, .. } => target == error.to_string(),
        _ => false,
    }
}

fn result_is_not_equal_to_error(calc_result: &CalcResult, target: &str) -> bool {
    match calc_result {
        CalcResult::Error { error, .. } => target != error.to_string(),
        _ => true,
    }
}

// EMPTY ///
//*******///

// Note that these two are not inverse of each other.
// In particular, you can never match an empty cell.

fn result_is_not_equal_to_empty(calc_result: &CalcResult) -> bool {
    !matches!(calc_result, CalcResult::EmptyCell)
}

fn result_is_equal_to_empty(calc_result: &CalcResult) -> bool {
    match calc_result {
        CalcResult::Number(f) => (f - 0.0).abs() < f64::EPSILON,
        _ => false,
    }
}

/// This returns a function (closure) of signature fn(&CalcResult) -> bool
/// It is Boxed because it returns different closures, so the size cannot be known at compile time
/// The lifetime (a) of value has to be longer or equal to the lifetime of the returned closure
pub(crate) fn build_criteria<'a>(value: &'a CalcResult) -> Box<dyn Fn(&CalcResult) -> bool + 'a> {
    match value {
        CalcResult::String(s) => {
            if let Some(v) = s.strip_prefix("<=") {
                // TODO: I am not implementing <= ERROR or <= BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_less_or_equal_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else {
                    Box::new(move |x| result_is_less_or_equal_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix(">=") {
                // TODO: I am not implementing >= ERROR or >= BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_greater_or_equal_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else {
                    Box::new(move |x| result_is_greater_or_equal_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix("<>") {
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_not_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_not_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_not_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    if let Ok(reg) = from_wildcard_to_regex(&v.to_lowercase(), true) {
                        Box::new(move |x| !result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else if v.is_empty() {
                    Box::new(result_is_not_equal_to_empty)
                } else {
                    Box::new(move |x| result_is_not_equal_to_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix('<') {
                // TODO: I am not implementing < ERROR or < BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_less_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else {
                    Box::new(move |x| result_is_less_than_string(x, &v.to_lowercase()))
                }
            } else if let Some(v) = s.strip_prefix('>') {
                // TODO: I am not implementing > ERROR or > BOOLEAN
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_greater_than_number(x, f))
                } else if v.is_empty() {
                    Box::new(move |_x| false)
                } else {
                    Box::new(move |x| result_is_greater_than_string(x, &v.to_lowercase()))
                }
            } else {
                let v = if let Some(a) = s.strip_prefix('=') {
                    a
                } else {
                    s
                };
                if let Ok(f) = v.parse::<f64>() {
                    Box::new(move |x| result_is_equal_to_number(x, f))
                } else if let Ok(b) = v.to_lowercase().parse::<bool>() {
                    Box::new(move |x| result_is_equal_to_bool(x, b))
                } else if is_english_error_string(v) {
                    Box::new(move |x| result_is_equal_to_error(x, v))
                } else if v.contains('*') || v.contains('?') {
                    if let Ok(reg) = from_wildcard_to_regex(&v.to_lowercase(), true) {
                        Box::new(move |x| result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else {
                    Box::new(move |x| result_is_equal_to_string(x, &v.to_lowercase()))
                }
            }
        }
        CalcResult::Number(target) => Box::new(move |x| result_is_equal_to_number(x, *target)),
        CalcResult::Boolean(b) => Box::new(move |x| result_is_equal_to_bool(x, *b)),
        CalcResult::Error { error, .. } => {
            // An error will match an error (never a string that is an error)
            Box::new(move |x| result_is_equal_to_error(x, &error.to_string()))
        }
        CalcResult::Range { left: _, right: _ } => Box::new(move |_x| false),
        CalcResult::Array(_) => Box::new(move |_x| false),
        CalcResult::EmptyCell | CalcResult::EmptyArg => Box::new(result_is_equal_to_empty),
    }
}

// ---------------------------------------------------------------------------
// Generic numeric collector with configurable behaviour
// ---------------------------------------------------------------------------
/// Walks every argument node applying Excel-compatible coercion rules and
/// returns a flat `Vec<f64>`.
/// Behaviour is controlled through `CollectOpts` so that one routine can serve
/// AVERAGE, STDEVA, CORREL, etc.
pub(crate) fn collect_numeric_values(
    model: &mut Model,
    args: &[Node],
    cell: CellReferenceIndex,
    opts: CollectOpts,
) -> Result<Vec<f64>, CalcResult> {
    let mut values = Vec::new();

    for arg in args {
        match model.evaluate_node_in_context(arg, cell) {
            CalcResult::Number(v) => values.push(v),
            CalcResult::Boolean(b) => {
                if matches!(arg, Node::ReferenceKind { .. }) {
                    if opts.include_bool_refs {
                        values.push(if b { 1.0 } else { 0.0 });
                    }
                } else {
                    values.push(if b { 1.0 } else { 0.0 });
                }
            }
            CalcResult::String(s) => {
                // String literals – we always try to coerce to number.
                if !matches!(arg, Node::ReferenceKind { .. }) {
                    if let Ok(t) = s.parse::<f64>() {
                        values.push(t);
                    } else {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument cannot be cast into number".to_string(),
                        ));
                    }
                    continue;
                }
                // String coming from reference
                if opts.string_ref_as_zero {
                    values.push(0.0);
                } // else: silently skip non-numeric string references (Excel behaviour)
            }
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                for row in left.row..=right.row {
                    for column in left.column..=right.column {
                        match model.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }) {
                            CalcResult::Number(v) => values.push(v),
                            CalcResult::Boolean(b) => {
                                if opts.include_bool_refs {
                                    values.push(if b { 1.0 } else { 0.0 });
                                }
                            }
                            CalcResult::String(_) => {
                                if opts.string_ref_as_zero {
                                    values.push(0.0);
                                }
                            }
                            error @ CalcResult::Error { .. } => return Err(error),
                            CalcResult::Range { .. } => {
                                return Err(CalcResult::new_error(
                                    Error::ERROR,
                                    cell,
                                    "Unexpected Range".to_string(),
                                ))
                            }
                            CalcResult::Array(_) => {
                                return Err(CalcResult::Error {
                                    error: Error::NIMPL,
                                    origin: cell,
                                    message: "Arrays not supported yet".to_string(),
                                })
                            }
                            CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return Err(error),
            CalcResult::Array(_) => {
                return Err(CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "Arrays not supported yet".to_string(),
                })
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => {}
        }
    }

    Ok(values)
}

/// Options for scanning ranges
#[derive(Clone, Copy, Default)]
pub(crate) struct ScanRangeOpts {
    /// Whether to expand whole-row/column ranges to actual data bounds
    pub expand_full_ranges: bool,
}

/// Scans a range and applies a closure to each cell result, collecting results into a Vec.
///
/// This utility extracts the common pattern found in statistical functions like LARGE, SMALL,
/// QUARTILE, PERCENTILE, RANK, etc. that need to:
/// 1. Check cross-sheet ranges (returns error if different sheets)
/// 2. Optionally expand whole-row/column ranges to actual data bounds
/// 3. Iterate through each cell and apply custom logic
/// 4. Collect results or propagate errors
///
/// # Arguments
/// * `model` - The spreadsheet model
/// * `range` - The range to scan
/// * `cell` - The cell context for error reporting
/// * `opts` - Options for scanning behavior
/// * `cell_fn` - Closure that processes each cell result and returns an optional result
///
/// # Returns
/// `Ok(Vec<T>)` with collected results, or `Err(CalcResult)` on error
pub(crate) fn scan_range<T, F>(
    model: &mut Model,
    range: &crate::calc_result::Range,
    cell: crate::expressions::types::CellReferenceIndex,
    opts: ScanRangeOpts,
    mut cell_fn: F,
) -> Result<Vec<T>, CalcResult>
where
    F: FnMut(&CalcResult) -> Result<Option<T>, CalcResult>,
{
    use crate::constants::{LAST_COLUMN, LAST_ROW};

    // Check cross-sheet ranges
    if range.left.sheet != range.right.sheet {
        return Err(CalcResult::new_error(
            crate::expressions::token::Error::VALUE,
            cell,
            "Ranges are in different sheets".to_string(),
        ));
    }

    let row1 = range.left.row;
    let mut row2 = range.right.row;
    let column1 = range.left.column;
    let mut column2 = range.right.column;

    // Expand whole-row/column ranges if requested
    if opts.expand_full_ranges {
        if row1 == 1 && row2 == LAST_ROW {
            row2 = match model.workbook.worksheet(range.left.sheet) {
                Ok(s) => s.dimension().max_row,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        crate::expressions::token::Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", range.left.sheet),
                    ));
                }
            };
        }
        if column1 == 1 && column2 == LAST_COLUMN {
            column2 = match model.workbook.worksheet(range.left.sheet) {
                Ok(s) => s.dimension().max_column,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        crate::expressions::token::Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", range.left.sheet),
                    ));
                }
            };
        }
    }

    let mut results = Vec::new();

    // Iterate through the range
    for row in row1..=row2 {
        for column in column1..=column2 {
            let cell_result = model.evaluate_cell(crate::expressions::types::CellReferenceIndex {
                sheet: range.left.sheet,
                row,
                column,
            });

            if let Some(value) = cell_fn(&cell_result)? {
                results.push(value);
            }
        }
    }

    Ok(results)
}

/// Collect a numeric series preserving positional information.
///
/// Given a single argument (range, reference, literal, or array), returns a
/// vector with the same length as the flattened input. Each position contains
/// `Some(f64)` when the corresponding element is numeric and `None` when it is
/// non-numeric or empty. Errors are propagated immediately.
///
/// Behaviour mirrors Excel's rules used by paired-data statistical functions
/// (SLOPE, INTERCEPT, CORREL, etc.):
/// - Booleans/string literals are coerced to numbers, literals coming from
///   references are ignored.
/// - Non-numeric cells become `None`, keeping the alignment between two series.
/// - Ranges crossing sheets cause a `#VALUE!` error.
/// - When `expand_full_rows_cols` is true, whole-row/whole-column ranges are
///   reduced to the sheet's actual dimensions.
pub(crate) fn collect_series(
    model: &mut Model,
    node: &Node,
    cell: CellReferenceIndex,
    expand_full_rows_cols: bool,
) -> Result<Vec<Option<f64>>, CalcResult> {
    let is_reference = matches!(
        node,
        Node::ReferenceKind { .. } | Node::RangeKind { .. } | Node::OpRangeKind { .. }
    );

    match model.evaluate_node_in_context(node, cell) {
        CalcResult::Number(v) => Ok(vec![Some(v)]),
        CalcResult::Boolean(b) => {
            if is_reference {
                Ok(vec![None])
            } else {
                Ok(vec![Some(if b { 1.0 } else { 0.0 })])
            }
        }
        CalcResult::String(s) => {
            if is_reference {
                Ok(vec![None])
            } else if let Ok(v) = s.parse::<f64>() {
                Ok(vec![Some(v)])
            } else {
                Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Argument cannot be cast into number".to_string(),
                ))
            }
        }
        CalcResult::Range { left, right } => {
            if left.sheet != right.sheet {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ));
            }
            let row1 = left.row;
            let mut row2 = right.row;
            let col1 = left.column;
            let mut col2 = right.column;

            if expand_full_rows_cols {
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = model
                        .workbook
                        .worksheet(left.sheet)
                        .map_err(|_| {
                            CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            )
                        })?
                        .dimension()
                        .max_row;
                }
                if col1 == 1 && col2 == LAST_COLUMN {
                    col2 = model
                        .workbook
                        .worksheet(left.sheet)
                        .map_err(|_| {
                            CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            )
                        })?
                        .dimension()
                        .max_column;
                }
            }

            let mut values = Vec::new();
            for row in row1..=row2 {
                for column in col1..=col2 {
                    let cell_result = model.evaluate_cell(CellReferenceIndex {
                        sheet: left.sheet,
                        row,
                        column,
                    });
                    match cell_result {
                        CalcResult::Number(n) => values.push(Some(n)),
                        error @ CalcResult::Error { .. } => {
                            return Err(error);
                        }
                        _ => values.push(None),
                    }
                }
            }
            Ok(values)
        }
        CalcResult::Array(arr) => {
            let mut values = Vec::new();
            for row in arr {
                for val in row {
                    match val {
                        ArrayNode::Number(n) => values.push(Some(n)),
                        ArrayNode::Boolean(b) => values.push(Some(if b { 1.0 } else { 0.0 })),
                        ArrayNode::String(s) => match s.parse::<f64>() {
                            Ok(v) => values.push(Some(v)),
                            Err(_) => {
                                return Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Argument cannot be cast into number".to_string(),
                                ))
                            }
                        },
                        ArrayNode::Error(e) => {
                            return Err(CalcResult::Error {
                                error: e,
                                origin: cell,
                                message: "Error in array".to_string(),
                            })
                        }
                    }
                }
            }
            Ok(values)
        }
        CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![None]),
        error @ CalcResult::Error { .. } => Err(error),
    }
}
