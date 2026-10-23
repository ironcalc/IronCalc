#![allow(clippy::unwrap_used)]

use crate::language::get_language;
use crate::locale::get_locale;

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::TokenType::*,
};

fn new_lexer(formula: &str, a1_mode: bool) -> Lexer<'_> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mode = if a1_mode {
        LexerMode::A1
    } else {
        LexerMode::R1C1
    };
    Lexer::new(formula, mode, locale, language)
}

// `Sheet!Name` where Name is not a reference is a sheet-qualified identifier
#[test]
fn qualified_identifier() {
    for a1_mode in [true, false] {
        let mut lx = new_lexer("Data!Scope_Test+1", a1_mode);
        assert_eq!(
            lx.next_token(),
            QualifiedIdent {
                sheet: "Data".to_string(),
                name: "Scope_Test".to_string()
            }
        );
        assert_eq!(
            lx.next_token(),
            Addition(crate::expressions::token::OpSum::Add)
        );
        assert_eq!(lx.next_token(), Number(1.0));
        assert_eq!(lx.next_token(), EOF);
    }
}

#[test]
fn qualified_identifier_quoted_sheet() {
    let mut lx = new_lexer("'Q1 Sales'!Total", true);
    assert_eq!(
        lx.next_token(),
        QualifiedIdent {
            sheet: "Q1 Sales".to_string(),
            name: "Total".to_string()
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

// A reference after the sheet is still a reference
#[test]
fn qualified_reference_is_a_reference() {
    let mut lx = new_lexer("Data!A1", true);
    assert!(matches!(lx.next_token(), Reference { .. }));
    let mut lx = new_lexer("Data!R1C1", false);
    assert!(matches!(lx.next_token(), Reference { .. }));
}

// Nothing valid after the bang is still an error
#[test]
fn qualified_nothing_is_illegal() {
    for formula in ["Data!", "Data!+1", "Data!3.4:5"] {
        let mut lx = new_lexer(formula, true);
        assert!(matches!(lx.next_token(), Illegal(_)), "{formula}");
    }
}

// A reference that is only the start of a longer identifier is a defined name
#[test]
fn qualified_identifier_starting_like_a_reference() {
    for (formula, a1_mode) in [
        ("Data!Q1_Total", true),
        ("Data!Q1Total", true),
        ("Data!R1C1_Total", false),
    ] {
        let mut lx = new_lexer(formula, a1_mode);
        let name = formula.split('!').nth(1).unwrap().to_string();
        assert_eq!(
            lx.next_token(),
            QualifiedIdent {
                sheet: "Data".to_string(),
                name
            },
            "{formula}"
        );
        assert_eq!(lx.next_token(), EOF);
    }
}

// Complete references and ranges are still references and ranges
#[test]
fn qualified_complete_references() {
    for formula in ["Data!Q1", "Data!$Q$1", "Data!Q1+1", "Data!Q1 "] {
        let mut lx = new_lexer(formula, true);
        assert!(matches!(lx.next_token(), Reference { .. }), "{formula}");
    }
    for formula in ["Data!A:A", "Data!$1:$1", "Data!A1:B2", "Data!A1:B2+1"] {
        let mut lx = new_lexer(formula, true);
        assert!(matches!(lx.next_token(), Range { .. }), "{formula}");
    }
    let mut lx = new_lexer("Data!R1C1:R2C2", false);
    assert!(matches!(lx.next_token(), Range { .. }));
}

// A reference followed by identifier characters that do not make a valid
// name (here one longer than 255 characters)
#[test]
fn qualified_reference_followed_by_an_invalid_name_is_illegal() {
    let formula = format!("Data!A1{}", "x".repeat(260));
    let mut lx = new_lexer(&formula, true);
    assert!(matches!(lx.next_token(), Illegal(_)));
}
