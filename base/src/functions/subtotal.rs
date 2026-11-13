use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{parse_range, ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    types::Cell,
};

/// Excel has a complicated way of filtering + hidden rows
/// As a first a approximation a table can either have filtered rows or hidden rows, but not both.
/// Internally hey both will be marked as hidden rows. Hidden rows
/// The behaviour is the same for SUBTOTAL(100s,) => ignore those
/// But changes for the SUBTOTAL(1-11, ) those ignore filtered but take hidden into account.
/// In Excel filters are non-dynamic. Once you apply filters in a table (say value in column 1 should > 20) they
/// stay that way, even if you change the value of the values in the table after the fact.
/// If you try to hide rows in a table with filtered rows they will behave as if filtered
/// // Also subtotals ignore subtotals
///
#[derive(Clone, Copy)]
enum SubTotalMode {
    Full,
    SkipHidden,
    /// AGGREGATE says in its second argument what it leaves out
    Aggregate {
        hidden: bool,
        errors: bool,
        nested: bool,
    },
}

impl SubTotalMode {
    fn skips_row(self, status: &CellTableStatus) -> bool {
        match self {
            SubTotalMode::Full => *status == CellTableStatus::Filtered,
            SubTotalMode::SkipHidden => *status != CellTableStatus::Normal,
            SubTotalMode::Aggregate { hidden, .. } => hidden && *status != CellTableStatus::Normal,
        }
    }

    fn skips_errors(self) -> bool {
        matches!(self, SubTotalMode::Aggregate { errors: true, .. })
    }

    // SUBTOTAL always leaves out the cells that hold another SUBTOTAL or an AGGREGATE
    fn skips_nested(self) -> bool {
        !matches!(self, SubTotalMode::Aggregate { nested: false, .. })
    }

    // AGGREGATE takes references. For Excel a number or a calculation written
    // as an argument is #VALUE!
    fn takes_only_references(self) -> bool {
        matches!(self, SubTotalMode::Aggregate { .. })
    }
}

// A reference, or an error that the argument has of its own.
fn is_reference(arg: &Node, value: &CalcResult) -> bool {
    match value {
        CalcResult::Range { .. } | CalcResult::Error { .. } => true,
        CalcResult::Array(_) | CalcResult::Lambda(_) => false,
        // the value of one cell
        _ => may_give_a_cell(arg),
    }
}

// INDEX, IF, CHOOSE, XLOOKUP and a defined name can stand for one cell. For Excel
// that cell is a reference. Here they give its value, so this looks at what they
// pick from: `IF(A1>0,B2)` can give a cell, `IF(A1>0,5)` cannot
fn may_give_a_cell(node: &Node) -> bool {
    match node {
        Node::ReferenceKind { .. } | Node::RangeKind { .. } | Node::DefinedNameKind { .. } => true,
        // OFFSET and INDIRECT
        Node::FunctionKind { kind, .. } if kind.returns_reference() => true,
        Node::FunctionKind {
            kind: Function::Index,
            args,
        } => args.first().is_some_and(may_give_a_cell),
        Node::FunctionKind {
            kind: Function::If | Function::Choose,
            args,
        } => args.iter().skip(1).any(may_give_a_cell),
        Node::FunctionKind {
            kind: Function::Xlookup,
            args,
        } => args.get(2).is_some_and(may_give_a_cell),
        _ => false,
    }
}

fn not_a_reference(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "AGGREGATE takes references here, not values".to_string(),
    )
}

/// True if the formula holds a SUBTOTAL or an AGGREGATE anywhere, as in
/// `=SUBTOTAL(9,A1:A3)+1` or `=IF(B1,AGGREGATE(9,6,A1:A3),0)`. Excel leaves
/// such a cell out of another SUBTOTAL, and out of an AGGREGATE whose option
/// says so. It does not look into the formula of a defined name.
fn holds_subtotal(node: &Node) -> bool {
    match node {
        Node::FunctionKind { kind, args } => {
            matches!(kind, Function::Subtotal | Function::Aggregate)
                || args.iter().any(holds_subtotal)
        }
        Node::NamedFunctionKind { args, .. } => args.iter().any(holds_subtotal),
        Node::OpRangeKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::CompareKind { left, right, .. } => holds_subtotal(left) || holds_subtotal(right),
        Node::UnaryKind { right, .. } => holds_subtotal(right),
        Node::ImplicitIntersection { child, .. } | Node::SpillRangeOperator { child } => {
            holds_subtotal(child)
        }
        Node::LambdaDefKind { body, .. } => holds_subtotal(body),
        Node::LambdaCallKind { lambda, args } => {
            holds_subtotal(lambda) || args.iter().any(holds_subtotal)
        }
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::DefinedNameKind { .. }
        | Node::TableNameKind(_)
        | Node::NamedVariableKind { .. }
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => false,
    }
}

