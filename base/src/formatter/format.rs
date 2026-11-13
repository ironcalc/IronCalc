use chrono::Datelike;

use crate::{locale::Locale, number_format::parse_finite_number};

use super::{
    dates::{date_to_serial_number, from_excel_date},
    parser::{ParsePart, Parser, TextToken},
};

pub struct Formatted {
    pub color: Option<i32>,
    pub text: String,
    pub error: Option<String>,
}

/// The absolute value of a number as Excel shows it.
///
/// Excel keeps 15 significant digits of a number, and when it shows the number
/// it rounds that decimal half away from zero, not the binary number underneath:
/// 1.005 is 1.00499999999999989... in binary, but its 15 digits are
/// 1.00500000000000, so "0.00" shows 1.01.
///
/// The number is 0.d1d2d3... × 10^point: `digits` are d1, d2, d3, ... without
/// trailing zeros, and empty for zero. 1.25 is [1, 2, 5] with point 1 and
/// 0.0125 is [1, 2, 5] with point -1.
struct Digits {
    digits: Vec<u8>,
    point: i32,
}

impl Digits {
    fn new(value: f64) -> Digits {
        let value = value.abs();
        if value == 0.0 || !value.is_finite() {
            return Digits {
                digits: vec![],
                point: 0,
            };
        }
        // 15 significant digits: "1.23450000000000e4"
        let formatted = format!("{value:.14e}");
        let (mantissa, exponent) = formatted.split_once('e').unwrap_or((&formatted, "0"));
        let mut digits: Vec<u8> = mantissa
            .bytes()
            .filter(u8::is_ascii_digit)
            .map(|c| c - b'0')
            .collect();
        while digits.last() == Some(&0) {
            digits.pop();
        }
        Digits {
            digits,
            point: exponent.parse::<i32>().unwrap_or(0) + 1,
        }
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Multiplies the number by 10^n
    fn shift(&mut self, n: i32) {
        if !self.is_zero() {
            self.point += n;
        }
    }

    /// Rounds the number to `decimals` digits after the decimal point, half away from zero
    fn round(&mut self, decimals: i32) {
        let keep = self.point + decimals;
        if keep >= self.digits.len() as i32 {
            return;
        }
        // The first digit that goes is digits[keep], or a zero before d1
        let round_up = keep >= 0 && self.digits[keep as usize] >= 5;
        self.digits.truncate(keep.max(0) as usize);
        if round_up {
            let mut carry = true;
            for digit in self.digits.iter_mut().rev() {
                if *digit == 9 {
                    *digit = 0;
                } else {
                    *digit += 1;
                    carry = false;
                    break;
                }
            }
            if carry {
                // 0.96 to 1.0
                self.digits.insert(0, 1);
                self.point += 1;
            }
        }
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        if self.is_zero() {
            self.point = 0;
        }
    }

    /// The digits before the decimal point, none if it is zero: 1234.5 is ['1', '2', '3', '4']
    fn integer_part(&self) -> Vec<char> {
        (0..self.point).map(|i| self.digit(i)).collect()
    }

    /// The digits after the decimal point, without trailing zeros: 3.05 is ['0', '5']
    fn fractional_part(&self) -> Vec<char> {
        (self.point..self.digits.len() as i32)
            .map(|i| self.digit(i))
            .collect()
    }

    fn digit(&self, index: i32) -> char {
        let digit = if index < 0 {
            0
        } else {
            self.digits.get(index as usize).copied().unwrap_or(0)
        };
        char::from(b'0' + digit)
    }
}

/// Return true if we need to add a separator in position digit_index
/// It normally happens if if digit_index -1 is 3, 6, 9,... digit_index ≡ 1 mod 3
fn use_group_separator(use_thousands: bool, digit_index: i32, group_sizes: &str) -> bool {
    if use_thousands {
        if group_sizes == "#,##0.###" {
            if digit_index > 1 && (digit_index - 1) % 3 == 0 {
                return true;
            }
        } else if group_sizes == "#,##,##0.###"
            && (digit_index == 3 || (digit_index > 3 && digit_index % 2 == 0))
        {
            return true;
        }
    }
    false
}

pub fn format_number(value_original: f64, format: &str, locale: &Locale) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts;
    // There are four parts:
    // 1) Positive numbers
    // 2) Negative numbers
    // 3) Zero
    // 4) Text
    // If you specify only one section of format code, the code in that section is used for all numbers.
    // If you specify two sections of format code, the first section of code is used
    // for positive numbers and zeros, and the second section of code is used for negative numbers.
    // When you skip code sections in your number format,
    // you must include a semicolon for each of the missing sections of code.
    // You can use the ampersand (&) text operator to join, or concatenate, two values.
    let mut value = value_original;
    let part;
    match parts.len() {
        1 => {
            part = &parts[0];
        }
        2 => {
            if value >= 0.0 {
                part = &parts[0]
            } else {
                value = -value;
                part = &parts[1];
            }
        }
        3 => {
            if value > 0.0 {
                part = &parts[0]
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        4 => {
            if value > 0.0 {
                part = &parts[0]
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        _ => {
            return Formatted {
                text: "#VALUE!".to_owned(),
                color: None,
                error: Some("Too many parts".to_owned()),
            };
        }
    }
    match part {
        ParsePart::Error(..) => Formatted {
            text: "#VALUE!".to_owned(),
            color: None,
            error: Some("Problem parsing format string".to_owned()),
        },
        ParsePart::General(..) => {
            // FIXME: This is "General formatting"
            // We should have different codepaths for general formatting and errors
            let mut number = Digits::new(value);
            let sign = if value < 0.0 { "-" } else { "" };
            // 1e-8 <= |value| < 1e11
            if (-7..=11).contains(&number.point) && !number.is_zero() {
                number.round(9);
                let mut int_part: String = number.integer_part().into_iter().collect();
                if int_part.is_empty() {
                    int_part = "0".to_string();
                }
                let fract_part: String = number.fractional_part().into_iter().collect();
                let mut text = if fract_part.is_empty() {
                    format!("{sign}{int_part}")
                } else {
                    format!("{sign}{int_part}.{fract_part}")
                };
                if locale.numbers.symbols.decimal != "." {
                    text = text.replace('.', &locale.numbers.symbols.decimal.to_string());
                }
                Formatted {
                    text,
                    color: None,
                    error: None,
                }
            } else {
                if number.is_zero() {
                    return Formatted {
                        text: "0".to_string(),
                        color: None,
                        error: None,
                    };
                }
                // Six significant digits
                number.round(6 - number.point);
                let exponent = number.point - 1;
                number.shift(-exponent);
                let exponent_sign = if exponent < 0 { '-' } else { '+' };
                let fract_part: String = number.fractional_part().into_iter().collect();
                let mantissa = if fract_part.is_empty() {
                    format!("{}", number.digit(0))
                } else {
                    format!("{}.{fract_part}", number.digit(0))
                };
                let mut text = format!("{sign}{mantissa}E{exponent_sign}{:02}", exponent.abs());
                if locale.numbers.symbols.decimal != "." {
                    text = text.replace('.', &locale.numbers.symbols.decimal.to_string());
                }
                Formatted {
                    text,
                    color: None,
                    error: None,
                }
            }
        }
        ParsePart::Date(p) => {
            let tokens = &p.tokens;
            let mut text = "".to_string();
            let time_fract = value.fract();
            let hours = (time_fract * 24.0).floor();
            let minutes = ((time_fract * 24.0 - hours) * 60.0).floor();
            let seconds = ((((time_fract * 24.0 - hours) * 60.0) - minutes) * 60.0).round();
            let date = from_excel_date(value as i64).ok();
            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Digit(_) => {}
                    TextToken::Period => {}
                    TextToken::Day => match date {
                        Some(date) => {
                            let day = date.day() as usize;
                            text = format!("{text}{day}");
                        }
                        None => {
                            return Formatted {
                                text: "#VALUE!".to_owned(),
                                color: None,
                                error: Some(format!("Invalid date value: '{value}'")),
                            }
                        }
                    },
                    TextToken::DayPadded => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let day = date.day() as usize;
                        text = format!("{text}{day:02}");
                    }
                    TextToken::DayNameShort => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let mut day = date.weekday().number_from_monday() as usize;
                        if day == 7 {
                            day = 0;
                        }
                        text = format!("{}{}", text, locale.dates.day_names_short[day]);
                    }
                    TextToken::DayName => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let mut day = date.weekday().number_from_monday() as usize;
                        if day == 7 {
                            day = 0;
                        }
                        text = format!("{}{}", text, locale.dates.day_names[day]);
                    }
                    TextToken::Month => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{text}{month}");
                    }
                    TextToken::MonthPadded => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{text}{month:02}");
                    }
                    TextToken::MonthNameShort => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{}{}", text, locale.dates.months_short[month - 1]);
                    }
                    TextToken::MonthName => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{}{}", text, locale.dates.months[month - 1]);
                    }
                    TextToken::MonthLetter => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        let months_letter = &locale.dates.months_letter[month - 1];
                        text = format!("{text}{months_letter}");
                    }
                    TextToken::YearShort => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        text = format!("{}{}", text, date.format("%y"));
                    }
                    TextToken::Year => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        text = format!("{}{}", text, date.year());
                    }
                    TextToken::Hour => {
                        let mut hour = hours as i32;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour}");
                    }
                    TextToken::HourPadded => {
                        let mut hour = hours as i32;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour:02}");
                    }
                    TextToken::Second => {
                        let second = seconds as i32;
                        text = format!("{text}{second}");
                    }
                    TextToken::SecondPadded => {
                        let second = seconds as i32;
                        text = format!("{text}{second:02}");
                    }
                    TextToken::AMPM => {
                        let ampm = if hours < 12.0 { "AM" } else { "PM" };
                        text = format!("{text}{ampm}");
                    }
                    TextToken::Minute => {
                        let minute = minutes as i32;
                        text = format!("{text}{minute}");
                    }
                    TextToken::MinutePadded => {
                        let minute = minutes as i32;
                        text = format!("{text}{minute:02}");
                    }
                    TextToken::ElapsedHour => {
                        let hour = (value * 24.0).floor() as i32;
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedHourPadded => {
                        let hour = (value * 24.0).floor() as i32;
                        text = format!("{text}{hour:02}");
                    }
                    TextToken::ElapsedMinute => {
                        let minute = (value * 24.0 * 60.0).floor() as i32;
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedMinutePadded => {
                        let minute = (value * 24.0 * 60.0).floor() as i32;
                        text = format!("{text}{minute:02}");
                    }
                    TextToken::ElapsedSecond => {
                        let second = (value * 24.0 * 60.0 * 60.0).floor() as i32;
                        text = format!("{text}{second}");
                    }
                    TextToken::ElapsedSecondPadded => {
                        let second = (value * 24.0 * 60.0 * 60.0).floor() as i32;
                        text = format!("{text}{second:02}");
                    }
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
        ParsePart::Number(p) => {
            let mut text = "".to_string();
            if let Some(c) = p.currency {
                text = format!("{c}");
            }
            let tokens = &p.tokens;
            // Each % multiplies the number by 100 and each trailing comma divides it by 1000
            let mut number = Digits::new(value);
            number.shift(2 * p.percent - 3 * p.comma);
            let mut exponent_part: Vec<char> = vec![];
            let mut exponent_is_negative = false;
            if p.is_scientific {
                if number.is_zero() {
                    exponent_part = vec!['0'];
                } else {
                    // TODO: Implement engineering formatting.
                    // One digit before the decimal point: the mantissa rounds to
                    // p.precision decimals, and rounding up can change the exponent
                    number.round(p.precision + 1 - number.point);
                    let exponent = number.point - 1;
                    number.shift(-exponent);
                    exponent_part = format!("{}", exponent.abs()).chars().collect();
                    exponent_is_negative = exponent < 0;
                }
            } else {
                // p.precision is the number of digits _after_ the decimal point
                number.round(p.precision);
            }
            let l_exp = exponent_part.len() as i32;
            let int_part = number.integer_part();
            let fract_part = number.fractional_part();
            // ln is the number of digits of the integer part of the value
            let ln = int_part.len() as i32;
            // digit count is the number of digit tokens ('0', '?' and '#') to the left of the decimal point
            let digit_count = p.digit_count;
            // digit_index points to the digit index in value that we have already formatted
            let mut digit_index = 0;

            let symbols = &locale.numbers.symbols;
            let group_sizes = locale.numbers.decimal_formats.standard.to_owned();
            let group_separator = symbols.group.to_owned();
            let decimal_separator = symbols.decimal.to_owned();
            // A negative number that rounds to zero shows no sign
            let is_negative = value < 0.0 && !number.is_zero();
            let mut needs_period = false;

            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Period => {
                        // if !fract_part.is_empty() &&  {
                        //     text = format!("{text}{decimal_separator}");
                        // }
                        needs_period = true;
                    }
                    TextToken::Digit(digit) => {
                        if digit.number.is_integer() {
                            // 1. Integer part
                            let index = digit.index;
                            let number_index = ln - digit_count + index;
                            if index == 0 && is_negative {
                                text = format!("-{text}");
                            }
                            if ln <= digit_count {
                                // The number of digits is less or equal than the number of digit tokens
                                // i.e. the value is 123 and the format_code is ##### (ln = 3 and digit_count = 5)
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '0' {
                                            '0'
                                        } else {
                                            // digit.kind = '?'
                                            ' '
                                        }
                                    } else {
                                        int_part[number_index as usize]
                                    };
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        ln - digit_index,
                                        &group_sizes,
                                    ) {
                                        &group_separator
                                    } else {
                                        ""
                                    };
                                    text = format!("{text}{c}{sep}");
                                }
                                digit_index += 1;
                            } else {
                                // The number is larger than the formatting code 12345 and 0##
                                // We just hit the first formatting digit (0 in the example above) so we write as many digits as we can (123 in the example)
                                for i in digit_index..number_index + 1 {
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        ln - i,
                                        &group_sizes,
                                    ) {
                                        &group_separator
                                    } else {
                                        ""
                                    };
                                    text = format!("{}{}{}", text, int_part[i as usize], sep);
                                }
                                digit_index = number_index + 1;
                            }
                        } else if digit.number.is_decimal() {
                            // 2. After the decimal point
                            let index = digit.index as usize;
                            if index < fract_part.len() {
                                if needs_period {
                                    text = format!(
                                        "{}{}{}",
                                        text, decimal_separator, fract_part[index]
                                    );
                                } else {
                                    text = format!("{}{}", text, fract_part[index]);
                                }
                            } else if digit.kind == '0' {
                                if needs_period {
                                    text = format!("{text}{}0", decimal_separator);
                                } else {
                                    text = format!("{text}0");
                                }
                            } else if digit.kind == '?' {
                                text = format!("{text} ");
                            } else if digit.kind == '#' && needs_period {
                                // FIXME: This is what Excel does, but it transforms "3" into "3."
                                text = format!("{text}{}", decimal_separator);
                            }
                            needs_period = false;
                        } else if digit.number.is_exponent() {
                            // 3. Exponent part
                            let index = digit.index;
                            if index == 0 {
                                if exponent_is_negative {
                                    text = format!("{text}E-");
                                } else if p.scientific_minus {
                                    text = format!("{text}E");
                                } else {
                                    text = format!("{text}E+");
                                }
                            }
                            let number_index = l_exp - (p.exponent_digit_count - index);
                            if l_exp <= p.exponent_digit_count {
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '?' {
                                            ' '
                                        } else {
                                            '0'
                                        }
                                    } else {
                                        exponent_part[number_index as usize]
                                    };

                                    text = format!("{text}{c}");
                                }
                            } else {
                                for i in 0..number_index + 1 {
                                    text = format!("{}{}", text, exponent_part[i as usize]);
                                }
                                digit_index += number_index + 1;
                            }
                        }
                    }
                    // Date tokens should not be present
                    TextToken::Day => {}
                    TextToken::DayPadded => {}
                    TextToken::DayNameShort => {}
                    TextToken::DayName => {}
                    TextToken::Month => {}
                    TextToken::MonthPadded => {}
                    TextToken::MonthNameShort => {}
                    TextToken::MonthName => {}
                    TextToken::MonthLetter => {}
                    TextToken::YearShort => {}
                    TextToken::Year => {}
                    TextToken::Hour => {}
                    TextToken::HourPadded => {}
                    TextToken::Minute => {}
                    TextToken::MinutePadded => {}
                    TextToken::Second => {}
                    TextToken::SecondPadded => {}
                    TextToken::AMPM => {}
                    TextToken::ElapsedHour => {}
                    TextToken::ElapsedHourPadded => {}
                    TextToken::ElapsedMinute => {}
                    TextToken::ElapsedMinutePadded => {}
                    TextToken::ElapsedSecond => {}
                    TextToken::ElapsedSecondPadded => {}
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
    }
}

