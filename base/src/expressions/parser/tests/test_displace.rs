#![allow(clippy::panic, clippy::unwrap_used)]

//! `displaced_node` must agree with the long way round: writing the formula
//! out with the displacement applied and parsing it back where the cell is
//! now. Inserting and deleting rows and columns went the long way for every
//! formula of the workbook until the node-level version replaced it.

use std::collections::HashMap;

use crate::expressions::parser::displace::{displaced_node, Displacement};
use crate::expressions::parser::stringify::{to_rc_format, to_string_displaced, DisplaceData};
use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::Node;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceRC;
use crate::language::get_default_language;
use crate::locale::get_default_locale;

const FORMULAS: &[&str] = &[
    "A1",
    "$A$1",
    "A$1",
    "$A1",
    "A5",
    "C3",
    "E9",
    "A1:B2",
    "A3:A10",
    "$A$3:$B$10",
    "A$3:B10",
    "A:A",
    "C:E",
    "3:3",
    "2:6",
    "SUM(A1:A10)",
    "SUM(A:A)",
    "SUM(A1,B2:B3,Sheet2!C1)",
    "A1*B1+1",
    "-A1",
    "A1%",
    "-(A1+B1)",
    "IF(A1>0,A2,A3)",
    "A1&B1",
    "A1^2",
    "(A1+A2)*A3",
    "{1,2;3,4}",
    "TRUE",
    "\"text\"",
    "#N/A",
    "1.5",
    "LAMBDA(x,x+A1)(A2)",
    "LET(x,A1,x*2)",
    "@A1:A3",
    "A1#",
    "Sheet2!A1",
    "Sheet2!$B$2",
    "Sheet2!A1:B5",
    "Sheet2!A:A",
    "Sheet3!A1",
    "Sheet3!A1:B2",
    "Sheet3!A:A",
    "SUM(A1:A3",
    "1+",
];

fn displacements() -> Vec<DisplaceData> {
    let mut all = vec![DisplaceData::None];
    for sheet in 0..2 {
        all.extend([
            DisplaceData::Row {
                sheet,
                row: 3,
                delta: 2,
            },
            DisplaceData::Row {
                sheet,
                row: 3,
                delta: -2,
            },
            DisplaceData::Row {
                sheet,
                row: 1,
                delta: -1,
            },
            DisplaceData::Row {
                sheet,
                row: 1,
                delta: 1,
            },
            DisplaceData::Column {
                sheet,
                column: 2,
                delta: 3,
            },
            DisplaceData::Column {
                sheet,
                column: 2,
                delta: -1,
            },
            DisplaceData::Column {
                sheet,
                column: 1,
                delta: -2,
            },
            DisplaceData::CellHorizontal {
                sheet,
                row: 1,
                column: 1,
                delta: 2,
            },
            DisplaceData::CellVertical {
                sheet,
                row: 1,
                column: 1,
                delta: -1,
            },
            DisplaceData::RowMove {
                sheet,
                row: 5,
                delta: 3,
            },
            DisplaceData::RowMove {
                sheet,
                row: 5,
                delta: -3,
            },
            DisplaceData::ColumnMove {
                sheet,
                column: 2,
                delta: 1,
            },
            DisplaceData::ColumnMove {
                sheet,
                column: 3,
                delta: -2,
            },
        ]);
    }
    all
}

/// `(row, column)` of a cell
type Position = (i32, i32);

/// Where the cell was and where it is: not moved, moved by the displacement
/// itself, and moved anywhere else, which exercises the change of context
/// on its own.
const POSITIONS: &[(Position, Position)] = &[
    ((5, 3), (5, 3)),
    ((5, 3), (7, 3)),
    ((5, 3), (3, 3)),
    ((5, 3), (5, 6)),
    ((5, 3), (5, 2)),
    ((1, 1), (1, 1)),
    ((5, 3), (1, 1)),
    ((1, 1), (9, 9)),
];

fn reference(row: i32, column: i32) -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row,
        column,
    }
}

