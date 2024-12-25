#![allow(clippy::panic)]

use std::collections::HashMap;

use crate::expressions::parser::stringify::to_string;
use crate::expressions::parser::Parser;
use crate::expressions::types::CellReferenceRC;

#[test]
fn exp_order() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = Parser::new(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let t = parser.parse("(1 + 2)^3  + 4", &cell_reference);
    assert_eq!(to_string(&t, &cell_reference), "(1+2)^3+4");

    let t = parser.parse("(C5 + 3)^R4", &cell_reference);
    assert_eq!(to_string(&t, &cell_reference), "(C5+3)^R4");

    let t = parser.parse("(C5 + 3)^(R4*6)", &cell_reference);
    assert_eq!(to_string(&t, &cell_reference), "(C5+3)^(R4*6)");

    let t = parser.parse("(C5)^(R4)", &cell_reference);
    assert_eq!(to_string(&t, &cell_reference), "C5^R4");

    let t = parser.parse("(5)^(4)", &cell_reference);
    assert_eq!(to_string(&t, &cell_reference), "5^4");
}