fn parse_day(day_str: &str) -> Result<(u32, String), String> {
    let bytes = day_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len <= 2 {
        match day_str.parse::<u32>() {
            Ok(y) => {
                if bytes_len == 2 {
                    return Ok((y, "dd".to_string()));
                } else {
                    return Ok((y, "d".to_string()));
                }
            }
            Err(_) => return Err("Not a valid day".to_string()),
        }
    }
    Err("Not a valid day".to_string())
}

fn parse_month(month_str: &str, locale: &Locale) -> Result<(u32, String), String> {
    let bytes = month_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len <= 2 {
        match month_str.parse::<u32>() {
            Ok(y) => {
                if bytes_len == 2 {
                    return Ok((y, "mm".to_string()));
                } else {
                    return Ok((y, "m".to_string()));
                }
            }
            Err(_) => return Err("Not a valid month".to_string()),
        }
    }

    let month_names_short = &locale.dates.months_short;
    let month_names_long = &locale.dates.months;
    if let Some(m) = month_names_short.iter().position(|r| r == month_str) {
        return Ok((m as u32 + 1, "mmm".to_string()));
    }
    if let Some(m) = month_names_long.iter().position(|r| r == month_str) {
        return Ok((m as u32 + 1, "mmmm".to_string()));
    }
    Err("Not a valid month".to_string())
}

