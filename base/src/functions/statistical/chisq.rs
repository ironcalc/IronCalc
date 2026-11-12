use statrs::distribution::{ChiSquared, Continuous, ContinuousCDF};

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
const MAX_DEGREES_OF_FREEDOM: f64 = 10_000_000_000.0;

impl<'a> Model<'a> {
    // CHISQ.DIST(x, deg_freedom, cumulative)
    pub(crate) fn fn_chisq_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        let cumulative = match self.get_boolean(&args[2], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        if x < 0.0 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "x must be >= 0 in CHISQ.DIST".to_string(),
            );
        }
        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.DIST".to_string(),
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

    // CHISQ.DIST.RT(x, deg_freedom)
    pub(crate) fn fn_chisq_dist_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number(&args[1], cell) {
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

        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.DIST.RT".to_string(),
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

    // CHISQ.INV(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc(),
            Err(e) => return e,
        };

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV".to_string(),
            );
        }

        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.INV".to_string(),
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

    // CHISQ.INV.RT(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df_raw = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let df = df_raw.trunc();

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "probability must be in [0,1] in CHISQ.INV.RT".to_string(),
            );
        }
        // if degrees of freedom < 1 or > 10^10 → #NUM!
        if !(1.0..=MAX_DEGREES_OF_FREEDOM).contains(&df) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "degrees of freedom must be in [1, 10^10] in CHISQ.INV.RT".to_string(),
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

    pub(crate) fn values_from_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let mut values = Vec::new();
        for row_offset in 0..=(right.row - left.row) {
            for col_offset in 0..=(right.column - left.column) {
                let cell_ref = CellReferenceIndex {
                    sheet: left.sheet,
                    row: left.row + row_offset,
                    column: left.column + col_offset,
                };
                let cell_value = self.evaluate_cell(cell_ref);
                match cell_value {
                    CalcResult::Number(v) => {
                        values.push(Some(v));
                    }
                    error @ CalcResult::Error { .. } => return Err(error),
                    _ => {
                        values.push(None);
                    }
                }
            }
        }
        Ok(values)
    }

    /// `values_from_range` for a function that skips empty cells (LARGE,
    /// SMALL): a whole column or row is read only as far as the sheet's used
    /// area, since the cells past it are all empty (`clip_to_used_area`).
    pub(crate) fn values_from_used_range(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<f64>>, CalcResult> {
        let (row, column) = self
            .clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
            .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))?;
        let right = CellReferenceIndex {
            sheet: left.sheet,
            row,
            column,
        };
        self.values_from_range(left, right)
    }

    /// The bottom-right corners to read two ranges of the same shape to, for a
    /// function that pairs their cells position by position and gives a pair
    /// of empty cells no weight. A whole column or row is read only as far as
    /// the used area of its sheet, and both ranges as far as the longer of the
    /// two, so the pairs stay aligned and only pairs of two empty cells are
    /// left out. Ranges of different shapes are read whole.
    pub(crate) fn paired_used_corners(
        &mut self,
        (left1, right1): (CellReferenceIndex, CellReferenceIndex),
        (left2, right2): (CellReferenceIndex, CellReferenceIndex),
        cell: CellReferenceIndex,
    ) -> Result<(CellReferenceIndex, CellReferenceIndex), CalcResult> {
        if right1.row - left1.row != right2.row - left2.row
            || right1.column - left1.column != right2.column - left2.column
        {
            return Ok((right1, right2));
        }
        let mut clip = |left: CellReferenceIndex, right: CellReferenceIndex| {
            self.clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
                .map_err(|message| CalcResult::new_error(Error::ERROR, cell, message))
        };
        let (row1, column1) = clip(left1, right1)?;
        let (row2, column2) = clip(left2, right2)?;
        let height = (row1 - left1.row).max(row2 - left2.row);
        let width = (column1 - left1.column).max(column2 - left2.column);
        let corner = |left: CellReferenceIndex| CellReferenceIndex {
            sheet: left.sheet,
            row: left.row + height,
            column: left.column + width,
        };
        Ok((corner(left1), corner(left2)))
    }

    pub(crate) fn values_from_array(
        &mut self,
        array: Vec<Vec<ArrayNode>>,
    ) -> Result<Vec<Option<f64>>, Error> {
        let mut values = Vec::new();
        for row in array {
            for item in row {
                match item {
                    ArrayNode::Number(f) => {
                        values.push(Some(f));
                    }
                    ArrayNode::Error(error) => {
                        return Err(error);
                    }
                    _ => {
                        values.push(None);
                    }
                }
            }
        }
        Ok(values)
    }

    // CHISQ.TEST(actual_range, expected_range)
    pub(crate) fn fn_chisq_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Every pair counts here, a pair of empty cells too (it is taken as 1
        // observed against 1 expected), so whole columns are read whole.
        let (width, height, values_left, values_right) =
            match self.fn_get_two_matrices_unclipped(args, cell) {
                Ok(v) => v,
                Err(r) => return r,
            };

        let mut values = Vec::with_capacity(values_left.len());

        // Now we have:
        // - values: flattened (observed, expected)
        // - width, height: shape
        for i in 0..values_left.len() {
            match (values_left[i], values_right[i]) {
                (Some(v1), Some(v2)) => {
                    values.push((v1, v2));
                }
                _ => {
                    values.push((1.0, 1.0));
                }
            }
        }
        if width == 0 || height == 0 || values.len() < 2 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "CHISQ.TEST requires at least two data points".to_string(),
            );
        }

        let mut chi2 = 0.0;
        for (obs, exp) in &values {
            if *obs < 0.0 || *exp < 0.0 {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Negative value in CHISQ.TEST data".to_string(),
                );
            }
            if *exp == 0.0 {
                return CalcResult::new_error(
                    Error::DIV,
                    cell,
                    "Zero expected value in CHISQ.TEST".to_string(),
                );
            }
            let diff = obs - exp;
            chi2 += (diff * diff) / exp;
        }

        if chi2 < 0.0 && chi2 > -1e-12 {
            chi2 = 0.0;
        }

        let total = width * height;
        if total <= 1 {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "CHISQ.TEST degrees of freedom is zero".to_string(),
            );
        }

        let df = if width > 1 && height > 1 {
            (width - 1) * (height - 1)
        } else {
            total - 1
        };

        let dist = match ChiSquared::new(df as f64) {
            Ok(d) => d,
            Err(_) => {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "Invalid degrees of freedom in CHISQ.TEST".to_string(),
                );
            }
        };

        let mut p = 1.0 - dist.cdf(chi2);

        // clamp tiny fp noise
        if p < 0.0 && p > -1e-15 {
            p = 0.0;
        }
        if p > 1.0 && p < 1.0 + 1e-15 {
            p = 1.0;
        }

        CalcResult::Number(p)
    }
}
