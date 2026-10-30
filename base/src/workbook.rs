use std::vec::Vec;

use crate::{
    expressions::{
        parser::{new_parser_english, stringify::to_english_string, DefinedNameS, Node},
        types::CellReferenceRC,
    },
    types::*,
};

impl Workbook {
    /// Moves every defined name whose formula is a `LAMBDA` definition to the
    /// named functions, keeping its scope and its comment as the description.
    /// That is how Excel stores named functions in a file. Formulas are
    /// expected in the internal (English) form.
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
                        name: defined_name.name,
                        sheet_id: defined_name.sheet_id,
                        description: defined_name.comment,
                        parameters: parameters
                            .iter()
                            .map(|p| NamedFunctionParameter {
                                name: p.name.clone(),
                                description: String::new(),
                                optional: p.is_optional,
                            })
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
