use std::collections::HashMap;

use crate::{
    expressions::{
        parser::{DefinedNameS, Node, Parser},
        types::CellReferenceRC,
    },
    language::Language,
    locale::Locale,
    types::Table,
};

use crate::expressions::parser::stringify::to_localized_string;

pub fn to_english_localized_string(t: &Node, cell_reference: &CellReferenceRC) -> String {
    let locale = Locale::default();
    let language = Language::default();
    to_localized_string(t, cell_reference, &locale, &language)
}

pub fn new_parser(
    worksheets: Vec<String>,
    defined_names: Vec<DefinedNameS>,
    tables: HashMap<String, Table>,
) -> Parser {
    let locale = Locale::default();
    let language = Language::default();
    Parser::new(worksheets, defined_names, tables, &locale, &language)
}
