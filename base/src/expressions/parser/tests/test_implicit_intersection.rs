#![allow(clippy::panic)]

use crate::expressions::parser::{Node, Parser};
use crate::expressions::types::CellReferenceRC;
use std::collections::HashMap;

#[test]
fn simple_tes() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = Parser::new(worksheets, HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse("@A1:A10", &Some(cell_reference));
    let child = Node::RangeKind {
        sheet_name: None,
        sheet_index: 0,
        absolute_row1: false,
        absolute_column1: false,
        row1: 0,
        column1: 0,
        absolute_row2: false,
        absolute_column2: false,
        row2: 9,
        column2: 0,
    };
    assert_eq!(
        t,
        Node::ImplicitIntersection {
            automatic: false,
            child: Box::new(child)
        }
    )
}
