#![allow(clippy::unwrap_used)]

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::{TableReference, TableSpecifier, TokenType::*},
};
use crate::language::get_language;
use crate::locale::get_locale;

fn new_lexer(formula: &str) -> Lexer<'_> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    Lexer::new(formula, LexerMode::A1, locale, language)
}

#[test]
fn table_this_row() {
    let mut lx = new_lexer("tbInfo[[#This Row], [Jan]:[Dec]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::ThisRow),
            table_reference: Some(TableReference::RangeReference((
                "Jan".to_string(),
                "Dec".to_string()
            )))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier() {
    let mut lx = new_lexer("tbInfo[December]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("December".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier_white_spaces() {
    let mut lx = new_lexer("tbInfo[[First Month]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("First Month".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_totals_no_reference() {
    let mut lx = new_lexer("tbInfo[#Totals]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::Totals),
            table_reference: None
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn tokens_of_the_short_form() {
    use crate::expressions::lexer::util::get_tokens;
    let tokens = get_tokens("=[@Qty]*2+[");
    assert_eq!(
        tokens[1].token,
        StructuredReference {
            table_name: "".to_string(),
            specifier: Some(TableSpecifier::ThisRow),
            table_reference: Some(TableReference::ColumnReference("Qty".to_string())),
        }
    );
    assert_eq!((tokens[1].start, tokens[1].end), (1, 7));
    // A bracket that starts nothing stays a bracket
    assert_eq!(tokens.last().unwrap().token, LeftBracket);
}

#[test]
fn an_unclosed_column_name_is_an_error() {
    // Not a column called "Price"
    let mut lx = new_lexer("tbInfo[Price");
    assert!(matches!(lx.next_token(), Illegal(_)));
}
