use crate::calc_result::CalcResult;
use crate::cf_types::CfCellResult;
use crate::collab::fractional_index::{FractionalIndex, FractionalKey};
use crate::expressions::parser::static_analysis::StaticResult;
use crate::expressions::parser::{NamedVariable, Node, Parser};
use crate::expressions::types::CellReferenceIndex;
use crate::language::Language;
use crate::locale::Locale;
use crate::model::{CellOrRange, CellState, ParsedDefinedName};
use crate::types::Workbook;
use crate::tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A description of a continuous range of cells, described using stable identifiers, which can be
/// used to keep track of cell position under various concurrent operations (ex. adding/removing
/// rows or columns).
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StableRange {
    pub row_hi: FractionalKey,
    pub col_hi: FractionalKey,
    pub row_lo: FractionalKey,
    pub col_lo: FractionalKey,
}

/// A cell position, described using stable identifiers, which can be used to keep track of cell
/// position under various concurrent operations (ex. adding/removing rows or columns).
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct StableCellAddress {
    pub row: FractionalKey,
    pub col: FractionalKey,
}

impl StableCellAddress {
    pub fn new(row: FractionalKey, col: FractionalKey) -> Self {
        StableCellAddress { row, col }
    }
}

pub struct ColabModel<'a> {
    rows: FractionalIndex,
    cols: FractionalIndex,
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
    /// A stack of variables used for LET function evaluation. The key is the variable id, and the value is the variable value.
    pub(crate) variable_stack: HashMap<usize, CalcResult>,
    /// Last variable id used. It is incremented every time a new variable is created (for example, when evaluating a LET function).
    pub(crate) last_variable_id: usize,
    /// Lambdas
    pub(crate) lambdas: HashMap<usize, (Vec<NamedVariable>, Node)>,
    /// Last lambda id used. It is incremented every time a new lambda is created.
    pub(crate) last_lambda_id: usize,
    /// The list of cells that might spill
    pub(crate) spill_cells: Vec<CellReferenceIndex>,
    /// A dictionary to keep track of which cells or ranges support a given cell.
    pub(crate) support: HashMap<CellReferenceIndex, Vec<CellOrRange>>,
    /// Evaluated CF results per cell, keyed by (sheet_index, row, column).
    /// Rebuilt from scratch on every call to evaluate_conditional_formatting().
    pub(crate) cf_cache: HashMap<(u32, i32, i32), Vec<CfCellResult>>,
}
