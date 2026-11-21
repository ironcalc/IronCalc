// [$$-en-US]#,##0.00_);[Red]([$$-en-US]#,##0.00)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::formatter::lexer::{Lexer, Token};

#[test]
fn dollar_en_us() {
    let mut lexer = Lexer::new("[$$-en-US]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::Currency(ref c) if c == "$"));
}

#[test]
fn dollar_409() {
    let mut lexer = Lexer::new("[$$-409]");
    let token = lexer.next_token();
    assert!(matches!(token, Token::Currency(ref c) if c == "$"));
}

#[test]
fn symbol_of_several_characters() {
    let mut lexer = Lexer::new("[$ kr]");
    assert_eq!(lexer.next_token(), Token::Currency(" kr".to_string()));
    let mut lexer = Lexer::new("[$CHF-de-CH]");
    assert_eq!(lexer.next_token(), Token::Currency("CHF".to_string()));
}

#[test]
fn locale_without_a_symbol() {
    let mut lexer = Lexer::new("[$-409]");
    assert_eq!(lexer.next_token(), Token::Currency("".to_string()));
}

fn format(value: f64, format: &str) -> crate::formatter::format::Formatted {
    crate::formatter::format::format_number(value, format, crate::locale::get_locale("en").unwrap())
}

#[test]
fn symbol_is_written_where_it_stands() {
    // Google Sheets exports kronor like this
    let r = format(-1234.0, "#,##0[$ kr]");
    assert_eq!(r.error, None);
    assert_eq!(r.text, "-1,234 kr");
    assert_eq!(format(1234.5, "#,##0.00[$ €-fr-FR]").text, "1,234.50 €");
    assert_eq!(format(1234.0, "[$$-409]#,##0").text, "$1,234");
    assert_eq!(format(1234.0, "[$CHF-de-CH] #,##0").text, "CHF 1,234");
}

#[test]
fn a_locale_alone_writes_nothing() {
    assert_eq!(format(1234.0, "[$-409]#,##0").text, "1,234");
}