// The numbers that AGGREGATE hands to another function, as an array of one row
fn array_of(values: Vec<f64>) -> Node {
    Node::ArrayKind(vec![values.into_iter().map(ArrayNode::Number).collect()])
}

#[derive(PartialEq, Debug)]
pub enum CellTableStatus {
    Normal,
    Hidden,
    Filtered,
}

impl<'a> Model<'a> {
    fn get_table_for_cell(&self, sheet_index: u32, row: i32, column: i32) -> bool {
        let worksheet = match self.workbook.worksheet(sheet_index) {
            Ok(ws) => ws,
            Err(_) => return false,
        };
        for table in self.workbook.tables.values() {
            if worksheet.name != table.sheet_name {
                continue;
            }
            // (column, row, column, row)
            if let Ok((column1, row1, column2, row2)) = parse_range(&table.reference) {
                if ((column >= column1) && (column <= column2)) && ((row >= row1) && (row <= row2))
                {
                    return table.has_filters;
                }
            }
        }
        false
    }

    fn cell_hidden_status(
        &self,
        sheet_index: u32,
        row: i32,
        column: i32,
    ) -> Result<CellTableStatus, String> {
        let worksheet = self.workbook.worksheet(sheet_index)?;
        let mut hidden = false;
        for row_style in &worksheet.rows {
            if row_style.r == row {
                hidden = row_style.hidden;
                break;
            }
        }
        if !hidden {
            return Ok(CellTableStatus::Normal);
        }
        // The row is hidden we need to know if the table has filters
        if self.get_table_for_cell(sheet_index, row, column) {
            Ok(CellTableStatus::Filtered)
        } else {
            Ok(CellTableStatus::Hidden)
        }
    }

    // FIXME(TD): This is too much
    fn cell_is_subtotal(&self, sheet_index: u32, row: i32, column: i32) -> bool {
        let worksheet = &self.workbook.worksheets[sheet_index as usize];
        let mut cell = worksheet.cell(row, column);
        // A cell that a formula spills into is judged by that formula
        if let Some(&Cell::SpillCell { a: anchor, .. }) = cell.as_deref() {
            cell = worksheet.cell(anchor.0, anchor.1);
        }

        match cell.and_then(|c| c.get_formula()) {
            Some(f) => holds_subtotal(&self.parsed_formulas[sheet_index as usize][f as usize].0),
            None => false,
        }
    }

