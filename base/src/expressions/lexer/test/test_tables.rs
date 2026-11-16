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
fn table_headers_and_data() {
    let mut lx = new_lexer("tbInfo[[#Headers], [#Data], [Amount]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::HeadersData),
            table_reference: Some(TableReference::ColumnReference("Amount".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_data_and_totals_range() {
    let mut lx = new_lexer("tbInfo[[#Data],[#Totals],[Jan]:[Dec]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::DataTotals),
            table_reference: Some(TableReference::RangeReference((
                "Jan".to_string(),
                "Dec".to_string()
            )))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_headers_and_data_no_reference() {
    let mut lx = new_lexer("tbInfo[[#Headers],[#Data]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::HeadersData),
            table_reference: None
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_invalid_item_combination() {
    let mut lx = new_lexer("tbInfo[[#Headers],[#Totals],[Jan]]");
    assert!(matches!(lx.next_token(), Illegal(_)));
}

#[test]
fn table_totals_in_brackets_no_reference() {
    let mut lx = new_lexer("tbInfo[[#Totals]]");
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