fn parse_year(year_str: &str) -> Result<(i32, String), String> {
    // year is either 2 digits or 4 digits
    // 23 -> 2023
    // 75 -> 1975
    // 30 is the split number (yeah, that's not going to be a problem any time soon)
    // 30 => 1930
    // 29 => 2029
    let bytes = year_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len != 2 && bytes_len != 4 {
        return Err("Not a valid year".to_string());
    }
    match year_str.parse::<i32>() {
        Ok(y) => {
            if y < 30 {
                Ok((2000 + y, "yy".to_string()))
            } else if y < 100 {
                Ok((1900 + y, "yy".to_string()))
            } else {
                Ok((y, "yyyy".to_string()))
            }
        }
        Err(_) => Err("Not a valid year".to_string()),
    }
}

// Check if it is a date. Other spreadsheet engines support a wide variety of dates formats
// Here we support a small subset of them.
//
// The grammar is:
//
// date -> long_date | short_date | iso-date
// short_date -> month separator year
// long_date -> day separator month separator year
// iso_date -> long_year separator number_month separator number_day
// separator -> "/" | "-" | "."
// day -> number | padded number
// month -> number_month | name_month
// number_month -> number | padded number |
// name_month -> short name | full name
// year -> short_year | long year
//
// NOTE 1: The separator has to be the same
// NOTE 2: In some engines "2/3" is implemented ad "2/March of the present year"
// NOTE 3: I did not implement the "short date"
pub(crate) fn parse_date(value: &str, locale: &Locale) -> Result<(i32, String), String> {
    let separator = if value.contains('/') {
        '/'
    } else if value.contains('-') {
        '-'
    } else if value.contains('.') {
        '.'
    } else {
        return Err("Not a valid date".to_string());
    };

    let parts: Vec<&str> = value.split(separator).collect();
    let mut is_iso_date = false;
    let mut day_first = true;
    let (day_str, month_str, year_str) = if parts.len() == 3 {
        if parts[0].len() == 4 {
            // ISO date  yyyy-mm-dd
            if !parts[1].chars().all(char::is_numeric) {
                return Err("Not a valid date".to_string());
            }
            if !parts[2].chars().all(char::is_numeric) {
                return Err("Not a valid date".to_string());
            }
            is_iso_date = true;
            (parts[2], parts[1], parts[0])
        } else {
            //  localized date dd-mm-yyyy or mm-dd-yyyy
            // TODO: A bit hacky, but works for now
            if locale.dates.date_formats.short.starts_with('d') {
                (parts[0], parts[1], parts[2])
            } else {
                day_first = false;
                (parts[1], parts[0], parts[2])
            }
        }
    } else {
        return Err("Not a valid date".to_string());
    };
    let (day, day_format) = parse_day(day_str)?;
    let (month, month_format) = parse_month(month_str, locale)?;
    let (year, year_format) = parse_year(year_str)?;
    let serial_number = match date_to_serial_number(day, month, year) {
        Ok(n) => n,
        Err(_) => return Err("Not a valid date".to_string()),
    };
    if is_iso_date {
        Ok((
            serial_number,
            format!("yyyy{separator}{month_format}{separator}{day_format}"),
        ))
    } else if !day_first {
        Ok((
            serial_number,
            format!("{month_format}{separator}{day_format}{separator}{year_format}"),
        ))
    } else {
        Ok((
            serial_number,
            format!("{day_format}{separator}{month_format}{separator}{year_format}"),
        ))
    }
}

