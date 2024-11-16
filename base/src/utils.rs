use crate::expressions::token::get_error_by_name;
use crate::expressions::types::CellReferenceIndex;
use crate::language::Language;

use crate::{
    expressions::{
        lexer::{Lexer, LexerMode},
        token::TokenType,
    },
    language::get_language,
    locale::Locale,
};

#[derive(Debug, Eq, PartialEq)]
pub enum ParsedReference {
    CellReference(CellReferenceIndex),
    Range(CellReferenceIndex, CellReferenceIndex),
}

impl ParsedReference {
    /// Parses reference in formula format. For example:  `Sheet1!A1`, `Sheet1!$A$1:$B$9`.
    /// Absolute references (`$`) do not affect parsing.
    ///
    /// # Arguments
    ///
    /// * `sheet_index_context` - if available, sheet index can be provided so references
    ///   without explicit sheet name can be recognized
    /// * `reference` - text string to parse as reference
    /// * `locale` - locale that will be used to set-up parser
    /// * `get_sheet_index_by_name` - function that allows to translate sheet name to index
    pub(crate) fn parse_reference_formula<F: Fn(&str) -> Option<u32>>(
        sheet_index_context: Option<u32>,
        reference: &str,
        locale: &Locale,
        get_sheet_index_by_name: F,
    ) -> Result<ParsedReference, String> {
        #[allow(clippy::expect_used)]
        let language = get_language("en").expect("");
        let mut lexer = Lexer::new(reference, LexerMode::A1, locale, language);

        let reference_token = lexer.next_token();
        let eof_token = lexer.next_token();

        if TokenType::EOF != eof_token {
            return Err("Invalid reference. Expected only one token.".to_string());
        }

        match reference_token {
            TokenType::Reference {
                sheet: sheet_name,
                column: column_id,
                row: row_id,
                ..
            } => {
                let sheet_index;
                if let Some(name) = sheet_name {
                    match get_sheet_index_by_name(&name) {
                        Some(i) => sheet_index = i,
                        None => {
                            return Err(format!(
                                "Invalid reference. Sheet \"{}\" could not be found.",
                                name.as_str(),
                            ));
                        }
                    }
                } else if let Some(sheet_index_context) = sheet_index_context {
                    sheet_index = sheet_index_context;
                } else {
                    return Err(
                        "Reference doesn't contain sheet name and relative cell is not known."
                            .to_string(),
                    );
                }

                Ok(ParsedReference::CellReference(CellReferenceIndex {
                    sheet: sheet_index,
                    row: row_id,
                    column: column_id,
                }))
            }
            TokenType::Range {
                sheet: sheet_name,
                left,
                right,
            } => {
                let sheet_index;
                if let Some(name) = sheet_name {
                    match get_sheet_index_by_name(&name) {
                        Some(i) => sheet_index = i,
                        None => {
                            return Err(format!(
                                "Invalid reference. Sheet \"{}\" could not be found.",
                                name.as_str(),
                            ));
                        }
                    }
                } else if let Some(sheet_index_context) = sheet_index_context {
                    sheet_index = sheet_index_context;
                } else {
                    return Err(
                        "Reference doesn't contain sheet name and relative cell is not known."
                            .to_string(),
                    );
                }

                Ok(ParsedReference::Range(
                    CellReferenceIndex {
                        sheet: sheet_index,
                        row: left.row,
                        column: left.column,
                    },
                    CellReferenceIndex {
                        sheet: sheet_index,
                        row: right.row,
                        column: right.column,
                    },
                ))
            }
            _ => Err("Invalid reference. First token is not a reference.".to_string()),
        }
    }
}

/// Returns true if the string value could be interpreted as:
///  * a formula
///  * a number
///  * a boolean
///  * an error (i.e "#VALUE!")
pub(crate) fn value_needs_quoting(value: &str, language: &Language) -> bool {
    value.starts_with('=')
        || value.parse::<f64>().is_ok()
        || value.to_lowercase().parse::<bool>().is_ok()
        || get_error_by_name(&value.to_uppercase(), language).is_some()
}

