use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::spill_functions::transpose_array,
    model::Model,
};

impl<'a> Model<'a> {
    // ── TRANSPOSE ─────────────────────────────────────────────────────────────

    /// `=TRANSPOSE(array)`
    ///
    /// Returns the transpose of the array: rows become columns and vice versa.
    pub(crate) fn fn_transpose(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        CalcResult::Array(transpose_array(data))
    }
}
