//! Tests for partial parsing / completion (`parse_at_cursor`).
//!
//! Note: as elsewhere in the parser tests, formulas are written *without* the
//! leading `=`. The cursor is a char offset into that string; `|` in the doc
//! comments marks where it sits.

#![allow(clippy::panic)]

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::{CompletionContext, ExpectedTokens};
use crate::expressions::types::CellReferenceRC;

/// Parse `formula` with the cursor at `cursor` (char offset) in a single-sheet
/// workbook whose active cell is `Sheet1!A1`.
fn complete_at(formula: &str, cursor: usize) -> CompletionContext {
    let worksheets = vec!["Sheet1".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let cell_reference = CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    };
    parser.parse_at_cursor(formula, cursor, &cell_reference)
}

/// Convenience: cursor sitting at the very end of `formula`.
fn complete_at_end(formula: &str) -> CompletionContext {
    complete_at(formula, formula.chars().count())
}

// ---------------------------------------------------------------------------
// (a) Incomplete prefix → EOF frame stamps `expecting`
// ---------------------------------------------------------------------------

#[test]
fn open_function_call_expects_first_argument() {
    // SUM(|
    let ctx = complete_at_end("SUM(");
    assert_eq!(
        ctx.expecting,
        vec![
            ExpectedTokens::Argument("SUM".to_string(), 1),
            ExpectedTokens::Range,
        ]
    );
    // Nothing to replace: the completion is inserted fresh at the cursor.
    assert_eq!(ctx.replace_from, 4);
}

#[test]
fn third_argument_of_user_function() {
    // MyFormula(3,4,|
    let ctx = complete_at_end("MyFormula(3,4,");
    assert_eq!(
        ctx.expecting,
        vec![
            ExpectedTokens::Argument("MyFormula".to_string(), 3),
            ExpectedTokens::Range,
        ]
    );
}

#[test]
fn second_argument_after_separator() {
    // SUM(A1,|
    let ctx = complete_at_end("SUM(A1,");
    assert_eq!(
        ctx.expecting,
        vec![
            ExpectedTokens::Argument("SUM".to_string(), 2),
            ExpectedTokens::Range,
        ]
    );
}

#[test]
fn nested_call_reports_innermost_context() {
    // SUM(AVERAGE(|  → we are in argument 1 of AVERAGE, not SUM
    let ctx = complete_at_end("SUM(AVERAGE(");
    assert_eq!(
        ctx.expecting,
        vec![
            ExpectedTokens::Argument("AVERAGE".to_string(), 1),
            ExpectedTokens::Range,
        ]
    );
}

#[test]
fn dangling_binary_operator_expects_an_operand() {
    // A1+|  → an operand goes here, and a range is valid
    let ctx = complete_at_end("A1+");
    assert!(ctx.expecting.contains(&ExpectedTokens::Range));
}

#[test]
fn empty_formula_expects_an_expression() {
    // | (just typed `=`)
    let ctx = complete_at_end("");
    assert!(ctx.expecting.contains(&ExpectedTokens::Range));
}

// ---------------------------------------------------------------------------
// (b) Valid-but-mid-edit identifier → parse SUCCEEDS, no ParseErrorKind
// ---------------------------------------------------------------------------

#[test]
fn trailing_name_offers_function_completion() {
    // A1+F|  → user is typing a function/name starting with `F`
    let ctx = complete_at_end("A1+F");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("F".to_string())]
    );
    // The UI should replace the `F` it is completing.
    assert_eq!(ctx.replace_from, 3);
}

#[test]
fn bare_partial_function_name() {
    // SU|  → `SUM`, `SUMIF`, ...
    let ctx = complete_at_end("SU");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("SU".to_string())]
    );
    assert_eq!(ctx.replace_from, 0);
}

#[test]
fn bare_partial_function_name_in_the_middle() {
    // IF(VLOOK|  → even though the call is still open, the cursor sits on the
    // partial name `VLOOK`; we offer name completion and replace only `VLOOK`.
    let ctx = complete_at_end("IF(VLOOK");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("VLOOK".to_string())]
    );
    assert_eq!(ctx.replace_from, 3);
}

#[test]
fn partial_function_name_in_nested_call() {
    // SUM(AVERAGE(MEDI|  → still typing `MEDI` two calls deep.
    let ctx = complete_at_end("SUM(AVERAGE(MEDI");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("MEDI".to_string())]
    );
    assert_eq!(ctx.replace_from, 12);
}

#[test]
fn partial_name_after_operator_inside_argument() {
    // SUM(A1+SQR|  → `A1` is complete, the cursor is on the partial name `SQR`.
    let ctx = complete_at_end("SUM(A1+SQR");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("SQR".to_string())]
    );
    assert_eq!(ctx.replace_from, 7);
}

