use std::collections::HashMap;

use crate::expressions::{
    parser::{stringify::to_string, Parser},
    types::{CellReferenceIndex, CellReferenceRC},
};

use crate::expressions::parser::static_analysis::add_implicit_intersection;

#[test]
fn simple_test() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = Parser::new(worksheets, vec![], HashMap::new());

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    let formula = "A1:A10*SUM(A1:A10)";
    let mut t = parser.parse(formula, &cell_reference);
    add_implicit_intersection(
        &mut t,
        &CellReferenceIndex {
            sheet: 0,
            column: cell_reference.column,
            row: cell_reference.row,
        },
        true,
    );
    let r = to_string(&t, &cell_reference);
    assert_eq!(r, "@A1:A10*SUM(A1:A10)");
}
