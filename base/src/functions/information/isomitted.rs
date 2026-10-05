use crate::types::Position;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    model::Model,
};

impl<'a, A: Position> Model<'a, A> {
    pub(crate) fn fn_isomitted(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.evaluate_node_in_context(&args[0], cell) {
            CalcResult::EmptyArg => CalcResult::Boolean(true),
            _ => CalcResult::Boolean(false),
        }
    }
}
