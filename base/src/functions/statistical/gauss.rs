use super::normal::standard_normal_cdf;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl<'a> Model<'a> {
    pub(crate) fn fn_gauss(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let z = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let result = standard_normal_cdf(z) - 0.5;

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid result for GAUSS".to_string(),
            };
        }

        CalcResult::Number(result)
    }
}