/// Parses a formatted number, returning the numeric value together with the format
/// Uses heuristics to guess the format string
/// "$ 123,345.678" => (123345.678, "$#,##0.00")
/// "30.34%" => (0.3034, "0.00%")
/// 100€ => (100, "100€")
pub(crate) fn parse_formatted_number(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
) -> Result<(f64, Option<String>), String> {
    let value = original.trim();
    let scientific_format = "0.00E+00";

    let symbols = &locale.numbers.symbols;
    let decimal_separator = symbols.decimal.chars().next().unwrap_or('.');
    let group_separator = symbols.group.chars().next().unwrap_or(',');

    // Check if it is a percentage
    if let Some(p) = value.strip_suffix('%') {
        let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
        if options.is_scientific {
            return Ok((f / 100.0, Some(scientific_format.to_string())));
        }
        // We ignore the separator
        if options.decimal_digits > 0 {
            // Percentage format with decimals
            return Ok((f / 100.0, Some("#,##0.00%".to_string())));
        }
        // Percentage format standard
        return Ok((f / 100.0, Some("#,##0%".to_string())));
    }

    // check if it is a currency in currencies
    for currency in currencies {
        if let Some(p) = value.strip_prefix(&format!("-{currency}")) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                return Ok((-f, Some(format!("{currency}#,##0.00"))));
            }
            return Ok((-f, Some(format!("{currency}#,##0"))));
        } else if let Some(p) = value.strip_prefix(currency) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                return Ok((f, Some(format!("{currency}#,##0.00"))));
            }
            return Ok((f, Some(format!("{currency}#,##0"))));
        } else if let Some(p) = value.strip_suffix(currency) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                let currency_format = &format!("#,##0.00{currency}");
                return Ok((f, Some(currency_format.to_string())));
            }
            let currency_format = &format!("#,##0{currency}");
            return Ok((f, Some(currency_format.to_string())));
        }
    }

    // check if it is a date. NOTE: we don't trim the original here
    if let Ok((serial_number, format)) = parse_date(original, locale) {
        return Ok((serial_number as f64, Some(format)));
    }

    // Lastly we check if it is a number
    let (f, options) = parse_number(value, decimal_separator, group_separator)?;
    if options.is_scientific {
        return Ok((f, Some(scientific_format.to_string())));
    }
    if options.has_commas {
        if options.decimal_digits > 0 {
            // group separator and two decimal points
            return Ok((f, Some("#,##0.00".to_string())));
        }
        // Group separator and no decimal points
        return Ok((f, Some("#,##0".to_string())));
    }
    Ok((f, None))
}

