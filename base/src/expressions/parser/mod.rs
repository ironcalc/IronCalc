/*!
# GRAMMAR

<pre class="rust">
opComp   => '=' | '<' | '>' | '<=' } '>=' | '<>'
opFactor => '*' | '/'
unaryOp  => '-' | '+'

expr    => concat (opComp concat)*
concat  => term ('&' term)*
term    => factor (opFactor factor)*
factor  => prod (opProd prod)*
prod    => power ('^' power)*
power   => (unaryOp)* range '%'*
range   => implicit (':' primary)?
implicit=> '@' primary | primary
primary => '(' expr ')'
        => number
        => function '(' f_args ')'
        => name
        => string
        => '{' a_args '}'
        => bool
        => bool()
        => error

f_args  => e (',' e)*
</pre>
*/

use std::collections::HashMap;

use crate::functions::Function;
use crate::language::get_language;
use crate::locale::get_locale;
use crate::types::Table;

use super::lexer;
use super::token;
use super::token::OpUnary;
use super::token::TableReference;
use super::token::TokenType;
use super::types::*;
use super::utils::number_to_column;

use token::OpCompare;

pub mod move_formula;
pub mod static_analysis;
pub mod stringify;

#[cfg(test)]
mod tests;

pub(crate) fn parse_range(formula: &str) -> Result<(i32, i32, i32, i32), String> {
    let mut lexer = lexer::Lexer::new(
        formula,
        lexer::LexerMode::A1,
        #[allow(clippy::expect_used)]
        get_locale("en").expect(""),
        #[allow(clippy::expect_used)]
        get_language("en").expect(""),
    );
    if let TokenType::Range {
        left,
        right,
        sheet: _,
    } = lexer.next_token()
    {
        Ok((left.column, left.row, right.column, right.row))
    } else {
        Err("Not a range".to_string())
    }
}

fn get_table_column_by_name(table_column_name: &str, table: &Table) -> Option<i32> {
    for (index, table_column) in table.columns.iter().enumerate() {
        if table_column.name == table_column_name {
            return Some(index as i32);
        }
    }
    None
}

// DefinedNameS is a tuple with the name of the defined name, the index of the sheet and the formula
pub type DefinedNameS = (String, Option<u32>, String);

