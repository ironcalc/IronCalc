use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // PEARSON(array1, array2)
    pub(crate) fn fn_pearson(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let (_, _, values_left, values_right) = match self.fn_get_two_matrices(args, cell) {
            Ok(result) => result,
            Err(e) => return e,
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

        CalcResult::Number(num / denom)
    }
}
