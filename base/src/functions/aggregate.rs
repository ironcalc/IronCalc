use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
};

/// What AGGREGATE's second argument asks it to leave out.
#[derive(Clone, Copy)]
struct Skip {
    hidden_rows: bool,
    errors: bool,
    nested: bool,
}

/// The values AGGREGATE works on.
#[derive(Default)]
struct Values {
    numbers: Vec<f64>,
    // Cells or values that are not empty (for COUNTA).
    non_empty: usize,
}

fn is_nested_total(node: &Node) -> bool {
    matches!(
        node,
        Node::FunctionKind {
            kind: Function::Subtotal | Function::Aggregate,
            ..
        }
    )
}

fn percentile_inc(sorted: &[f64], k: f64) -> Option<f64> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&k) {
        return None;
    }
    let rank = k * (sorted.len() - 1) as f64;
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    Some(sorted[low] + (rank - low as f64) * (sorted[high] - sorted[low]))
}

fn percentile_exc(sorted: &[f64], k: f64) -> Option<f64> {
    let n = sorted.len() as f64;
    let rank = k * (n + 1.0);
    if sorted.is_empty() || k <= 0.0 || k >= 1.0 || rank < 1.0 || rank > n {
        return None;
    }
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    Some(sorted[low - 1] + (rank - low as f64) * (sorted[high - 1] - sorted[low - 1]))
}

fn variance(numbers: &[f64], sample: bool) -> Option<f64> {
    let n = numbers.len();
    if n == 0 || (sample && n < 2) {
        return None;
    }
    let mean = numbers.iter().sum::<f64>() / n as f64;
    let sum_sq: f64 = numbers.iter().map(|v| (v - mean).powi(2)).sum();
    Some(sum_sq / if sample { (n - 1) as f64 } else { n as f64 })
}

