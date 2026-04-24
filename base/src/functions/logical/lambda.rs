use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    model::Model,
};

impl<'a> Model<'a> {
    pub(crate) fn fn_lambda(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // LAMBDA([parameter1, parameter2, ...], calculation)
        // Requires at least 1 argument (the body); meaningful use requires >= 2.
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        todo!()
    }
}
