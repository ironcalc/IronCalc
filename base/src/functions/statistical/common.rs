use statrs::distribution::{Continuous, ContinuousCDF, Gamma as GammaDist};
use statrs::distribution::{Binomial, Discrete, DiscreteCDF};
use statrs::distribution::{ChiSquared, FisherSnedecor, Normal, StudentsT};
use statrs::distribution::{Hypergeometric, Poisson};
use statrs::function::gamma::{gamma, ln_gamma};

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::build_criteria;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::Node,
    expressions::token::Error,
    model::Model,
};

impl Model {
    pub(crate) fn fn_average(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    count += 1.0;
                    sum += value;
                }
                CalcResult::Boolean(b) => {
                    if let Node::ReferenceKind { .. } = arg {
                    } else {
                        sum += if b { 1.0 } else { 0.0 };
                        count += 1.0;
                    }
                }
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
                                CalcResult::Number(value) => {
                                    count += 1.0;
                                    sum += value;
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::Range { .. } => {
                                    return CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(s) => {
                    if let Node::ReferenceKind { .. } = arg {
                        // Do nothing
                    } else if let Ok(t) = s.parse::<f64>() {
                        sum += t;
                        count += 1.0;
                    } else {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Argument cannot be cast into number".to_string(),
                        };
                    }
                }
                _ => {
                    // Ignore everything else
                }
            };
        }
        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::Number(sum / count)
    }

    pub(crate) fn fn_averagea(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
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
                                CalcResult::String(_) => count += 1.0,
                                CalcResult::Number(value) => {
                                    count += 1.0;
                                    sum += value;
                                }
                                CalcResult::Boolean(b) => {
                                    if b {
                                        sum += 1.0;
                                    }
                                    count += 1.0;
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::Range { .. } => {
                                    return CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    );
                                }
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::Array(_) => {
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
                CalcResult::Number(value) => {
                    count += 1.0;
                    sum += value;
                }
                CalcResult::String(s) => {
                    if let Node::ReferenceKind { .. } = arg {
                        // Do nothing
                        count += 1.0;
                    } else if let Ok(t) = s.parse::<f64>() {
                        sum += t;
                        count += 1.0;
                    } else {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Argument cannot be cast into number".to_string(),
                        };
                    }
                }
                CalcResult::Boolean(b) => {
                    count += 1.0;
                    if b {
                        sum += 1.0;
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                CalcResult::Array(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
            };
        }
        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::Number(sum / count)
    }

    pub(crate) fn fn_count(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(_) => {
                    result += 1.0;
                }
                CalcResult::Boolean(_) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        result += 1.0;
                    }
                }
                CalcResult::String(s) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) && s.parse::<f64>().is_ok() {
                        result += 1.0;
                    }
                }
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
                            if let CalcResult::Number(_) = self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                result += 1.0;
                            }
                        }
                    }
                }
                _ => {
                    // Ignore everything else
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_counta(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
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
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                _ => {
                                    result += 1.0;
                                }
                            }
                        }
                    }
                }
                _ => {
                    result += 1.0;
                }
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_countblank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // COUNTBLANK requires only one argument
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 0.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::EmptyCell | CalcResult::EmptyArg => result += 1.0,
                CalcResult::String(s) => {
                    if s.is_empty() {
                        result += 1.0
                    }
                }
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
                                CalcResult::EmptyCell | CalcResult::EmptyArg => result += 1.0,
                                CalcResult::String(s) => {
                                    if s.is_empty() {
                                        result += 1.0
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_countif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let arguments = vec![args[0].clone(), args[1].clone()];
            self.fn_countifs(&arguments, cell)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    /// AVERAGEIF(criteria_range, criteria, [average_range])
    /// if average_rage is missing then criteria_range will be used
    pub(crate) fn fn_averageif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let arguments = vec![args[0].clone(), args[0].clone(), args[1].clone()];
            self.fn_averageifs(&arguments, cell)
        } else if args.len() == 3 {
            let arguments = vec![args[2].clone(), args[0].clone(), args[1].clone()];
            self.fn_averageifs(&arguments, cell)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    // FIXME: This function shares a lot of code with apply_ifs. Can we merge them?
    pub(crate) fn fn_countifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count < 2 || !args_count.is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }

        let case_count = args_count / 2;
        // NB: this is a beautiful example of the borrow checker
        // The order of these two definitions cannot be swapped.
        let mut criteria = Vec::new();
        let mut fn_criteria = Vec::new();
        let ranges = &mut Vec::new();
        for case_index in 0..case_count {
            let criterion = self.evaluate_node_in_context(&args[case_index * 2 + 1], cell);
            criteria.push(criterion);
            // NB: We cannot do:
            // fn_criteria.push(build_criteria(&criterion));
            // because criterion doesn't live long enough
            let result = self.evaluate_node_in_context(&args[case_index * 2], cell);
            if result.is_error() {
                return result;
            }
            if let CalcResult::Range { left, right } = result {
                if left.sheet != right.sheet {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    );
                }
                // TODO test ranges are of the same size as sum_range
                ranges.push(Range { left, right });
            } else {
                return CalcResult::new_error(Error::VALUE, cell, "Expected a range".to_string());
            }
        }
        for criterion in criteria.iter() {
            fn_criteria.push(build_criteria(criterion));
        }

        let mut total = 0.0;
        let first_range = &ranges[0];
        let left_row = first_range.left.row;
        let left_column = first_range.left.column;
        let right_row = first_range.right.row;
        let right_column = first_range.right.column;

        let dimension = match self.workbook.worksheet(first_range.left.sheet) {
            Ok(s) => s.dimension(),
            Err(_) => {
                return CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    format!("Invalid worksheet index: '{}'", first_range.left.sheet),
                )
            }
        };
        let max_row = dimension.max_row;
        let max_column = dimension.max_column;

        let open_row = left_row == 1 && right_row == LAST_ROW;
        let open_column = left_column == 1 && right_column == LAST_COLUMN;

        for row in left_row..right_row + 1 {
            if open_row && row > max_row {
                // If the row is larger than the max row in the sheet then all cells are empty.
                // We compute it only once
                let mut is_true = true;
                for fn_criterion in fn_criteria.iter() {
                    if !fn_criterion(&CalcResult::EmptyCell) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    total += ((LAST_ROW - max_row) * (right_column - left_column + 1)) as f64;
                }
                break;
            }
            for column in left_column..right_column + 1 {
                if open_column && column > max_column {
                    // If the column is larger than the max column in the sheet then all cells are empty.
                    // We compute it only once
                    let mut is_true = true;
                    for fn_criterion in fn_criteria.iter() {
                        if !fn_criterion(&CalcResult::EmptyCell) {
                            is_true = false;
                            break;
                        }
                    }
                    if is_true {
                        total += (LAST_COLUMN - max_column) as f64;
                    }
                    break;
                }
                let mut is_true = true;
                for case_index in 0..case_count {
                    // We check if value in range n meets criterion n
                    let range = &ranges[case_index];
                    let fn_criterion = &fn_criteria[case_index];
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - first_range.left.row,
                        column: range.left.column + column - first_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    total += 1.0;
                }
            }
        }
        CalcResult::Number(total)
    }

    pub(crate) fn apply_ifs<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mut apply: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        let args_count = args.len();
        if args_count < 3 || args_count.is_multiple_of(2) {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let arg_0 = self.evaluate_node_in_context(&args[0], cell);
        if arg_0.is_error() {
            return Err(arg_0);
        }
        let sum_range = if let CalcResult::Range { left, right } = arg_0 {
            if left.sheet != right.sheet {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ));
            }
            Range { left, right }
        } else {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a range".to_string(),
            ));
        };

        let case_count = (args_count - 1) / 2;
        // NB: this is a beautiful example of the borrow checker
        // The order of these two definitions cannot be swapped.
        let mut criteria = Vec::new();
        let mut fn_criteria = Vec::new();
        let ranges = &mut Vec::new();
        for case_index in 1..=case_count {
            let criterion = self.evaluate_node_in_context(&args[case_index * 2], cell);
            // NB: criterion might be an error. That's ok
            criteria.push(criterion);
            // NB: We cannot do:
            // fn_criteria.push(build_criteria(&criterion));
            // because criterion doesn't live long enough
            let result = self.evaluate_node_in_context(&args[case_index * 2 - 1], cell);
            if result.is_error() {
                return Err(result);
            }
            if let CalcResult::Range { left, right } = result {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                // TODO test ranges are of the same size as sum_range
                ranges.push(Range { left, right });
            } else {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expected a range".to_string(),
                ));
            }
        }
        for criterion in criteria.iter() {
            fn_criteria.push(build_criteria(criterion));
        }

        let left_row = sum_range.left.row;
        let left_column = sum_range.left.column;
        let mut right_row = sum_range.right.row;
        let mut right_column = sum_range.right.column;

        if left_row == 1 && right_row == LAST_ROW {
            right_row = match self.workbook.worksheet(sum_range.left.sheet) {
                Ok(s) => s.dimension().max_row,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", sum_range.left.sheet),
                    ));
                }
            };
        }
        if left_column == 1 && right_column == LAST_COLUMN {
            right_column = match self.workbook.worksheet(sum_range.left.sheet) {
                Ok(s) => s.dimension().max_column,
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", sum_range.left.sheet),
                    ));
                }
            };
        }

        for row in left_row..right_row + 1 {
            for column in left_column..right_column + 1 {
                let mut is_true = true;
                for case_index in 0..case_count {
                    // We check if value in range n meets criterion n
                    let range = &ranges[case_index];
                    let fn_criterion = &fn_criteria[case_index];
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - sum_range.left.row,
                        column: range.left.column + column - sum_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    let v = self.evaluate_cell(CellReferenceIndex {
                        sheet: sum_range.left.sheet,
                        row,
                        column,
                    });
                    match v {
                        CalcResult::Number(n) => apply(n),
                        CalcResult::Error { .. } => return Err(v),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn fn_averageifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut total = 0.0;
        let mut count = 0.0;

        let average = |value: f64| {
            total += value;
            count += 1.0;
        };
        if let Err(e) = self.apply_ifs(args, cell, average) {
            return e;
        }

        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "division by 0".to_string(),
            };
        }
        CalcResult::Number(total / count)
    }

    pub(crate) fn fn_minifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut min = f64::INFINITY;
        let apply_min = |value: f64| min = value.min(min);
        if let Err(e) = self.apply_ifs(args, cell, apply_min) {
            return e;
        }

        if min.is_infinite() {
            min = 0.0;
        }
        CalcResult::Number(min)
    }

    pub(crate) fn fn_maxifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let mut max = -f64::INFINITY;
        let apply_max = |value: f64| max = value.max(max);
        if let Err(e) = self.apply_ifs(args, cell, apply_max) {
            return e;
        }
        if max.is_infinite() {
            max = 0.0;
        }
        CalcResult::Number(max)
    }

    pub(crate) fn fn_geomean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut product = 1.0;
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => {
                    count += 1.0;
                    product *= value;
                }
                CalcResult::Boolean(b) => {
                    if let Node::ReferenceKind { .. } = arg {
                    } else {
                        product *= if b { 1.0 } else { 0.0 };
                        count += 1.0;
                    }
                }
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
                                CalcResult::Number(value) => {
                                    count += 1.0;
                                    product *= value;
                                }
                                error @ CalcResult::Error { .. } => return error,
                                CalcResult::Range { .. } => {
                                    return CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(s) => {
                    if let Node::ReferenceKind { .. } = arg {
                        // Do nothing
                    } else if let Ok(t) = s.parse::<f64>() {
                        product *= t;
                        count += 1.0;
                    } else {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Argument cannot be cast into number".to_string(),
                        };
                    }
                }
                _ => {
                    // Ignore everything else
                }
            };
        }
        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::Number(product.powf(1.0 / count))
    }


    pub(crate) fn fn_binom_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        // number_s
        let number_s = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // trials
        let trials = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // probability_s
        let p = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // cumulative (logical)
        let cumulative = match self.evaluate_node_in_context(&args[3], cell) {
            CalcResult::Boolean(b) => b,
            CalcResult::Number(n) => n != 0.0,
            CalcResult::String(s) => {
                let up = s.to_ascii_uppercase();
                if up == "TRUE" {
                    true
                } else if up == "FALSE" {
                    false
                } else {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid cumulative argument".to_string(),
                    );
                }
            }
            e @ CalcResult::Error { .. } => return e,
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid cumulative argument".to_string(),
                )
            }
        };

        // Excel truncates integer arguments
        let number_s_trunc = number_s.trunc();
        let trials_trunc = trials.trunc();

        // Domain checks
        if trials_trunc < 0.0
            || number_s_trunc < 0.0
            || number_s_trunc > trials_trunc
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
        {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid parameters for BINOM.DIST".to_string(),
            );
        }

        // Limit to u64
        if trials_trunc > u64::MAX as f64 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Number of trials too large".to_string(),
            );
        }

        let n = trials_trunc as u64;
        let k = number_s_trunc as u64;

        let dist = match Binomial::new(p, n) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for binomial distribution".to_string(),
                )
            }
        };

        let prob = if cumulative { dist.cdf(k) } else { dist.pmf(k) };

        if prob.is_nan() || prob.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for BINOM.DIST".to_string(),
            );
        }

        CalcResult::Number(prob)
    }

    pub(crate) fn fn_binom_dist_range(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // 3 or 4 args
        if args.len() < 3 || args.len() > 4 {
            return CalcResult::new_args_number_error(cell);
        }

        // trials
        let trials = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // probability_s
        let p = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // number_s (lower)
        let number_s = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // number_s2 (upper, optional)
        let number_s2 = if args.len() == 4 {
            match self.get_number_no_bools(&args[3], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        } else {
            number_s
        };

        let trials_trunc = trials.trunc();
        let s1_trunc = number_s.trunc();
        let s2_trunc = number_s2.trunc();

        // Domain checks (Excel-style)
        if trials_trunc < 0.0
            || s1_trunc < 0.0
            || s2_trunc < 0.0
            || s1_trunc > s2_trunc
            || s2_trunc > trials_trunc
            || p.is_nan()
            || p < 0.0
            || p > 1.0
        {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid parameters for BINOM.DIST.RANGE".to_string(),
            );
        }

        if trials_trunc > u64::MAX as f64 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Number of trials too large".to_string(),
            );
        }

        let n = trials_trunc as u64;
        let lower = s1_trunc as u64;
        let upper = s2_trunc as u64;

        let dist = match Binomial::new(p, n) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for binomial distribution".to_string(),
                )
            }
        };

        let prob = if lower == 0 {
            dist.cdf(upper)
        } else {
            let cdf_upper = dist.cdf(upper);
            let cdf_below_lower = dist.cdf(lower - 1);
            cdf_upper - cdf_below_lower
        };

        if prob.is_nan() || prob.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for BINOM.DIST.RANGE".to_string(),
            );
        }

        CalcResult::Number(prob)
    }

    pub(crate) fn fn_binom_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // trials
        let trials = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // probability_s
        let p = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // alpha
        let alpha = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let trials_trunc = trials.trunc();

        if trials_trunc < 0.0
            || trials_trunc > u64::MAX as f64
            || p.is_nan()
            || !(0.0..=1.0).contains(&p)
            || alpha.is_nan()
            || !(0.0..=1.0).contains(&alpha)
        {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid parameters for BINOM.INV".to_string(),
            );
        }

        let n = trials_trunc as u64;

        let dist = match Binomial::new(p, n) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for binomial distribution".to_string(),
                )
            }
        };

        // DiscreteCDF::inverse_cdf returns u64 for binomial
        let k = statrs::distribution::DiscreteCDF::inverse_cdf(&dist, alpha);

        CalcResult::Number(k as f64)
    }

    pub(crate) fn fn_chisq_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // CHISQ.DIST(x, deg_freedom, cumulative)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel truncates non-integer degrees of freedom
        let df = df_raw.trunc();

        // cumulative: accept bool, number, or "TRUE"/"FALSE"
        let cumulative = match self.evaluate_node_in_context(&args[2], cell) {
            CalcResult::Boolean(b) => b,
            CalcResult::Number(n) => n != 0.0,
            CalcResult::String(s) => {
                let up = s.to_ascii_uppercase();
                if up == "TRUE" {
                    true
                } else if up == "FALSE" {
                    false
                } else {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "cumulative must be TRUE/FALSE or numeric".to_string(),
                    };
                }
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid cumulative argument".to_string(),
                }
            }
        };

        // Excel domain checks
        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in CHISQ.DIST".to_string(),
            );
        }
        if df < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in CHISQ.DIST".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_chisq_dist_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // CHISQ.DIST.RT(x, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in CHISQ.DIST.RT".to_string(),
            );
        }
        if df < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in CHISQ.DIST.RT".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        // Right-tail probability: P(X > x).
        // Use sf(x) directly for better numerical properties than 1 - cdf(x).
        let result = dist.sf(x);

        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.DIST.RT".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_chisq_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // CHISQ.INV(probability, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        // Excel: if probability < 0 or > 1 → #NUM!
        if p < 0.0 || p > 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV".to_string(),
            );
        }
        if df < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in CHISQ.INV".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        let x = dist.inverse_cdf(p);

        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.INV".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_chisq_inv_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // CHISQ.INV.RT(probability, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        // Excel: if probability < 0 or > 1 → #NUM!
        // (Docs for .RT say probability <= 0 or > 1 gives #NUM!,
        // but inverse_cdf(1.0) and inverse_cdf(0.0) are handled below via inf/NaN checks.)
        if p < 0.0 || p > 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV.RT".to_string(),
            );
        }
        if df < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in CHISQ.INV.RT".to_string(),
            );
        }

        let dist = match ChiSquared::new(df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Chi-squared distribution".to_string(),
                )
            }
        };

        // Right-tail inverse: p = P(X > x) = SF(x) = 1 - CDF(x)
        // So x = inverse_cdf(1 - p).
        let x = dist.inverse_cdf(1.0 - p);

        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for CHISQ.INV.RT".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_chisq_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_confidence_norm(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let alpha = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let size = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.floor(),
            Err(e) => return e,
        };

        if alpha <= 0.0 || alpha >= 1.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for CONFIDENCE.NORM".to_string(),
            };
        }
        if size < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Sample size must be at least 1".to_string(),
            };
        }

        let normal = match Normal::new(0.0, 1.0) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    "Failed to construct normal distribution".to_string(),
                )
            }
        };

        let quantile = normal.inverse_cdf(1.0 - alpha / 2.0);
        if !quantile.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid quantile for CONFIDENCE.NORM".to_string(),
            };
        }

        let margin = quantile * std_dev / size.sqrt();
        CalcResult::Number(margin)
    }

    pub(crate) fn fn_confidence_t(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let alpha = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let size = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Domain checks
        if alpha <= 0.0 || alpha >= 1.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for CONFIDENCE.T".to_string(),
            };
        }

        // Excel truncates size to an integer
        let n = size.trunc();

        // Need at least 2 observations so df = n - 1 > 0
        if n < 2.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Sample size must be at least 2".to_string(),
            };
        }

        let df = n - 1.0;

        let t_dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    "Failed to construct Student's t distribution".to_string(),
                )
            }
        };

        // Two-sided CI => use 1 - alpha/2
        let t_crit = t_dist.inverse_cdf(1.0 - alpha / 2.0);
        if !t_crit.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid quantile for CONFIDENCE.T".to_string(),
            };
        }

        let margin = t_crit * std_dev / n.sqrt();
        CalcResult::Number(margin)
    }

    pub(crate) fn fn_covariance_p(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_covariance_s(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_devsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_expon_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // EXPON.DIST(x, lambda, cumulative)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let lambda = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[2], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 || lambda <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for EXPON.DIST".to_string(),
            };
        }

        let result = if cumulative {
            // CDF
            1.0 - (-lambda * x).exp()
        } else {
            // PDF
            lambda * (-lambda * x).exp()
        };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for EXPON.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_f_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // F.DIST(x, deg_freedom1, deg_freedom2, cumulative)
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df1_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df2_raw = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel truncates non-integer degrees of freedom
        let df1 = df1_raw.trunc();
        let df2 = df2_raw.trunc();

        // cumulative argument: handle bool / number / "TRUE"/"FALSE"
        let cumulative = match self.evaluate_node_in_context(&args[3], cell) {
            CalcResult::Boolean(b) => b,
            CalcResult::Number(n) => n != 0.0,
            CalcResult::String(s) => {
                let up = s.to_ascii_uppercase();
                if up == "TRUE" {
                    true
                } else if up == "FALSE" {
                    false
                } else {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "cumulative must be TRUE/FALSE or numeric".to_string(),
                    };
                }
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid cumulative argument".to_string(),
                }
            }
        };

        // Excel domain checks
        if x < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "x must be >= 0 in F.DIST".to_string());
        }
        if df1 < 1.0 || df2 < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.DIST".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_f_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // F.DIST.RT(x, deg_freedom1, deg_freedom2)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df1_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df2_raw = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df1 = df1_raw.trunc();
        let df2 = df2_raw.trunc();

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in F.DIST.RT".to_string(),
            );
        }
        if df1 < 1.0 || df2 < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.DIST.RT".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        // Right-tail probability: P(F > x) = 1 - CDF(x)
        let result = 1.0 - dist.cdf(x);

        if result.is_nan() || result.is_infinite() || result < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.DIST.RT".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_f_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // F.INV(probability, deg_freedom1, deg_freedom2)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df1_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df2_raw = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df1 = df1_raw.trunc();
        let df2 = df2_raw.trunc();

        // Excel: probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in F.INV".to_string(),
            );
        }
        if df1 < 1.0 || df2 < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.INV".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        let x = dist.inverse_cdf(p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid result for F.INV".to_string());
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_f_inv_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // F.INV.RT(probability, deg_freedom1, deg_freedom2)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df1_raw = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let df2_raw = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df1 = df1_raw.trunc();
        let df2 = df2_raw.trunc();

        // Excel: 0 < p <= 1
        if p <= 0.0 || p > 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in (0,1] in F.INV.RT".to_string(),
            );
        }
        if df1 < 1.0 || df2 < 1.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be >= 1 in F.INV.RT".to_string(),
            );
        }

        let dist = match FisherSnedecor::new(df1, df2) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for F distribution".to_string(),
                )
            }
        };

        // p is right-tail: p = P(F > x) = 1 - CDF(x)
        let x = dist.inverse_cdf(1.0 - p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for F.INV.RT".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_fisher(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // FISHER(x) = 0.5 * ln((1 + x) / (1 - x))
        // Excel: x must be in (-1, 1); otherwise #NUM!
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Domain check like Excel
        if x <= -1.0 || x >= 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "x must be between -1 and 1 (exclusive) in FISHER".to_string(),
            };
        }

        let ratio = (1.0 + x) / (1.0 - x);
        let result = 0.5 * ratio.ln();

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for FISHER".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_fisher_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // FISHERINV(y) = (e^(2y) - 1) / (e^(2y) + 1) = tanh(y)
        // Excel: defined for all real y (no domain restriction)
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let y = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Use tanh directly to avoid overflow from exp(2y)
        let result = y.tanh();

        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for FISHERINV".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_gamma(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if x < 0.0 && x.floor() == x {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for Gamma function".to_string(),
            };
        }
        let result = gamma(x);
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for Gamma function".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_gamma_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // GAMMA.DIST(x, alpha, beta, cumulative)
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_scale = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in GAMMA.DIST".to_string(),
            );
        }
        if alpha <= 0.0 || beta_scale <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in GAMMA.DIST".to_string(),
            );
        }

        let rate = 1.0 / beta_scale;

        let dist = match GammaDist::new(alpha, rate) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Gamma distribution".to_string(),
                )
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if result.is_nan() || result.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for GAMMA.DIST".to_string(),
            );
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_gamma_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // GAMMA.INV(probability, alpha, beta)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let alpha = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let beta_scale = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in GAMMA.INV".to_string(),
            );
        }

        if alpha <= 0.0 || beta_scale <= 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "alpha and beta must be > 0 in GAMMA.INV".to_string(),
            );
        }

        let rate = 1.0 / beta_scale;

        let dist = match GammaDist::new(alpha, rate) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for Gamma distribution".to_string(),
                )
            }
        };

        let x = dist.inverse_cdf(p);
        if x.is_nan() || x.is_infinite() || x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Invalid result for GAMMA.INV".to_string(),
            );
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_gamma_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if x < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for Gamma function".to_string(),
            };
        }
        let result = ln_gamma(x);
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for Gamma Ln function".to_string(),
            };
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_gamma_ln_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.fn_gamma_ln(args, cell)
    }

    // =HYPGEOM.DIST(sample_s, number_sample, population_s, number_pop, cumulative)
    pub(crate) fn fn_hyp_geom_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 5 {
            return CalcResult::new_args_number_error(cell);
        }

        // sample_s (number of successes in the sample)
        let sample_s = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // number_sample (sample size)
        let number_sample = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // population_s (number of successes in the population)
        let population_s = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // number_pop (population size)
        let number_pop = match self.get_number_no_bools(&args[3], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[4], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if sample_s < 0.0 || sample_s > f64::min(number_sample, population_s) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for HYPGEOM.DIST".to_string(),
            };
        }

        if sample_s < f64::max(0.0, number_sample + population_s - number_pop) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for HYPGEOM.DIST".to_string(),
            };
        }

        if number_sample <= 0.0 || number_sample > number_pop {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for HYPGEOM.DIST".to_string(),
            };
        }

        if population_s <= 0.0 || population_s > number_pop {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for HYPGEOM.DIST".to_string(),
            };
        }

        let n_pop = number_pop as u64;
        let k_pop = population_s as u64;
        let n_sample = number_sample as u64;
        let k = sample_s as u64;

        let dist = match Hypergeometric::new(n_pop, k_pop, n_sample) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid parameters for hypergeometric distribution".to_string(),
                )
            }
        };

        let prob = if cumulative { dist.cdf(k) } else { dist.pmf(k) };

        if !prob.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for HYPGEOM.DIST".to_string(),
            };
        }

        CalcResult::Number(prob)
    }

    pub(crate) fn fn_log_norm_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use statrs::distribution::{Continuous, ContinuousCDF, LogNormal};

        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        // Excel domain checks
        if x <= 0.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.DIST".to_string(),
            };
        }

        let dist = match LogNormal::new(mean, std_dev) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameter for LOGNORM.DIST".to_string(),
                }
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_log_norm_inv(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use statrs::distribution::{ContinuousCDF, LogNormal};

        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel domain checks
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.INV".to_string(),
            };
        }

        let dist = match LogNormal::new(mean, std_dev) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameter for LOGNORM.INV".to_string(),
                }
            }
        };

        let result = dist.inverse_cdf(p);

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.INV".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_negbinom_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use statrs::distribution::{Discrete, DiscreteCDF, NegativeBinomial};

        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let number_f = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };
        let number_s = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };
        let probability_s = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if number_f < 0.0 || number_s < 1.0 || !(0.0..=1.0).contains(&probability_s) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for NEGBINOM.DIST".to_string(),
            };
        }

        // Guard against absurdly large failures that won't fit in u64
        if number_f > (u64::MAX as f64) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for NEGBINOM.DIST".to_string(),
            };
        }

        let dist = match NegativeBinomial::new(number_s, probability_s) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameter for NEGBINOM.DIST".to_string(),
                }
            }
        };

        let f_u = number_f as u64;
        let result = if cumulative {
            dist.cdf(f_u)
        } else {
            dist.pmf(f_u)
        };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for NEGBINOM.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_norm_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // NORM.DIST(x, mean, standard_dev, cumulative)
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        // Excel: standard_dev must be > 0
        if std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "standard_dev must be > 0 in NORM.DIST".to_string(),
            };
        }

        let dist = match Normal::new(mean, std_dev) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for NORM.DIST".to_string(),
                }
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for NORM.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_norm_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // NORM.INV(probability, mean, standard_dev)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel domain checks
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for NORM.INV".to_string(),
            };
        }

        let dist = match Normal::new(mean, std_dev) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for NORM.INV".to_string(),
                }
            }
        };

        let x = dist.inverse_cdf(p);

        if !x.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for NORM.INV".to_string(),
            };
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_norm_s_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // NORM.S.DIST(z, cumulative)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let z = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let cumulative = match self.get_boolean(&args[1], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        let dist = match Normal::new(0.0, 1.0) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Failed to construct standard normal distribution".to_string(),
                }
            }
        };

        let result = if cumulative { dist.cdf(z) } else { dist.pdf(z) };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for NORM.S.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_norm_s_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // NORM.S.INV(probability)
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel domain: 0 < p < 1
        if p <= 0.0 || p >= 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "probability must be in (0,1) in NORM.S.INV".to_string(),
            };
        }

        let dist = match Normal::new(0.0, 1.0) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Failed to construct standard normal distribution".to_string(),
                }
            }
        };

        let z = dist.inverse_cdf(p);

        if !z.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for NORM.S.INV".to_string(),
            };
        }

        CalcResult::Number(z)
    }

    pub(crate) fn fn_pearson(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_phi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // PHI(x) = standard normal PDF at x
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Standard normal PDF: (1 / sqrt(2π)) * exp(-x^2 / 2)
        let result = (-(x * x) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();

        if !result.is_finite() {
            // In practice this shouldn't happen, but keep it consistent with other funcs
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for PHI".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // =POISSON.DIST(x, mean, cumulative)
    pub(crate) fn fn_poisson_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // x
        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // mean (lambda)
        let lambda = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[2], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 || lambda < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for POISSON.DIST".to_string(),
            };
        }

        // Guard against insane k for u64
        if x < 0.0 || x > (u64::MAX as f64) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for POISSON.DIST".to_string(),
            };
        }

        let k = x as u64;

        // Special-case lambda = 0: degenerate distribution at 0
        if lambda == 0.0 {
            let result = if cumulative {
                // For x >= 0, P(X <= x) = 1
                1.0
            } else {
                // P(X = 0) = 1, P(X = k>0) = 0
                if k == 0 {
                    1.0
                } else {
                    0.0
                }
            };
            return CalcResult::Number(result);
        }

        let dist = match Poisson::new(lambda) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for POISSON.DIST".to_string(),
                }
            }
        };

        let prob = if cumulative { dist.cdf(k) } else { dist.pmf(k) };

        if !prob.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for POISSON.DIST".to_string(),
            };
        }

        CalcResult::Number(prob)
    }

    pub(crate) fn fn_standardize(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // STANDARDIZE(x, mean, standard_dev)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number_no_bools(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        if std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "standard_dev must be > 0 in STANDARDIZE".to_string(),
            };
        }

        let z = (x - mean) / std_dev;

        CalcResult::Number(z)
    }

    pub(crate) fn fn_stdev_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_stdev_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_stdeva(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_stdevpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_t_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // T.DIST(x, deg_freedom, cumulative)
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[2], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        // Excel: df >= 1
        if df < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom must be >= 1 in T.DIST".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST".to_string(),
                }
            }
        };

        let result = if cumulative { dist.cdf(x) } else { dist.pdf(x) };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_t_dist_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // T.DIST.2T(x, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        if x < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "x must be >= 0 in T.DIST.2T".to_string(),
            };
        }

        if df < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom must be >= 1 in T.DIST.2T".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST.2T".to_string(),
                }
            }
        };

        let ax = x.abs();
        let upper_tail = 1.0 - dist.cdf(ax);
        let mut result = 2.0 * upper_tail;

        result = result.clamp(0.0, 1.0);

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST.2T".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_t_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // T.DIST.RT(x, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        if df < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "deg_freedom must be >= 1 in T.DIST.RT".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.DIST.RT".to_string(),
                }
            }
        };

        let result = 1.0 - dist.cdf(x);

        if !result.is_finite() || result < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.DIST.RT".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_t_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // T.INV(probability, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // Excel: 0 < p < 1, df >= 1
        if p <= 0.0 || p >= 1.0 || df < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for T.INV".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.INV".to_string(),
                }
            }
        };

        let x = dist.inverse_cdf(p);

        if !x.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.INV".to_string(),
            };
        }

        CalcResult::Number(x)
    }

    pub(crate) fn fn_t_inv_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // T.INV.2T(probability, deg_freedom)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        if p <= 0.0 || p > 1.0 || df < 1.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameters for T.INV.2T".to_string(),
            };
        }

        let dist = match StudentsT::new(0.0, 1.0, df) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Invalid parameters for T.INV.2T".to_string(),
                }
            }
        };

        // Two-sided: F(x) = 1 - p/2
        let target_cdf = 1.0 - p / 2.0;
        let x = dist.inverse_cdf(target_cdf);

        if !x.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for T.INV.2T".to_string(),
            };
        }

        CalcResult::Number(x.abs()) // Excel returns the positive root
    }

    pub(crate) fn fn_t_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_var_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_var_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_varpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }

    pub(crate) fn fn_z_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }
}