impl<'a> Model<'a> {
    fn aggregate_cell_is_total(&self, sheet: u32, row: i32, column: i32) -> bool {
        let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) else {
            return false;
        };
        let Some(formula) = worksheet
            .sheet_data
            .get(&row)
            .and_then(|r| r.get(&column))
            .and_then(|c| c.get_formula())
        else {
            return false;
        };
        self.parsed_formulas
            .get(sheet as usize)
            .and_then(|f| f.get(formula as usize))
            .is_some_and(|(node, _)| is_nested_total(node))
    }

    fn aggregate_row_hidden(&self, sheet: u32, row: i32) -> bool {
        self.workbook
            .worksheet(sheet)
            .map(|ws| ws.rows.iter().any(|r| r.r == row && r.hidden))
            .unwrap_or(false)
    }

    // Adds one value; returns the error to stop with, if any.
    fn aggregate_push(
        values: &mut Values,
        value: CalcResult,
        skip: Skip,
    ) -> Result<(), CalcResult> {
        match value {
            CalcResult::Number(f) => {
                values.numbers.push(f);
                values.non_empty += 1;
            }
            CalcResult::String(_) | CalcResult::Boolean(_) => values.non_empty += 1,
            error @ CalcResult::Error { .. } if !skip.errors => return Err(error),
            _ => {}
        }
        Ok(())
    }

    fn aggregate_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        skip: Skip,
    ) -> Result<Values, CalcResult> {
        let mut values = Values::default();
        for arg in args {
            if skip.nested && is_nested_total(arg) {
                continue;
            }
            match self.evaluate_node_with_reference(arg, cell) {
                CalcResult::Range { left, right } => {
                    if left.sheet != right.sheet {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Ranges are in different sheets".to_string(),
                        ));
                    }
                    let (row1, row2) = (left.row.min(right.row), left.row.max(right.row));
                    let (column1, column2) =
                        (left.column.min(right.column), left.column.max(right.column));
                    // A whole column or row: only look where there is data.
                    let row2 = row2.min(
                        self.workbook
                            .worksheet(left.sheet)
                            .map(|ws| ws.sheet_data.keys().copied().max().unwrap_or(0))
                            .unwrap_or(0),
                    );
                    for row in row1..=row2 {
                        if skip.hidden_rows && self.aggregate_row_hidden(left.sheet, row) {
                            continue;
                        }
                        for column in column1..=column2 {
                            if skip.nested && self.aggregate_cell_is_total(left.sheet, row, column)
                            {
                                continue;
                            }
                            let value = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            });
                            Self::aggregate_push(&mut values, value, skip)?;
                        }
                    }
                }
                CalcResult::Array(rows) => {
                    for item in rows.into_iter().flatten() {
                        let value = match item {
                            ArrayNode::Number(f) => CalcResult::Number(f),
                            ArrayNode::String(s) => CalcResult::String(s),
                            ArrayNode::Boolean(b) => CalcResult::Boolean(b),
                            ArrayNode::Error(error) => {
                                CalcResult::new_error(error, cell, "Error in array".to_string())
                            }
                            ArrayNode::Empty => CalcResult::EmptyCell,
                        };
                        Self::aggregate_push(&mut values, value, skip)?;
                    }
                }
                CalcResult::Lambda(_) => {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Expected a reference".to_string(),
                    ))
                }
                value => Self::aggregate_push(&mut values, value, skip)?,
            }
        }
        Ok(values)
    }

    /// `=AGGREGATE(function_num, options, ref1, [ref2], ...)` for functions 1-13
    /// `=AGGREGATE(function_num, options, array, k)` for functions 14-19
    ///
    /// function_num: 1 AVERAGE, 2 COUNT, 3 COUNTA, 4 MAX, 5 MIN, 6 PRODUCT,
    /// 7 STDEV.S, 8 STDEV.P, 9 SUM, 10 VAR.S, 11 VAR.P, 12 MEDIAN, 13 MODE.SNGL,
    /// 14 LARGE, 15 SMALL, 16 PERCENTILE.INC, 17 QUARTILE.INC,
    /// 18 PERCENTILE.EXC, 19 QUARTILE.EXC.
    ///
    /// options: 0 (or empty) leaves out nested SUBTOTAL and AGGREGATE,
    /// 1 also hidden rows, 2 also errors, 3 also both, 4 nothing,
    /// 5 only hidden rows, 6 only errors, 7 hidden rows and errors.
    pub(crate) fn fn_aggregate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let function = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i32,
            Err(e) => return e,
        };
        let options = match &args[1] {
            Node::EmptyArgKind => 0,
            node => match self.get_number(node, cell) {
                Ok(f) => f.trunc() as i32,
                Err(e) => return e,
            },
        };
        if !(1..=19).contains(&function) || !(0..=7).contains(&options) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid function or option number".to_string(),
            );
        }
        let skip = Skip {
            hidden_rows: matches!(options, 1 | 3 | 5 | 7),
            errors: matches!(options, 2 | 3 | 6 | 7),
            nested: options <= 3,
        };
        let div0 = || CalcResult::new_error(Error::DIV, cell, "Division by 0!".to_string());
        let num = || CalcResult::new_error(Error::NUM, cell, "Invalid value".to_string());

        if function >= 14 {
            if args.len() != 4 {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "This function needs an array and k".to_string(),
                );
            }
            let k = match self.get_number(&args[3], cell) {
                Ok(f) => f,
                Err(e) => return e,
            };
            let values = match self.aggregate_values(&args[2..3], cell, skip) {
                Ok(v) => v,
                Err(e) => return e,
            };
            let mut sorted = values.numbers;
            sorted.sort_by(|a, b| a.total_cmp(b));
            let result = match function {
                14 | 15 => {
                    let k = k.ceil();
                    if k < 1.0 || k > sorted.len() as f64 {
                        None
                    } else if function == 14 {
                        Some(sorted[sorted.len() - k as usize])
                    } else {
                        Some(sorted[k as usize - 1])
                    }
                }
                16 => percentile_inc(&sorted, k),
                17 => {
                    let q = k.trunc();
                    if (0.0..=4.0).contains(&q) {
                        percentile_inc(&sorted, q / 4.0)
                    } else {
                        None
                    }
                }
                18 => percentile_exc(&sorted, k),
                _ => {
                    let q = k.trunc();
                    if (1.0..=3.0).contains(&q) {
                        percentile_exc(&sorted, q / 4.0)
                    } else {
                        None
                    }
                }
            };
            return match result {
                Some(f) => CalcResult::Number(f),
                None => num(),
            };
        }

        let values = match self.aggregate_values(&args[2..], cell, skip) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let numbers = values.numbers;
        let n = numbers.len();
        match function {
            1 => {
                if n == 0 {
                    div0()
                } else {
                    CalcResult::Number(numbers.iter().sum::<f64>() / n as f64)
                }
            }
            2 => CalcResult::Number(n as f64),
            3 => CalcResult::Number(values.non_empty as f64),
            4 => CalcResult::Number(numbers.iter().copied().fold(f64::NAN, f64::max).max(0.0)),
            5 => CalcResult::Number(if n == 0 {
                0.0
            } else {
                numbers.iter().copied().fold(f64::NAN, f64::min)
            }),
            6 => CalcResult::Number(if n == 0 {
                0.0
            } else {
                numbers.iter().product()
            }),
            7 | 8 | 10 | 11 => {
                let sample = function == 7 || function == 10;
                match variance(&numbers, sample) {
                    Some(v) if function <= 8 => CalcResult::Number(v.sqrt()),
                    Some(v) => CalcResult::Number(v),
                    None => div0(),
                }
            }
            9 => CalcResult::Number(numbers.iter().sum()),
            12 => {
                let mut sorted = numbers;
                sorted.sort_by(|a, b| a.total_cmp(b));
                match percentile_inc(&sorted, 0.5) {
                    Some(f) => CalcResult::Number(f),
                    None => num(),
                }
            }
            _ => {
                // MODE.SNGL: the most frequent value; the first one on a tie.
                let mut best: Option<(f64, usize)> = None;
                for (i, v) in numbers.iter().enumerate() {
                    if numbers[..i].contains(v) {
                        continue;
                    }
                    let count = numbers[i..].iter().filter(|w| *w == v).count();
                    if count > 1 && best.is_none_or(|(_, c)| count > c) {
                        best = Some((*v, count));
                    }
                }
                match best {
                    Some((v, _)) => CalcResult::Number(v),
                    None => CalcResult::new_error(Error::NA, cell, "No repeated value".to_string()),
                }
            }
        }
    }
}
