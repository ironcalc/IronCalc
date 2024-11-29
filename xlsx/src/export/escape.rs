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

enum Process<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> Process<'a> {
    fn process(&mut self, (i, next): (usize, Value)) {
        match next {
            Value::Str(s) => match *self {
                Process::Owned(ref mut o) => o.push_str(s),
                Process::Borrowed(b) => {
                    let mut r = String::with_capacity(b.len() + s.len());
                    r.push_str(&b[..i]);
                    r.push_str(s);
                    *self = Process::Owned(r);
                }
            },
            Value::Char(c) => match *self {
                Process::Borrowed(_) => {}
                Process::Owned(ref mut o) => o.push(c),
            },
        }
    }

    fn into_result(self) -> Cow<'a, str> {
        match self {
            Process::Borrowed(b) => Cow::Borrowed(b),
            Process::Owned(o) => Cow::Owned(o),
        }
    }
}

impl Extend<(usize, Value)> for Process<'_> {
    fn extend<I: IntoIterator<Item = (usize, Value)>>(&mut self, it: I) {
        for v in it.into_iter() {
            self.process(v);
        }
    }
}

/// Performs escaping of common XML characters inside an attribute value.
///
/// This function replaces several important markup characters with their
/// entity equivalents:
///
/// * `<` → `&lt;`
/// * `>` → `&gt;`
/// * `"` → `&quot;`
/// * `'` → `&apos;`
/// * `&` → `&amp;`
///
/// The resulting string is safe to use inside XML attribute values.
///
/// Does not perform allocations if the given string does not contain escapable characters.
pub fn escape_xml(s: &str) -> Cow<str> {
    let mut p = Process::Borrowed(s);
    p.extend(s.char_indices().map(|(ind, c)| (ind, escape_char(c))));
    p.into_result()
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