    fn subtotal_get_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut result: Vec<f64> = Vec::new();
        for arg in args {
            match arg {
                Node::FunctionKind {
                    kind: Function::Subtotal,
                    args: _,
                } if !mode.takes_only_references() => {
                    // skip
                }
                _ => {
                    let value = self.evaluate_node_with_reference(arg, cell);
                    if mode.takes_only_references() && !is_reference(arg, &value) {
                        return Err(not_a_reference(cell));
                    }
                    match value {
                        CalcResult::String(_) | CalcResult::Boolean(_) => {
                            // Skip
                        }
                        CalcResult::Number(f) => result.push(f),
                        error @ CalcResult::Error { .. } => {
                            return Err(error);
                        }
                        CalcResult::Range { left, right } => {
                            if left.sheet != right.sheet {
                                return Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Ranges are in different sheets".to_string(),
                                ));
                            }
                            // We are not expecting subtotal to have open ranges
                            let row1 = left.row;
                            let row2 = right.row;
                            let column1 = left.column;
                            let column2 = right.column;

                            for row in row1..=row2 {
                                let cell_status = self
                                    .cell_hidden_status(left.sheet, row, column1)
                                    .map_err(|message| {
                                        CalcResult::new_error(Error::ERROR, cell, message)
                                    })?;
                                if mode.skips_row(&cell_status) {
                                    continue;
                                }
                                for column in column1..=column2 {
                                    if mode.skips_nested()
                                        && self.cell_is_subtotal(left.sheet, row, column)
                                    {
                                        continue;
                                    }
                                    match self.evaluate_cell(CellReferenceIndex {
                                        sheet: left.sheet,
                                        row,
                                        column,
                                    }) {
                                        CalcResult::Number(value) => {
                                            result.push(value);
                                        }
                                        CalcResult::Error { .. } if mode.skips_errors() => {}
                                        error @ CalcResult::Error { .. } => return Err(error),
                                        _ => {
                                            // We ignore booleans and strings
                                        }
                                    }
                                }
                            }
                        }
                        // an empty cell that INDEX or IF gives to AGGREGATE is not a number
                        CalcResult::EmptyCell if mode.takes_only_references() => {}
                        CalcResult::EmptyCell | CalcResult::EmptyArg => result.push(0.0),
                        CalcResult::Array(_) | CalcResult::Lambda(_) => {
                            return Err(CalcResult::Error {
                                error: Error::NIMPL,
                                origin: cell,
                                message: "Arrays not supported yet".to_string(),
                            })
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn fn_subtotal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        match value {
            1 => self.subtotal_average(&args[1..], cell, SubTotalMode::Full),
            2 => self.subtotal_count(&args[1..], cell, SubTotalMode::Full),
            3 => self.subtotal_counta(&args[1..], cell, SubTotalMode::Full),
            4 => self.subtotal_max(&args[1..], cell, SubTotalMode::Full),
            5 => self.subtotal_min(&args[1..], cell, SubTotalMode::Full),
            6 => self.subtotal_product(&args[1..], cell, SubTotalMode::Full),
            7 => self.subtotal_stdevs(&args[1..], cell, SubTotalMode::Full),
            8 => self.subtotal_stdevp(&args[1..], cell, SubTotalMode::Full),
            9 => self.subtotal_sum(&args[1..], cell, SubTotalMode::Full),
            10 => self.subtotal_vars(&args[1..], cell, SubTotalMode::Full),
            11 => self.subtotal_varp(&args[1..], cell, SubTotalMode::Full),
            101 => self.subtotal_average(&args[1..], cell, SubTotalMode::SkipHidden),
            102 => self.subtotal_count(&args[1..], cell, SubTotalMode::SkipHidden),
            103 => self.subtotal_counta(&args[1..], cell, SubTotalMode::SkipHidden),
            104 => self.subtotal_max(&args[1..], cell, SubTotalMode::SkipHidden),
            105 => self.subtotal_min(&args[1..], cell, SubTotalMode::SkipHidden),
            106 => self.subtotal_product(&args[1..], cell, SubTotalMode::SkipHidden),
            107 => self.subtotal_stdevs(&args[1..], cell, SubTotalMode::SkipHidden),
            108 => self.subtotal_stdevp(&args[1..], cell, SubTotalMode::SkipHidden),
            109 => self.subtotal_sum(&args[1..], cell, SubTotalMode::SkipHidden),
            110 => self.subtotal_vars(&args[1..], cell, SubTotalMode::SkipHidden),
            111 => self.subtotal_varp(&args[1..], cell, SubTotalMode::SkipHidden),
            _ => CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Invalid value for SUBTOTAL: {value}"),
            ),
        }
    }

    // The array of function 14 to 19 is a range or a calculation.
    // A calculation has no rows to hide and no nested totals, only errors.
    fn aggregate_array_values(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> Result<Vec<f64>, CalcResult> {
        let array = match self.evaluate_node_with_reference(arg, cell) {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                // The range goes on as a plain reference, so that what gave it is
                // not evaluated a second time
                let range = Node::RangeKind {
                    sheet_name: None,
                    sheet_index: left.sheet,
                    absolute_row1: true,
                    absolute_column1: true,
                    row1: left.row,
                    column1: left.column,
                    absolute_row2: true,
                    absolute_column2: true,
                    row2: right.row,
                    column2: right.column,
                };
                return self.subtotal_get_values(&[range], cell, mode);
            }
            CalcResult::Array(array) => array,
            // an argument that is an error gives that error, whatever the option
            error @ CalcResult::Error { .. } => return Err(error),
            // one value is an array of one
            value => vec![vec![calc_result_to_array_node(value)]],
        };
        let mut result = Vec::new();
        for value in array.into_iter().flatten() {
            match value {
                ArrayNode::Number(f) => result.push(f),
                ArrayNode::Error(_) if mode.skips_errors() => {}
                ArrayNode::Error(error) => {
                    return Err(CalcResult::new_error(
                        error,
                        cell,
                        "Error in array".to_string(),
                    ))
                }
                _ => {}
            }
        }
        Ok(result)
    }

    /// `=AGGREGATE(function_num, options, ref1, [ref2], ...)`
    ///
    /// `=AGGREGATE(function_num, options, array, k)` for function_num 14 to 19
    ///
    /// function_num 1 to 11 are the functions of SUBTOTAL, in the same order.
    /// 12 is MEDIAN and 13 is MODE.SNGL.
    /// 14 is LARGE, 15 SMALL, 16 PERCENTILE.INC, 17 QUARTILE.INC,
    /// 18 PERCENTILE.EXC and 19 QUARTILE.EXC.
    ///
    /// options 0 to 3 leave out nested SUBTOTAL and AGGREGATE, 4 to 7 do not.
    /// On top of that 1, 3, 5 and 7 leave out hidden rows,
    /// and 2, 3, 6 and 7 leave out errors.
    pub(crate) fn fn_aggregate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let function = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        let options = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        if !(0..=7).contains(&options) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Invalid options for AGGREGATE: {options}"),
            );
        }
        let mode = SubTotalMode::Aggregate {
            hidden: options & 1 != 0,
            errors: options & 2 != 0,
            nested: options & 4 == 0,
        };
        let args = &args[2..];
        match function {
            1 => self.subtotal_average(args, cell, mode),
            2 => self.subtotal_count(args, cell, mode),
            3 => self.subtotal_counta(args, cell, mode),
            4 => self.subtotal_max(args, cell, mode),
            5 => self.subtotal_min(args, cell, mode),
            6 => self.subtotal_product(args, cell, mode),
            7 => self.subtotal_stdevs(args, cell, mode),
            8 => self.subtotal_stdevp(args, cell, mode),
            9 => self.subtotal_sum(args, cell, mode),
            10 => self.subtotal_vars(args, cell, mode),
            11 => self.subtotal_varp(args, cell, mode),
            12 | 13 => {
                // MEDIAN and MODE.SNGL get the numbers that are left, as an array
                let values = match self.subtotal_get_values(args, cell, mode) {
                    Ok(s) => s,
                    Err(s) => return s,
                };
                let numbers = [array_of(values)];
                if function == 12 {
                    self.fn_median(&numbers, cell)
                } else {
                    self.fn_mode_sngl(&numbers, cell)
                }
            }
            14..=19 => {
                // LARGE and the others get the numbers that are left, and k
                if args.len() != 2 {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        format!("AGGREGATE function {function} takes an array and k"),
                    );
                }
                let values = match self.aggregate_array_values(&args[0], cell, mode) {
                    Ok(s) => s,
                    Err(s) => return s,
                };
                let args = [array_of(values), args[1].clone()];
                match function {
                    14 => self.fn_large(&args, cell),
                    15 => self.fn_small(&args, cell),
                    16 => self.fn_percentile_inc(&args, cell),
                    17 => self.fn_quartile_inc(&args, cell),
                    18 => self.fn_percentile_exc(&args, cell),
                    _ => self.fn_quartile_exc(&args, cell),
                }
            }
            _ => CalcResult::new_error(
                Error::VALUE,
                cell,
                format!("Invalid function for AGGREGATE: {function}"),
            ),
        }
    }

    fn subtotal_vars(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64 - 1.0)
        }

        CalcResult::Number(result)
    }

    fn subtotal_varp(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64)
        }
        CalcResult::Number(result)
    }

    fn subtotal_stdevs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64 - 1.0)
        }

        CalcResult::Number(result.sqrt())
    }

    fn subtotal_stdevp(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in &values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        // average
        let average = result / (l as f64);
        let mut result = 0.0;
        for value in &values {
            result += (value - average).powi(2) / (l as f64)
        }
        CalcResult::Number(result.sqrt())
    }

    fn subtotal_counta(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let mut counta = 0;
        for arg in args {
            match arg {
                Node::FunctionKind {
                    kind: Function::Subtotal,
                    args: _,
                } if !mode.takes_only_references() => {
                    // skip
                }
                _ => {
                    let value = self.evaluate_node_with_reference(arg, cell);
                    if mode.takes_only_references() && !is_reference(arg, &value) {
                        return not_a_reference(cell);
                    }
                    match value {
                        CalcResult::EmptyCell | CalcResult::EmptyArg => {
                            // skip
                        }
                        CalcResult::Range { left, right } => {
                            if left.sheet != right.sheet {
                                return CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Ranges are in different sheets".to_string(),
                                );
                            }
                            // We are not expecting subtotal to have open ranges
                            let row1 = left.row;
                            let row2 = right.row;
                            let column1 = left.column;
                            let column2 = right.column;

                            for row in row1..=row2 {
                                let cell_status = match self
                                    .cell_hidden_status(left.sheet, row, column1)
                                {
                                    Ok(s) => s,
                                    Err(message) => {
                                        return CalcResult::new_error(Error::ERROR, cell, message);
                                    }
                                };
                                if mode.skips_row(&cell_status) {
                                    continue;
                                }
                                for column in column1..=column2 {
                                    if mode.skips_nested()
                                        && self.cell_is_subtotal(left.sheet, row, column)
                                    {
                                        continue;
                                    }
                                    match self.evaluate_cell(CellReferenceIndex {
                                        sheet: left.sheet,
                                        row,
                                        column,
                                    }) {
                                        CalcResult::EmptyCell | CalcResult::EmptyArg => {
                                            // skip
                                        }
                                        CalcResult::Error { .. } if mode.skips_errors() => {}
                                        _ => counta += 1,
                                    }
                                }
                            }
                        }
                        // for AGGREGATE an argument that is an error gives that error
                        error @ CalcResult::Error { .. } if mode.takes_only_references() => {
                            return error;
                        }
                        CalcResult::String(_)
                        | CalcResult::Number(_)
                        | CalcResult::Boolean(_)
                        | CalcResult::Error { .. } => counta += 1,
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
        CalcResult::Number(counta as f64)
    }

    fn subtotal_count(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let mut count = 0;
        for arg in args {
            match arg {
                Node::FunctionKind {
                    kind: Function::Subtotal,
                    args: _,
                } if !mode.takes_only_references() => {
                    // skip
                }
                _ => {
                    let value = self.evaluate_node_with_reference(arg, cell);
                    if mode.takes_only_references() && !is_reference(arg, &value) {
                        return not_a_reference(cell);
                    }
                    match value {
                        CalcResult::Range { left, right } => {
                            if left.sheet != right.sheet {
                                return CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Ranges are in different sheets".to_string(),
                                );
                            }
                            // We are not expecting subtotal to have open ranges
                            let row1 = left.row;
                            let row2 = right.row;
                            let column1 = left.column;
                            let column2 = right.column;

                            for row in row1..=row2 {
                                let cell_status = match self
                                    .cell_hidden_status(left.sheet, row, column1)
                                {
                                    Ok(s) => s,
                                    Err(message) => {
                                        return CalcResult::new_error(Error::ERROR, cell, message);
                                    }
                                };
                                if mode.skips_row(&cell_status) {
                                    continue;
                                }
                                for column in column1..=column2 {
                                    if mode.skips_nested()
                                        && self.cell_is_subtotal(left.sheet, row, column)
                                    {
                                        continue;
                                    }
                                    if let CalcResult::Number(_) =
                                        self.evaluate_cell(CellReferenceIndex {
                                            sheet: left.sheet,
                                            row,
                                            column,
                                        })
                                    {
                                        count += 1;
                                    }
                                }
                            }
                        }
                        // for AGGREGATE an argument that is an error gives that error
                        error @ CalcResult::Error { .. } if mode.takes_only_references() => {
                            return error;
                        }
                        // This hasn't been tested
                        CalcResult::Number(_) => count += 1,
                        _ => {}
                    }
                }
            }
        }
        CalcResult::Number(count as f64)
    }

    fn subtotal_average(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        let l = values.len();
        for value in values {
            result += value;
        }
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0!".to_string(),
            };
        }
        CalcResult::Number(result / (l as f64))
    }

    fn subtotal_sum(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = 0.0;
        for value in values {
            result += value;
        }
        CalcResult::Number(result)
    }

    fn subtotal_product(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        if values.is_empty() {
            // as PRODUCT
            return CalcResult::Number(0.0);
        }
        let mut result = 1.0;
        for value in values {
            result *= value;
        }
        CalcResult::Number(result)
    }

    fn subtotal_max(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = f64::NAN;
        for value in values {
            result = value.max(result);
        }
        if result.is_nan() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    fn subtotal_min(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: SubTotalMode,
    ) -> CalcResult {
        let values = match self.subtotal_get_values(args, cell, mode) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut result = f64::NAN;
        for value in values {
            result = value.min(result);
        }
        if result.is_nan() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }
}
