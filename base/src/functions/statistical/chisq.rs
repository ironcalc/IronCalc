use statrs::distribution::{Beta, ChiSquared, Continuous, ContinuousCDF};

use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // CHISQ.DIST(x, deg_freedom, cumulative)
    pub(crate) fn fn_chisq_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

    // CHISQ.DIST.RT(x, deg_freedom)
    pub(crate) fn fn_chisq_dist_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
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

    // CHISQ.INV(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
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

    // CHISQ.INV.RT(probability, deg_freedom)
    pub(crate) fn fn_chisq_inv_rt(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
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

        // if probability < 0 or > 1 → #NUM!
        if !(0.0..=1.0).contains(&p) {
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
}
