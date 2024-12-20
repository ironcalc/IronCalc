use serde::{Deserialize, Serialize};

use crate::expressions::token;
use crate::language::get_language;
use crate::locale::get_locale;

use super::{Lexer, LexerMode};

/// A MarkedToken is a token together with its position on a formula
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkedToken {
    pub token: token::TokenType,
    pub start: i32,
    pub end: i32,
}

/// Returns a list of marked tokens for a formula
///
/// # Examples
/// ```
/// use ironcalc_base::expressions::{
///      lexer::util::{get_tokens, MarkedToken},
///      token::{OpSum, TokenType},
/// };
///
/// let marked_tokens = get_tokens("A1+1");
/// let first_t = MarkedToken {
///     token: TokenType::Reference {
///         sheet: None,
///         row: 1,
///         column: 1,
///         absolute_column: false,
///         absolute_row: false,
///     },
///     start: 0,
///     end: 2,
/// };
/// let second_t = MarkedToken {
///     token: TokenType::Addition(OpSum::Add),
///     start:2,
///     end: 3
/// };
/// let third_t = MarkedToken {
///     token: TokenType::Number(1.0),
///     start:3,
///     end: 4
/// };
/// assert_eq!(marked_tokens, vec![first_t, second_t, third_t]);
/// ```
pub fn get_tokens(formula: &str) -> Vec<MarkedToken> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(
        formula,
        LexerMode::A1,
        #[allow(clippy::expect_used)]
        get_locale("en").expect(""),
        #[allow(clippy::expect_used)]
        get_language("en").expect(""),
    );
    let mut start = lexer.get_position();
    let mut next_token = lexer.next_token();
    let mut end = lexer.get_position();
    loop {
        match next_token {
            token::TokenType::EOF => {
                break;
            }
            _ => {
                tokens.push(MarkedToken {
                    start,
                    end,
                    token: next_token,
                });
                start = lexer.get_position();
                next_token = lexer.next_token();
                end = lexer.get_position();
            }
        }
    }
    tokens
}
