use std::cmp::Ordering;

use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl<'a> Model<'a> {
    /// Helper to collect numeric values for QUARTILE functions
    fn collect_quartile_values(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let values = match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Array(array) => match self.values_from_array(array) {
                Ok(v) => v,
                Err(e) => {
                    return Err(CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: format!("Unsupported array argument: {}", e),
                    })
                }
            },
            CalcResult::Range { left, right } => self.values_from_range(left, right)?,
            CalcResult::Number(value) => vec![Some(value)],
            _ => {
                return Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Unsupported argument type".to_string(),
                })
            }
        };

        let numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        Ok(numeric_values)
    }

    /// QUARTILE(array, quart)
    /// Compatibility function - same as QUARTILE.INC
    pub(crate) fn fn_quartile(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fn_quartile_inc(args, cell)
    }

    /// QUARTILE.INC(array, quart)
    /// Returns the quartile of a data set (inclusive method)
    pub(crate) fn fn_quartile_inc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.collect_quartile_values(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let quart = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.quartile_impl(values, quart, true, cell)
    }

    /// QUARTILE.EXC(array, quart)
    /// Returns the quartile of a data set (exclusive method)
    pub(crate) fn fn_quartile_exc(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.collect_quartile_values(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let quart = match self.get_number(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        self.quartile_impl(values, quart, false, cell)
    }

    /// Internal implementation for QUARTILE functions
    fn quartile_impl(
        &self,
        mut values: Vec<f64>,
        quart: f64,
        inclusive: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
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
