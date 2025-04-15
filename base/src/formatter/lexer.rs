pub struct Lexer {
    position: usize,
    len: usize,
    chars: Vec<char>,
    error_message: String,
    error_position: usize,
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Color(i32),              // [Red] or [Color 23]
    Condition(Compare, f64), // [<=100] (Comparator, number)
    Currency(char),          // [$€] ($ currency symbol)
    Literal(char), // €, $, (, ), /, :, +, -, ^, ', {, }, <, =, !, ~, > and space or scaped \X
    Spacer(char),  // *X
    Ghost(char),   // _X
    Text(String),  // "Text"
    Separator,     // ;
    Raw,           // @
    Percent,       // %
    Comma,         // ,
    Period,        // .
    Sharp,         // #
    Zero,          // 0
    QuestionMark,  // ?
    Scientific,    // E+
    ScientificMinus, // E-
    General,       // General
    // Dates
    Day,            // d
    DayPadded,      // dd
    DayNameShort,   // ddd
    DayName,        // dddd+
    Month,          // m
    MonthPadded,    // mm
    MonthNameShort, // mmm
    MonthName,      // mmmm or mmmmmm+
    MonthLetter,    // mmmmm
    YearShort,      // y or yy
    Year,           // yyy+
    // TODO: Hours Minutes and Seconds
    ILLEGAL,
    EOF,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Compare {
    Equal,
    LessThan,
    GreaterThan,
    LessOrEqualThan,
    GreaterOrEqualThan,
}

impl Token {
    pub fn is_digit(&self) -> bool {
        (self == &Token::Zero) || (self == &Token::Sharp) || (self == &Token::QuestionMark)
    }

    pub fn is_date(&self) -> bool {
        self == &Token::Day
            || self == &Token::DayPadded
            || self == &Token::DayNameShort
            || self == &Token::DayName
            || self == &Token::MonthName
            || self == &Token::MonthNameShort
            || self == &Token::Month
            || self == &Token::MonthPadded
            || self == &Token::MonthLetter
            || self == &Token::YearShort
            || self == &Token::Year
    }
}

impl Lexer {
    pub fn new(format: &str) -> Lexer {
        let chars: Vec<char> = format.chars().collect();
        let len = chars.len();
        Lexer {
            chars,
            position: 0,
            len,
            error_message: "".to_string(),
            error_position: 0,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            self.position = position + 1;
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn set_error(&mut self, error: &str) {
        self.error_message = error.to_string();
        self.error_position = self.position;
        self.position = self.len;
    }

    fn consume_string(&mut self) -> Option<String> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        while position < len {
            let x = self.chars[position];
            position += 1;
            if x != '"' {
                chars.push(x);
            } else if position < len && self.chars[position] == '"' {
                chars.push(x);
                chars.push(self.chars[position]);
                position += 1;
            } else {
                self.position = position;
                return Some(chars);
            }
        }
        None
    }

    fn consume_number(&mut self) -> Option<f64> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        // numbers before the '.'
        while position < len {
            let x = self.chars[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        if position < len && self.chars[position] == '.' {
            // numbers after the'.'
            chars.push('.');
            position += 1;
            while position < len {
                let x = self.chars[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
        if position + 1 < len && self.chars[position] == 'e' {
            // exponential side
            let x = self.chars[position + 1];
            if x == '-' || x == '+' || x.is_ascii_digit() {
                chars.push('e');
                chars.push(x);
                position += 2;
                while position < len {
                    let x = self.chars[position];
                    if x.is_ascii_digit() {
                        chars.push(x);
                    } else {
                        break;
                    }
                    position += 1;
                }
            }
        }
        self.position = position;
        chars.parse::<f64>().ok()
    }

    fn consume_condition(&mut self) -> Option<(Compare, f64)> {
        let cmp;
        match self.read_next_char() {
            Some('<') => {
                if let Some('=') = self.peek_char() {
                    self.read_next_char();
                    cmp = Compare::LessOrEqualThan;
                } else {
                    cmp = Compare::LessThan;
                }
            }
            Some('>') => {
                if let Some('=') = self.peek_char() {
                    self.read_next_char();
                    cmp = Compare::GreaterOrEqualThan;
                } else {
                    cmp = Compare::GreaterThan;
                }
            }
            Some('=') => {
                cmp = Compare::Equal;
            }
            _ => {
                return None;
            }
        }
        if let Some(v) = self.consume_number() {
            return Some((cmp, v));
        }
        None
    }

    fn consume_color(&mut self) -> Option<i32> {
        let colors = [
            "black", "white", "red", "green", "blue", "yellow", "magenta",
        ];
        let mut chars = "".to_string();
        while let Some(ch) = self.read_next_char() {
            if ch == ']' {
                if let Some(index) = colors.iter().position(|&x| x == chars.to_lowercase()) {
                    return Some(index as i32);
                }
                if !chars.starts_with("Color") {
                    return None;
                }
                if let Ok(index) = chars[5..].trim().parse::<i32>() {
                    if index < 57 && index > 0 {
                        return Some(index);
                    } else {
                        return None;
                    }
                }
                return None;
            } else {
                chars.push(ch);
            }
        }
        None
    }

    pub fn peek_token(&mut self) -> Token {
        let position = self.position;
        let token = self.next_token();
        self.position = position;
        token
    }

    pub fn next_token(&mut self) -> Token {
        let ch = self.read_next_char();
        match ch {
            Some(x) => match x {
                '$' | '€' | '(' | ')' | '/' | ':' | '+' | '-' | '^' | '\'' | '{' | '}' | '<'
                | '=' | '!' | '~' | '>' | ' ' => Token::Literal(x),
                '?' => Token::QuestionMark,
                ';' => Token::Separator,
                '#' => Token::Sharp,
                ',' => Token::Comma,
                '.' => Token::Period,
                '0' => Token::Zero,
                '@' => Token::Raw,
                '%' => Token::Percent,
                '[' => {
                    if let Some(c) = self.peek_char() {
                        if c == '<' || c == '>' || c == '=' {
                            // Condition
                            if let Some((cmp, value)) = self.consume_condition() {
                                Token::Condition(cmp, value)
                            } else {
                                self.set_error("Failed to parse condition");
                                Token::ILLEGAL
                            }
                        } else if c == '$' {
                            // currency
                            self.read_next_char();
                            if let Some(currency) = self.read_next_char() {
                                self.read_next_char();
                                return Token::Currency(currency);
                            }
                            self.set_error("Failed to parse currency");
                            Token::ILLEGAL
                        } else {
                            // Color
                            if let Some(index) = self.consume_color() {
                                return Token::Color(index);
                            }
                            self.set_error("Failed to parse color");
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '_' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Ghost(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '*' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Spacer(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '\\' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Literal(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '"' => {
                    if let Some(s) = self.consume_string() {
                        Token::Text(s)
                    } else {
                        self.set_error("Did not find end of text string");
                        Token::ILLEGAL
                    }
                }
                'E' => {
                    if let Some(s) = self.read_next_char() {
                        if s == '+' {
                            Token::Scientific
                        } else if s == '-' {
                            Token::ScientificMinus
                        } else {
                            self.set_error(&format!("Unexpected char: {}. Expected + or -", s));
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                'd' => {
                    let mut d = 1;
                    while let Some('d') = self.peek_char() {
                        d += 1;
                        self.read_next_char();
                    }
                    match d {
                        1 => Token::Day,
                        2 => Token::DayPadded,
                        3 => Token::DayNameShort,
                        _ => Token::DayName,
                    }
                }
                'm' => {
                    let mut m = 1;
                    while let Some('m') = self.peek_char() {
                        m += 1;
                        self.read_next_char();
                    }
                    match m {
                        1 => Token::Month,
                        2 => Token::MonthPadded,
                        3 => Token::MonthNameShort,
                        4 => Token::MonthName,
                        5 => Token::MonthLetter,
                        _ => Token::MonthName,
                    }
                }
                'y' => {
                    let mut y = 1;
                    while let Some('y') = self.peek_char() {
                        y += 1;
                        self.read_next_char();
                    }
                    if y == 1 || y == 2 {
                        Token::YearShort
                    } else {
                        Token::Year
                    }
                }
                'g' | 'G' => {
                    for c in "eneral".chars() {
                        let cc = self.read_next_char();
                        if Some(c) != cc {
                            self.set_error(&format!("Unexpected character: {}", x));
                            return Token::ILLEGAL;
                        }
                    }
                    Token::General
                }
                _ => {
                    self.set_error(&format!("Unexpected character: {}", x));
                    Token::ILLEGAL
                }
            },
            None => Token::EOF,
        }
    }
}

pub fn is_likely_date_number_format(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            return false;
        }
        if token.is_date() {
            return true;
        }
    }
}
