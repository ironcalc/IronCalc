// Grammar:
// structured references -> table_name "[" arguments "]"
//  arguments -> table_reference | "["specifier"]"  "," table_reference
//  specifier > "#All"      |
//              "#This Row" |
//              "#Data"     |
//              "#Headers"  |
//              "#Totals"
// table_reference -> column_reference | range_reference
// Excel's short form: table_name "[@" (column_name | column_reference | range_reference)? "]"
// The table name can be left out inside a table: "[@Price]" (the parser fills it in).
// column reference -> column_name | "["column_name"]"
// range_reference -> column_reference":"column_reference

use crate::expressions::token::TokenType;
use crate::expressions::token::{TableReference, TableSpecifier};

use super::Result;
use super::{Lexer, LexerError};

/// Column names escape `[`, `]`, `#`, `@` and `'` with a quote: `Price 'Q1'` is `Price ''Q1''`.
fn unescape_column_name(chars: &str) -> String {
    chars
        .replace("'[", "[")
        .replace("']", "]")
        .replace("'#", "#")
        .replace("'@", "@")
        .replace("''", "'")
}

impl<'a> Lexer<'a> {
    fn consume_table_specifier(&mut self) -> Result<Option<TableSpecifier>> {
        if self.peek_char() == Some('#') {
            // It's a specifier
            // TODO(TD): There are better ways of doing this :)
            let rest_of_formula: String = self.chars[self.position..self.len].iter().collect();
            let specifier = if rest_of_formula.starts_with("#This Row]") {
                self.position += "#This Row]".len();
                TableSpecifier::ThisRow
            } else if rest_of_formula.starts_with("#All]") {
                self.position += "#All]".len();
                TableSpecifier::All
            } else if rest_of_formula.starts_with("#Data]") {
                self.position += "#Data]".len();
                TableSpecifier::Data
            } else if rest_of_formula.starts_with("#Headers]") {
                self.position += "#Headers]".len();
                TableSpecifier::Headers
            } else if rest_of_formula.starts_with("#Totals]") {
                self.position += "#Totals]".len();
                TableSpecifier::Totals
            } else {
                return Err(LexerError {
                    position: self.position,
                    message: "Invalid structured reference".to_string(),
                });
            };
            Ok(Some(specifier))
        } else {
            Ok(None)
        }
    }

    fn consume_column_reference(&mut self) -> Result<String> {
        self.consume_whitespace();
        let end_char = if self.peek_char() == Some('[') {
            self.position += 1;
            ']'
        } else {
            ')'
        };

        let mut position = self.position;
        while position < self.len {
            let next_char = self.chars[position];
            if next_char != end_char {
                position += 1;
                if next_char == '\'' {
                    if position == self.len {
                        return Err(LexerError {
                            position: self.position,
                            message: "Invalid column name".to_string(),
                        });
                    }
                    // skip next char
                    position += 1
                }
            } else {
                break;
            }
        }
        let chars: String = self.chars[self.position..position].iter().collect();
        if end_char == ']' {
            if position >= self.len {
                // "[Price" with no closing bracket
                return Err(LexerError {
                    position: self.position,
                    message: "Invalid column name".to_string(),
                });
            }
            position += 1;
        }
        self.position = position;
        Ok(unescape_column_name(&chars))
    }

