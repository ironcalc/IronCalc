#![deny(missing_docs)]

use std::collections::HashMap;
use std::vec::Vec;

use crate::expressions::parser::static_analysis::run_static_analysis_on_node;
use crate::{
    calc_result::{CalcResult, Range},
    cell::CellValue,
    constants::{self, LAST_COLUMN, LAST_ROW},
    expressions::{
        lexer::LexerMode,
        parser::{
            move_formula::{move_formula, MoveContext},
            static_analysis::StaticResult,
            stringify::{rename_defined_name_in_node, to_localized_string, to_rc_format},
            ArrayNode, Node, Parser,
        },
        token::{get_error_by_name, Error, OpCompare, OpProduct, OpSum, OpUnary},
        types::*,
        utils::{self, is_valid_column_number, is_valid_identifier, is_valid_row},
    },
    formatter::{
        format::{format_number, parse_formatted_number},
        lexer::is_likely_date_number_format,
    },
    functions::util::compare_values,
    implicit_intersection::implicit_intersection,
    language::{get_default_language, get_language, Language},
    locale::{get_locale, Locale},
    types::*,
    utils as common,
};

use chrono_tz::Tz;

#[cfg(test)]
pub use crate::mock_time::get_milliseconds_since_epoch;

/// Number of milliseconds since January 1, 1970
/// Used by time and date functions. It takes the value from the environment:
/// * The Operative System
/// * The JavaScript environment
/// * Or mocked for tests
#[cfg(not(test))]
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::expect_used)]
pub fn get_milliseconds_since_epoch() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("problem with system time")
        .as_millis() as i64
}

/// Number of milliseconds since January 1, 1970
/// Used by time and date functions. It takes the value from the environment:
/// * The Operative System
/// * The JavaScript environment
/// * Or mocked for tests
#[cfg(not(test))]
#[cfg(target_arch = "wasm32")]
pub fn get_milliseconds_since_epoch() -> i64 {
    use js_sys::Date;
    Date::now() as i64
}

// The structure of a cell.
// It can be:
// * A single cell
// * The anchor of an array formula
// * The anchor of a dynamic formula
// * A part of an array formula spill
// * A part of a dynamic formula spill
pub(crate) enum CellStructure {
    SingleCell,
    ArrayFormula {
        range: (i32, i32),
    },
    DynamicFormula {
        range: (i32, i32),
    },
    SpillArray {
        anchor: (i32, i32),
        range: (i32, i32),
    },
    SpillDynamic {
        anchor: (i32, i32),
        range: (i32, i32),
    },
}

/// A cell might be evaluated or being evaluated
#[derive(Clone)]
pub(crate) enum CellState {
    /// The cell has already been evaluated
    Evaluated,
    /// The cell is being evaluated
    Evaluating,
}

/// A parsed formula for a defined name
#[derive(Clone)]
pub(crate) enum ParsedDefinedName {
    /// CellReference (`=C4`)
    CellReference(CellReferenceIndex),
    /// A Range (`=C4:D6`)
    RangeReference(Range),
    /// `=SomethingElse`
    InvalidDefinedNameFormula,
}

/// Formatting settings for a locale
pub struct FmtSettings {
    /// Currency format
    pub currency: String,
    /// Currency format with symbol
    pub currency_format: String,
    /// Short date format
    pub short_date: String,
    /// Example of short date format
    pub short_date_example: String,
    /// Long date format
    pub long_date: String,
    /// Example of long date format
    pub long_date_example: String,
    /// Number format
    pub number_fmt: String,
    /// Example of number format
    pub number_example: String,
}

fn array_node_to_formula_value(node: ArrayNode) -> FormulaValue {
    match node {
        ArrayNode::Boolean(b) => FormulaValue::Boolean(b),
        ArrayNode::Number(n) => FormulaValue::Number(n),
        ArrayNode::String(s) => FormulaValue::Text(s),
        ArrayNode::Error(ei) => FormulaValue::Error {
            ei,
            o: String::new(),
            m: String::new(),
        },
    }
}

fn array_node_to_spill_value(node: ArrayNode) -> SpillValue {
    match node {
        ArrayNode::Boolean(b) => SpillValue::Boolean(b),
        ArrayNode::Number(n) => SpillValue::Number(n),
        ArrayNode::String(s) => SpillValue::Text(s),
        ArrayNode::Error(ei) => SpillValue::Error(ei),
    }
}

fn formula_value_to_spill_value(v: &FormulaValue) -> SpillValue {
    match v {
        FormulaValue::Unevaluated => SpillValue::Error(Error::ERROR),
        FormulaValue::Boolean(b) => SpillValue::Boolean(*b),
        FormulaValue::Number(n) => SpillValue::Number(*n),
        FormulaValue::Text(s) => SpillValue::Text(s.clone()),
        FormulaValue::Error { ei, .. } => SpillValue::Error(ei.clone()),
    }
}

/// A dynamical IronCalc model.
///
/// Its is composed of a `Workbook`. Everything else are dynamical quantities:
///
/// * The Locale: a parsed version of the Workbook's locale
/// * The Timezone: an object representing the Workbook's timezone
/// * The language. Note that the timezone and the locale belong to the workbook while
///   the language can be different for different users looking _at the same_ workbook.
/// * Parsed Formulas: All the formulas in the workbook are parsed here (runtime only)
/// * A list of cells with its status (evaluating, evaluated, not evaluated)
/// * A dictionary with the shared strings and their indices.
///   This is an optimization for large files (~1 million rows)
pub struct Model<'a> {
    /// A Rust internal representation of an Excel workbook
    pub workbook: Workbook,
    /// A list of parsed formulas
    pub parsed_formulas: Vec<Vec<(Node, StaticResult)>>,
    /// A list of parsed defined names
    pub(crate) parsed_defined_names: HashMap<(Option<u32>, String), ParsedDefinedName>,
    /// An optimization to lookup strings faster
    pub(crate) shared_strings: HashMap<String, usize>,
    /// An instance of the parser
    pub(crate) parser: Parser<'a>,
    /// The list of cells with formulas that are evaluated or being evaluated
    pub(crate) cells: HashMap<(u32, i32, i32), CellState>,
    /// The locale of the model
    pub(crate) locale: &'a Locale,
    /// The language used
    pub(crate) language: &'a Language,
    /// The timezone used to evaluate the model
    pub(crate) tz: Tz,
    /// The view id. A view consists of a selected sheet and ranges.
    pub(crate) view_id: u32,
}

// FIXME: Maybe this should be the same as CellReference
/// A struct pointing to a cell
pub struct CellIndex {
    /// Sheet index (0-indexed)
    pub index: u32,
    /// Row index
    pub row: i32,
    /// Column index
    pub column: i32,
}

