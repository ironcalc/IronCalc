use super::Parser;
use crate::expressions::parser::stringify::*;
use crate::expressions::types::CellReferenceRC;
use std::collections::HashMap;

fn make_parser() -> Parser {
    let worksheets = vec!["Sheet1".to_string()];
    Parser::new(worksheets, HashMap::new())
}

#[test]
fn test_to_rc_format() {
    let mut parser = make_parser();

    // with no context
    let node = parser.parse("$C$5", &None);
    let formatted = to_rc_format(&node);
    assert_eq!(formatted, "$C$5");

    // Reference cell is Sheet1!A1
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };

    let node = parser.parse("$C$5", &Some(cell_reference));
    let formatted = to_rc_format(&node);
    assert_eq!(formatted, "R5C3");

    let cell_reference_2 = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 99,
        column: 101,
    };

    let node3 = parser.parse("$C$5", &Some(cell_reference_2));
    let formatted3 = to_rc_format(&node3);
    assert_eq!(formatted3, "R5C3");
}

#[test]
fn test_to_string_displaced() {
    let mut parser = make_parser();

    // Reference cell is Sheet1!A1
    let cell_reference: CellReferenceRC = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };

    let node = parser.parse("$C$5", &Some(cell_reference.clone()));

    let output = to_string_displaced(&node, &cell_reference, &DisplaceData::None);
    assert_eq!(output, "$C$5");

    let output = to_string_displaced(
        &node,
        &cell_reference,
        &DisplaceData::Row {
            sheet: 0,
            row: 1,
            delta: 2,
        },
    );
    assert_eq!(output, "$C$7");

    let output = to_string_displaced(
        &node,
        &cell_reference,
        &DisplaceData::Row {
            sheet: 0,
            row: 1,
            delta: -2,
        },
    );
    assert_eq!(output, "$C$3");

    let output = to_string_displaced(
        &node,
        &cell_reference,
        &DisplaceData::Column {
            sheet: 0,
            column: 1,
            delta: 10,
        },
    );
    assert_eq!(output, "$M$5");
}

#[test]

fn test_to_string() {
    let mut parser = make_parser();

    // Reference cell is Sheet1!A1
    let context: Option<CellReferenceRC> = Some(CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    });

    let node = parser.parse("$C$5", &context);

    assert_eq!(to_string(&node, &context.clone().unwrap()), "$C$5");

    let cell_reference: CellReferenceRC = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 9,
        column: 9,
    };
    let node = parser.parse("$C$5", &Some(cell_reference.clone()));
   
    assert_eq!(to_string(&node, &cell_reference.clone()), "$C$5");
}

#[test]

fn test_to_excel_string() {
    let mut parser = make_parser();

    // Reference cell is Sheet1!A1
    let context: Option<CellReferenceRC> = Some(CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    });

    let node = parser.parse("$C$5", &context);

    assert_eq!(to_excel_string(&node, &context.clone().unwrap()), "$C$5");

    let cell_reference: CellReferenceRC = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 9,
        column: 9,
    };
    let node = parser.parse("$C$5", &Some(cell_reference.clone()));
   
    assert_eq!(to_excel_string(&node, &cell_reference), "$C$5");
}