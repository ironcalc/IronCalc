use statrs::distribution::{Continuous, ContinuousCDF, StudentsT};

use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    // T.DIST(x, deg_freedom, cumulative)
    pub(crate) fn fn_t_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

    // T.DIST.2T(x, deg_freedom)
    pub(crate) fn fn_t_dist_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

        let upper_tail = 1.0 - dist.cdf(x);
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

    // T.DIST.RT(x, deg_freedom)
    pub(crate) fn fn_t_dist_rt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

    // T.INV(probability, deg_freedom)
    pub(crate) fn fn_t_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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

    // T.INV.2T(probability, deg_freedom)
    pub(crate) fn fn_t_inv_2t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
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
}