impl<'a> Model<'a> {
    pub(crate) fn evaluate_node_with_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node {
            Node::ReferenceKind {
                sheet_name: _,
                sheet_index,
                absolute_row,
                absolute_column,
                row,
                column,
            } => {
                let mut row1 = *row;
                let mut column1 = *column;
                if !absolute_row {
                    row1 += cell.row;
                }
                if !absolute_column {
                    column1 += cell.column;
                }
                CalcResult::Range {
                    left: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: row1,
                        column: column1,
                    },
                    right: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: row1,
                        column: column1,
                    },
                }
            }
            Node::RangeKind {
                sheet_name: _,
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
                let mut row_left = *row1;
                let mut column_left = *column1;
                if !absolute_row1 {
                    row_left += cell.row;
                }
                if !absolute_column1 {
                    column_left += cell.column;
                }
                let mut row_right = *row2;
                let mut column_right = *column2;
                if !absolute_row2 {
                    row_right += cell.row;
                }
                if !absolute_column2 {
                    column_right += cell.column;
                }
                // FIXME: HACK. The parser is currently parsing Sheet3!A1:A10 as Sheet3!A1:(present sheet)!A10
                CalcResult::Range {
                    left: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: row_left,
                        column: column_left,
                    },
                    right: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: row_right,
                        column: column_right,
                    },
                }
            }
            Node::ImplicitIntersection {
                automatic: _,
                child,
            } => match self.evaluate_node_with_reference(child, cell) {
                CalcResult::Range { left, right } => CalcResult::Range { left, right },
                _ => CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    format!("Error with Implicit Intersection in cell {cell:?}"),
                ),
            },
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

    fn get_range(&mut self, left: &Node, right: &Node, cell: CellReferenceIndex) -> CalcResult {
        let left_result = self.evaluate_node_with_reference(left, cell);
        let right_result = self.evaluate_node_with_reference(right, cell);
        match (left_result, right_result) {
            (
                CalcResult::Range {
                    left: left1,
                    right: right1,
                },
                CalcResult::Range {
                    left: left2,
                    right: right2,
                },
            ) => {
                if left1.row == right1.row
                    && left1.column == right1.column
                    && left2.row == right2.row
                    && left2.column == right2.column
                {
                    return CalcResult::Range {
                        left: left1,
                        right: right2,
                    };
                }
                CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid range".to_string(),
                }
            }
            _ => CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Invalid range".to_string(),
            },
        }
    }

    fn formula_without_prefix<'b>(&self, value: &'b str) -> Option<&'b str> {
        if let Some(stripped) = value.strip_prefix('=') {
            if stripped.is_empty() {
                None
            } else {
                Some(stripped)
            }
        } else if let Some(stripped) = value.strip_prefix(['+', '-']) {
            if stripped.is_empty() || self.cast_number(stripped).is_some() {
                None
            } else {
                Some(value)
            }
        } else {
            None
        }
    }

    pub(crate) fn evaluate_node_in_context(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use Node::*;
        match node {
            OpSumKind { kind, left, right } => match kind {
                OpSum::Add => self.handle_arithmetic(left, right, cell, &|f1, f2| Ok(f1 + f2)),
                OpSum::Minus => self.handle_arithmetic(left, right, cell, &|f1, f2| Ok(f1 - f2)),
            },
            NumberKind(value) => CalcResult::Number(*value),
            StringKind(value) => CalcResult::String(value.replace(r#""""#, r#"""#)),
            BooleanKind(value) => CalcResult::Boolean(*value),
            ReferenceKind {
                sheet_name: _,
                sheet_index,
                absolute_row,
                absolute_column,
                row,
                column,
            } => {
                let mut row1 = *row;
                let mut column1 = *column;
                if !absolute_row {
                    row1 += cell.row;
                }
                if !absolute_column {
                    column1 += cell.column;
                }
                self.evaluate_cell(CellReferenceIndex {
                    sheet: *sheet_index,
                    row: row1,
                    column: column1,
                })
            }
            WrongReferenceKind { .. } => {
                CalcResult::new_error(Error::REF, cell, "Wrong reference".to_string())
            }
            OpRangeKind { left, right } => self.get_range(left, right, cell),
            WrongRangeKind { .. } => {
                CalcResult::new_error(Error::REF, cell, "Wrong range".to_string())
            }
            RangeKind {
                sheet_index,
                row1,
                column1,
                row2,
                column2,
                absolute_column1,
                absolute_row2,
                absolute_row1,
                absolute_column2,
                sheet_name: _,
            } => {
                let r1 = if *absolute_row1 {
                    *row1
                } else {
                    *row1 + cell.row
                };
                let r2 = if *absolute_row2 {
                    *row2
                } else {
                    *row2 + cell.row
                };
                let c1 = if *absolute_column1 {
                    *column1
                } else {
                    *column1 + cell.column
                };
                let c2 = if *absolute_column2 {
                    *column2
                } else {
                    *column2 + cell.column
                };
                CalcResult::Range {
                    left: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: r1.min(r2),
                        column: c1.min(c2),
                    },
                    right: CellReferenceIndex {
                        sheet: *sheet_index,
                        row: r1.max(r2),
                        column: c1.max(c2),
                    },
                }
            }
            OpConcatenateKind { left, right } => {
                let l = match self.get_string(left, cell) {
                    Ok(f) => f,
                    Err(s) => {
                        return s;
                    }
                };
                let r = match self.get_string(right, cell) {
                    Ok(f) => f,
                    Err(s) => {
                        return s;
                    }
                };
                let result = format!("{l}{r}");
                CalcResult::String(result)
            }
            OpProductKind { kind, left, right } => match kind {
                OpProduct::Times => {
                    self.handle_arithmetic(left, right, cell, &|f1, f2| Ok(f1 * f2))
                }
                OpProduct::Divide => self.handle_arithmetic(left, right, cell, &|f1, f2| {
                    if f2 == 0.0 {
                        Err(Error::DIV)
                    } else {
                        Ok(f1 / f2)
                    }
                }),
            },
            OpPowerKind { left, right } => {
                self.handle_arithmetic(left, right, cell, &|f1, f2| Ok(f1.powf(f2)))
            }
            FunctionKind { kind, args } => self.evaluate_function(kind, args, cell),
            InvalidFunctionKind { name, args: _ } => {
                CalcResult::new_error(Error::NAME, cell, format!("Invalid function: {name}"))
            }
            ArrayKind(s) => CalcResult::Array(s.to_owned()),
            DefinedNameKind((name, scope, _)) => {
                if let Ok(Some(parsed_defined_name)) = self.get_parsed_defined_name(name, *scope) {
                    match parsed_defined_name {
                        ParsedDefinedName::CellReference(reference) => {
                            self.evaluate_cell(reference)
                        }
                        ParsedDefinedName::RangeReference(range) => CalcResult::Range {
                            left: range.left,
                            right: range.right,
                        },
                        ParsedDefinedName::InvalidDefinedNameFormula => CalcResult::new_error(
                            Error::NAME,
                            cell,
                            format!("Defined name \"{name}\" is not a reference."),
                        ),
                    }
                } else {
                    CalcResult::new_error(
                        Error::NAME,
                        cell,
                        format!("Defined name \"{name}\" not found."),
                    )
                }
            }
            TableNameKind(s) => CalcResult::new_error(
                Error::NAME,
                cell,
                format!("table name \"{s}\" not supported."),
            ),
            WrongVariableKind(s) => CalcResult::new_error(
                Error::NAME,
                cell,
                format!("Variable name \"{s}\" not found."),
            ),
            CompareKind { kind, left, right } => {
                let l = self.evaluate_node_in_context(left, cell);
                if l.is_error() {
                    return l;
                }
                let r = self.evaluate_node_in_context(right, cell);
                if r.is_error() {
                    return r;
                }
                let compare = compare_values(&l, &r);
                match kind {
                    OpCompare::Equal => {
                        if compare == 0 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                    OpCompare::LessThan => {
                        if compare == -1 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                    OpCompare::GreaterThan => {
                        if compare == 1 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                    OpCompare::LessOrEqualThan => {
                        if compare < 1 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                    OpCompare::GreaterOrEqualThan => {
                        if compare > -1 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                    OpCompare::NonEqual => {
                        if compare != 0 {
                            CalcResult::Boolean(true)
                        } else {
                            CalcResult::Boolean(false)
                        }
                    }
                }
            }
            UnaryKind { kind, right } => {
                let r = match self.get_number(right, cell) {
                    Ok(f) => f,
                    Err(s) => {
                        return s;
                    }
                };
                match kind {
                    OpUnary::Minus => CalcResult::Number(-r),
                    OpUnary::Percentage => CalcResult::Number(r / 100.0),
                }
            }
            ErrorKind(kind) => CalcResult::new_error(kind.clone(), cell, "".to_string()),
            ParseErrorKind {
                formula,
                message,
                position: _,
            } => CalcResult::new_error(
                Error::ERROR,
                cell,
                format!("Error parsing {formula}: {message}"),
            ),
            EmptyArgKind => CalcResult::EmptyArg,
            ImplicitIntersection {
                automatic: _,
                child,
            } => match self.evaluate_node_with_reference(child, cell) {
                CalcResult::Range { left, right } => {
                    match implicit_intersection(&cell, &Range { left, right }) {
                        Some(cell_reference) => self.evaluate_cell(cell_reference),
                        None => CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error with Implicit Intersection in cell {cell:?}"),
                        ),
                    }
                }
                _ => self.evaluate_node_in_context(child, cell),
            },
        }
    }

    fn cell_reference_to_string(
        &self,
        cell_reference: &CellReferenceIndex,
    ) -> Result<String, String> {
        let sheet = self.workbook.worksheet(cell_reference.sheet)?;
        let column = utils::number_to_column(cell_reference.column)
            .ok_or_else(|| "Invalid column".to_string())?;
        if !is_valid_row(cell_reference.row) {
            return Err("Invalid row".to_string());
        }
        Ok(format!("{}!{}{}", sheet.name, column, cell_reference.row))
    }

    fn get_value_from_array(
        &self,
        array: &[Vec<ArrayNode>],
        row: i32,
        column: i32,
    ) -> Option<ArrayNode> {
        let width = array[0].len() as i32;
        let height = array.len() as i32;
        if row < 1 || row > height || column < 1 || column > width {
            return None;
        }
        let value = &array[(row - 1) as usize][(column - 1) as usize];
        Some(value.clone())
    }

    /// Sets `result` in the cell given by `sheet` sheet index, row and column
    /// Note that will panic if the cell does not exist
    /// It will do nothing if the cell does not have a formula
    /// If the result is an array it will spill over other cells
    /// If the formula is an array formula it will update the spill area.
    ///    If the array is smaller than the spill area it will fill the remaining cells with #N/A error
    ///    If the array is just one element it will fill the original range with that element
    fn set_cells_with_result(
        &mut self,
        cell_reference: CellReferenceIndex,
        cell: &Cell,
        result: &CalcResult,
    ) -> Result<(), String> {
        let CellReferenceIndex { sheet, column, row } = cell_reference;
        let original_range = match cell {
            Cell::ArrayFormula {
                r,
                kind: ArrayKind::Cse,
                ..
            } => Some((false, (r.0, r.1))),
            Cell::ArrayFormula {
                r,
                kind: ArrayKind::Dynamic,
                ..
            } => Some((true, (r.0, r.1))),
            _ => None,
        };
        let s = cell.get_style();
        let formula = match cell.get_formula() {
            Some(f) => f,
            None => return Ok(()),
        };
        // Handle array results separately: they always return early, writing all cells
        // themselves. By dispatching here we avoid needing an unreachable arm in the
        // `new_cell` match below.
        if let CalcResult::Array(array) = result {
            if array.is_empty() || array[0].is_empty() {
                return Ok(());
            }
            let array_width = array[0].len() as i32;
            let array_height = array.len() as i32;

            match original_range {
                Some((true, _)) => {
                    // Check that the full spill area (based on actual result dimensions) is clear.
                    // The stored range may be (1,1) on first evaluation, so we must re-check here.
                    let sheet_data = &self.workbook.worksheets[sheet as usize].sheet_data;
                    for r in row..row + array_height {
                        let row_data = sheet_data.get(&r);
                        for c in column..column + array_width {
                            if r == row && c == column {
                                continue;
                            }
                            let blocking = row_data
                                .and_then(|row_map| row_map.get(&c))
                                .map(|spill_cell| !matches!(spill_cell, Cell::EmptyCell { .. }))
                                .unwrap_or(false);
                            if blocking {
                                return self.set_cells_with_result(
                                    cell_reference,
                                    cell,
                                    &CalcResult::new_error(
                                        Error::SPILL,
                                        cell_reference,
                                        "Cannot spill array result".to_string(),
                                    ),
                                );
                            }
                        }
                    }
                    // Dynamic formula: spill the array into adjacent cells.
                    // Cells are created on demand via update_cell since they may not exist yet.
                    for r in row..row + array_height {
                        for c in column..column + array_width {
                            let value = array[(r - row) as usize][(c - column) as usize].clone();
                            let cell = if r == row && c == column {
                                Cell::ArrayFormula {
                                    f: formula,
                                    s,
                                    r: (array_width, array_height),
                                    kind: ArrayKind::Dynamic,
                                    v: array_node_to_formula_value(value),
                                }
                            } else {
                                Cell::SpillCell {
                                    a: (row, column),
                                    s,
                                    v: array_node_to_spill_value(value),
                                }
                            };
                            self.workbook.worksheets[sheet as usize].update_cell(r, c, cell)?;
                        }
                    }
                    return Ok(());
                }
                Some((false, (original_width, original_height))) => {
                    // CSE array formula: fill the declared range with the array values.
                    // Use relative indices for get_value_from_array (1-based).
                    for r in row..row + original_height {
                        for c in column..column + original_width {
                            let rel_row = r - row + 1;
                            let rel_col = c - column + 1;
                            let value = self.get_value_from_array(array, rel_row, rel_col);
                            let new_cell = if r == row && c == column {
                                let fv = match value {
                                    Some(node) => array_node_to_formula_value(node),
                                    None => FormulaValue::Error {
                                        ei: Error::NIMPL,
                                        o: "".to_string(),
                                        m: "Unexpected array result".to_string(),
                                    },
                                };
                                Cell::ArrayFormula {
                                    f: formula,
                                    s,
                                    r: (original_width, original_height),
                                    kind: ArrayKind::Cse,
                                    v: fv,
                                }
                            } else {
                                let sv = match value {
                                    Some(node) => array_node_to_spill_value(node),
                                    None => SpillValue::Error(Error::VALUE),
                                };
                                Cell::SpillCell {
                                    s,
                                    a: (row, column),
                                    v: sv,
                                }
                            };
                            *self.workbook.worksheets[sheet as usize]
                                .sheet_data
                                .get_mut(&r)
                                .ok_or("expected a row")?
                                .get_mut(&c)
                                .ok_or("expected a column")? = new_cell;
                        }
                    }
                    // All cells (anchor + spills) have been written above.
                    return Ok(());
                }
                None => {
                    // this should not happen, but we need to handle it anyway
                    // It means we got an array in a cell that we 'thought' could only return a scalar
                    // It means we made a mistake in parse time
                    debug_assert!(false, "Unexpected array result in non-array formula");
                    *self.workbook.worksheets[sheet as usize]
                        .sheet_data
                        .get_mut(&row)
                        .ok_or("expected a row")?
                        .get_mut(&column)
                        .ok_or("expected a column")? = Cell::CellFormula {
                        f: formula,
                        s,
                        v: FormulaValue::Error {
                            ei: Error::NIMPL,
                            o: "".to_string(),
                            m: "Unexpected array result".to_string(),
                        },
                    };
                    return Err("Unexpected array result".to_string());
                }
            }
        }

        let formula_value = match result {
            CalcResult::Number(value) => {
                // safety belt
                if value.is_nan() || value.is_infinite() {
                    // This should never happen, is there a way we can log this events?
                    return self.set_cells_with_result(
                        cell_reference,
                        cell,
                        &CalcResult::Error {
                            error: Error::NUM,
                            origin: cell_reference,
                            message: "".to_string(),
                        },
                    );
                }
                FormulaValue::Number(*value)
            }
            CalcResult::String(value) => FormulaValue::Text(value.clone()),
            CalcResult::Boolean(value) => FormulaValue::Boolean(*value),
            CalcResult::Error {
                error,
                origin,
                message,
            } => {
                let o = match self.cell_reference_to_string(origin) {
                    Ok(s) => s,
                    Err(_) => "".to_string(),
                };
                FormulaValue::Error {
                    ei: error.clone(),
                    o,
                    m: message.to_string(),
                }
            }
            CalcResult::Range { .. } => {
                // This should never happen
                debug_assert!(false, "Unexpected range result in non-array formula");
                return Err("Cannot set a range as cell value".to_string());
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => {
                // We treat empty cells as number 0.
                return self.set_cells_with_result(cell_reference, cell, &CalcResult::Number(0.0));
            }
            // CalcResult::Array is handled before this match (see above); it always returns early.
            CalcResult::Array(_) => {
                debug_assert!(false, "Unexpected array result in non-array formula");
                return Err("Unexpected array result in non-array formula".to_string());
            }
        };

        let new_cell = match original_range {
            Some((is_dynamic, (width, height))) => {
                let (kind, r) = if is_dynamic {
                    (ArrayKind::Dynamic, (1, 1))
                } else {
                    (ArrayKind::Cse, (width, height))
                };
                Cell::ArrayFormula {
                    f: formula,
                    s,
                    r,
                    kind,
                    v: formula_value.clone(),
                }
            }
            None => Cell::CellFormula {
                f: formula,
                s,
                v: formula_value.clone(),
            },
        };

        // If the cell is the anchor of a CSE array formula, fill all spill cells
        if let Some((false, (width, height))) = original_range {
            let spill_cell = Cell::SpillCell {
                a: (row, column),
                s,
                v: formula_value_to_spill_value(&formula_value),
            };
            let ws = &mut self.workbook.worksheets[sheet as usize];
            for r in row..row + height {
                for c in column..column + width {
                    if r == row && c == column {
                        continue;
                    }
                    ws.update_cell(r, c, spill_cell.clone())?;
                }
            }
        }

        self.workbook.worksheets[sheet as usize].update_cell(row, column, new_cell)?;
        Ok(())
    }

    /// Sets the color of the sheet tab.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// assert_eq!(model.workbook.worksheet(0)?.color, None);
    /// model.set_sheet_color(0, "#DBBE29")?;
    /// assert_eq!(model.workbook.worksheet(0)?.color, Some("#DBBE29".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_sheet_color(&mut self, sheet: u32, color: &str) -> Result<(), String> {
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        if color.is_empty() {
            worksheet.color = None;
            return Ok(());
        }
        if common::is_valid_hex_color(color) {
            worksheet.color = Some(color.to_string());
            return Ok(());
        }
        Err(format!("Invalid color: {color}"))
    }

    /// Changes the visibility of a sheet
    pub fn set_sheet_state(&mut self, sheet: u32, state: SheetState) -> Result<(), String> {
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        worksheet.state = state;
        Ok(())
    }

    /// Makes the grid lines in the sheet visible (`true`) or hidden (`false`)
    pub fn set_show_grid_lines(&mut self, sheet: u32, show_grid_lines: bool) -> Result<(), String> {
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        worksheet.show_grid_lines = show_grid_lines;
        Ok(())
    }

    // Returns the 'single' value of a cell. Not arrays or ranges.
    fn get_cell_value(&self, cell: &Cell, cell_reference: CellReferenceIndex) -> CalcResult {
        use Cell::*;
        match cell {
            EmptyCell { .. } => CalcResult::EmptyCell,
            BooleanCell { v, .. } => CalcResult::Boolean(*v),
            NumberCell { v, .. } => CalcResult::Number(*v),
            ErrorCell { ei, .. } => {
                let message = ei.to_localized_error_string(self.language);
                CalcResult::new_error(ei.clone(), cell_reference, message)
            }
            SharedString { si, .. } => {
                if let Some(s) = self.workbook.shared_strings.get(*si as usize) {
                    CalcResult::String(s.clone())
                } else {
                    let message = "Invalid shared string".to_string();
                    CalcResult::new_error(Error::ERROR, cell_reference, message)
                }
            }
            CellFormula {
                v: FormulaValue::Unevaluated,
                ..
            }
            | ArrayFormula {
                v: FormulaValue::Unevaluated,
                ..
            } => CalcResult::Error {
                error: Error::ERROR,
                origin: cell_reference,
                message: "Unevaluated formula".to_string(),
            },
            CellFormula {
                v: FormulaValue::Boolean(v),
                ..
            }
            | ArrayFormula {
                v: FormulaValue::Boolean(v),
                ..
            } => CalcResult::Boolean(*v),
            CellFormula {
                v: FormulaValue::Number(v),
                ..
            }
            | ArrayFormula {
                v: FormulaValue::Number(v),
                ..
            } => CalcResult::Number(*v),
            CellFormula {
                v: FormulaValue::Text(v),
                ..
            }
            | ArrayFormula {
                v: FormulaValue::Text(v),
                ..
            } => CalcResult::String(v.clone()),
            CellFormula {
                v: FormulaValue::Error { ei, o, m },
                ..
            }
            | ArrayFormula {
                v: FormulaValue::Error { ei, o, m },
                ..
            } => {
                if let Some(cell_reference) = self.parse_reference(o) {
                    CalcResult::new_error(ei.clone(), cell_reference, m.clone())
                } else {
                    CalcResult::Error {
                        error: ei.clone(),
                        origin: cell_reference,
                        message: ei.to_localized_error_string(self.language),
                    }
                }
            }
            SpillCell {
                v: SpillValue::Number(v),
                ..
            } => CalcResult::Number(*v),
            SpillCell {
                v: SpillValue::Boolean(v),
                ..
            } => CalcResult::Boolean(*v),
            SpillCell {
                v: SpillValue::Text(v),
                ..
            } => CalcResult::String(v.clone()),
            SpillCell {
                v: SpillValue::Error(ei),
                ..
            } => {
                let message = ei.to_localized_error_string(self.language);
                CalcResult::new_error(ei.clone(), cell_reference, message)
            }
        }
    }

    /// Returns `true` if the cell is completely empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// assert_eq!(model.is_empty_cell(0, 1, 1)?, true);
    /// model.set_user_input(0, 1, 1, "Attention is all you need".to_string());
    /// assert_eq!(model.is_empty_cell(0, 1, 1)?, false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty_cell(&self, sheet: u32, row: i32, column: i32) -> Result<bool, String> {
        self.workbook.worksheet(sheet)?.is_empty_cell(row, column)
    }

    /// Evaluates all cells in a given range and returns the results in a 2D vector.
    pub(crate) fn evaluate_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        let mut result = Vec::new();
        for r in left.row..=right.row {
            let mut row_result = Vec::new();
            for c in left.column..=right.column {
                let cell_reference = CellReferenceIndex {
                    sheet: left.sheet,
                    row: r,
                    column: c,
                };
                let value = match self.evaluate_cell(cell_reference) {
                    CalcResult::Number(n) => ArrayNode::Number(n),
                    CalcResult::Boolean(b) => ArrayNode::Boolean(b),
                    CalcResult::String(s) => ArrayNode::String(s),
                    CalcResult::Error { error, .. } => ArrayNode::Error(error),
                    CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Number(0.0),
                    CalcResult::Range { .. } | CalcResult::Array(_) => {
                        // This should never happen, but we need to handle it anyway
                        debug_assert!(false, "Unexpected array result in non-array formula");
                        ArrayNode::Error(Error::NIMPL)
                    }
                };
                row_result.push(value);
            }
            result.push(row_result);
        }
        result
    }

    #[inline(always)]
    fn fetch_cell(&self, cell_reference: CellReferenceIndex) -> Option<&Cell> {
        self.workbook.worksheets[cell_reference.sheet as usize]
            .sheet_data
            .get(&cell_reference.row)?
            .get(&cell_reference.column)
    }

    // Evaluates a cell and returns the value in the cell
    // FIXME: CalcResult cannot be Array or Range, should we have a different type?
    pub(crate) fn evaluate_cell(&mut self, cell_reference: CellReferenceIndex) -> CalcResult {
        let original_cell = match self.fetch_cell(cell_reference) {
            Some(c) => c.clone(),
            None => return CalcResult::EmptyCell,
        };

        if let Cell::SpillCell { a, .. } = original_cell {
            // If it is part of an array or dynamic formula we need to evaluate the anchor cell
            // strictly speaking we don't need to evaluate the anchor cell of a dynamic array formula
            // but it is most likely a good guess anyway
            let anchor_cell_reference = CellReferenceIndex {
                sheet: cell_reference.sheet,
                column: a.1,
                row: a.0,
            };
            // evaluate the anchor and discard the result
            let _ = self.evaluate_cell(anchor_cell_reference);
            // refetch the cell after evaluating the spill reference
            let cell = match self.fetch_cell(cell_reference) {
                Some(c) => c,
                None => return CalcResult::EmptyCell,
            };
            // and return its value
            return self.get_cell_value(cell, cell_reference);
        };

        match original_cell.get_formula() {
            Some(f) => {
                let key = (
                    cell_reference.sheet,
                    cell_reference.row,
                    cell_reference.column,
                );
                if let Some(state) = self.cells.get(&key) {
                    match state {
                        CellState::Evaluating => {
                            return CalcResult::new_error(
                                Error::CIRC,
                                cell_reference,
                                "Circular reference detected".to_string(),
                            );
                        }
                        CellState::Evaluated => {
                            return self.get_cell_value(&original_cell, cell_reference);
                        }
                    }
                }
                // Clear the pre-existing spill area of a dynamic formula before re-evaluating.
                // This must happen after the CellState check so that a recursive call from a
                // spill cell does not wipe out spill cells that were just written.
                if let Cell::ArrayFormula {
                    r,
                    kind: ArrayKind::Dynamic,
                    ..
                } = &original_cell
                {
                    let (width, height) = *r;
                    let ws = match self.workbook.worksheet_mut(cell_reference.sheet) {
                        Ok(ws) => ws,
                        Err(_) => {
                            return CalcResult::new_error(
                                Error::ERROR,
                                cell_reference,
                                "Invalid sheet".to_string(),
                            )
                        }
                    };
                    for r in cell_reference.row..cell_reference.row + height {
                        for c in cell_reference.column..cell_reference.column + width {
                            if r == cell_reference.row && c == cell_reference.column {
                                continue;
                            }
                            let _ = ws.cell_clear_contents(r, c);
                        }
                    }
                }
                // mark cell as being evaluated
                self.cells.insert(key, CellState::Evaluating);
                let (node, _static_result) =
                    &self.parsed_formulas[cell_reference.sheet as usize][f as usize];
                let result = self.evaluate_node_in_context(&node.clone(), cell_reference);

                // At this point a range needs to be transformed into an array
                let result = if let CalcResult::Range { left, right } = result {
                    if left.sheet == right.sheet
                        && left.row == right.row
                        && left.column == right.column
                    {
                        // it is a single cell range, we can just return the value of the cell
                        self.evaluate_cell(left)
                    } else {
                        let array = self.evaluate_range(left, right);
                        CalcResult::Array(array)
                    }
                } else {
                    result
                };

                if let Err(e) = self.set_cells_with_result(cell_reference, &original_cell, &result)
                {
                    self.cells.insert(key, CellState::Evaluated);
                    // TODO: I _think_ this can never happen. Maybe we should  refactor things in a way that this is apparent
                    return CalcResult::new_error(Error::ERROR, cell_reference, e);
                };

                // mark cell as evaluated
                self.cells.insert(key, CellState::Evaluated);

                // return the result of the evaluation.
                match result {
                    CalcResult::Array(a) => {
                        // If it is an array, we return the value of the first cell
                        match a[0][0] {
                            ArrayNode::Number(n) => CalcResult::Number(n),
                            ArrayNode::Boolean(b) => CalcResult::Boolean(b),
                            ArrayNode::String(ref s) => CalcResult::String(s.clone()),
                            ArrayNode::Error(ref error) => {
                                let message = error.to_localized_error_string(self.language);
                                CalcResult::new_error(error.clone(), cell_reference, message)
                            }
                        }
                    }
                    _ => result,
                }
            }
            None => self.get_cell_value(&original_cell, cell_reference),
        }
    }

    pub(crate) fn get_sheet_index_by_name(&self, name: &str) -> Option<u32> {
        let worksheets = &self.workbook.worksheets;
        for (index, worksheet) in worksheets.iter().enumerate() {
            if worksheet.get_name().to_uppercase() == name.to_uppercase() {
                return Some(index as u32);
            }
        }
        None
    }

    /// Returns a model from an internal binary representation of a workbook
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::cell::CellValue;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "Stella!".to_string());
    /// let model2 = Model::from_bytes(&model.to_bytes(), "en")?;
    /// assert_eq!(
    ///     model2.get_cell_value_by_index(0, 1, 1),
    ///     Ok(CellValue::String("Stella!".to_string()))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::to_bytes]
    pub fn from_bytes(s: &[u8], language_id: &'a str) -> Result<Model<'a>, String> {
        let workbook: Workbook =
            bitcode::decode(s).map_err(|e| format!("Error parsing workbook: {e}"))?;
        Model::from_workbook(workbook, language_id)
    }

    /// Returns a model from a Workbook object
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::cell::CellValue;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "Stella!".to_string());
    /// let model2 = Model::from_workbook(model.workbook, "en")?;
    /// assert_eq!(
    ///     model2.get_cell_value_by_index(0, 1, 1),
    ///     Ok(CellValue::String("Stella!".to_string()))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_workbook(workbook: Workbook, language_id: &str) -> Result<Model<'_>, String> {
        let parsed_formulas = Vec::new();
        let worksheets = &workbook.worksheets;

        let worksheet_names = worksheets.iter().map(|s| s.get_name()).collect();

        let defined_names = workbook.get_defined_names_with_scope();
        // add all tables
        // let mut tables = Vec::new();
        // for worksheet in worksheets {
        //     let mut tables_in_sheet = HashMap::new();
        //     for table in &worksheet.tables {
        //         tables_in_sheet.insert(table.name.clone(), table.clone());
        //     }
        //     tables.push(tables_in_sheet);
        // }

        let cells = HashMap::new();
        let locale =
            get_locale(&workbook.settings.locale).map_err(|_| "Invalid locale".to_string())?;
        let tz: Tz = workbook
            .settings
            .tz
            .parse()
            .map_err(|_| format!("Invalid timezone: {}", workbook.settings.tz))?;

        let language = match get_language(language_id) {
            Ok(lang) => lang,
            Err(_) => return Err("Invalid language".to_string()),
        };
        let parser = Parser::new(
            worksheet_names,
            defined_names,
            workbook.tables.clone(),
            locale,
            language,
        );
        let mut shared_strings = HashMap::new();
        for (index, s) in workbook.shared_strings.iter().enumerate() {
            shared_strings.insert(s.to_string(), index);
        }

        let mut model = Model {
            workbook,
            parsed_formulas,
            shared_strings,
            parsed_defined_names: HashMap::new(),
            parser,
            cells,
            language,
            locale,
            tz,
            view_id: 0,
        };

        model.parse_formulas();
        model.parse_defined_names();

        Ok(model)
    }

    /// Parses a reference like "Sheet1!B4" into {0, 2, 4}
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::expressions::types::CellReferenceIndex;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "Stella!".to_string());
    /// let reference = model.parse_reference("Sheet1!D40");
    /// assert_eq!(reference, Some(CellReferenceIndex {sheet: 0, row: 40, column: 4}));
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_reference(&self, s: &str) -> Option<CellReferenceIndex> {
        let bytes = s.as_bytes();
        let mut sheet_name = "".to_string();
        let mut column = "".to_string();
        let mut row = "".to_string();
        let mut state = "sheet"; // "sheet", "col", "row"
        for &byte in bytes {
            match state {
                "sheet" => {
                    if byte == b'!' {
                        state = "col"
                    } else {
                        sheet_name.push(byte as char);
                    }
                }
                "col" => {
                    if byte.is_ascii_alphabetic() {
                        column.push(byte as char);
                    } else {
                        state = "row";
                        row.push(byte as char);
                    }
                }
                _ => {
                    row.push(byte as char);
                }
            }
        }
        let sheet = self.get_sheet_index_by_name(&sheet_name)?;
        let row = match row.parse::<i32>() {
            Ok(r) => r,
            Err(_) => return None,
        };
        if !(1..=constants::LAST_ROW).contains(&row) {
            return None;
        }

        let column = match utils::column_to_number(&column) {
            Ok(column) => {
                if is_valid_column_number(column) {
                    column
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        };

        Some(CellReferenceIndex { sheet, row, column })
    }

    /// Moves the formula `value` from `source` (in `area`) to `target`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::expressions::types::{Area, CellReferenceIndex};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let source = CellReferenceIndex { sheet: 0, row: 3, column: 1};
    /// let target = CellReferenceIndex { sheet: 0, row: 50, column: 1};
    /// let area = Area { sheet: 0, row: 1, column: 1, width: 5, height: 4};
    /// let result = model.move_cell_value_to_area("=B1", &source, &target, &area)?;
    /// assert_eq!(&result, "=B48");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::extend_to()]
    /// * [Model::extend_copied_value()]
    pub fn move_cell_value_to_area(
        &mut self,
        value: &str,
        source: &CellReferenceIndex,
        target: &CellReferenceIndex,
        area: &Area,
    ) -> Result<String, String> {
        let source_sheet_name = self
            .workbook
            .worksheet(source.sheet)
            .map_err(|e| format!("Could not find source worksheet: {e}"))?
            .get_name();
        if source.sheet != area.sheet {
            return Err("Source and area are in different sheets".to_string());
        }
        if source.row < area.row || source.row >= area.row + area.height {
            return Err("Source is outside the area".to_string());
        }
        if source.column < area.column || source.column >= area.column + area.width {
            return Err("Source is outside the area".to_string());
        }
        let target_sheet_name = self
            .workbook
            .worksheet(target.sheet)
            .map_err(|e| format!("Could not find target worksheet: {e}"))?
            .get_name();
        if let Some(formula) = self.formula_without_prefix(value) {
            let cell_reference = CellReferenceRC {
                sheet: source_sheet_name.to_owned(),
                row: source.row,
                column: source.column,
            };
            let formula_str = move_formula(
                &self.parser.parse(formula, &cell_reference),
                &MoveContext {
                    source_sheet_name: &source_sheet_name,
                    row: source.row,
                    column: source.column,
                    area,
                    target_sheet_name: &target_sheet_name,
                    row_delta: target.row - source.row,
                    column_delta: target.column - source.column,
                },
                self.locale,
                self.language,
            );
            Ok(format!("={formula_str}"))
        } else {
            Ok(value.to_string())
        }
    }

    /// 'Extends' the value from cell (`sheet`, `row`, `column`) to (`target_row`, `target_column`) in the same sheet
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "=B1*D4".to_string());
    /// let (target_row, target_column) = (30, 1);
    /// let result = model.extend_to(sheet, row, column, target_row, target_column)?;
    /// assert_eq!(&result, "=B30*D33");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::extend_copied_value()]
    /// * [Model::move_cell_value_to_area()]
    pub fn extend_to(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
        target_row: i32,
        target_column: i32,
    ) -> Result<String, String> {
        let cell = self.workbook.worksheet(sheet)?.cell(row, column);
        let result = match cell {
            Some(cell) => match cell.get_formula() {
                None => cell.get_localized_text(
                    &self.workbook.shared_strings,
                    self.locale,
                    self.language,
                ),
                Some(i) => {
                    let (formula, _static_result) =
                        &self.parsed_formulas[sheet as usize][i as usize];
                    let cell_ref = CellReferenceRC {
                        sheet: self.workbook.worksheets[sheet as usize].get_name(),
                        row: target_row,
                        column: target_column,
                    };
                    format!(
                        "={}",
                        to_localized_string(formula, &cell_ref, self.locale, self.language)
                    )
                }
            },
            None => "".to_string(),
        };
        Ok(result)
    }

    /// 'Extends' the formula `value` from `source` to `target`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::expressions::types::CellReferenceIndex;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let source = CellReferenceIndex {sheet: 0, row: 1, column: 1};
    /// let target = CellReferenceIndex {sheet: 0, row: 30, column: 1};
    /// let result = model.extend_copied_value("=B1*D4", &source, &target)?;
    /// assert_eq!(&result, "=B30*D33");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::extend_to()]
    /// * [Model::move_cell_value_to_area()]
    pub fn extend_copied_value(
        &mut self,
        value: &str,
        source: &CellReferenceIndex,
        target: &CellReferenceIndex,
    ) -> Result<String, String> {
        let source_sheet_name = match self.workbook.worksheets.get(source.sheet as usize) {
            Some(ws) => ws.get_name(),
            None => {
                return Err("Invalid worksheet index".to_owned());
            }
        };
        let target_sheet_name = match self.workbook.worksheets.get(target.sheet as usize) {
            Some(ws) => ws.get_name(),
            None => {
                return Err("Invalid worksheet index".to_owned());
            }
        };

        if let Some(formula_str) = self.formula_without_prefix(value) {
            let cell_reference = CellReferenceRC {
                sheet: source_sheet_name.to_string(),
                row: source.row,
                column: source.column,
            };
            let formula = &self.parser.parse(formula_str, &cell_reference);
            let cell_reference = CellReferenceRC {
                sheet: target_sheet_name,
                row: target.row,
                column: target.column,
            };
            return Ok(format!(
                "={}",
                to_localized_string(formula, &cell_reference, self.locale, self.language)
            ));
        }
        Ok(value.to_string())
    }

    /// Returns the formula in (`sheet`, `row`, `column`) if any
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "=SIN(B1*C3)+1".to_string());
    /// model.evaluate();
    /// let result = model.get_cell_formula(sheet, row, column)?;
    /// assert_eq!(result, Some("=SIN(B1*C3)+1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::get_localized_cell_content()]
    pub fn get_cell_formula(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<String>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        match worksheet.cell(row, column) {
            Some(cell) => match cell.get_formula() {
                Some(formula_index) => {
                    let (formula, _static_result) = &self
                        .parsed_formulas
                        .get(sheet as usize)
                        .ok_or("missing sheet")?
                        .get(formula_index as usize)
                        .ok_or("missing formula")?;
                    let cell_ref = CellReferenceRC {
                        sheet: worksheet.get_name(),
                        row,
                        column,
                    };
                    Ok(Some(format!(
                        "={}",
                        to_localized_string(formula, &cell_ref, self.locale, self.language)
                    )))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// Returns the text for the formula in (`sheet`, `row`, `column`) in English if any
    ///
    /// See also:
    /// * [Model::get_localized_cell_content()]
    pub(crate) fn get_english_cell_formula(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<String>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        match worksheet.cell(row, column) {
            Some(cell) => match cell.get_formula() {
                Some(formula_index) => {
                    let (formula, _static_result) = &self
                        .parsed_formulas
                        .get(sheet as usize)
                        .ok_or("missing sheet")?
                        .get(formula_index as usize)
                        .ok_or("missing formula")?;
                    let cell_ref = CellReferenceRC {
                        sheet: worksheet.get_name(),
                        row,
                        column,
                    };
                    let language_en = get_default_language();
                    Ok(Some(format!(
                        "={}",
                        to_localized_string(formula, &cell_ref, self.locale, language_en)
                    )))
                }
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    /// Updates the value of a cell with some text
    /// It does not change the style unless needs to add "quoting"
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "Hello!".to_string())?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "Hello!".to_string());
    ///
    /// model.update_cell_with_text(sheet, row, column, "Goodbye!")?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "Goodbye!".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::set_user_input()]
    /// * [Model::update_cell_with_number()]
    /// * [Model::update_cell_with_bool()]
    /// * [Model::update_cell_with_formula()]
    pub fn update_cell_with_text(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(), String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        let new_style_index;
        if common::value_needs_quoting(value, self.language) {
            new_style_index = self
                .workbook
                .styles
                .get_style_with_quote_prefix(style_index)?;
        } else if self.workbook.styles.style_is_quote_prefix(style_index) {
            new_style_index = self
                .workbook
                .styles
                .get_style_without_quote_prefix(style_index)?;
        } else {
            new_style_index = style_index;
        }

        self.set_cell_with_string(sheet, row, column, value, new_style_index)
    }

    /// Updates the value of a cell with a boolean value
    /// It does not change the style
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "TRUE".to_string())?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "TRUE".to_string());
    ///
    /// model.update_cell_with_bool(sheet, row, column, false)?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "FALSE".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::set_user_input()]
    /// * [Model::update_cell_with_number()]
    /// * [Model::update_cell_with_text()]
    /// * [Model::update_cell_with_formula()]
    pub fn update_cell_with_bool(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: bool,
    ) -> Result<(), String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        let new_style_index = if self.workbook.styles.style_is_quote_prefix(style_index) {
            self.workbook
                .styles
                .get_style_without_quote_prefix(style_index)?
        } else {
            style_index
        };
        self.set_cell_with_boolean(sheet, row, column, value, new_style_index)
    }

    /// Updates the value of a cell with a number
    /// It does not change the style
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "42".to_string())?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "42".to_string());
    ///
    /// model.update_cell_with_number(sheet, row, column, 23.0)?;
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "23".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::set_user_input()]
    /// * [Model::update_cell_with_text()]
    /// * [Model::update_cell_with_bool()]
    /// * [Model::update_cell_with_formula()]
    pub fn update_cell_with_number(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: f64,
    ) -> Result<(), String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        let new_style_index = if self.workbook.styles.style_is_quote_prefix(style_index) {
            self.workbook
                .styles
                .get_style_without_quote_prefix(style_index)?
        } else {
            style_index
        };
        self.set_cell_with_number(sheet, row, column, value, new_style_index)
    }

    /// Updates the formula of given cell
    /// It does not change the style unless needs to add "quoting"
    /// Expects the formula to start with "="
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "=A2*2".to_string())?;
    /// model.evaluate();
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "=A2*2".to_string());
    ///
    /// model.update_cell_with_formula(sheet, row, column, "=A3*2".to_string())?;
    /// model.evaluate();
    /// assert_eq!(model.get_localized_cell_content(sheet, row, column)?, "=A3*2".to_string());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::set_user_input()]
    /// * [Model::update_cell_with_number()]
    /// * [Model::update_cell_with_bool()]
    /// * [Model::update_cell_with_text()]
    pub fn update_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: String,
    ) -> Result<(), String> {
        let mut style_index = self.get_cell_style_index(sheet, row, column)?;
        if self.workbook.styles.style_is_quote_prefix(style_index) {
            style_index = self
                .workbook
                .styles
                .get_style_without_quote_prefix(style_index)?;
        }

        if let Some(new_formula) = self.formula_without_prefix(&formula) {
            self.set_cell_with_formula(sheet, row, column, new_formula, style_index)?;
            Ok(())
        } else {
            Err(format!("\"{formula}\" is not a valid formula"))
        }
    }

    // If we are writing in (sheet, row, column). If it is:
    // - A single cell => do nothing
    // - Part of an array formula => we bail
    // - Anchor of an array formula => we delete the formula and we clear the spill
    // - Part of a dynamic array formula => we delete the formula and we clear the spill
    // - Anchor of a dynamic array formula
    //     => we clear the spill and we set an unevaluated dynamic formula.
    fn prepare_cell_for_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<(), String> {
        match self.get_cell_structure(sheet, row, column)? {
            CellStructure::SingleCell => {
                // noop
            }
            CellStructure::ArrayFormula { range } => {
                // We cannot write in a cell that is part of an array formula
                let (width, height) = range;
                if width > 1 || height > 1 {
                    return Err(
                        "Cannot write in a cell that is part of an array formula".to_string()
                    );
                }
            }
            CellStructure::DynamicFormula { range } => {
                // clear the spill of the dynamic formula
                let (width, height) = range;
                let ws = self.workbook.worksheet_mut(sheet)?;
                for r in row..row + height {
                    for c in column..column + width {
                        // We ignore errors here
                        let _ = ws.cell_clear_contents(r, c);
                    }
                }
            }
            CellStructure::SpillArray { .. } => {
                return Err("Cannot write in a cell that is part of an array formula".to_string());
            }
            CellStructure::SpillDynamic { anchor, range } => {
                // It is part of a dynamic array formula, but it is not the anchor.
                // We can write in it but we need to clear the spill and reset the anchor
                // to an unevaluated dynamic formula so it will re-spill on next evaluate().
                let (anchor_row, anchor_column) = anchor;
                let (width, height) = range;
                let ws = self.workbook.worksheet_mut(sheet)?;
                // Extract formula index and style from the anchor before mutating
                let (formula_index, anchor_style) = {
                    let anchor_cell = ws
                        .cell(anchor_row, anchor_column)
                        .ok_or_else(|| "Dynamic formula anchor not found".to_string())?;
                    let fi = anchor_cell
                        .get_formula()
                        .ok_or_else(|| "Dynamic formula anchor has no formula".to_string())?;
                    let s = anchor_cell.get_style();
                    (fi, s)
                };
                ws.set_cell_with_dynamic_formula(
                    anchor_row,
                    anchor_column,
                    formula_index,
                    anchor_style,
                    1,
                    1,
                )?;
                for r in anchor_row..anchor_row + height {
                    for c in anchor_column..anchor_column + width {
                        if r == anchor_row && c == anchor_column {
                            continue;
                        }
                        // We ignore errors here
                        let _ = ws.cell_clear_contents(r, c);
                    }
                }
            }
        };
        Ok(())
    }

    /// Sets a cell parametrized by (`sheet`, `row`, `column`) with `value`.
    ///
    /// This mimics a user entering a value on a cell.
    ///
    /// If you enter a currency `$100` it will set as a number and update the style
    ///  Note that for currencies/percentage there is only one possible style
    ///  The value is always a string, so we need to try to cast it into numbers/booleans/errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::cell::CellValue;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "100$".to_string());
    /// model.set_user_input(0, 2, 1, "125$".to_string());
    /// model.set_user_input(0, 3, 1, "-10$".to_string());
    /// model.set_user_input(0, 1, 2, "=SUM(A:A)".to_string());
    /// model.evaluate();
    /// assert_eq!(model.get_cell_value_by_index(0, 1, 2), Ok(CellValue::Number(215.0)));
    /// assert_eq!(model.get_formatted_cell_value(0, 1, 2), Ok("215$".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See also:
    /// * [Model::update_cell_with_formula()]
    /// * [Model::update_cell_with_number()]
    /// * [Model::update_cell_with_bool()]
    /// * [Model::update_cell_with_text()]
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: String,
    ) -> Result<(), String> {
        // first we make sure we can write in the cell and clear the spills.
        self.prepare_cell_for_user_input(sheet, row, column)?;

        // If value starts with "'" then we force the style to be quote_prefix
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        if let Some(new_value) = value.strip_prefix('\'') {
            let new_style = self
                .workbook
                .styles
                .get_style_with_quote_prefix(style_index)?;
            self.set_cell_with_string(sheet, row, column, new_value, new_style)?;
        } else {
            let mut new_style_index = style_index;
            if self.workbook.styles.style_is_quote_prefix(style_index) {
                new_style_index = self
                    .workbook
                    .styles
                    .get_style_without_quote_prefix(style_index)?;
            }
            if let Some(formula) = self.formula_without_prefix(&value) {
                let formula_index =
                    self.set_cell_with_formula(sheet, row, column, formula, new_style_index)?;
                // Update the style if needed
                let cell = CellReferenceIndex { sheet, row, column };
                let (parsed_formula, _static_result) =
                    &self.parsed_formulas[sheet as usize][formula_index as usize];
                if let Some(units) = self.compute_node_units(parsed_formula, &cell) {
                    let new_style_index = self
                        .workbook
                        .styles
                        .get_style_with_format(new_style_index, &units.get_num_fmt())?;
                    let style = self.workbook.styles.get_style(new_style_index)?;
                    self.set_cell_style(sheet, row, column, &style)?;
                }
            } else {
                // The list of currencies is '$', '€' and the local currency
                let mut currencies = vec!["$", "€"];
                let currency = &self.locale.currency.symbol;
                if !currencies.iter().any(|e| e == currency) {
                    currencies.push(currency);
                }

                //  We try to parse as number
                if let Ok((v, number_format)) =
                    parse_formatted_number(&value, &currencies, self.locale)
                {
                    if let Some(num_fmt) = number_format {
                        // Should not apply the format in the following cases:
                        // - we assign a date to already date-formatted cell
                        let should_apply_format = !(is_likely_date_number_format(
                            &self.workbook.styles.get_style(new_style_index)?.num_fmt,
                        ) && is_likely_date_number_format(&num_fmt));
                        if should_apply_format {
                            new_style_index = self
                                .workbook
                                .styles
                                .get_style_with_format(new_style_index, &num_fmt)?;
                        }
                    }
                    let worksheet = self.workbook.worksheet_mut(sheet)?;
                    worksheet.set_cell_with_number(row, column, v, new_style_index)?;
                    return Ok(());
                }
                // We try to parse as boolean
                if let Ok(v) = value.to_lowercase().parse::<bool>() {
                    let worksheet = self.workbook.worksheet_mut(sheet)?;
                    worksheet.set_cell_with_boolean(row, column, v, new_style_index)?;
                    return Ok(());
                }
                // Check is it is error value
                let upper = value.to_uppercase();
                let worksheet = self.workbook.worksheet_mut(sheet)?;
                match get_error_by_name(&upper, self.language) {
                    Some(error) => {
                        worksheet.set_cell_with_error(row, column, error, new_style_index)?;
                    }
                    None => {
                        self.set_cell_with_string(sheet, row, column, &value, new_style_index)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Sets an array formula in an area (CSE formula)
    pub fn set_user_array_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
        value: &str,
    ) -> Result<(), String> {
        if matches!(
            self.get_cell_structure(sheet, row, column)?,
            CellStructure::SpillArray { .. }
        ) {
            return Err("Cannot write in a cell that is part of an array formula".to_string());
        }
        // If value starts with "'" then we force the style to be quote_prefix
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        if value.strip_prefix('\'').is_none() {
            let mut new_style_index = style_index;
            if self.workbook.styles.style_is_quote_prefix(style_index) {
                new_style_index = self
                    .workbook
                    .styles
                    .get_style_without_quote_prefix(style_index)?;
            }
            if let Some(formula) = value.strip_prefix('=') {
                // It is a formula, we mark it as an array formulas and fill the "spills" with placeholders
                let formula_index = self.set_cell_with_array_formula(
                    sheet,
                    row,
                    column,
                    formula,
                    new_style_index,
                    width,
                    height,
                )?;

                // Update the style if needed
                let cell = CellReferenceIndex { sheet, row, column };
                let (parsed_formula, _static_result) =
                    &self.parsed_formulas[sheet as usize][formula_index as usize];

                if let Some(units) = self.compute_node_units(parsed_formula, &cell) {
                    let new_style_index = self
                        .workbook
                        .styles
                        .get_style_with_format(new_style_index, &units.get_num_fmt())?;
                    let style = self.workbook.styles.get_style(new_style_index)?;
                    self.set_cell_style(sheet, row, column, &style)?;
                }
                // Update the "spill" area with placeholders
                for r in row..row + height {
                    for c in column..column + width {
                        if r == row && c == column {
                            continue;
                        }
                        let mut new_style_index_spill =
                            self.get_cell_style_index(sheet, row, column)?;
                        if self
                            .workbook
                            .styles
                            .style_is_quote_prefix(new_style_index_spill)
                        {
                            new_style_index_spill = self
                                .workbook
                                .styles
                                .get_style_without_quote_prefix(new_style_index_spill)?;
                        }

                        self.set_cell_with_string(sheet, r, c, "", new_style_index_spill)?;
                    }
                }
                return Ok(());
            }
        }
        // just use set user input on every cell
        for r in row..row + height {
            for c in column..column + width {
                self.set_user_input(sheet, r, c, value.to_string())?;
            }
        }

        Ok(())
    }

    pub(crate) fn get_cell_structure(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<CellStructure, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        worksheet.get_cell_structure(row, column)
    }

    fn set_cell_with_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
        style: i32,
    ) -> Result<i32, String> {
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        let cell_reference = CellReferenceRC {
            sheet: worksheet.get_name(),
            row,
            column,
        };
        let shared_formulas = &mut worksheet.shared_formulas;
        let mut parsed_formula = self.parser.parse(formula, &cell_reference);
        // If the formula fails to parse try adding a parenthesis
        // SUM(A1:A3  => SUM(A1:A3)
        if let Node::ParseErrorKind { .. } = parsed_formula {
            let new_parsed_formula = self.parser.parse(&format!("{formula})"), &cell_reference);
            match new_parsed_formula {
                Node::ParseErrorKind { .. } => {}
                _ => parsed_formula = new_parsed_formula,
            }
        }
        let static_result = run_static_analysis_on_node(&parsed_formula);
        let is_dynamic = !matches!(static_result, StaticResult::Scalar);

        let s = to_rc_format(&parsed_formula);
        let mut formula_index: i32 = -1;
        if let Some(index) = shared_formulas.iter().position(|x| x == &s) {
            formula_index = index as i32;
        }
        if formula_index == -1 {
            shared_formulas.push(s);
            self.parsed_formulas[sheet as usize].push((parsed_formula, static_result));
            formula_index = (shared_formulas.len() as i32) - 1;
        }
        if is_dynamic {
            worksheet.set_cell_with_dynamic_formula(row, column, formula_index, style, 1, 1)?;
        } else {
            worksheet.set_cell_with_formula(row, column, formula_index, style)?;
        }
        Ok(formula_index)
    }

    // FIXME
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_cell_with_array_formula(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        formula: &str,
        style: i32,
        width: i32,
        height: i32,
    ) -> Result<i32, String> {
        let worksheet = self.workbook.worksheet_mut(sheet)?;
        let cell_reference = CellReferenceRC {
            sheet: worksheet.get_name(),
            row,
            column,
        };
        let shared_formulas = &mut worksheet.shared_formulas;
        let mut parsed_formula = self.parser.parse(formula, &cell_reference);
        // If the formula fails to parse try adding a parenthesis
        // SUM(A1:A3  => SUM(A1:A3)
        if let Node::ParseErrorKind { .. } = parsed_formula {
            let new_parsed_formula = self.parser.parse(&format!("{formula})"), &cell_reference);
            match new_parsed_formula {
                Node::ParseErrorKind { .. } => {}
                _ => parsed_formula = new_parsed_formula,
            }
        }
        let static_result = run_static_analysis_on_node(&parsed_formula);

        let s = to_rc_format(&parsed_formula);
        let mut formula_index: i32 = -1;
        if let Some(index) = shared_formulas.iter().position(|x| x == &s) {
            formula_index = index as i32;
        }
        if formula_index == -1 {
            shared_formulas.push(s);
            self.parsed_formulas[sheet as usize].push((parsed_formula, static_result));
            formula_index = (shared_formulas.len() as i32) - 1;
        }
        worksheet.set_cell_with_array_formula(row, column, formula_index, style, width, height)?;
        Ok(formula_index)
    }

    pub(crate) fn set_cell_with_string(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
        style: i32,
    ) -> Result<(), String> {
        match self.shared_strings.get(value) {
            Some(string_index) => {
                self.workbook.worksheet_mut(sheet)?.set_cell_with_string(
                    row,
                    column,
                    *string_index as i32,
                    style,
                )?;
            }
            None => {
                let string_index = self.workbook.shared_strings.len();
                self.workbook.shared_strings.push(value.to_string());
                self.shared_strings.insert(value.to_string(), string_index);
                self.workbook.worksheet_mut(sheet)?.set_cell_with_string(
                    row,
                    column,
                    string_index as i32,
                    style,
                )?;
            }
        }
        Ok(())
    }

    fn set_cell_with_boolean(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: bool,
        style: i32,
    ) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_with_boolean(row, column, value, style)
    }

    fn set_cell_with_number(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: f64,
        style: i32,
    ) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_cell_with_number(row, column, value, style)
    }

    // Helper function that returns a defined name given the name and scope
    fn get_parsed_defined_name(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<Option<ParsedDefinedName>, String> {
        let name_upper = name.to_uppercase();

        for (key, df) in &self.parsed_defined_names {
            if key.1.to_uppercase() == name_upper && key.0 == scope {
                return Ok(Some(df.clone()));
            }
        }
        Ok(None)
    }

    // Returns the formula for a defined name
    pub(crate) fn get_defined_name_formula(
        &self,
        name: &str,
        scope: Option<u32>,
    ) -> Result<String, String> {
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        for df in defined_names {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                return Ok(df.formula.clone());
            }
        }
        Err("Defined name not found".to_string())
    }

    /// Gets the Excel Value (Bool, Number, String) of a cell
    ///
    /// See also:
    /// * [Model::get_cell_value_by_index()]
    pub fn get_cell_value_by_ref(&self, cell_ref: &str) -> Result<CellValue, String> {
        let cell_reference = match self.parse_reference(cell_ref) {
            Some(c) => c,
            None => return Err(format!("Error parsing reference: '{cell_ref}'")),
        };
        let sheet_index = cell_reference.sheet;
        let column = cell_reference.column;
        let row = cell_reference.row;

        self.get_cell_value_by_index(sheet_index, row, column)
    }

    /// Returns the cell value for (`sheet`, `row`, `column`)
    ///
    /// See also:
    /// * [Model::get_formatted_cell_value()]
    pub fn get_cell_value_by_index(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
    ) -> Result<CellValue, String> {
        let cell = self
            .workbook
            .worksheet(sheet_index)?
            .cell(row, column)
            .cloned()
            .unwrap_or_default();
        let cell_value = cell.value(&self.workbook.shared_strings, self.language);
        Ok(cell_value)
    }

    /// Returns the formatted cell value for (`sheet`, `row`, `column`)
    ///
    /// See also:
    /// * [Model::get_cell_value_by_index()]
    /// * [Model::get_cell_value_by_ref]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "=1/3".to_string());
    /// model.evaluate();
    /// let result = model.get_formatted_cell_value(sheet, row, column)?;
    /// assert_eq!(result, "0.333333333".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_formatted_cell_value(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
    ) -> Result<String, String> {
        match self.workbook.worksheet(sheet_index)?.cell(row, column) {
            Some(cell) => {
                let format = self.get_style_for_cell(sheet_index, row, column)?.num_fmt;
                let formatted_value =
                    cell.formatted_value(&self.workbook.shared_strings, self.language, |value| {
                        format_number(value, &format, self.locale).text
                    });
                Ok(formatted_value)
            }
            None => Ok("".to_string()),
        }
    }

    /// Return the typeof a cell
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<CellType, String> {
        Ok(match self.workbook.worksheet(sheet)?.cell(row, column) {
            Some(c) => c.get_type(),
            None => CellType::Number,
        })
    }

    /// Returns a string with the cell content in the given language and locale.
    /// If there is a formula returns the formula
    /// If the cell is empty returns the empty string
    /// Returns an error if there is no worksheet
    /// If the cell has quote prefix style it adds a ' at the beginning of the value
    /// If the cell is date formatted it tries to format it as date
    pub fn get_localized_cell_content(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<String, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let cell = match worksheet.cell(row, column) {
            Some(c) => c,
            None => return Ok("".to_string()),
        };
        match cell.get_formula() {
            Some(formula_index) => {
                let formula = &self.parsed_formulas[sheet as usize][formula_index as usize].0;
                let cell_ref = CellReferenceRC {
                    sheet: worksheet.get_name(),
                    row,
                    column,
                };
                Ok(format!(
                    "={}",
                    to_localized_string(formula, &cell_ref, self.locale, self.language)
                ))
            }
            None => {
                let style_index = cell.get_style();
                let style = self.workbook.styles.get_style(style_index)?;
                if style.quote_prefix {
                    Ok(format!(
                        "'{}",
                        cell.get_localized_text(
                            &self.workbook.shared_strings,
                            self.locale,
                            self.language,
                        )
                    ))
                } else {
                    // If it is a date formatted cell we try to format it as date, if it fails we return the raw value
                    if is_likely_date_number_format(&style.num_fmt) {
                        let value = cell.value(&self.workbook.shared_strings, self.language);
                        if let CellValue::Number(n) = value {
                            let formatted = format_number(n, &style.num_fmt, self.locale);
                            if formatted.error.is_none() {
                                return Ok(formatted.text);
                            }
                        }
                    }
                    Ok(cell.get_localized_text(
                        &self.workbook.shared_strings,
                        self.locale,
                        self.language,
                    ))
                }
            }
        }
    }

    /// Returns a list of all cells
    pub fn get_all_cells(&self) -> Vec<CellIndex> {
        let mut cells = Vec::new();
        for (index, sheet) in self.workbook.worksheets.iter().enumerate() {
            let mut sorted_rows: Vec<_> = sheet.sheet_data.keys().collect();
            sorted_rows.sort_unstable();
            for row in sorted_rows {
                let row_data = &sheet.sheet_data[row];
                let mut sorted_columns: Vec<_> = row_data.keys().collect();
                sorted_columns.sort_unstable();
                for column in sorted_columns {
                    cells.push(CellIndex {
                        index: index as u32,
                        row: *row,
                        column: *column,
                    });
                }
            }
        }
        cells
    }

    /// Evaluates the model with a top-down recursive algorithm
    pub fn evaluate(&mut self) {
        // clear all computation artifacts
        self.cells.clear();

        let cells = self.get_all_cells();

        for cell in cells {
            self.evaluate_cell(CellReferenceIndex {
                sheet: cell.index,
                row: cell.row,
                column: cell.column,
            });
        }
    }

    /// Removes the content of every cell in the range but leaves the style.
    ///
    /// See also:
    /// * [Model::range_clear_all()]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::expressions::types::Area;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "100$".to_string());
    /// let area = Area {
    ///     sheet,
    ///     row,
    ///     column,
    ///     width: 1,
    ///     height: 1,
    /// };
    /// model.range_clear_contents(&area)?;
    /// model.set_user_input(sheet, row, column, "10".to_string());
    /// let result = model.get_formatted_cell_value(sheet, row, column)?;
    /// assert_eq!(result, "10$".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn range_clear_contents(&mut self, range: &Area) -> Result<(), String> {
        if !self.can_clear_range(range)? {
            return Err("Cannot clear the range because it contains array formulas".to_string());
        }
        let sheet = range.sheet;
        let ws = self.workbook.worksheet_mut(sheet)?;
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                let structure = ws.get_cell_structure(row, column)?;
                match structure {
                    CellStructure::DynamicFormula { range }
                    | CellStructure::ArrayFormula { range, .. } => {
                        let (width, height) = range;
                        for r in row..row + height {
                            for c in column..column + width {
                                let _ = ws.cell_clear_contents(r, c);
                            }
                        }
                    }
                    _ => {
                        let _ = ws.cell_clear_contents(row, column);
                    }
                }
            }
        }
        Ok(())
    }

    // Returns true if for every array formula in the range, the whole spill is included in the range,
    // false otherwise.
    pub(crate) fn can_clear_range(&self, range: &Area) -> Result<bool, String> {
        let sheet = range.sheet;
        for row in range.row..range.row + range.height {
            for column in range.column..range.column + range.width {
                match self.get_cell_structure(sheet, row, column)? {
                    CellStructure::ArrayFormula { range: r } => {
                        let (width, height) = r;
                        if column + width > range.column + range.width
                            || row + height > range.row + range.height
                        {
                            return Ok(false);
                        }
                    }
                    CellStructure::SpillArray {
                        anchor: a,
                        range: r,
                    } => {
                        let (anchor_row, anchor_column) = a;
                        let (width, height) = r;
                        if anchor_column < range.column
                            || anchor_row < range.row
                            || anchor_column + width > range.column + range.width
                            || anchor_row + height > range.row + range.height
                        {
                            return Ok(false);
                        }
                    }
                    _ => {
                        // noop
                    }
                }
            }
        }
        Ok(true)
    }

    /// Deletes a range by removing it from worksheet data. All content and style is removed.
    /// It fails if it deletes part of an array formula.
    /// Deletes the whole spill if it is part of a dynamic array formula.
    ///
    /// See also:
    /// * [Model::range_clear_contents()]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::expressions::types::Area;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// let (sheet, row, column) = (0, 1, 1);
    /// model.set_user_input(sheet, row, column, "100$".to_string());
    /// let area = Area {
    ///     sheet,
    ///     row,
    ///     column,
    ///     width: 1,
    ///     height: 1,
    /// };
    /// model.range_clear_all(&area)?;
    /// model.set_user_input(sheet, row, column, "10".to_string());
    /// let result = model.get_formatted_cell_value(sheet, row, column)?;
    /// assert_eq!(result, "10".to_string());
    /// # Ok(())
    /// # }
    pub fn range_clear_all(&mut self, area: &Area) -> Result<(), String> {
        if !self.can_clear_range(area)? {
            return Err("Cannot clear the range because it contains array formulas".to_string());
        }
        let worksheet = self.workbook.worksheet_mut(area.sheet)?;

        let sheet_data = &mut worksheet.sheet_data;
        let mut cells_to_clear = Vec::new();
        for row in area.row..area.row + area.height {
            if let Some(row_data) = sheet_data.get_mut(&row) {
                for column in area.column..area.column + area.width {
                    // If it is part of a dynamic array we need to clear the spill
                    if let Some(Cell::ArrayFormula {
                        r,
                        kind: ArrayKind::Dynamic,
                        ..
                    }) = row_data.get(&column)
                    {
                        // clear the spill of the dynamic formula
                        let (width, height) = r;
                        for r in row..row + height {
                            for c in column..column + width {
                                cells_to_clear.push((r, c));
                            }
                        }
                    }
                    row_data.remove(&column);
                }
            }
        }
        for (row, column) in cells_to_clear {
            // we ignore errors here because the cell might have already been cleared as part of an array formula
            let _ = worksheet.cell_clear_contents(row, column);
        }
        Ok(())
    }

    // Finds all the dynamic array formulas that spills:
    // * Delete the spilled cells
    // * Update the formula cell to be DynamicFormula with r = (1,1)
    pub(crate) fn reset_dynamic_array_spills(&mut self, sheet: u32) -> Result<(), String> {
        // Collect anchor info first — can't mutate sheet_data while iterating over it.
        let anchors: Vec<(i32, i32, i32, i32, i32, i32)> = {
            let ws = self.workbook.worksheet(sheet)?;
            let mut result = Vec::new();
            for (row, row_data) in &ws.sheet_data {
                for (column, cell) in row_data {
                    if let Cell::ArrayFormula {
                        r,
                        f,
                        s,
                        kind: ArrayKind::Dynamic,
                        ..
                    } = cell
                    {
                        let (width, height) = *r;
                        result.push((*row, *column, *f, *s, width, height));
                    }
                }
            }
            result
        };

        for (row, column, f, s, width, height) in anchors {
            let ws = self.workbook.worksheet_mut(sheet)?;
            // Reset the anchor cell to DynamicFormula with r = (1, 1)
            if let Some(row_data) = ws.sheet_data.get_mut(&row) {
                row_data.insert(
                    column,
                    Cell::ArrayFormula {
                        f,
                        s,
                        r: (1, 1),
                        kind: ArrayKind::Dynamic,
                        v: FormulaValue::Unevaluated,
                    },
                );
            }
            // Delete all spill cells
            for r in row..row + height {
                for c in column..column + width {
                    if r == row && c == column {
                        continue;
                    }
                    if let Some(row_data) = ws.sheet_data.get_mut(&r) {
                        row_data.remove(&c);
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns the style index for cell (`sheet`, `row`, `column`)
    pub fn get_cell_style_index(&self, sheet: u32, row: i32, column: i32) -> Result<i32, String> {
        // First check the cell, then row, the column
        let cell = self.workbook.worksheet(sheet)?.cell(row, column);

        match cell {
            Some(cell) => Ok(cell.get_style()),
            None => {
                let rows = &self.workbook.worksheet(sheet)?.rows;
                for r in rows {
                    if r.r == row {
                        if r.custom_format {
                            return Ok(r.s);
                        }
                        break;
                    }
                }
                let cols = &self.workbook.worksheet(sheet)?.cols;
                for c in cols.iter() {
                    let min = c.min;
                    let max = c.max;
                    if column >= min && column <= max {
                        return Ok(c.style.unwrap_or(0));
                    }
                }
                Ok(0)
            }
        }
    }

    /// Returns the style for cell (`sheet`, `row`, `column`)
    /// If the cell does not have a style defined we check the row, otherwise the column and finally a default
    pub fn get_style_for_cell(&self, sheet: u32, row: i32, column: i32) -> Result<Style, String> {
        let style_index = self.get_cell_style_index(sheet, row, column)?;
        let style = self.workbook.styles.get_style(style_index)?;
        Ok(style)
    }

    /// Returns the style defined in a cell if any.
    pub fn get_cell_style_or_none(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<Style>, String> {
        let style = self
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .map(|c| self.workbook.styles.get_style(c.get_style()))
            .transpose();
        style
    }

    /// Returns an internal binary representation of the workbook
    ///
    /// See also:
    /// * [Model::from_bytes]
    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(&self.workbook)
    }

    /// Returns data about the worksheets
    pub fn get_worksheets_properties(&self) -> Vec<SheetProperties> {
        self.workbook
            .worksheets
            .iter()
            .map(|worksheet| SheetProperties {
                name: worksheet.get_name(),
                state: worksheet.state.to_string(),
                color: worksheet.color.clone(),
                sheet_id: worksheet.sheet_id,
            })
            .collect()
    }

    /// Returns markup representation of the given `sheet`.
    pub fn get_sheet_markup(&self, sheet: u32) -> Result<String, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let dimension = worksheet.dimension();

        let mut rows = Vec::new();

        for row in 1..(dimension.max_row + 1) {
            let mut row_markup: Vec<String> = Vec::new();

            for column in 1..(dimension.max_column + 1) {
                let mut cell_markup = match self.get_cell_formula(sheet, row, column)? {
                    Some(formula) => formula,
                    None => self.get_formatted_cell_value(sheet, row, column)?,
                };
                let style = self.get_style_for_cell(sheet, row, column)?;
                if style.font.b {
                    cell_markup = format!("**{cell_markup}**")
                }
                row_markup.push(cell_markup);
            }

            rows.push(row_markup.join("|"));
        }

        Ok(rows.join("\n"))
    }

    /// Returns the number of frozen rows in `sheet`
    pub fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32, String> {
        if let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) {
            Ok(worksheet.frozen_rows)
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Return the number of frozen columns in `sheet`
    pub fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32, String> {
        if let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) {
            Ok(worksheet.frozen_columns)
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Sets the number of frozen rows to `frozen_rows` in the workbook.
    /// Fails if `frozen`_rows` is either too small (<0) or too large (>LAST_ROW)`
    pub fn set_frozen_rows(&mut self, sheet: u32, frozen_rows: i32) -> Result<(), String> {
        if let Some(worksheet) = self.workbook.worksheets.get_mut(sheet as usize) {
            if frozen_rows < 0 {
                return Err("Frozen rows cannot be negative".to_string());
            }
            if frozen_rows >= LAST_ROW {
                return Err("Too many rows".to_string());
            }
            worksheet.frozen_rows = frozen_rows;
            Ok(())
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Sets the number of frozen columns to `frozen_column` in the workbook.
    /// Fails if `frozen`_columns` is either too small (<0) or too large (>LAST_COLUMN)`
    pub fn set_frozen_columns(&mut self, sheet: u32, frozen_columns: i32) -> Result<(), String> {
        if let Some(worksheet) = self.workbook.worksheets.get_mut(sheet as usize) {
            if frozen_columns < 0 {
                return Err("Frozen columns cannot be negative".to_string());
            }
            if frozen_columns >= LAST_COLUMN {
                return Err("Too many columns".to_string());
            }
            worksheet.frozen_columns = frozen_columns;
            Ok(())
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Returns the width of a column
    #[inline]
    pub fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64, String> {
        self.workbook.worksheet(sheet)?.get_column_width(column)
    }

    /// Sets the width of a column
    #[inline]
    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_column_width(column, width)
    }

    /// Sets whether a column is hidden
    #[inline]
    pub fn set_column_hidden(
        &mut self,
        sheet: u32,
        column: i32,
        hidden: bool,
    ) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_column_hidden(column, hidden)
    }

    /// Sets whether a row is hidden
    #[inline]
    pub fn set_row_hidden(&mut self, sheet: u32, row: i32, hidden: bool) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_row_hidden(row, hidden)
    }

    /// Returns whether a column is hidden
    #[inline]
    pub fn is_column_hidden(&self, sheet: u32, column: i32) -> Result<bool, String> {
        self.workbook.worksheet(sheet)?.is_column_hidden(column)
    }

    /// Returns whether a row is hidden
    #[inline]
    pub fn is_row_hidden(&self, sheet: u32, row: i32) -> Result<bool, String> {
        self.workbook.worksheet(sheet)?.is_row_hidden(row)
    }

    /// Returns the height of a row
    #[inline]
    pub fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64, String> {
        self.workbook.worksheet(sheet)?.row_height(row)
    }

    /// Sets the height of a row
    #[inline]
    pub fn set_row_height(&mut self, sheet: u32, column: i32, height: f64) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .set_row_height(column, height)
    }

    /// Adds a new defined name
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        let sheet_id = self.is_valid_defined_name(name, scope, formula)?;
        self.workbook.defined_names.push(DefinedName {
            name: name.to_string(),
            formula: formula.to_string(),
            sheet_id,
        });
        self.reset_parsed_structures();

        Ok(())
    }

    /// Validates if a defined name can be created
    pub fn is_valid_defined_name(
        &self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<Option<u32>, String> {
        if !is_valid_identifier(name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => match self.workbook.worksheet(index) {
                Ok(ws) => Some(ws.sheet_id),
                Err(_) => return Err("Scope: Invalid sheet index".to_string()),
            },
            None => None,
        };
        // if the defined name already exist return error
        for df in defined_names {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                return Err("Name: Defined name already exists".to_string());
            }
        }

        // Make sure the formula is valid
        match common::ParsedReference::parse_reference_formula(None, formula, self.locale, |name| {
            self.get_sheet_index_by_name(name)
        }) {
            Ok(_) => {}
            Err(_) => {
                return Err("Formula: Invalid defined name formula".to_string());
            }
        };

        Ok(sheet_id)
    }

    /// Delete defined name of name and scope
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        if let Some(i) = index {
            self.workbook.defined_names.remove(i);
            self.reset_parsed_structures();
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }

    /// Update defined name
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        };
        let name_upper = name.to_uppercase();
        let new_name_upper = new_name.to_uppercase();

        if name_upper != new_name_upper || scope != new_scope {
            for key in self.parsed_defined_names.keys() {
                if key.1.to_uppercase() == new_name_upper && key.0 == new_scope {
                    return Err("Name: Defined name already exists".to_string());
                }
            }
        }
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let new_sheet_id = match new_scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        if let Some(i) = index {
            if let Some(df) = self.workbook.defined_names.get_mut(i) {
                if new_name != df.name {
                    // We need to rename the name in every formula:

                    // Parse all formulas with the old name
                    // All internal formulas are R1C1
                    self.parser.set_lexer_mode(LexerMode::R1C1);
                    let worksheets = &mut self.workbook.worksheets;
                    for worksheet in worksheets {
                        let cell_reference = CellReferenceRC {
                            sheet: worksheet.get_name(),
                            row: 1,
                            column: 1,
                        };
                        let mut formulas = Vec::new();
                        for formula in &worksheet.shared_formulas {
                            let mut t = self.parser.parse(formula, &cell_reference);
                            rename_defined_name_in_node(&mut t, name, scope, new_name);
                            formulas.push(to_rc_format(&t));
                        }
                        worksheet.shared_formulas = formulas;
                    }
                    // Se the mode back to A1
                    self.parser.set_lexer_mode(LexerMode::A1);
                }
                df.name = new_name.to_string();
                df.sheet_id = new_sheet_id;
                df.formula = new_formula.to_string();
                self.reset_parsed_structures();
            }
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }
    /// Returns the style object of a column, if any
    pub fn get_column_style(&self, sheet: u32, column: i32) -> Result<Option<Style>, String> {
        if let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) {
            let cols = &worksheet.cols;
            for col in cols {
                if column >= col.min && column <= col.max {
                    if let Some(style_index) = col.style {
                        let style = self.workbook.styles.get_style(style_index)?;
                        return Ok(Some(style));
                    }
                    return Ok(None);
                }
            }
            Ok(None)
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Returns the style object of a row, if any
    pub fn get_row_style(&self, sheet: u32, row: i32) -> Result<Option<Style>, String> {
        if let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) {
            let rows = &worksheet.rows;
            for r in rows {
                if row == r.r {
                    let style = self.workbook.styles.get_style(r.s)?;
                    return Ok(Some(style));
                }
            }
            Ok(None)
        } else {
            Err("Invalid sheet".to_string())
        }
    }

    /// Sets a column with style
    pub fn set_column_style(
        &mut self,
        sheet: u32,
        column: i32,
        style: &Style,
    ) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_or_create(style);
        self.workbook
            .worksheet_mut(sheet)?
            .set_column_style(column, style_index)
    }

    /// Sets a row with style
    pub fn set_row_style(&mut self, sheet: u32, row: i32, style: &Style) -> Result<(), String> {
        let style_index = self.workbook.styles.get_style_index_or_create(style);
        self.workbook
            .worksheet_mut(sheet)?
            .set_row_style(row, style_index)
    }

    /// Deletes the style of a column if the is any
    pub fn delete_column_style(&mut self, sheet: u32, column: i32) -> Result<(), String> {
        self.workbook
            .worksheet_mut(sheet)?
            .delete_column_style(column)
    }

    /// Deletes the style of a row if there is any
    pub fn delete_row_style(&mut self, sheet: u32, row: i32) -> Result<(), String> {
        self.workbook.worksheet_mut(sheet)?.delete_row_style(row)
    }

    /// Sets the locale of the model
    pub fn set_locale(&mut self, locale_id: &str) -> Result<(), String> {
        let locale = match get_locale(locale_id) {
            Ok(l) => l,
            Err(_) => return Err(format!("Invalid locale: {locale_id}")),
        };
        self.parser.set_locale(locale);
        self.locale = locale;
        self.workbook.settings.locale = locale_id.to_string();
        self.evaluate();
        Ok(())
    }

    /// Sets the timezone of the model
    pub fn set_timezone(&mut self, timezone: &str) -> Result<(), String> {
        let tz: Tz = match &timezone.parse() {
            Ok(tz) => *tz,
            Err(_) => return Err(format!("Invalid timezone: {}", &timezone)),
        };
        self.tz = tz;
        self.workbook.settings.tz = timezone.to_string();
        self.evaluate();
        Ok(())
    }

    /// Sets the language
    pub fn set_language(&mut self, language_id: &str) -> Result<(), String> {
        let language = match get_language(language_id) {
            Ok(l) => l,
            Err(_) => return Err(format!("Invalid language: {language_id}")),
        };
        self.parser.set_language(language);
        self.language = language;
        Ok(())
    }

    /// Gets the current language
    pub fn get_language(&self) -> String {
        self.language.code.clone()
    }

    /// Gets the timezone of the model
    pub fn get_timezone(&self) -> String {
        self.workbook.settings.tz.clone()
    }

    /// Gets the locale of the model
    pub fn get_locale(&self) -> String {
        self.workbook.settings.locale.clone()
    }

    /// Gets the formatting settings based on the locale
    pub fn get_fmt_settings(&self) -> FmtSettings {
        let day_example = 46006.0; // December 15, 2025
        let currency = self.locale.currency.iso.clone();
        let currency_symbol = &self.locale.currency.symbol;
        // "M/d/yy"
        let short_date = &self.locale.dates.date_formats.short;
        // "M/d/yyyy"
        let long_date = &self.locale.dates.date_formats.long;
        let short_date_example = format_number(day_example, short_date, self.locale).text;
        let long_date_example = format_number(day_example, long_date, self.locale).text;
        // Number format ("#,##0.###")
        // The CLDR formats are a bit different than Excel's
        // let number_fmt = self.locale.numbers.decimal_formats.standard.clone();
        // "#,##0.00 ¤" Currency format might have weird spaces
        let currency_format_template = &self.locale.numbers.currency_formats.standard;
        let currency_format = currency_format_template
            .replace("¤", &format!("\"{}\"", currency_symbol))
            .replace(" ", " ");

        let number_fmt = "#,##0.00".to_string();
        let number_example = format_number(1234.567, &number_fmt, self.locale).text;
        FmtSettings {
            currency,
            currency_format,
            short_date: short_date.clone(),
            long_date: long_date.clone(),
            short_date_example,
            long_date_example,
            number_fmt,
            number_example,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::CellReferenceIndex as CellReference;
    use crate::{test::util::new_empty_model, types::Cell};

    #[test]
    fn test_cell_reference_to_string() {
        let model = new_empty_model();
        let reference = CellReference {
            sheet: 0,
            row: 32,
            column: 16,
        };
        assert_eq!(
            model.cell_reference_to_string(&reference),
            Ok("Sheet1!P32".to_string())
        )
    }

    #[test]
    fn test_cell_reference_to_string_invalid_worksheet() {
        let model = new_empty_model();
        let reference = CellReference {
            sheet: 10,
            row: 1,
            column: 1,
        };
        assert_eq!(
            model.cell_reference_to_string(&reference),
            Err("Invalid sheet index".to_string())
        )
    }

    #[test]
    fn test_cell_reference_to_string_invalid_column() {
        let model = new_empty_model();
        let reference = CellReference {
            sheet: 0,
            row: 1,
            column: 20_000,
        };
        assert_eq!(
            model.cell_reference_to_string(&reference),
            Err("Invalid column".to_string())
        )
    }

    #[test]
    fn test_cell_reference_to_string_invalid_row() {
        let model = new_empty_model();
        let reference = CellReference {
            sheet: 0,
            row: 2_000_000,
            column: 1,
        };
        assert_eq!(
            model.cell_reference_to_string(&reference),
            Err("Invalid row".to_string())
        )
    }

    #[test]
    fn test_get_cell() {
        let mut model = new_empty_model();
        model._set("A1", "35");
        model._set("A2", "");
        let worksheet = model.workbook.worksheet(0).expect("Invalid sheet");

        assert_eq!(
            worksheet.cell(1, 1),
            Some(&Cell::NumberCell { v: 35.0, s: 0 })
        );

        assert_eq!(
            worksheet.cell(2, 1),
            Some(&Cell::SharedString { si: 0, s: 0 })
        );
        assert_eq!(worksheet.cell(3, 1), None)
    }

    #[test]
    fn test_get_cell_invalid_sheet() {
        let model = new_empty_model();
        assert_eq!(
            model.workbook.worksheet(5),
            Err("Invalid sheet index".to_string()),
        )
    }

    #[test]
    fn test_update_cell_with_sign_prefixed_formulas() {
        let mut model = new_empty_model();

        let update_result = model.update_cell_with_formula(0, 1, 1, "-A2*2".to_string());
        model.evaluate();
        assert_eq!(update_result, Ok(()));
        assert_eq!(model._get_formula("A1"), *"=-A2*2");
    }
}
