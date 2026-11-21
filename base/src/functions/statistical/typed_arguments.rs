use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl<'a> Model<'a> {
    /// A value typed directly into the argument list of STDEV, STDEV.P, VAR, VAR.P or SUMSQ, as Excel
    /// counts it: a boolean is 1 or 0 and text that holds a number is that number; other text is
    /// #VALUE!. A value read from a cell (`A1`, a name, an intersection) is not typed, so it is left
    /// out, as are empty arguments. Excel: `STDEV("1","2")` is 0.7071…, `VAR(TRUE,FALSE,TRUE)` is
    /// 0.333… and `SUMSQ("2","3",4)` is 29 (Apache POI's FormulaEvalTestData_Copy.xlsx,
    /// EverythingTests rows 1288, 1448 and 1332, saved by Excel).
    ///
    /// * `arg` - the argument as written.
    /// * `value` - what it evaluated to.
    /// * `cell` - the formula's cell.
    ///
    /// Returns the number to count, `Ok(None)` for a value that isn't counted, or the #VALUE! error.
    pub(crate) fn typed_number(
        &self,
        arg: &Node,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Option<f64>, CalcResult> {
        if matches!(
            arg,
            Node::ReferenceKind { .. }
                | Node::RangeKind { .. }
                | Node::OpRangeKind { .. }
                | Node::DefinedNameKind { .. }
                | Node::TableNameKind(_)
                | Node::NamedVariableKind { .. }
                | Node::ImplicitIntersection { .. }
        ) {
            return Ok(None);
        }
        match value {
            CalcResult::Boolean(b) => Ok(Some(if b { 1.0 } else { 0.0 })),
            CalcResult::String(s) => match self.cast_number(&s) {
                Some(f) => Ok(Some(f)),
                None => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    format!("\"{s}\" is not a number"),
                )),
            },
            _ => Ok(None),
        }
    }
}