    /// What follows the '@' of `MyTable[@...]`, Excel's short form of `[#This Row]`.
    fn consume_this_row_reference(&mut self, table_name: &str) -> Result<TokenType> {
        let specifier = Some(TableSpecifier::ThisRow);
        let table_reference = match self.peek_char() {
            Some(']') => None,
            Some('[') => {
                let left = self.consume_column_reference()?;
                if self.peek_char() == Some(':') {
                    self.position += 1;
                    let right = self.consume_column_reference()?;
                    Some(TableReference::RangeReference((left, right)))
                } else {
                    Some(TableReference::ColumnReference(left))
                }
            }
            _ => Some(TableReference::ColumnReference(
                self.consume_bare_column_name()?,
            )),
        };
        self.expect(TokenType::RightBracket)?;
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
        })
    }

    /// A column name up to (not including) the closing ']', as in `[@Price]`.
    fn consume_bare_column_name(&mut self) -> Result<String> {
        let start = self.position;
        let mut position = self.position;
        while position < self.len && self.chars[position] != ']' {
            // a quote escapes the next character
            position += if self.chars[position] == '\'' { 2 } else { 1 };
        }
        if position >= self.len || position == start {
            return Err(LexerError {
                position: self.position,
                message: "Invalid column name".to_string(),
            });
        }
        let chars: String = self.chars[start..position].iter().collect();
        self.position = position;
        Ok(unescape_column_name(&chars))
    }

    /// A structured reference with no table name (`[@Price]`, `[Sales]`), which
    /// means the table the formula is in. The parser calls this after reading
    /// the '[' as a token.
    pub(crate) fn consume_implicit_structured_reference(&mut self) -> Result<TokenType> {
        self.position -= 1;
        self.consume_structured_reference("")
    }

    /// After a '[' token: the structured reference it starts, if it is one;
    /// otherwise nothing is consumed.
    pub(crate) fn try_implicit_structured_reference(&mut self) -> Option<TokenType> {
        let position = self.position;
        match self.consume_implicit_structured_reference() {
            Ok(token @ TokenType::StructuredReference { .. }) => Some(token),
            _ => {
                self.position = position;
                None
            }
        }
    }

    /// Possibilities:
    ///  1. MyTable[#Totals] or MyTable[#This Row]
    ///  2. MyTable[MyColumn]
    ///  3. MyTable[[My Column]]
    ///  4. MyTable[[#This Row], [My Column]]
    ///  5. MyTable[[#Totals], [MyColumn]]
    ///  6. MyTable[[#This Row], [Jan]:[Dec]]
    ///  7. MyTable[]
    ///
    /// Multiple specifiers are not supported yet:
    ///  1. MyTable[[#Data], [#Totals], [MyColumn]]
    ///
    /// In particular note that names of columns are escaped only when they are in the first argument
    /// We use '[' and ']'
    /// When there is only a specifier but not a reference the specifier is not in brackets
    ///
    /// Invalid:
    /// * MyTable[#Totals, [Jan]:[March]] => MyTable[[#Totals], [Jan]:[March]]
    //
    // NOTES:
    // * MyTable[[#Totals]] is translated into MyTable[#Totals]
    // * Excel shows '@' instead of '#This Row':
    //     MyTable[[#This Row], [Jan]:[Dec]] => MyTable[@[Jan]:[Dec]]
    //   Both forms are read (see `consume_this_row_reference`); formulas show '@'.
    pub(crate) fn consume_structured_reference(&mut self, table_name: &str) -> Result<TokenType> {
        self.expect(TokenType::LeftBracket)?;
        let peek_char = self.peek_char();
        if peek_char == Some('@') {
            // MyTable[@], MyTable[@Col], MyTable[@[My Col]], MyTable[@[Jan]:[Dec]]
            self.position += 1;
            return self.consume_this_row_reference(table_name);
        }
        if peek_char == Some(']') {
            // This is just a reference to the full table
            self.expect(TokenType::RightBracket)?;
            return Ok(TokenType::Ident(table_name.to_string()));
        }
        if peek_char == Some('#') {
            // Expecting MyTable[#Totals]
            if let Some(specifier) = self.consume_table_specifier()? {
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier: Some(specifier),
                    table_reference: None,
                });
            } else {
                return Err(LexerError {
                    position: self.position,
                    message: "Invalid structured reference".to_string(),
                });
            }
        } else if peek_char != Some('[') {
            // Expecting MyTable[MyColumn]
            self.position -= 1;
            let column_name = self.consume_column_reference()?;
            return Ok(TokenType::StructuredReference {
                table_name: table_name.to_string(),
                specifier: None,
                table_reference: Some(TableReference::ColumnReference(column_name)),
            });
        }
        self.expect(TokenType::LeftBracket)?;
        let specifier = self.consume_table_specifier()?;
        if specifier.is_some() {
            let peek_token = self.peek_token();
            if peek_token == TokenType::Comma {
                self.advance_token();
                self.expect(TokenType::LeftBracket)?;
            } else if peek_token == TokenType::RightBracket {
                return Ok(TokenType::StructuredReference {
                    table_name: table_name.to_string(),
                    specifier,
                    table_reference: None,
                });
            }
        }

        // Now it's either:
        // [Column Name]
        // [Column Name]:[Column Name]
        self.position -= 1;
        let column_reference = self.consume_column_reference()?;
        let table_reference = if self.peek_char() == Some(':') {
            self.position += 1;
            let column_reference_right = self.consume_column_reference()?;
            self.expect(TokenType::RightBracket)?;
            Some(TableReference::RangeReference((
                column_reference,
                column_reference_right,
            )))
        } else {
            self.expect(TokenType::RightBracket)?;
            Some(TableReference::ColumnReference(column_reference))
        };
        Ok(TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
        })
    }
}
