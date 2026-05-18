// Taken from :

// https://docs.rs/xml-rs/latest/src/xml/escape.rs.html#1-125

//! Contains functions for performing XML special characters escaping.

use std::borrow::Cow;

enum Value {
    Char(char),
    Str(&'static str),
}

fn escape_char(c: char) -> Value {
    match c {
        '<' => Value::Str("&lt;"),
        '>' => Value::Str("&gt;"),
        '"' => Value::Str("&quot;"),
        '\'' => Value::Str("&apos;"),
        '&' => Value::Str("&amp;"),
        '\n' => Value::Str("&#xA;"),
        '\r' => Value::Str("&#xD;"),
        _ => Value::Char(c),
    }
}

/// Returns true for characters that are restricted/forbidden in XML 1.0 and
/// must be encoded using Excel's `_xXXXX_` convention.
fn needs_xlsx_escape(c: char) -> bool {
    let cp = c as u32;
    // XML 1.0 forbidden: 0x01-0x08, 0x0B, 0x0C, 0x0E-0x1F
    // (0x09=TAB, 0x0A=LF, 0x0D=CR are valid in XML and handled above)
    matches!(cp, 0x01..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F)
}

/// Performs escaping of common XML characters inside an attribute value.
///
/// Also encodes control characters (U+0001-U+0008, U+000B, U+000C, U+000E-U+001F)
/// using Excel's `_xXXXX_` convention so the output is valid XML 1.0.
///
/// Does not perform allocations if the given string does not contain escapable characters.
pub fn escape_xml(s: &'_ str) -> Cow<'_, str> {
    // Fast path: if no special characters, return borrowed slice.
    if !s
        .chars()
        .any(|c| matches!(c, '<' | '>' | '"' | '\'' | '&' | '\n' | '\r') || needs_xlsx_escape(c))
    {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if needs_xlsx_escape(c) {
            result.push_str(&format!("_x{:04X}_", c as u32));
        } else {
            match escape_char(c) {
                Value::Str(s) => result.push_str(s),
                Value::Char(c) => result.push(c),
            }
        }
    }
    Cow::Owned(result)
}

// A simpler function that allocates memory for each replacement
// fn escape_xml(value: &str) -> String {
//     value
//         .replace('&', "&amp")
//         .replace('<', "&lt;")
//         .replace('>', "&gt;")
//         .replace('"', "&quot;")
//         .replace('\'', "&apos;")
// }

// See also:
// https://docs.rs/shell-escape/0.1.5/src/shell_escape/lib.rs.html#17-23
// https://aaronerhardt.github.io/docs/relm4/src/quick_xml/escapei.rs.html#69-106
