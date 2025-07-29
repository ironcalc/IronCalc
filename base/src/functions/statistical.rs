use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::Node,
    expressions::token::Error,
    model::Model,
};

use super::util::{build_criteria, collect_numeric_values, collect_series};
use std::cmp::Ordering;

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
        if args_count < 2 || args_count % 2 == 1 {
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
        if args_count < 3 || args_count % 2 == 0 {
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
        let values = match collect_numeric_values(self, args, cell) {
            Ok(v) => v,
            Err(err) => return err,
        };

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        let product: f64 = values.iter().product();
        let count = values.len() as f64;
        CalcResult::Number(product.powf(1.0 / count))
    }
    pub(crate) fn fn_var_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_var_generic(args, cell, true)
    }

    pub(crate) fn fn_var_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_var_generic(args, cell, false)
    }

    fn fn_var_generic(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        sample: bool,
    ) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut values = Vec::new();
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Boolean(b) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        values.push(if b { 1.0 } else { 0.0 });
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
                    for row in row1..=row2 {
                        for column in column1..=column2 {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(v) => values.push(v),
                                error @ CalcResult::Error { .. } => return error,
                                _ => {}
                            }
                        }
                    }
                }
                CalcResult::String(s) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        if let Ok(t) = s.parse::<f64>() {
                            values.push(t);
                        } else {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Argument cannot be cast into number".to_string(),
                            };
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::Array(_) => {
                    return CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    }
                }
                _ => {}
            }
        }
        let count = values.len() as f64;
        if (sample && count < 2.0) || (!sample && count == 0.0) {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let mut sum = 0.0;
        for v in &values {
            sum += *v;
        }
        let mean = sum / count;
        let mut var = 0.0;
        for v in &values {
            var += (*v - mean).powi(2);
        }
        if sample {
            var /= count - 1.0;
        } else {
            var /= count;
        }
        CalcResult::Number(var)
    }

    pub(crate) fn fn_correl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let (data1, len1) = match self.correl_collect(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let (data2, len2) = match self.correl_collect(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if len1 != len2 {
            return CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Arrays must be of the same size".to_string(),
            };
        }
        let mut pairs = Vec::new();
        for i in 0..len1 {
            if let (Some(x), Some(y)) = (data1[i], data2[i]) {
                pairs.push((x, y));
            }
        }
        let n = pairs.len() as f64;
        if n < 2.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for (x, y) in &pairs {
            sum_x += *x;
            sum_y += *y;
        }
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        let mut num = 0.0;
        let mut sx = 0.0;
        let mut sy = 0.0;
        for (x, y) in &pairs {
            let dx = *x - mean_x;
            let dy = *y - mean_y;
            num += dx * dy;
            sx += dx * dx;
            sy += dy * dy;
        }
        if sx == 0.0 || sy == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        CalcResult::Number(num / (sx.sqrt() * sy.sqrt()))
    }

    fn correl_collect(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<Option<f64>>, usize), CalcResult> {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(f) => Ok((vec![Some(f)], 1)),
            CalcResult::Boolean(b) => {
                if matches!(node, Node::ReferenceKind { .. }) {
                    Ok((vec![None], 1))
                } else {
                    Ok((vec![Some(if b { 1.0 } else { 0.0 })], 1))
                }
            }
            CalcResult::String(s) => {
                if matches!(node, Node::ReferenceKind { .. }) {
                    Ok((vec![None], 1))
                } else if let Ok(t) = s.parse::<f64>() {
                    Ok((vec![Some(t)], 1))
                } else {
                    Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Argument cannot be cast into number".to_string(),
                    })
                }
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok((vec![None], 1)),
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
                let column1 = left.column;
                let mut column2 = right.column;
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = match self.workbook.worksheet(left.sheet) {
                        Ok(s) => s.dimension().max_row,
                        Err(_) => {
                            return Err(CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            ));
                        }
                    };
                }
                if column1 == 1 && column2 == LAST_COLUMN {
                    column2 = match self.workbook.worksheet(left.sheet) {
                        Ok(s) => s.dimension().max_column,
                        Err(_) => {
                            return Err(CalcResult::new_error(
                                Error::ERROR,
                                cell,
                                format!("Invalid worksheet index: '{}'", left.sheet),
                            ));
                        }
                    };
                }
                let mut v = Vec::new();
                for row in row1..=row2 {
                    for column in column1..=column2 {
                        match self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }) {
                            CalcResult::Number(f) => v.push(Some(f)),
                            error @ CalcResult::Error { .. } => return Err(error),
                            _ => v.push(None),
                        }
                    }
                }
                let len = v.len();
                Ok((v, len))
            }
            CalcResult::Array(_) => Err(CalcResult::Error {
                error: Error::NIMPL,
                origin: cell,
                message: "Arrays not supported yet".to_string(),
            }),
            error @ CalcResult::Error { .. } => Err(error),
        }
    }

    pub(crate) fn fn_large(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values = Vec::new();
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Number(v) => values.push(v),
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    );
                }
                for row in left.row..=right.row {
                    for column in left.column..=right.column {
                        match self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }) {
                            CalcResult::Number(v) => values.push(v),
                            error @ CalcResult::Error { .. } => return error,
                            _ => {}
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {}
        }

        let k = match self.get_number(&args[1], cell) {
            Ok(v) => {
                if v < 1.0 {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "k must be a positive number".to_string(),
                    );
                }
                v as usize
            }
            Err(e) => return e,
        };

        if k > values.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "k is larger than the number of values".to_string(),
            );
        }
        values.sort_by(|a, b| b.total_cmp(a));
        CalcResult::Number(values[k - 1])
    }

    pub(crate) fn fn_small(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values = Vec::new();
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::Number(v) => values.push(v),
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    );
                }
                for row in left.row..=right.row {
                    for column in left.column..=right.column {
                        match self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        }) {
                            CalcResult::Number(v) => values.push(v),
                            error @ CalcResult::Error { .. } => return error,
                            _ => {}
                        }
                    }
                }
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {}
        }

        let k = match self.get_number(&args[1], cell) {
            Ok(v) => {
                if v < 1.0 {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "k must be a positive number".to_string(),
                    );
                }
                v as usize
            }
            Err(e) => return e,
        };

        if k > values.len() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "k is larger than the number of values".to_string(),
            );
        }
        values.sort_by(|a, b| a.total_cmp(b));
        CalcResult::Number(values[k - 1])
    }

    pub(crate) fn fn_median(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match collect_numeric_values(self, args, cell) {
            Ok(v) => v,
            Err(err) => return err,
        };

        // Filter out NaN values to ensure proper sorting
        let mut values: Vec<f64> = values.into_iter().filter(|v| !v.is_nan()).collect();

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        // Sort values - NaN values have been filtered out, but use unwrap_or for safety
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = values.len();
        if len % 2 == 1 {
            CalcResult::Number(values[len / 2])
        } else {
            CalcResult::Number((values[len / 2 - 1] + values[len / 2]) / 2.0)
        }
    }

    pub(crate) fn fn_stdev_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match collect_numeric_values(self, args, cell) {
            Ok(v) => v,
            Err(err) => return err,
        };
        self.stdev(&values, true, cell)
    }

    pub(crate) fn fn_stdev_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match collect_numeric_values(self, args, cell) {
            Ok(v) => v,
            Err(err) => return err,
        };
        self.stdev(&values, false, cell)
    }

    fn stdev(&self, values: &[f64], sample: bool, cell: CellReferenceIndex) -> CalcResult {
        let n = values.len();
        if (sample && n < 2) || (!sample && n == 0) {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        let sum: f64 = values.iter().sum();
        let mean = sum / n as f64;
        let mut variance = 0.0;
        for v in values {
            variance += (*v - mean).powi(2);
        }
        if sample {
            variance /= n as f64 - 1.0;
        } else {
            variance /= n as f64;
        }
        CalcResult::Number(variance.sqrt())
    }

    fn get_a_values(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
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
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(v) => values.push(v),
                                CalcResult::Boolean(b) => {
                                    values.push(if b { 1.0 } else { 0.0 });
                                }
                                CalcResult::String(_) => values.push(0.0),
                                error @ CalcResult::Error { .. } => return Err(error),
                                CalcResult::Range { .. } => {
                                    return Err(CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    ))
                                }
                                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                                CalcResult::Array(_) => {
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
                CalcResult::Number(v) => values.push(v),
                CalcResult::Boolean(b) => values.push(if b { 1.0 } else { 0.0 }),
                CalcResult::String(s) => {
                    if let Node::ReferenceKind { .. } = arg {
                        values.push(0.0);
                    } else if let Ok(t) = s.parse::<f64>() {
                        values.push(t);
                    } else {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument cannot be cast into number".to_string(),
                        ));
                    }
                }
                error @ CalcResult::Error { .. } => return Err(error),
                CalcResult::EmptyCell | CalcResult::EmptyArg => {}
                CalcResult::Array(_) => {
                    return Err(CalcResult::Error {
                        error: Error::NIMPL,
                        origin: cell,
                        message: "Arrays not supported yet".to_string(),
                    })
                }
            }
        }
        Ok(values)
    }

    pub(crate) fn fn_stdeva(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_a_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let l = values.len();
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let sum: f64 = values.iter().sum();
        let mean = sum / l as f64;
        let mut var = 0.0;
        for v in &values {
            var += (v - mean).powi(2);
        }
        var /= l as f64 - 1.0;
        CalcResult::Number(var.sqrt())
    }

    pub(crate) fn fn_stdevpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_a_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let l = values.len();
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let sum: f64 = values.iter().sum();
        let mean = sum / l as f64;
        let mut var = 0.0;
        for v in &values {
            var += (v - mean).powi(2);
        }
        var /= l as f64;
        CalcResult::Number(var.sqrt())
    }

    pub(crate) fn fn_vara(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_a_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let l = values.len();
        if l < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let sum: f64 = values.iter().sum();
        let mean = sum / l as f64;
        let mut var = 0.0;
        for v in &values {
            var += (v - mean).powi(2);
        }
        var /= l as f64 - 1.0;
        CalcResult::Number(var)
    }

    pub(crate) fn fn_varpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_a_values(args, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let l = values.len();
        if l == 0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by 0".to_string(),
            };
        }
        let sum: f64 = values.iter().sum();
        let mean = sum / l as f64;
        let mut var = 0.0;
        for v in &values {
            var += (v - mean).powi(2);
        }
        var /= l as f64;
        CalcResult::Number(var)
    }

    pub(crate) fn fn_skew(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut values = Vec::new();
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Boolean(b) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        values.push(if b { 1.0 } else { 0.0 });
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
                    for row in left.row..=right.row {
                        for column in left.column..=right.column {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(v) => values.push(v),
                                CalcResult::Boolean(_)
                                | CalcResult::EmptyCell
                                | CalcResult::EmptyArg => {}
                                CalcResult::Range { .. } => {
                                    return CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    );
                                }
                                error @ CalcResult::Error { .. } => return error,
                                _ => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(s) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        if let Ok(t) = s.parse::<f64>() {
                            values.push(t);
                        } else {
                            return CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "Argument cannot be cast into number".to_string(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        let n = values.len();
        if n < 3 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }

        let mean = values.iter().sum::<f64>() / n as f64;
        let mut var = 0.0;
        for &v in &values {
            var += (v - mean).powi(2);
        }
        let std = (var / (n as f64 - 1.0)).sqrt();
        if std == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "division by 0".to_string());
        }
        let mut sum3 = 0.0;
        for &v in &values {
            sum3 += ((v - mean) / std).powi(3);
        }
        let result = n as f64 / ((n as f64 - 1.0) * (n as f64 - 2.0)) * sum3;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_skew_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut values = Vec::new();
        for arg in args {
            match self.evaluate_node_in_context(arg, cell) {
                CalcResult::Number(value) => values.push(value),
                CalcResult::Boolean(b) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        values.push(if b { 1.0 } else { 0.0 });
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
                    for row in left.row..=right.row {
                        for column in left.column..=right.column {
                            match self.evaluate_cell(CellReferenceIndex {
                                sheet: left.sheet,
                                row,
                                column,
                            }) {
                                CalcResult::Number(v) => values.push(v),
                                CalcResult::Boolean(_)
                                | CalcResult::EmptyCell
                                | CalcResult::EmptyArg => {}
                                CalcResult::Range { .. } => {
                                    return CalcResult::new_error(
                                        Error::ERROR,
                                        cell,
                                        "Unexpected Range".to_string(),
                                    );
                                }
                                error @ CalcResult::Error { .. } => return error,
                                _ => {}
                            }
                        }
                    }
                }
                error @ CalcResult::Error { .. } => return error,
                CalcResult::String(s) => {
                    if !matches!(arg, Node::ReferenceKind { .. }) {
                        if let Ok(t) = s.parse::<f64>() {
                            values.push(t);
                        } else {
                            return CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "Argument cannot be cast into number".to_string(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        let n = values.len();
        if n == 0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }

        let mean = values.iter().sum::<f64>() / n as f64;
        let mut var = 0.0;
        for &v in &values {
            var += (v - mean).powi(2);
        }
        let std = (var / n as f64).sqrt();
        if std == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "division by 0".to_string());
        }
        let mut sum3 = 0.0;
        for &v in &values {
            sum3 += ((v - mean) / std).powi(3);
        }
        let result = sum3 / n as f64;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_quartile(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_quartile_inc(args, cell)
    }

    pub(crate) fn fn_rank_eq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let range = match self.get_reference(&args[1], cell) {
            Ok(r) => r,
            Err(e) => return e,
        };
        let order = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f != 0.0,
                Err(e) => return e,
            }
        } else {
            false
        };

        let mut values = Vec::new();
        if range.left.sheet != range.right.sheet {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            );
        }
        for row in range.left.row..=range.right.row {
            for column in range.left.column..=range.right.column {
                match self.evaluate_cell(CellReferenceIndex {
                    sheet: range.left.sheet,
                    row,
                    column,
                }) {
                    CalcResult::Number(v) => values.push(v),
                    CalcResult::Error { .. } => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Invalid value".to_string(),
                        )
                    }
                    _ => {}
                }
            }
        }

        if values.is_empty() {
            return CalcResult::new_error(Error::NUM, cell, "Empty range".to_string());
        }

        let mut greater = 0;
        let mut found = false;
        for v in &values {
            if order {
                if *v < number {
                    greater += 1;
                } else if (*v - number).abs() < f64::EPSILON {
                    found = true;
                }
            } else if *v > number {
                greater += 1;
            } else if (*v - number).abs() < f64::EPSILON {
                found = true;
            }
        }

        if !found {
            return CalcResult::new_error(Error::NA, cell, "Number not found in range".to_string());
        }

        let rank = (greater + 1) as f64;
        CalcResult::Number(rank)
    }

    pub(crate) fn fn_rank_avg(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let number = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let range = match self.get_reference(&args[1], cell) {
            Ok(r) => r,
            Err(e) => return e,
        };
        let order = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f != 0.0,
                Err(e) => return e,
            }
        } else {
            false
        };

        if range.left.sheet != range.right.sheet {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            );
        }
        let mut values = Vec::new();
        for row in range.left.row..=range.right.row {
            for column in range.left.column..=range.right.column {
                match self.evaluate_cell(CellReferenceIndex {
                    sheet: range.left.sheet,
                    row,
                    column,
                }) {
                    CalcResult::Number(v) => values.push(v),
                    CalcResult::Error { .. } => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Invalid value".to_string(),
                        )
                    }
                    _ => {}
                }
            }
        }

        if values.is_empty() {
            return CalcResult::new_error(Error::NUM, cell, "Empty range".to_string());
        }

        let mut greater = 0;
        let mut equal = 0;
        for v in &values {
            if order {
                if *v < number {
                    greater += 1;
                } else if (*v - number).abs() < f64::EPSILON {
                    equal += 1;
                }
            } else if *v > number {
                greater += 1;
            } else if (*v - number).abs() < f64::EPSILON {
                equal += 1;
            }
        }

        if equal == 0 {
            return CalcResult::new_error(Error::NA, cell, "Number not found in range".to_string());
        }

        let rank = greater as f64 + ((equal as f64 + 1.0) / 2.0);
        CalcResult::Number(rank)
    }

    pub(crate) fn fn_rank(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_rank_eq(args, cell)
    }

    fn get_array_of_numbers_stat(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let mut values = Vec::new();
        let result = self.evaluate_node_in_context(arg, cell);
        match result {
            CalcResult::Number(value) => values.push(value),
            CalcResult::Boolean(b) => {
                if !matches!(arg, Node::ReferenceKind { .. }) {
                    values.push(if b { 1.0 } else { 0.0 });
                }
            }
            CalcResult::String(s) => {
                if !matches!(arg, Node::ReferenceKind { .. }) {
                    if let Ok(v) = s.parse::<f64>() {
                        values.push(v);
                    } else {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument cannot be cast into number".to_string(),
                        ));
                    }
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
                let column1 = left.column;
                let mut column2 = right.column;
                if row1 == 1 && row2 == LAST_ROW {
                    row2 = self
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
                if column1 == 1 && column2 == LAST_COLUMN {
                    column2 = self
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
                for row in row1..=row2 {
                    for column in column1..=column2 {
                        let v = self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row,
                            column,
                        });
                        match v {
                            CalcResult::Number(num) => values.push(num),
                            CalcResult::Error { .. } => return Err(v),
                            _ => {}
                        }
                    }
                }
            }
            CalcResult::Error { .. } => return Err(result),
            CalcResult::Array(_) => {
                return Err(CalcResult::Error {
                    error: Error::NIMPL,
                    origin: cell,
                    message: "Arrays not supported yet".to_string(),
                })
            }
            CalcResult::EmptyCell | CalcResult::EmptyArg => {}
        }
        Ok(values)
    }

    pub(crate) fn fn_percentile_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.percentile(values, k, true, cell)
    }

    pub(crate) fn fn_percentile_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.percentile(values, k, false, cell)
    }

    pub(crate) fn fn_percentrank_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let x = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let decimals = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(v) => v as i32,
                Err(e) => return e,
            }
        } else {
            3
        };
        self.percentrank(values, x, true, decimals, cell)
    }

    pub(crate) fn fn_percentrank_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let x = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let decimals = if args.len() == 3 {
            match self.get_number(&args[2], cell) {
                Ok(v) => v as i32,
                Err(e) => return e,
            }
        } else {
            3
        };
        self.percentrank(values, x, false, decimals, cell)
    }

    pub(crate) fn fn_quartile_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let quart = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.quartile(values, quart, true, cell)
    }

    pub(crate) fn fn_quartile_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers_stat(&args[0], cell) {
            Ok(v) => v,
            Err(_) => {
                return CalcResult::new_error(Error::VALUE, cell, "Invalid value".to_string())
            }
        };
        let quart = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.quartile(values, quart, false, cell)
    }

    pub(crate) fn fn_slope(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let ys = match collect_series(self, &args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let xs = match collect_series(self, &args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if ys.len() != xs.len() {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "Ranges have different lengths".to_string(),
            );
        }
        let mut pairs = Vec::new();
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut n = 0.0;
        for (y, x) in ys.iter().zip(xs.iter()) {
            if let (Some(yy), Some(xx)) = (y, x) {
                pairs.push((*yy, *xx));
                sum_x += xx;
                sum_y += yy;
                n += 1.0;
            }
        }
        if n == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (yy, xx) in pairs {
            let dx = xx - mean_x;
            let dy = yy - mean_y;
            numerator += dx * dy;
            denominator += dx * dx;
        }
        if denominator == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }
        CalcResult::Number(numerator / denominator)
    }

    pub(crate) fn fn_intercept(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let ys = match collect_series(self, &args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let xs = match collect_series(self, &args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if ys.len() != xs.len() {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "Ranges have different lengths".to_string(),
            );
        }
        let mut pairs = Vec::new();
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut n = 0.0;
        for (y, x) in ys.iter().zip(xs.iter()) {
            if let (Some(yy), Some(xx)) = (y, x) {
                pairs.push((*yy, *xx));
                sum_x += xx;
                sum_y += yy;
                n += 1.0;
            }
        }
        if n == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }
        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for (yy, xx) in pairs {
            let dx = xx - mean_x;
            let dy = yy - mean_y;
            numerator += dx * dy;
            denominator += dx * dx;
        }
        if denominator == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by Zero".to_string());
        }
        let slope = numerator / denominator;
        CalcResult::Number(mean_y - slope * mean_x)
    }

    // =============================================================================
    // PERCENTILE / PERCENTRANK / QUARTILE shared helpers
    // =============================================================================
    fn percentile(
        &self,
        mut values: Vec<f64>,
        k: f64,
        inclusive: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if values.is_empty() {
            return CalcResult::new_error(Error::NUM, cell, "Empty array".to_string());
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let n = values.len() as f64;
        if inclusive {
            if !(0.0..=1.0).contains(&k) {
                return CalcResult::new_error(Error::NUM, cell, "k out of range".to_string());
            }
            let pos = k * (n - 1.0) + 1.0;
            let m = pos.floor();
            let g = pos - m;
            let idx = (m as usize).saturating_sub(1);
            if idx >= values.len() - 1 {
                let last_value = match values.last() {
                    Some(&v) => v,
                    None => {
                        return CalcResult::new_error(Error::NUM, cell, "Empty array".to_string())
                    }
                };
                return CalcResult::Number(last_value);
            }
            let result = values[idx] + g * (values[idx + 1] - values[idx]);
            CalcResult::Number(result)
        } else {
            if k <= 0.0 || k >= 1.0 {
                return CalcResult::new_error(Error::NUM, cell, "k out of range".to_string());
            }
            let pos = k * (n + 1.0);
            if pos < 1.0 || pos > n {
                return CalcResult::new_error(Error::NUM, cell, "k out of range".to_string());
            }
            let m = pos.floor();
            let g = pos - m;
            let idx = (m as usize).saturating_sub(1);
            if idx >= values.len() - 1 {
                let last_value = match values.last() {
                    Some(&v) => v,
                    None => {
                        return CalcResult::new_error(Error::NUM, cell, "Empty array".to_string())
                    }
                };
                return CalcResult::Number(last_value);
            }
            let result = values[idx] + g * (values[idx + 1] - values[idx]);
            CalcResult::Number(result)
        }
    }

    fn percentrank(
        &self,
        mut values: Vec<f64>,
        x: f64,
        inclusive: bool,
        decimals: i32,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use std::cmp::Ordering;
        if values.is_empty() {
            return CalcResult::new_error(Error::NUM, cell, "Empty array".to_string());
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let n_f = values.len() as f64;
        let n_usize = values.len();
        let factor = 10f64.powi(decimals);

        if inclusive {
            if n_usize == 1 {
                if (x - values[0]).abs() <= f64::EPSILON {
                    return CalcResult::Number((0.5 * factor).round() / factor);
                }
                return CalcResult::new_error(
                    Error::NA,
                    cell,
                    "Value not found in single element array".to_string(),
                );
            }

            if x < values[0] {
                return CalcResult::Number(0.0);
            }
            if x > values[n_usize - 1] {
                return CalcResult::Number(1.0);
            }
            let mut idx = 0usize;
            while idx < n_usize && values[idx] < x {
                idx += 1;
            }
            if idx >= n_usize {
                return CalcResult::Number(1.0);
            }
            let rank = if (x - values[idx]).abs() <= f64::EPSILON {
                idx as f64
            } else if idx == 0 {
                0.0
            } else {
                let lower = values[idx - 1];
                let upper = values[idx];
                (idx as f64 - 1.0) + (x - lower) / (upper - lower)
            };
            let mut result = rank / (n_f - 1.0);
            result = (result * factor).round() / factor;
            CalcResult::Number(result)
        } else {
            if x <= values[0] || x >= values[n_usize - 1] {
                return CalcResult::new_error(Error::NUM, cell, "x out of range".to_string());
            }
            let mut idx = 0usize;
            while idx < n_usize && values[idx] < x {
                idx += 1;
            }
            let rank = if (x - values[idx]).abs() > f64::EPSILON {
                let lower = values[idx - 1];
                let upper = values[idx];
                idx as f64 + (x - lower) / (upper - lower)
            } else {
                (idx + 1) as f64
            };
            let mut result = rank / (n_f + 1.0);
            result = (result * factor).round() / factor;
            CalcResult::Number(result)
        }
    }

    fn quartile(
        &self,
        mut values: Vec<f64>,
        quart: f64,
        inclusive: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        use std::cmp::Ordering;
        if quart.fract() != 0.0 {
            return CalcResult::new_error(Error::NUM, cell, "Invalid quart".to_string());
        }
        let q_int = quart as i32;
        if inclusive {
            if !(0..=4).contains(&q_int) {
                return CalcResult::new_error(Error::NUM, cell, "Invalid quart".to_string());
            }
        } else if !(1..=3).contains(&q_int) {
            return CalcResult::new_error(Error::NUM, cell, "Invalid quart".to_string());
        }

        if values.is_empty() {
            return CalcResult::new_error(Error::NUM, cell, "Empty array".to_string());
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let n = values.len() as f64;
        let k = quart / 4.0;

        if inclusive {
            let index = k * (n - 1.0);
            let i = index.floor() as usize;
            let f = index - (i as f64);
            if i + 1 >= values.len() {
                return CalcResult::Number(values[i]);
            }
            CalcResult::Number(values[i] + f * (values[i + 1] - values[i]))
        } else {
            let r = k * (n + 1.0);
            if r <= 1.0 || r >= n {
                return CalcResult::new_error(Error::NUM, cell, "Invalid quart".to_string());
            }
            let i = r.floor() as usize;
            let f = r - (i as f64);
            CalcResult::Number(values[i - 1] + f * (values[i] - values[i - 1]))
        }
    }
}
