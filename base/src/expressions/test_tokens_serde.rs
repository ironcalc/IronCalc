#![allow(clippy::unwrap_used)]

use super::lexer::util::{get_tokens, MarkedToken};

#[test]
fn test_simple_formula() {
    let formula = "123+23";
    let tokens = get_tokens(formula);
    let tokens_str = serde_json::to_string(&tokens).unwrap();
    let tokens_json: Vec<MarkedToken> = serde_json::from_str(&tokens_str).unwrap();
    assert_eq!(tokens_json.len(), 3);
}