pub(crate) struct Reference<'a> {
    sheet_name: &'a Option<String>,
    sheet_index: u32,
    absolute_row: bool,
    absolute_column: bool,
    row: i32,
    column: i32,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ArrayNode {
    Boolean(bool),
    Number(f64),
    String(String),
    Error(token::Error),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Node {
    BooleanKind(bool),
    NumberKind(f64),
    StringKind(String),
    ReferenceKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    RangeKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    WrongReferenceKind {
        sheet_name: Option<String>,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    WrongRangeKind {
        sheet_name: Option<String>,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    OpRangeKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    OpConcatenateKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    OpSumKind {
        kind: token::OpSum,
        left: Box<Node>,
        right: Box<Node>,
    },
    OpProductKind {
        kind: token::OpProduct,
        left: Box<Node>,
        right: Box<Node>,
    },
    OpPowerKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    FunctionKind {
        kind: Function,
        args: Vec<Node>,
    },
    InvalidFunctionKind {
        name: String,
        args: Vec<Node>,
    },
    ArrayKind(Vec<Vec<ArrayNode>>),
    DefinedNameKind(DefinedNameS),
    TableNameKind(String),
    WrongVariableKind(String),
    ImplicitIntersection {
        automatic: bool,
        child: Box<Node>,
    },
    CompareKind {
        kind: OpCompare,
        left: Box<Node>,
        right: Box<Node>,
    },
    UnaryKind {
        kind: OpUnary,
        right: Box<Node>,
    },
    ErrorKind(token::Error),
    ParseErrorKind {
        formula: String,
        message: String,
        position: usize,
    },
    EmptyArgKind,
}

#[derive(Clone)]
pub struct Parser {
    lexer: lexer::Lexer,
    worksheets: Vec<String>,
    defined_names: Vec<DefinedNameS>,
    context: CellReferenceRC,
    tables: HashMap<String, Table>,
}

impl Parser {
    pub fn new(
        worksheets: Vec<String>,
        defined_names: Vec<DefinedNameS>,
        tables: HashMap<String, Table>,
    ) -> Parser {
        let lexer = lexer::Lexer::new(
            "",
            lexer::LexerMode::A1,
            #[allow(clippy::expect_used)]
            get_locale("en").expect(""),
            #[allow(clippy::expect_used)]
            get_language("en").expect(""),
        );
        let context = CellReferenceRC {
            sheet: worksheets.first().map_or("", |v| v).to_string(),
            column: 1,
            row: 1,
        };
        Parser {
            lexer,
            worksheets,
            defined_names,
            context,
            tables,
        }
    }
    pub fn set_lexer_mode(&mut self, mode: lexer::LexerMode) {
        self.lexer.set_lexer_mode(mode)
    }

    pub fn set_worksheets_and_names(
        &mut self,
        worksheets: Vec<String>,
        defined_names: Vec<DefinedNameS>,
    ) {
        self.worksheets = worksheets;
        self.defined_names = defined_names;
    }

    pub fn parse(&mut self, formula: &str, context: &CellReferenceRC) -> Node {
        self.lexer.set_formula(formula);
        self.context = context.clone();
        self.parse_expr()
    }

    fn get_sheet_index_by_name(&self, name: &str) -> Option<u32> {
        let worksheets = &self.worksheets;
        for (i, sheet) in worksheets.iter().enumerate() {
            if sheet == name {
                return Some(i as u32);
            }
        }
        None
    }

    // Returns:
    //  * None: If there is no defined name by that name
    //  * Some((Some(index), formula)): If there is a defined name local to that sheet
    //  * Some(None): If there is a global defined name
    fn get_defined_name(&self, name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
        for (df_name, df_scope, df_formula) in &self.defined_names {
            if name.to_lowercase() == df_name.to_lowercase() && df_scope == &Some(sheet) {
                return Some((*df_scope, df_formula.to_owned()));
            }
        }
        for (df_name, df_scope, df_formula) in &self.defined_names {
            if name.to_lowercase() == df_name.to_lowercase() && df_scope.is_none() {
                return Some((None, df_formula.to_owned()));
            }
        }
        None
    }

    fn parse_expr(&mut self) -> Node {
        let mut t = self.parse_concat();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut next_token = self.lexer.peek_token();
        while let TokenType::Compare(op) = next_token {
            self.lexer.advance_token();
            let p = self.parse_concat();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            t = Node::CompareKind {
                kind: op,
                left: Box::new(t),
                right: Box::new(p),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_concat(&mut self) -> Node {
        let mut t = self.parse_term();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut next_token = self.lexer.peek_token();
        while next_token == TokenType::And {
            self.lexer.advance_token();
            let p = self.parse_term();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            t = Node::OpConcatenateKind {
                left: Box::new(t),
                right: Box::new(p),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_term(&mut self) -> Node {
        let mut t = self.parse_factor();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut next_token = self.lexer.peek_token();
        while let TokenType::Addition(op) = next_token {
            self.lexer.advance_token();
            let p = self.parse_factor();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            t = Node::OpSumKind {
                kind: op,
                left: Box::new(t),
                right: Box::new(p),
            };

            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_factor(&mut self) -> Node {
        let mut t = self.parse_prod();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut next_token = self.lexer.peek_token();
        while let TokenType::Product(op) = next_token {
            self.lexer.advance_token();
            let p = self.parse_prod();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            t = Node::OpProductKind {
                kind: op,
                left: Box::new(t),
                right: Box::new(p),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_prod(&mut self) -> Node {
        let mut t = self.parse_power();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let mut next_token = self.lexer.peek_token();
        while next_token == TokenType::Power {
            self.lexer.advance_token();
            let p = self.parse_power();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            t = Node::OpPowerKind {
                left: Box::new(t),
                right: Box::new(p),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_power(&mut self) -> Node {
        let mut next_token = self.lexer.peek_token();
        let mut sign = 1;
        while let TokenType::Addition(op) = next_token {
            self.lexer.advance_token();
            if op == token::OpSum::Minus {
                sign = -sign;
            }
            next_token = self.lexer.peek_token();
        }

        let mut t = self.parse_range();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        if sign == -1 {
            t = Node::UnaryKind {
                kind: token::OpUnary::Minus,
                right: Box::new(t),
            }
        }
        next_token = self.lexer.peek_token();
        while next_token == TokenType::Percent {
            self.lexer.advance_token();
            t = Node::UnaryKind {
                kind: token::OpUnary::Percentage,
                right: Box::new(t),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_range(&mut self) -> Node {
        let t = self.parse_implicit();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::Colon {
            self.lexer.advance_token();
            let p = self.parse_primary();
            if let Node::ParseErrorKind { .. } = p {
                return p;
            }
            return Node::OpRangeKind {
                left: Box::new(t),
                right: Box::new(p),
            };
        }
        t
    }

    fn parse_implicit(&mut self) -> Node {
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::At {
            self.lexer.advance_token();
            let t = self.parse_primary();
            if let Node::ParseErrorKind { .. } = t {
                return t;
            }
            return Node::ImplicitIntersection {
                automatic: false,
                child: Box::new(t),
            };
        }
        self.parse_primary()
    }

    fn parse_array_row(&mut self) -> Result<Vec<ArrayNode>, Node> {
        let mut row = Vec::new();
        // and array can only have numbers, string or booleans
        // otherwise it is a syntax error
        let first_element = match self.parse_expr() {
            Node::BooleanKind(s) => ArrayNode::Boolean(s),
            Node::NumberKind(s) => ArrayNode::Number(s),
            Node::StringKind(s) => ArrayNode::String(s),
            Node::ErrorKind(kind) => ArrayNode::Error(kind),
            error @ Node::ParseErrorKind { .. } => return Err(error),
            _ => {
                return Err(Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    message: "Invalid value in array".to_string(),
                    position: self.lexer.get_position() as usize,
                });
            }
        };
        row.push(first_element);
        let mut next_token = self.lexer.peek_token();
        // FIXME: this is not respecting the locale
        while next_token == TokenType::Comma {
            self.lexer.advance_token();
            let value = match self.parse_expr() {
                Node::BooleanKind(s) => ArrayNode::Boolean(s),
                Node::NumberKind(s) => ArrayNode::Number(s),
                Node::StringKind(s) => ArrayNode::String(s),
                Node::ErrorKind(kind) => ArrayNode::Error(kind),
                error @ Node::ParseErrorKind { .. } => return Err(error),
                _ => {
                    return Err(Node::ParseErrorKind {
                        formula: self.lexer.get_formula(),
                        message: "Invalid value in array".to_string(),
                        position: self.lexer.get_position() as usize,
                    });
                }
            };
            row.push(value);
            next_token = self.lexer.peek_token();
        }
        Ok(row)
    }

    fn parse_primary(&mut self) -> Node {
        let next_token = self.lexer.next_token();
        match next_token {
            TokenType::LeftParenthesis => {
                let t = self.parse_expr();
                if let Node::ParseErrorKind { .. } = t {
                    return t;
                }

                if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
                    return Node::ParseErrorKind {
                        formula: self.lexer.get_formula(),
                        position: err.position,
                        message: err.message,
                    };
                }
                t
            }
            TokenType::Number(s) => Node::NumberKind(s),
            TokenType::String(s) => Node::StringKind(s),
            TokenType::LeftBrace => {
                // It's an array. It's a collection of rows all of the same dimension

                let first_row = match self.parse_array_row() {
                    Ok(s) => s,
                    Err(error) => return error,
                };
                let length = first_row.len();

                let mut matrix = Vec::new();
                matrix.push(first_row);
                // FIXME: this is not respecting the locale
                let mut next_token = self.lexer.peek_token();
                while next_token == TokenType::Semicolon {
                    self.lexer.advance_token();
                    let row = match self.parse_array_row() {
                        Ok(s) => s,
                        Err(error) => return error,
                    };
                    next_token = self.lexer.peek_token();
                    if row.len() != length {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: self.lexer.get_position() as usize,
                            message: "All rows in an array should be the same length".to_string(),
                        };
                    }
                    matrix.push(row);
                }

                if let Err(err) = self.lexer.expect(TokenType::RightBrace) {
                    return Node::ParseErrorKind {
                        formula: self.lexer.get_formula(),
                        position: err.position,
                        message: err.message,
                    };
                }
                Node::ArrayKind(matrix)
            }
            TokenType::Reference {
                sheet,
                row,
                column,
                absolute_column,
                absolute_row,
            } => {
                let context = &self.context;
                let sheet_index = match &sheet {
                    Some(name) => self.get_sheet_index_by_name(name),
                    None => self.get_sheet_index_by_name(&context.sheet),
                };
                let a1_mode = self.lexer.is_a1_mode();
                let row = if absolute_row || !a1_mode {
                    row
                } else {
                    row - context.row
                };
                let column = if absolute_column || !a1_mode {
                    column
                } else {
                    column - context.column
                };
                match sheet_index {
                    Some(index) => Node::ReferenceKind {
                        sheet_name: sheet,
                        sheet_index: index,
                        row,
                        column,
                        absolute_row,
                        absolute_column,
                    },
                    None => Node::WrongReferenceKind {
                        sheet_name: sheet,
                        row,
                        column,
                        absolute_row,
                        absolute_column,
                    },
                }
            }
            TokenType::Range { sheet, left, right } => {
                let context = &self.context;
                let sheet_index = match &sheet {
                    Some(name) => self.get_sheet_index_by_name(name),
                    None => self.get_sheet_index_by_name(&context.sheet),
                };
                let mut row1 = left.row;
                let mut column1 = left.column;
                let mut row2 = right.row;
                let mut column2 = right.column;

                let mut absolute_column1 = left.absolute_column;
                let mut absolute_column2 = right.absolute_column;
                let mut absolute_row1 = left.absolute_row;
                let mut absolute_row2 = right.absolute_row;

                if row1 > row2 {
                    (row2, row1) = (row1, row2);
                    (absolute_row2, absolute_row1) = (absolute_row1, absolute_row2);
                }
                if column1 > column2 {
                    (column2, column1) = (column1, column2);
                    (absolute_column2, absolute_column1) = (absolute_column1, absolute_column2);
                }

                if self.lexer.is_a1_mode() {
                    if !absolute_row1 {
                        row1 -= context.row
                    };
                    if !absolute_column1 {
                        column1 -= context.column
                    };
                    if !absolute_row2 {
                        row2 -= context.row
                    };
                    if !absolute_column2 {
                        column2 -= context.column
                    };
                }

                match sheet_index {
                    Some(index) => Node::RangeKind {
                        sheet_name: sheet,
                        sheet_index: index,
                        row1,
                        column1,
                        row2,
                        column2,
                        absolute_column1,
                        absolute_column2,
                        absolute_row1,
                        absolute_row2,
                    },
                    None => Node::WrongRangeKind {
                        sheet_name: sheet,
                        row1,
                        column1,
                        row2,
                        column2,
                        absolute_column1,
                        absolute_column2,
                        absolute_row1,
                        absolute_row2,
                    },
                }
            }
            TokenType::Ident(name) => {
                let next_token = self.lexer.peek_token();
                if next_token == TokenType::LeftParenthesis {
                    // It's a function call "SUM(.."
                    self.lexer.advance_token();
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
                    if let Some(function_kind) = Function::get_function(&name) {
                        return Node::FunctionKind {
                            kind: function_kind,
                            args,
                        };
                    }
                    if &name == "_xlfn.SINGLE" {
                        if args.len() != 1 {
                            return Node::ParseErrorKind {
                                formula: self.lexer.get_formula(),
                                position: self.lexer.get_position() as usize,
                                message: "Implicit Intersection requires just one argument"
                                    .to_string(),
                            };
                        }
                        return Node::ImplicitIntersection {
                            automatic: false,
                            child: Box::new(args[0].clone()),
                        };
                    }
                    return Node::InvalidFunctionKind { name, args };
                }
                let context = &self.context;

                let context_sheet_index = match self.get_sheet_index_by_name(&context.sheet) {
                    Some(i) => i,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: 0,
                            message: "sheet not found".to_string(),
                        };
                    }
                };

                // Could be a defined name or a table
                if let Some((scope, formula)) = self.get_defined_name(&name, context_sheet_index) {
                    return Node::DefinedNameKind((name, scope, formula));
                }
                let name_lower = name.to_lowercase();
                for table_name in self.tables.keys() {
                    if table_name.to_lowercase() == name_lower {
                        return Node::TableNameKind(name);
                    }
                }
                Node::WrongVariableKind(name)
            }
            TokenType::Error(kind) => Node::ErrorKind(kind),
            TokenType::Illegal(error) => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: error.position,
                message: error.message,
            },
            TokenType::EOF => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: 0,
                message: "Unexpected end of input.".to_string(),
            },
            TokenType::Boolean(value) => {
                // Could be a function call "TRUE()"
                let next_token = self.lexer.peek_token();
                if next_token == TokenType::LeftParenthesis {
                    self.lexer.advance_token();
                    // We parse all the arguments, although technically this is moot
                    // But is has the upside of transforming `=TRUE( 4 )` into `=TRUE(4)`
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
                    if value {
                        return Node::FunctionKind {
                            kind: Function::True,
                            args,
                        };
                    } else {
                        return Node::FunctionKind {
                            kind: Function::False,
                            args,
                        };
                    }
                }
                Node::BooleanKind(value)
            }
            TokenType::Compare(_) => {
                // A primary Node cannot start with an operator
                Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: 0,
                    message: "Unexpected token: 'COMPARE'".to_string(),
                }
            }
            TokenType::Addition(_) => {
                // A primary Node cannot start with an operator
                Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: 0,
                    message: "Unexpected token: 'SUM'".to_string(),
                }
            }
            TokenType::Product(_) => {
                // A primary Node cannot start with an operator
                Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: 0,
                    message: "Unexpected token: 'PRODUCT'".to_string(),
                }
            }
            TokenType::Power => {
                // A primary Node cannot start with an operator
                Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: 0,
                    message: "Unexpected token: 'POWER'".to_string(),
                }
            }
            TokenType::At => {
                // A primary Node cannot start with an operator
                Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    position: 0,
                    message: "Unexpected token: '@'".to_string(),
                }
            }
            TokenType::RightParenthesis
            | TokenType::RightBracket
            | TokenType::Colon
            | TokenType::Semicolon
            | TokenType::RightBrace
            | TokenType::Comma
            | TokenType::Bang
            | TokenType::And
            | TokenType::Percent => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: 0,
                message: format!("Unexpected token: '{:?}'", next_token),
            },
            TokenType::LeftBracket => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                position: 0,
                message: "Unexpected token: '['".to_string(),
            },
            TokenType::StructuredReference {
                table_name,
                specifier,
                table_reference,
            } => {
                // We will try to convert to a normal reference
                // table_name[column_name] => cell1:cell2
                // table_name[[#This Row], [column_name]:[column_name]] => cell1:cell2
                let context = &self.context;
                let context_sheet_index = match self.get_sheet_index_by_name(&context.sheet) {
                    Some(i) => i,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: 0,
                            message: "sheet not found".to_string(),
                        };
                    }
                };
                // table-name => table
                let table = match self.tables.get(&table_name) {
                    Some(t) => t,
                    None => {
                        let message = format!(
                            "Table not found: '{table_name}' at '{}!{}{}'",
                            context.sheet,
                            number_to_column(context.column)
                                .unwrap_or(format!("{}", context.column)),
                            context.row
                        );
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: 0,
                            message,
                        };
                    }
                };
                let table_sheet_index = match self.get_sheet_index_by_name(&table.sheet_name) {
                    Some(i) => i,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            position: 0,
                            message: "sheet not found".to_string(),
                        };
                    }
                };

                let sheet_name = if table_sheet_index == context_sheet_index {
                    None
                } else {
                    Some(table.sheet_name.clone())
                };

                // context must be with tables.reference
                #[allow(clippy::expect_used)]
                let (column_start, mut row_start, column_end, mut row_end) =
                    parse_range(&table.reference).expect("Failed parsing range");

                let totals_row_count = table.totals_row_count as i32;
                let header_row_count = table.header_row_count as i32;
                row_end -= totals_row_count;

                match specifier {
                    Some(token::TableSpecifier::ThisRow) => {
                        row_start = context.row;
                        row_end = context.row;
                    }
                    Some(token::TableSpecifier::Totals) => {
                        if totals_row_count != 0 {
                            row_start = row_end + 1;
                            row_end = row_start;
                        } else {
                            // Table1[#Totals] is #REF! if Table1 does not have totals
                            return Node::ErrorKind(token::Error::REF);
                        }
                    }
                    Some(token::TableSpecifier::Headers) => {
                        row_end = row_start;
                    }
                    Some(token::TableSpecifier::Data) => {
                        row_start += header_row_count;
                    }
                    Some(token::TableSpecifier::All) => {
                        if totals_row_count != 0 {
                            row_end += 1;
                        }
                    }
                    None => {
                        // skip the headers
                        row_start += header_row_count;
                    }
                }
                match table_reference {
                    None => Node::RangeKind {
                        sheet_name,
                        sheet_index: table_sheet_index,
                        absolute_row1: true,
                        absolute_column1: true,
                        row1: row_start,
                        column1: column_start,
                        absolute_row2: true,
                        absolute_column2: true,
                        row2: row_end,
                        column2: column_end,
                    },
                    Some(TableReference::ColumnReference(s)) => {
                        let column_index = match get_table_column_by_name(&s, table) {
                            Some(s) => s + column_start,
                            None => {
                                return Node::ParseErrorKind {
                                    formula: self.lexer.get_formula(),
                                    position: self.lexer.get_position() as usize,
                                    message: format!("Expecting column: {s} in table {table_name}"),
                                };
                            }
                        };
                        if row_start == row_end {
                            return Node::ReferenceKind {
                                sheet_name,
                                sheet_index: table_sheet_index,
                                absolute_row: true,
                                absolute_column: true,
                                row: row_start,
                                column: column_index,
                            };
                        }
                        Node::RangeKind {
                            sheet_name,
                            sheet_index: table_sheet_index,
                            absolute_row1: true,
                            absolute_column1: true,
                            row1: row_start,
                            column1: column_index,
                            absolute_row2: true,
                            absolute_column2: true,
                            row2: row_end,
                            column2: column_index,
                        }
                    }
                    Some(TableReference::RangeReference((left, right))) => {
                        let left_column_index = match get_table_column_by_name(&left, table) {
                            Some(f) => f + column_start,
                            None => {
                                return Node::ParseErrorKind {
                                    formula: self.lexer.get_formula(),
                                    position: self.lexer.get_position() as usize,
                                    message: format!(
                                        "Expecting column: {left} in table {table_name}"
                                    ),
                                };
                            }
                        };

                        let right_column_index = match get_table_column_by_name(&right, table) {
                            Some(f) => f + column_start,
                            None => {
                                return Node::ParseErrorKind {
                                    formula: self.lexer.get_formula(),
                                    position: self.lexer.get_position() as usize,
                                    message: format!(
                                        "Expecting column: {right} in table {table_name}"
                                    ),
                                };
                            }
                        };
                        Node::RangeKind {
                            sheet_name,
                            sheet_index: table_sheet_index,
                            absolute_row1: true,
                            absolute_column1: true,
                            row1: row_start,
                            column1: left_column_index,
                            absolute_row2: true,
                            absolute_column2: true,
                            row2: row_end,
                            column2: right_column_index,
                        }
                    }
                }
            }
        }
    }

    fn parse_function_args(&mut self) -> Result<Vec<Node>, Node> {
        let mut args: Vec<Node> = Vec::new();
        let mut next_token = self.lexer.peek_token();
        if next_token == TokenType::RightParenthesis {
            return Ok(args);
        }
        if self.lexer.peek_token() == TokenType::Comma {
            args.push(Node::EmptyArgKind);
        } else {
            let t = self.parse_expr();
            if let Node::ParseErrorKind { .. } = t {
                return Err(t);
            }
            args.push(t);
        }
        next_token = self.lexer.peek_token();
        while next_token == TokenType::Comma {
            self.lexer.advance_token();
            if self.lexer.peek_token() == TokenType::Comma {
                args.push(Node::EmptyArgKind);
                next_token = TokenType::Comma;
            } else if self.lexer.peek_token() == TokenType::RightParenthesis {
                args.push(Node::EmptyArgKind);
                return Ok(args);
            } else {
                let p = self.parse_expr();
                if let Node::ParseErrorKind { .. } = p {
                    return Err(p);
                }
                next_token = self.lexer.peek_token();
                args.push(p);
            }
        }
        Ok(args)
    }
}
