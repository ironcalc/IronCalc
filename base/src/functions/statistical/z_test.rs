use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    pub(crate) fn fn_z_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        todo!()
    }
}
