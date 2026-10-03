//! A function name used on its own as a LAMBDA (`=BYROW(A1:B2,SUM)`,
//! `=MAP({1,4},SQRT)`).

use crate::{
    calc_result::CalcResult,
    expressions::parser::{NamedVariable, Node},
    functions::Function,
    language::get_default_language,
    model::Model,
};

impl<'a> Model<'a> {
    /// A function name used without brackets (`SUM`, or `_xleta.SUM` as
    /// Excel stores it) is a LAMBDA that calls that function. PERCENTOF takes
    /// two values, every other function one.
    pub(crate) fn eta_lambda(&mut self, name: &str) -> Option<CalcResult> {
        let bare = name
            .trim_start_matches("_xleta.")
            .trim_start_matches("_xlfn.");
        let kind = self
            .language
            .functions
            .lookup(bare)
            .or_else(|| get_default_language().functions.lookup(bare))?;
        if matches!(kind, Function::Lambda | Function::Let) {
            return None;
        }
        let count = if matches!(kind, Function::Percentof) {
            2
        } else {
            1
        };
        let parameters: Vec<NamedVariable> = (1..=count)
            .map(|i| NamedVariable {
                name: format!("_eta{i}"),
                id: None,
                is_optional: false,
            })
            .collect();
        let body = Node::FunctionKind {
            kind,
            args: parameters
                .iter()
                .map(|p| Node::NamedVariableKind {
                    name: p.name.clone(),
                    id: None,
                })
                .collect(),
        };
        let id = self.get_next_lambda_id();
        self.lambdas.insert(id, (parameters, body));
        Some(CalcResult::Lambda(id))
    }
}
