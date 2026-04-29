use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

use super::r#let::assign_variable_ids;

impl<'a> Model<'a> {
    /// Evaluates the body of a named lambda with the given call-site arguments.
    /// Handles parameter binding, body patching, and cleanup.
    pub(crate) fn call_lambda(
        &mut self,
        lambda_result: CalcResult,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (param_names, body) = match lambda_result {
            CalcResult::Lambda { id, .. } => match self.lambdas.get(&id) {
                Some(l) => l.clone(),
                None => {
                    return CalcResult::new_error(
                        Error::NAME,
                        cell,
                        "Lambda not found in store".to_string(),
                    )
                }
            },
            other if other.is_error() => return other,
            _ => return CalcResult::new_error(Error::VALUE, cell, "Expected a LAMBDA".to_string()),
        };

        if param_names.len() != call_args.len() {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                format!(
                    "LAMBDA expected {} argument(s), got {}",
                    param_names.len(),
                    call_args.len()
                ),
            );
        }

        // Clone the body so we can patch fresh variable ids into it without mutating the store.
        let mut patched_body = body;
        let mut bound_ids: Vec<usize> = Vec::with_capacity(param_names.len());

        for param_name in &param_names {
            let raw_id = self.get_next_variable_id();
            assign_variable_ids(&mut patched_body, param_name, raw_id as u32);
            bound_ids.push(raw_id);
        }

        // Eagerly evaluate each argument and push it onto the variable stack.
        for (raw_id, arg_node) in bound_ids.iter().zip(call_args.iter()) {
            let val = self.evaluate_node_in_context(arg_node, cell);
            self.variable_stack.insert(*raw_id, val);
        }

        let result = self.evaluate_node_in_context(&patched_body, cell);

        for raw_id in bound_ids {
            self.variable_stack.remove(&raw_id);
        }

        result
    }

    pub(crate) fn fn_lambda(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // This path is reached when LAMBDA was stored as FunctionKind::Lambda (e.g. imported xlsx).
        // Treat it identically to LambdaDefKind: register and return a Lambda value.
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let body = args.last().unwrap().clone();
        let param_names: Vec<String> = args[..args.len() - 1]
            .iter()
            .filter_map(|n| {
                if let Node::NamedVariableKind { name, .. } = n {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        let id = self.get_next_lambda_id();
        self.lambdas.insert(id, (param_names, body));
        CalcResult::Lambda { name: None, id }
    }
}
