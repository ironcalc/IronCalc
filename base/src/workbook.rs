use std::vec::Vec;

use crate::{
    expressions::{
        parser::{new_parser_english, stringify::to_english_string, DefinedNameS, Node},
        types::CellReferenceRC,
    },
    language::get_default_language,
    types::*,
};

/// Marks a function that is not built-in, in a formula and in the name of a
/// named function. It is what Excel writes in front of a function it does
/// not know.
pub(crate) const USER_FUNCTION_PREFIX: &str = "_xludf.";

/// The name of a named function without the `_xludf.` prefix. The prefix is
/// matched ignoring case.
pub(crate) fn strip_user_function_prefix(name: &str) -> &str {
    let length = USER_FUNCTION_PREFIX.len();
    match name.get(..length) {
        Some(prefix) if prefix.eq_ignore_ascii_case(USER_FUNCTION_PREFIX) => &name[length..],
        _ => name,
    }
}

impl NamedFunction {
    /// The name of the function as it is written in English, that is how it
    /// is written in an xlsx file: with the `_xludf.` prefix if a built-in
    /// function is called the same (`_xludf.NOW`).
    pub fn english_name(&self) -> String {
        if get_default_language()
            .functions
            .lookup(&self.name)
            .is_some()
        {
            format!("{USER_FUNCTION_PREFIX}{}", self.name)
        } else {
            self.name.clone()
        }
    }
}

impl Workbook {
    /// Moves every defined name whose formula is a `LAMBDA` definition to the
    /// named functions, keeping its scope and its comment as the description.
    /// That is how Excel stores named functions in a file. Formulas are
    /// expected in the internal (English) form. A function called like a
    /// built-in one has the `_xludf.` prefix in the file, that is not part
    /// of its name.
    pub fn lambda_defined_names_to_named_functions(&mut self) {
        let mut parser = new_parser_english(
            self.get_worksheet_names(),
            self.get_defined_names_with_scope(),
            self.tables.clone(),
        );
        let context = CellReferenceRC {
            sheet: self
                .worksheets
                .first()
                .map(|ws| ws.get_name())
                .unwrap_or_else(|| "Sheet1".to_string()),
            row: 1,
            column: 1,
        };
        let mut defined_names = Vec::new();
        for defined_name in self.defined_names.drain(..) {
            let formula = defined_name.formula.trim();
            let body = formula.strip_prefix('=').unwrap_or(formula);
            match parser.parse(body, &context) {
                Node::LambdaDefKind { parameters, body } => {
                    self.named_functions.push(NamedFunction {
                        name: strip_user_function_prefix(&defined_name.name).to_string(),
                        sheet_id: defined_name.sheet_id,
                        hidden: defined_name.hidden,
                        description: defined_name.comment,
                        parameters: parameters
                            .iter()
                            .map(|p| NamedFunctionParameter::new(&p.name).optional(p.is_optional))
                            .collect(),
                        formula: to_english_string(&body, &context),
                    });
                }
                _ => defined_names.push(defined_name),
            }
        }
        self.defined_names = defined_names;
    }

    pub fn get_worksheet_names(&self) -> Vec<String> {
        self.worksheets
            .iter()
            .map(|worksheet| worksheet.get_name())
            .collect()
    }
    pub fn get_worksheet_ids(&self) -> Vec<u32> {
        self.worksheets
            .iter()
            .map(|worksheet| worksheet.get_sheet_id())
            .collect()
    }

    pub fn worksheet(&self, worksheet_index: u32) -> Result<&Worksheet, String> {
        self.worksheets
            .get(worksheet_index as usize)
            .ok_or_else(|| "Invalid sheet index".to_string())
    }

    pub fn worksheet_mut(&mut self, worksheet_index: u32) -> Result<&mut Worksheet, String> {
        self.worksheets
            .get_mut(worksheet_index as usize)
            .ok_or_else(|| "Invalid sheet index".to_string())
    }

    /// Returns the list of named functions in the workbook with their scope,
    /// as a sheet index
    pub fn get_named_functions_with_scope(&self) -> Vec<(String, Option<u32>)> {
        self.named_functions
            .iter()
            .map(|nf| {
                let index = nf.sheet_id.and_then(|sheet_id| {
                    self.worksheets
                        .iter()
                        .position(|ws| ws.sheet_id == sheet_id)
                });
                (nf.name.clone(), index.map(|pos| pos as u32))
            })
            .collect()
    }

    /// Returns the a list of defined names in the workbook with their scope
    pub fn get_defined_names_with_scope(&self) -> Vec<DefinedNameS> {
        let sheet_id_index: Vec<u32> = self.worksheets.iter().map(|s| s.sheet_id).collect();

        let defined_names = self
            .defined_names
            .iter()
            .map(|dn| {
                let index = dn
                    .sheet_id
                    .and_then(|sheet_id| {
                        // returns an Option<usize>
                        sheet_id_index.iter().position(|&x| x == sheet_id)
                    })
                    // convert Option<usize> to Option<u32>
                    .map(|pos| pos as u32);

                (dn.name.clone(), index, dn.formula.clone())
            })
            .collect::<Vec<_>>();
        defined_names
    }
}
