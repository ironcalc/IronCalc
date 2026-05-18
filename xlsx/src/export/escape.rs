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
    // XML 1.0 forbidden: 0x00-0x08, 0x0B, 0x0C, 0x0E-0x1F
    // (0x09=TAB, 0x0A=LF, 0x0D=CR are valid in XML and handled above)
    matches!(cp, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F)
}

/// Returns true if `bytes` starts with `_xHHHH_` (7 bytes, 4 hex digits).
/// A literal `_` at such a position must be written as `_x005F_` so the
/// decoder does not misread the surrounding text as an escape sequence.
fn starts_xlsx_escape_pattern(bytes: &[u8]) -> bool {
    bytes.len() >= 7
        && bytes[0] == b'_'
        && bytes[1] == b'x'
        && bytes[6] == b'_'
        && bytes[2].is_ascii_hexdigit()
        && bytes[3].is_ascii_hexdigit()
        && bytes[4].is_ascii_hexdigit()
        && bytes[5].is_ascii_hexdigit()
}

/// Performs escaping of common XML characters inside an attribute value.
///
/// Also encodes control characters (U+0001-U+0008, U+000B, U+000C, U+000E-U+001F)
/// using Excel's `_xXXXX_` convention so the output is valid XML 1.0.
///
/// Literal underscores that begin a `_xHHHH_` look-alike are written as `_x005F_`
/// so the decoder cannot misread them as escape sequences.
///
/// Does not perform allocations if the given string does not contain escapable characters.
pub fn escape_xml(s: &'_ str) -> Cow<'_, str> {
    // Fast path: if no special characters, return borrowed slice.
    let needs_escape = s.char_indices().any(|(i, c)| {
        matches!(c, '<' | '>' | '"' | '\'' | '&' | '\n' | '\r')
            || needs_xlsx_escape(c)
            || (c == '_' && starts_xlsx_escape_pattern(&s.as_bytes()[i..]))
    });
    if !needs_escape {
        return Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len() + 8);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        let c = s[i..].chars().next().unwrap();
        if needs_xlsx_escape(c) {
            result.push_str(&format!("_x{:04X}_", c as u32));
        } else if c == '_' && starts_xlsx_escape_pattern(&bytes[i..]) {
            result.push_str("_x005F_");
        } else {
            match escape_char(c) {
                Value::Str(esc) => result.push_str(esc),
                Value::Char(ch) => result.push(ch),
            }
        }
        i += c.len_utf8();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::shared_strings::decode_xlsx_escapes;

    fn roundtrip(s: &str) -> String {
        decode_xlsx_escapes(&escape_xml(s))
    }

    // --- export output ---

    #[test]
    fn test_control_chars_encoded() {
        assert_eq!(escape_xml("\x01").as_ref(), "_x0001_");
        assert_eq!(escape_xml("\x0B").as_ref(), "_x000B_");
        assert_eq!(escape_xml("\x1F").as_ref(), "_x001F_");
        assert_eq!(escape_xml("\x00").as_ref(), "_x0000_");
    }

    #[test]
    fn test_literal_escape_sequence_encoded() {
        // A literal _xHHHH_ in the source has its underscore escaped to _x005F_
        assert_eq!(escape_xml("_x0001_").as_ref(), "_x005F_x0001_");
        assert_eq!(escape_xml("_x005F_").as_ref(), "_x005F_x005F_");
    }

    #[test]
    fn test_plain_underscore_not_encoded() {
        assert_eq!(escape_xml("_hello").as_ref(), "_hello");
        assert_eq!(escape_xml("a_b").as_ref(), "a_b");
        assert_eq!(escape_xml("_x").as_ref(), "_x");
    }

    // --- roundtrip ---

    #[test]
    fn test_control_chars_roundtrip() {
        assert_eq!(roundtrip("\x00"), "\x00");
        assert_eq!(roundtrip("\x01"), "\x01");
        assert_eq!(roundtrip("\x1F"), "\x1F");
        assert_eq!(roundtrip("\x0B"), "\x0B");
    }

    #[test]
    fn test_literal_escape_sequence_roundtrip() {
        assert_eq!(roundtrip("_x0001_"), "_x0001_");
        assert_eq!(roundtrip("_x005F_"), "_x005F_");
        assert_eq!(roundtrip("hello _x0001_ world"), "hello _x0001_ world");
    }

    #[test]
    fn test_plain_underscore_roundtrip() {
        assert_eq!(roundtrip("_hello"), "_hello");
        assert_eq!(roundtrip("a_b"), "a_b");
        assert_eq!(roundtrip("_x"), "_x");
    }
}
