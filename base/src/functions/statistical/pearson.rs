use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::chisq::is_same_shape_or_1d;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // PEARSON(array1, array2)
    pub(crate) fn fn_pearson(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let left_arg = self.evaluate_node_in_context(&args[0], cell);
        let right_arg = self.evaluate_node_in_context(&args[1], cell);

        let (values_left, values_right) = match (left_arg, right_arg) {
            (
                CalcResult::Range {
                    left: l1,
                    right: r1,
                },
                CalcResult::Range {
                    left: l2,
                    right: r2,
                },
            ) => {
                if l1.sheet != l2.sheet {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    );
                }

                let rows1 = r1.row - l1.row + 1;
                let cols1 = r1.column - l1.column + 1;
                let rows2 = r2.row - l2.row + 1;
                let cols2 = r2.column - l2.column + 1;

                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges must be of the same shape".to_string(),
                    );
                }

                let values_left = match self.values_from_range(l1, r1) {
                    Err(error) => return error,
                    Ok(v) => v,
                };
                let values_right = match self.values_from_range(l2, r2) {
                    Err(error) => return error,
                    Ok(v) => v,
                };

                (values_left, values_right)
            }
            (
                CalcResult::Array(left),
                CalcResult::Range {
                    left: l2,
                    right: r2,
                },
            ) => {
                let rows2 = r2.row - l2.row + 1;
                let cols2 = r2.column - l2.column + 1;

                let rows1 = left.len() as i32;
                let cols1 = if rows1 > 0 { left[0].len() as i32 } else { 0 };

                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Array and range must be of the same shape".to_string(),
                    );
                }

                let values_left = match self.values_from_array(left) {
                    Err(error) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in first array: {:?}", error),
                        );
                    }
                    Ok(v) => v,
                };
                let values_right = match self.values_from_range(l2, r2) {
                    Err(error) => return error,
                    Ok(v) => v,
                };

                (values_left, values_right)
            }
            (
                CalcResult::Range {
                    left: l1,
                    right: r1,
                },
                CalcResult::Array(right),
            ) => {
                let rows1 = r1.row - l1.row + 1;
                let cols1 = r1.column - l1.column + 1;

                let rows2 = right.len() as i32;
                let cols2 = if rows2 > 0 { right[0].len() as i32 } else { 0 };

                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Range and array must be of the same shape".to_string(),
                    );
                }

                let values_left = match self.values_from_range(l1, r1) {
                    Err(error) => return error,
                    Ok(v) => v,
                };
                let values_right = match self.values_from_array(right) {
                    Err(error) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in second array: {:?}", error),
                        );
                    }
                    Ok(v) => v,
                };

                (values_left, values_right)
            }
            (CalcResult::Array(left), CalcResult::Array(right)) => {
                let rows1 = left.len() as i32;
                let rows2 = right.len() as i32;
                let cols1 = if rows1 > 0 { left[0].len() as i32 } else { 0 };
                let cols2 = if rows2 > 0 { right[0].len() as i32 } else { 0 };

                if !is_same_shape_or_1d(rows1, cols1, rows2, cols2) {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Arrays must be of the same shape".to_string(),
                    );
                }

                let values_left = match self.values_from_array(left) {
                    Err(error) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in first array: {:?}", error),
                        );
                    }
                    Ok(v) => v,
                };
                let values_right = match self.values_from_array(right) {
                    Err(error) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            format!("Error in second array: {:?}", error),
                        );
                    }
                    Ok(v) => v,
                };

                (values_left, values_right)
            }
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Both arguments must be ranges or arrays".to_string(),
                );
            }
        };

        // Flatten into (x, y) pairs, skipping non-numeric entries (None)
        let mut n: f64 = 0.0;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        let mut sum_xy = 0.0;

        let len = values_left.len().min(values_right.len());
        for i in 0..len {
            match (values_left[i], values_right[i]) {
                (Some(x), Some(y)) => {
                    n += 1.0;
                    sum_x += x;
                    sum_y += y;
                    sum_x2 += x * x;
                    sum_y2 += y * y;
                    sum_xy += x * y;
                }
                _ => {
                    // Ignore pairs where at least one side is non-numeric
                }
            }
        }

        if n < 2.0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "PEARSON requires at least two numeric pairs".to_string(),
            );
        }

        // Pearson correlation:
        // r = [ n*Σxy - (Σx)(Σy) ] / sqrt( [n*Σx² - (Σx)²] [n*Σy² - (Σy)²] )
        let num = n * sum_xy - sum_x * sum_y;
        let denom_x = n * sum_x2 - sum_x * sum_x;
        let denom_y = n * sum_y2 - sum_y * sum_y;

        if denom_x.abs() < 1e-15 || denom_y.abs() < 1e-15 {
            // Zero variance in at least one series
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "PEARSON cannot be computed when one series has zero variance".to_string(),
            );
        }

        let denom = (denom_x * denom_y).sqrt();
        let r = num / denom;

        CalcResult::Number(r)
    }
}
