use crate::expressions::parser::{NamedVariable, Node, Parser};
use crate::expressions::token::TokenType;

impl<'a> Parser<'a> {
    // Called after `LAMBDA` and the opening `(` have been consumed.
    // Parses:  param*, body ')' ['(' call_args ')']
    // Returns LambdaDefKind, or LambdaCallKind if immediately invoked.
    pub(crate) fn parse_lambda(&mut self) -> Node {
        let args = match self.parse_function_args() {
            Ok(s) => s,
            Err(e) => return e,
        };

        if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: err.position,
                message: err.message,
            };
        }

        if args.is_empty() {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: self.lexer.get_position() as usize,
                message: "LAMBDA requires at least one argument (the body)".to_string(),
            };
        }

        // All args except the last are parameter name declarations; the last is the body.
        let body = args.last().unwrap().clone();
        let param_nodes = &args[..args.len() - 1];

        let mut parameters = Vec::with_capacity(param_nodes.len());
        for param in param_nodes {
            match param {
                Node::NamedVariableKind { name, id } => {
                    // Strip the _xlpm. prefix that Excel uses in serialised XLSX.
                    let clean = name.trim_start_matches("_xlpm.").to_string();
                    parameters.push(NamedVariable {
                        name: clean,
                        id: *id,
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
        }

        let def = Node::LambdaDefKind {
            parameters,
            body: Box::new(body),
        };

        // Immediate invocation syntax: LAMBDA(params, body)(call_args)
        if self.lexer.peek_token() == TokenType::LeftParenthesis {
            self.lexer.advance_token();
            let call_args = match self.parse_function_args() {
                Ok(s) => s,
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
