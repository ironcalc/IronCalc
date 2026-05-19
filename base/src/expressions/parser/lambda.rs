use crate::expressions::parser::{NamedVariable, Node, Parser};
use crate::expressions::token::TokenType;

/// Recursively strips the `_xlpm.` prefix from every `NamedVariableKind` name in `node`.
/// This converts XLSX-style body references (`_xlpm.x`) to clean names (`x`) so the
/// internal representation is prefix-free regardless of whether the formula came from a
/// hand-typed formula or was imported from an xlsx file.
fn strip_xlpm_prefix_in_body(node: &mut Node) {
    match node {
        Node::NamedVariableKind { name, .. } => {
            if let Some(clean) = name.strip_prefix("_xlpm.") {
                *name = clean.to_string();
            }
        }
        Node::FunctionKind { args, .. } => {
            for a in args.iter_mut() {
                strip_xlpm_prefix_in_body(a);
            }
        }
        Node::NamedFunctionKind { args, .. } => {
            for a in args.iter_mut() {
                strip_xlpm_prefix_in_body(a);
            }
        }
        Node::LambdaDefKind { body, .. } => strip_xlpm_prefix_in_body(body),
        Node::LambdaCallKind { lambda, args } => {
            strip_xlpm_prefix_in_body(lambda);
            for a in args.iter_mut() {
                strip_xlpm_prefix_in_body(a);
            }
        }
        Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::OpRangeKind { left, right } => {
            strip_xlpm_prefix_in_body(left);
            strip_xlpm_prefix_in_body(right);
        }
        Node::CompareKind { left, right, .. } => {
            strip_xlpm_prefix_in_body(left);
            strip_xlpm_prefix_in_body(right);
        }
        Node::UnaryKind { right, .. } => strip_xlpm_prefix_in_body(right),
        Node::ImplicitIntersection { child, .. } => strip_xlpm_prefix_in_body(child),
        _ => {}
    }
}

impl<'a> Parser<'a> {
    // Called after `LAMBDA` and the opening `(` have been consumed.
    // Parses:  (param | '[' param ']')*, body ')' ['(' call_args ')']
    // Returns LambdaDefKind, or LambdaCallKind if immediately invoked.
    //
    // Parameter names are always stored clean (no _xlpm./_xlop. prefix).
    // Optional parameters come from either [name] bracket syntax or the _xlop. XLSX prefix.
    // Body variable references are also stored clean (the _xlpm. prefix is stripped).
    pub(crate) fn parse_lambda(&mut self) -> Node {
        let arg_separator = self.get_argument_separator_token();

        if self.lexer.peek_token() == TokenType::RightParenthesis {
            self.lexer.advance_token();
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: self.lexer.get_position() as usize,
                message: "LAMBDA requires at least one argument (the body)".to_string(),
            };
        }

        let mut parameters: Vec<NamedVariable> = Vec::new();

        // Parse items one by one. Items followed by the separator are parameters (must
        // be plain names or [name]). The final item before ')' is the body expression.
        let mut body = loop {
            // Optional parameter syntax: [name]
            let bracket_optional = self.lexer.peek_token() == TokenType::LeftBracket;
            if bracket_optional {
                self.lexer.advance_token(); // consume '['
            }

            let expr = self.parse_expr();
            if let Node::ParseErrorKind { .. } = expr {
                return expr;
            }

            if bracket_optional {
                if let Err(err) = self.lexer.expect(TokenType::RightBracket) {
                    return Node::ParseErrorKind {
                        formula: self.lexer.get_formula(),
                        position: err.position,
                        message: "Expected ']' after optional parameter name".to_string(),
                    };
                }
            }

            let next = self.lexer.peek_token();
            if next == arg_separator {
                // This item must be a parameter name.
                self.lexer.advance_token(); // consume separator
                match expr {
                    Node::NamedVariableKind { name, id } => {
                        // Strip XLSX prefixes. _xlop. marks optional; both prefixes are
                        // removed so the stored name is always clean.
                        let (clean, is_optional) = if let Some(s) = name.strip_prefix("_xlop.") {
                            (s.to_string(), true)
                        } else if let Some(s) = name.strip_prefix("_xlpm.") {
                            (s.to_string(), bracket_optional)
                        } else {
                            (name, bracket_optional)
                        };
                        parameters.push(NamedVariable {
                            name: clean,
                            id,
                            is_optional,
                        });
                    }
                    _ => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: self.lexer.get_position() as usize,
                            message: "LAMBDA parameter must be a name".to_string(),
                        };
                    }
                }
            } else if next == TokenType::RightParenthesis {
                self.lexer.advance_token(); // consume ')'
                break expr;
            } else {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: self.lexer.get_position() as usize,
                    message: "Expected ',' or ')' in LAMBDA".to_string(),
                };
            }
        };

        // Strip _xlpm. prefix from variable references in the body so the internal
        // representation is always prefix-free (matches the clean param names above).
        strip_xlpm_prefix_in_body(&mut body);

        let def = Node::LambdaDefKind {
            parameters,
            body: Box::new(body),
        };

        // Immediate invocation: LAMBDA(params, body)(call_args)
        if self.lexer.peek_token() == TokenType::LeftParenthesis {
            self.lexer.advance_token(); // consume '('
            let call_args = match self.parse_function_args() {
                Ok(args) => args,
                Err(e) => return e,
            };
            if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: err.position,
                    message: err.message,
                };
            }
            return Node::LambdaCallKind {
                lambda: Box::new(def),
                args: call_args,
            };
        }

        def
    }
}