struct NumberOptions {
    has_commas: bool,
    is_scientific: bool,
    decimal_digits: usize,
}

// tries to parse 'value' as a number.
// If it is a number it either uses commas as thousands separator or it does not
fn parse_number(
    value: &str,
    decimal_separator: char,
    group_separator: char,
) -> Result<(f64, NumberOptions), String> {
    let mut position = 0;
    let characters: Vec<char> = value.chars().collect();
    let len = characters.len();
    if len == 0 {
        return Err("Cannot parse number".to_string());
    }
    let mut chars = String::from("");
    let mut group_separator_index = Vec::new();
    // get the sign
    let sign = if characters[0] == '-' {
        position += 1;
        -1.0
    } else if characters[0] == '+' {
        position += 1;
        1.0
    } else {
        1.0
    };

    if position >= len {
        return Err("Cannot parse number".to_string());
    }

    if characters[position] == group_separator {
        return Err("Cannot parse number".to_string());
    }

    // numbers before the decimal point
    while position < len {
        let x = characters[position];
        if x.is_ascii_digit() {
            chars.push(x);
        } else if x == group_separator {
            group_separator_index.push(chars.len());
        } else {
            break;
        }
        position += 1;
    }
    // Check the group separator is in multiples of three
    for index in &group_separator_index {
        if (chars.len() - index) % 3 != 0 {
            return Err("Cannot parse number".to_string());
        }
    }
    let mut decimal_digits = 0;
    if position < len && characters[position] == decimal_separator {
        // numbers after the decimal point
        chars.push('.');
        position += 1;
        let start_position = 0;
        while position < len {
            let x = characters[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        decimal_digits = position - start_position;
    }
    let mut is_scientific = false;
    if position + 1 < len && (characters[position] == 'e' || characters[position] == 'E') {
        // exponential side
        is_scientific = true;
        let x = characters[position + 1];
        if x == '-' || x == '+' || x.is_ascii_digit() {
            chars.push('e');
            chars.push(x);
            position += 2;
            while position < len {
                let x = characters[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
    }
    if position != len {
        return Err("Could not parse number".to_string());
    };
    match parse_finite_number(&chars) {
        None => Err("Failed to parse to double".to_string()),
        Some(v) => Ok((
            sign * v,
            NumberOptions {
                has_commas: !group_separator_index.is_empty(),
                is_scientific,
                decimal_digits,
            },
        )),
    }
}
