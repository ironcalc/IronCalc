#![allow(clippy::panic)]

use std::collections::HashMap;

use crate::expressions::parser::stringify::to_string;
use crate::expressions::parser::{Node, Parser};
use crate::expressions::types::CellReferenceRC;

#[test]
fn issue_483_parser() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = Parser::new(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 2,
        column: 2,
    };
    let t = parser.parse("-(A1^1.22)", &cell_reference);
    assert!(matches!(t, Node::UnaryKind { .. }));
    assert_eq!(to_string(&t, &cell_reference), "-(A1^1.22)");

    let t = parser.parse("-A1^1.22", &cell_reference);
    assert!(matches!(t, Node::OpPowerKind { .. }));
    assert_eq!(to_string(&t, &cell_reference), "-A1^1.22");
}
