#![allow(clippy::unwrap_used, clippy::panic)]

use std::collections::HashMap;

use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::{DefinedNameS, Node, Parser};
use crate::expressions::types::CellReferenceRC;

// Sheets: 0 Tests, 1 Data, 2 'Q1 Sales'
// Names: Scope_Test local to Data and a workbook Scope_Test, a workbook TaxRate
fn parser<'a>() -> Parser<'a> {
    let worksheets = vec![
        "Tests".to_string(),
        "Data".to_string(),
        "Q1 Sales".to_string(),
    ];
    let defined_names: Vec<DefinedNameS> = vec![
        ("Scope_Test".to_string(), Some(1), "Data!$C$2".to_string()),
        ("Scope_Test".to_string(), None, "Tests!$B$9".to_string()),
        ("TaxRate".to_string(), None, "Tests!$B$2".to_string()),
    ];
    new_parser(worksheets, defined_names, HashMap::new())
}

fn tests_a1() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Tests".to_string(),
        row: 1,
        column: 1,
    }
}

fn scope_and_sheet(node: &Node) -> (Option<u32>, Option<(u32, String)>) {
    match node {
        Node::DefinedNameKind { scope, sheet, .. } => (*scope, sheet.clone()),
        _ => panic!("not a defined name: {node:?}"),
    }
}

#[test]
fn bare_name_uses_the_context_sheet() {
    let mut parser = parser();
    // From Tests there is no local name: the workbook one
    let t = parser.parse("Scope_Test", &tests_a1());
    assert_eq!(scope_and_sheet(&t), (None, None));
    // From Data the local one
    let data_a1 = CellReferenceRC {
        sheet: "Data".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse("Scope_Test", &data_a1);
    assert_eq!(scope_and_sheet(&t), (Some(1), None));
    assert_eq!(to_english_localized_string(&t, &data_a1), "Scope_Test");
}

#[test]
fn qualified_name_uses_the_local_name() {
    let mut parser = parser();
    let t = parser.parse("Data!Scope_Test", &tests_a1());
    assert_eq!(
        scope_and_sheet(&t),
        (Some(1), Some((1, "Data".to_string())))
    );
    // The qualifier survives a round trip
    assert_eq!(
        to_english_localized_string(&t, &tests_a1()),
        "Data!Scope_Test"
    );
    assert_eq!(to_rc_format(&t), "Data!Scope_Test");

    let t = parser.parse("SUM(Data!Scope_Test)+1", &tests_a1());
    assert_eq!(
        to_english_localized_string(&t, &tests_a1()),
        "SUM(Data!Scope_Test)+1"
    );
}

#[test]
fn qualified_name_falls_back_to_the_workbook_name() {
    let mut parser = parser();
    let t = parser.parse("Data!TaxRate", &tests_a1());
    assert_eq!(scope_and_sheet(&t), (None, Some((1, "Data".to_string()))));
    assert_eq!(to_english_localized_string(&t, &tests_a1()), "Data!TaxRate");

    // A sheet with spaces is quoted
    let t = parser.parse("'Q1 Sales'!TaxRate", &tests_a1());
    assert_eq!(
        scope_and_sheet(&t),
        (None, Some((2, "Q1 Sales".to_string())))
    );
    assert_eq!(
        to_english_localized_string(&t, &tests_a1()),
        "'Q1 Sales'!TaxRate"
    );
}

#[test]
fn qualified_unknown_name_keeps_its_text() {
    let mut parser = parser();
    let t = parser.parse("Data!Nope", &tests_a1());
    match &t {
        Node::DefinedNameKind {
            name,
            scope,
            formula,
            sheet,
        } => {
            assert_eq!(name, "Nope");
            assert_eq!(*scope, Some(1));
            assert_eq!(formula, "");
            assert_eq!(*sheet, Some((1, "Data".to_string())));
        }
        _ => panic!("not a defined name: {t:?}"),
    }
    assert_eq!(to_english_localized_string(&t, &tests_a1()), "Data!Nope");
}

#[test]
fn qualified_name_unknown_sheet_is_an_error() {
    let mut parser = parser();
    let t = parser.parse("Nope!TaxRate", &tests_a1());
    assert!(matches!(t, Node::ParseErrorKind { .. }));
}
