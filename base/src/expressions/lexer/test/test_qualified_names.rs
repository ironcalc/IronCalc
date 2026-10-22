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
