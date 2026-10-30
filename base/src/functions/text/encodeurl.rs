use std::fmt::Write;

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Excel returns #VALUE! when the encoded text would reach the maximum length
const MAX_RESULT_LENGTH: usize = 32_766;

/// Percent-encodes every byte of the UTF-8 representation of `text` except
/// the RFC 3986 unreserved characters: A-Z, a-z, 0-9, '-', '.', '_' and '~'.
/// Hexadecimal digits are uppercase.
///
/// This deliberately differs from Excel and LibreOffice, which encode '~' as
/// "%7E" (LibreOffice also encodes '.' as "%2E").
fn encode_url(text: &str) -> Result<String, Error> {
    let mut result = String::with_capacity(text.len());
    for byte in text.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(*byte as char);
            }
            _ => {
                // Writing to a String cannot fail
                let _ = write!(result, "%{byte:02X}");
            }
        }
    }
    if result.len() > MAX_RESULT_LENGTH {
        return Err(Error::VALUE);
    }
    Ok(result)
}

impl<'a> Model<'a> {
    // ENCODEURL(text)
    // A range or array argument is encoded element by element.
    pub(crate) fn fn_encodeurl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.try_apply_text_unary(args, cell, encode_url)
    }
}