#[test]
fn agrees_with_writing_out_and_parsing_back() {
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let locale = get_default_locale();
    let language = get_default_language();
    let mut compared = 0;
    for formula in FORMULAS {
        for data in displacements() {
            for (from, to) in POSITIONS {
                let node = parser.parse(formula, &reference(from.0, from.1));
                let text =
                    to_string_displaced(&node, &reference(from.0, from.1), &data, locale, language);
                if text.contains("#REF!:") || text.contains(":#REF!") {
                    // A range with a deleted end is where the two part ways,
                    // by design: written out, `Sheet2!A1:B5` loses its sheet
                    // (`#REF!:B4`), `$A$3:#REF!` and `#REF!:A` do not parse
                    // back at all. Displaced in place it is the range operator
                    // on what is left. See `ranges_with_a_deleted_end`.
                    continue;
                }
                let expected = parser.parse(&text, &reference(to.0, to.1));
                let displacement = Displacement {
                    data: &data,
                    from: *from,
                    to: *to,
                };
                let displaced =
                    displaced_node(&node, &displacement).unwrap_or_else(|| node.clone());
                assert_eq!(
                    to_rc_format(&displaced),
                    to_rc_format(&expected),
                    "'{formula}' from {from:?} to {to:?} written out as '{text}'"
                );
                compared += 1;
            }
        }
    }
    assert!(compared > 9000);
}

#[test]
fn unchanged_formulas_are_not_copied() {
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    // Rows inserted above: the cell and everything it reads move together
    let data = DisplaceData::Row {
        sheet: 0,
        row: 3,
        delta: 2,
    };
    for formula in [
        "A5*B5",
        "SUM(A:A)",
        "$A$1",
        "Sheet2!A$1",
        "1+2",
        "SUM(A5:A10)",
    ] {
        let node = parser.parse(formula, &reference(5, 3));
        let displacement = Displacement {
            data: &data,
            from: (5, 3),
            to: (7, 3),
        };
        assert!(
            displaced_node(&node, &displacement).is_none(),
            "'{formula}' should not change"
        );
    }
    // The same cell staying put: what it reads moves away from it
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (5, 3),
    };
    let node = parser.parse("A5*B5", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "R[2]C[-2]*R[2]C[-1]");
    // A reference above the inserted rows, from a cell that moved
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (7, 3),
    };
    let node = parser.parse("A1", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "R[-6]C[-2]");
}

#[test]
fn deleted_references_become_ref_errors() {
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let data = DisplaceData::Row {
        sheet: 0,
        row: 2,
        delta: -2,
    };
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (3, 3),
    };
    let node = parser.parse("A2*2", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "#REF!*2");
    let node = parser.parse("SUM(A1:A3)", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "SUM(R[-2]C[-2]:#REF!)");
    let node = parser.parse("SUM(A2:A3)", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "SUM(#REF!:#REF!)");
}

#[test]
fn ranges_with_a_deleted_end() {
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let locale = get_default_locale();
    let language = get_default_language();

    // The plain case agrees with the formula written out and parsed back
    let data = DisplaceData::Row {
        sheet: 0,
        row: 2,
        delta: -1,
    };
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (4, 3),
    };
    let node = parser.parse("SUM(A2:A3)", &reference(5, 3));
    let text = to_string_displaced(&node, &reference(5, 3), &data, locale, language);
    assert_eq!(text, "SUM(#REF!:A2)");
    let expected = parser.parse(&text, &reference(4, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), to_rc_format(&expected));
    assert_eq!(to_rc_format(&displaced), "SUM(#REF!:R[-2]C[-2])");

    // An absolute end: written out it did not parse back
    let data = DisplaceData::Column {
        sheet: 0,
        column: 2,
        delta: -1,
    };
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (5, 2),
    };
    let node = parser.parse("SUM($A$3:$B$10)", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "SUM(R3C1:#REF!)");

    // On another sheet: written out the surviving end lost its sheet
    let data = DisplaceData::Row {
        sheet: 1,
        row: 1,
        delta: -1,
    };
    let displacement = Displacement {
        data: &data,
        from: (5, 3),
        to: (5, 3),
    };
    let node = parser.parse("SUM(Sheet2!A1:B5)", &reference(5, 3));
    let displaced = displaced_node(&node, &displacement).unwrap();
    assert_eq!(to_rc_format(&displaced), "SUM(#REF!:Sheet2!R[-1]C[-1])");
}

#[test]
fn a_reference_pushed_off_the_sheet_is_gone() {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let data = DisplaceData::Row {
        sheet: 0,
        row: 1,
        delta: 1,
    };
    let displacement = Displacement {
        data: &data,
        from: (1, 1),
        to: (2, 1),
    };
    let node = parser.parse("A1048576", &reference(1, 1));
    assert_eq!(
        displaced_node(&node, &displacement),
        Some(Node::ErrorKind(Error::REF))
    );
}