/// Valid hex colors are #FFAABB
/// #fff is not valid
pub(crate) fn is_valid_hex_color(color: &str) -> bool {
    if color.chars().count() != 7 {
        return false;
    }
    if !color.starts_with('#') {
        return false;
    }
    if let Ok(z) = i32::from_str_radix(&color[1..], 16) {
        if (0..=0xffffff).contains(&z) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::language::get_language;
    use crate::locale::{get_locale, Locale};

    fn get_test_locale() -> &'static Locale {
        #![allow(clippy::unwrap_used)]
        get_locale("en").unwrap()
    }

    fn get_sheet_index_by_name(sheet_names: &[&str], name: &str) -> Option<u32> {
        sheet_names
            .iter()
            .position(|&sheet_name| sheet_name == name)
            .map(|index| index as u32)
    }

    #[test]
    fn test_parse_cell_references() {
        let locale = get_test_locale();
        let sheet_names = vec!["Sheet1", "Sheet2", "Sheet3"];

        assert_eq!(
            ParsedReference::parse_reference_formula(Some(7), "A1", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::CellReference(CellReferenceIndex {
                sheet: 7,
                row: 1,
                column: 1,
            })),
        );

        assert_eq!(
            ParsedReference::parse_reference_formula(None, "Sheet1!A1", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::CellReference(CellReferenceIndex {
                sheet: 0,
                row: 1,
                column: 1,
            })),
        );

        assert_eq!(
            ParsedReference::parse_reference_formula(None, "Sheet1!$A$1", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::CellReference(CellReferenceIndex {
                sheet: 0,
                row: 1,
                column: 1,
            })),
        );

        assert_eq!(
            ParsedReference::parse_reference_formula(None, "Sheet2!$A$1", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::CellReference(CellReferenceIndex {
                sheet: 1,
                row: 1,
                column: 1,
            })),
        );
    }

    #[test]
    fn test_parse_range_references() {
        let locale = get_test_locale();
        let sheet_names = vec!["Sheet1", "Sheet2", "Sheet3"];

        assert_eq!(
            ParsedReference::parse_reference_formula(Some(5), "A1:A2", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::Range(
                CellReferenceIndex {
                    sheet: 5,
                    column: 1,
                    row: 1,
                },
                CellReferenceIndex {
                    sheet: 5,
                    column: 1,
                    row: 2,
                },
            )),
        );

        assert_eq!(
            ParsedReference::parse_reference_formula(None, "Sheet1!$A$1:$B$10", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::Range(
                CellReferenceIndex {
                    sheet: 0,
                    row: 1,
                    column: 1,
                },
                CellReferenceIndex {
                    sheet: 0,
                    row: 10,
                    column: 2,
                },
            )),
        );

        assert_eq!(
            ParsedReference::parse_reference_formula(None, "Sheet2!AA1:E$11", locale, |name| {
                get_sheet_index_by_name(&sheet_names, name)
            },),
            Ok(ParsedReference::Range(
                CellReferenceIndex {
                    sheet: 1,
                    row: 1,
                    column: 27,
                },
                CellReferenceIndex {
                    sheet: 1,
                    row: 11,
                    column: 5,
                },
            )),
        );
    }

    #[test]
    fn test_error_reject_assignments() {
        let locale = get_test_locale();
        let sheet_index = Some(1);
        assert_eq!(
            ParsedReference::parse_reference_formula(sheet_index, "=A1", locale, |_| Some(1)),
            Err("Invalid reference. Expected only one token.".to_string()),
        );
        assert_eq!(
            ParsedReference::parse_reference_formula(sheet_index, "=$A$1", locale, |_| { Some(1) }),
            Err("Invalid reference. Expected only one token.".to_string()),
        );
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "=Sheet1!A1", locale, |_| Some(1)),
            Err("Invalid reference. Expected only one token.".to_string()),
        );
    }

    #[test]
    fn test_error_reject_formulas_without_equal_sign() {
        let locale = get_test_locale();
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "SUM", locale, |_| Some(1)),
            Err("Invalid reference. First token is not a reference.".to_string()),
        );
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "SUM(A1:A2)", locale, |_| Some(1)),
            Err("Invalid reference. Expected only one token.".to_string()),
        );
    }

    #[test]
    fn test_error_reject_without_sheet_and_relative_cell() {
        let locale = get_test_locale();
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "A1", locale, |_| Some(1)),
            Err("Reference doesn't contain sheet name and relative cell is not known.".to_string()),
        );
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "A1:A2", locale, |_| Some(1)),
            Err("Reference doesn't contain sheet name and relative cell is not known.".to_string()),
        );
    }

    #[test]
    fn test_error_unrecognized_sheet_name() {
        let locale = get_test_locale();
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "SheetName!A1", locale, |_| None),
            Err("Invalid reference. Sheet \"SheetName\" could not be found.".to_string()),
        );
        assert_eq!(
            ParsedReference::parse_reference_formula(None, "SheetName2!A1:A4", locale, |_| None),
            Err("Invalid reference. Sheet \"SheetName2\" could not be found.".to_string()),
        );
    }

    #[test]
    fn test_value_needs_quoting() {
        let en_language = get_language("en").expect("en language expected");

        assert!(!value_needs_quoting("", en_language));
        assert!(!value_needs_quoting("hello", en_language));

        assert!(value_needs_quoting("12", en_language));
        assert!(value_needs_quoting("true", en_language));
        assert!(value_needs_quoting("False", en_language));

        assert!(value_needs_quoting("=A1", en_language));

        assert!(value_needs_quoting("#REF!", en_language));
        assert!(value_needs_quoting("#NAME?", en_language));
    }

    #[test]
    fn test_is_valid_hex_color() {
        assert!(is_valid_hex_color("#000000"));
        assert!(is_valid_hex_color("#ffffff"));

        assert!(!is_valid_hex_color("000000"));
        assert!(!is_valid_hex_color("ffffff"));

        assert!(!is_valid_hex_color("#gggggg"));

        // Not obvious cases unrecognized as colors
        assert!(!is_valid_hex_color("#ffffff "));
        assert!(!is_valid_hex_color("#fff")); // CSS shorthand
        assert!(!is_valid_hex_color("#ffffff00")); // with alpha channel
    }
}