#[test]
fn partial_name_in_second_argument() {
    // IF(A1,CONCAT|  → typing a name in the second argument of an open call.
    let ctx = complete_at_end("IF(A1,CONCAT");
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("CONCAT".to_string())]
    );
    assert_eq!(ctx.replace_from, 6);
}

#[test]
fn partial_name_with_cursor_before_existing_tail() {
    // MAX(AVER|, B2)  → cursor after `AVER`; the `, B2)` tail is ignored and we
    // replace only the `AVER` being typed.
    let ctx = complete_at("MAX(AVER, B2)", 8);
    assert_eq!(
        ctx.expecting,
        vec![ExpectedTokens::FunctionName("AVER".to_string())]
    );
    assert_eq!(ctx.replace_from, 4);
}

// ---------------------------------------------------------------------------
// (d) An identifier glued to the end of a number is not a function name being
//     typed — the number already completed an operand. (Nothing to do with
//     floating point / scientific notation; any letter behaves the same.)
// ---------------------------------------------------------------------------

#[test]
fn identifier_glued_to_a_number_is_not_a_function() {
    // 1m|  → `1` is a complete operand; the trailing `m` is not a function name.
    let ctx = complete_at_end("1m");
    assert!(
        !ctx.expecting
            .iter()
            .any(|e| matches!(e, ExpectedTokens::FunctionName(_))),
        "got {:?}",
        ctx.expecting
    );
}

#[test]
fn incomplete_scientific_notation_inside_argument_is_not_a_function() {
    // SUM(1e|  → still a partial number inside the call, not a function name.
    let ctx = complete_at_end("SUM(1e");
    assert!(
        !ctx.expecting
            .iter()
            .any(|e| matches!(e, ExpectedTokens::FunctionName(_))),
        "got {:?}",
        ctx.expecting
    );
}

// ---------------------------------------------------------------------------
// (e) The wrinkle: complete-looking argument then EOF
// ---------------------------------------------------------------------------

#[test]
fn complete_arg_without_closing_paren_keeps_function_context() {
    // SUM(A1|  → `A1` is complete but the call is still open; we should still
    // know we are inside argument 1 of SUM rather than falling back to Other.
    let ctx = complete_at_end("SUM(A1");
    assert!(ctx
        .expecting
        .contains(&ExpectedTokens::Argument("SUM".to_string(), 1)));
}

// ---------------------------------------------------------------------------
// Cursor not at end of input → only the prefix before the cursor matters
// ---------------------------------------------------------------------------

#[test]
fn cursor_in_the_middle_uses_only_the_prefix() {
    // SUM(|, B2)  → cursor right after the `(`, ignoring the `, B2)` tail
    let ctx = complete_at("SUM(, B2)", 4);
    assert_eq!(
        ctx.expecting,
        vec![
            ExpectedTokens::Argument("SUM".to_string(), 1),
            ExpectedTokens::Range,
        ]
    );
}

// ---------------------------------------------------------------------------
// Cursor inside a string literal
// ---------------------------------------------------------------------------

#[test]
fn cursor_inside_string_literal() {
    // SUM("bla|  → the cursor is inside an unterminated string, so no grammar
    // completion applies.
    let ctx = complete_at_end("SUM(\"bla");
    assert_eq!(ctx.expecting, vec![ExpectedTokens::Other]);
}

#[test]
fn top_level_string_literal() {
    // "hello wor|  → still inside a string even without an enclosing call.
    let ctx = complete_at_end("\"hello wor");
    assert_eq!(ctx.expecting, vec![ExpectedTokens::Other]);
}

#[test]
fn closed_string_is_not_in_string() {
    // SUM("bla"|  → the string is closed; we are back to expecting more of the
    // argument / a separator, not "in string".
    let ctx = complete_at_end("SUM(\"bla\"");
    assert!(!ctx.expecting.contains(&ExpectedTokens::Other));
}

#[test]
fn reopened_string_after_a_closed_one() {
    // CONCAT("a", "bl|  → first string closed, second one open.
    let ctx = complete_at_end("CONCAT(\"a\", \"bl");
    assert_eq!(ctx.expecting, vec![ExpectedTokens::Other]);
}

#[test]
fn escaped_quotes_inside_string_stay_open() {
    // SUM("say ""hi""| → the `""` are escaped quotes, the string is still open.
    let ctx = complete_at_end("SUM(\"say \"\"hi\"\"");
    assert_eq!(ctx.expecting, vec![ExpectedTokens::Other]);
}
