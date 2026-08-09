#![allow(clippy::unwrap_used)]

use ironcalc_base::colors::get_indexed_color;
use roxmltree::{ExpandedName, Node};

use crate::error::XlsxError;

use ironcalc_base::types::{Color, Theme};

pub(crate) fn get_number(node: Node, s: &str) -> i32 {
    node.attribute(s).unwrap_or("0").parse::<i32>().unwrap_or(0)
}

#[inline]
pub(super) fn get_attribute<'a, 'n, 'm, N>(
    node: &'a Node,
    attr_name: N,
) -> Result<&'a str, XlsxError>
where
    N: Into<ExpandedName<'n, 'm>>,
{
    let attr_name = attr_name.into();
    node.attribute(attr_name)
        .ok_or_else(|| XlsxError::Xml(format!("Missing \"{attr_name:?}\" XML attribute")))
}

pub(super) fn get_value_or_default(node: &Node, tag_name: &str, default: &str) -> String {
    let application_nodes = node
        .children()
        .filter(|n| n.has_tag_name(tag_name))
        .collect::<Vec<Node>>();
    if application_nodes.len() == 1 {
        application_nodes[0].text().unwrap_or(default).to_string()
    } else {
        default.to_string()
    }
}

pub(super) fn get_color(node: Node, theme: &Theme) -> Result<Color, XlsxError> {
    get_color_indexed(node, theme, None)
}

/// Like [`get_color`], but resolves an `indexed="n"` colour against the workbook's
/// `<colors><indexedColors>` palette override (OOXML §18.8.27) when the file supplies one.
/// `indexed` is that override as `#RRGGBB` strings positioned by index; `None` (the common
/// case, no override) keeps the legacy default indexed palette. An out-of-range or malformed
/// override entry also falls back to the default palette.
pub(super) fn get_color_indexed(
    node: Node,
    _theme: &Theme,
    indexed: Option<&[String]>,
) -> Result<Color, XlsxError> {
    // 18.3.1.15 color (Data Bar Color)
    if node.has_attribute("rgb") {
        let raw = node.attribute("rgb").unwrap();
        // Strip leading alpha byte from ARGB (e.g. "FF4472C4" → "#4472C4")
        let hex = if raw.len() == 8 {
            format!("#{}", raw[2..].to_ascii_uppercase())
        } else {
            format!("#{}", raw.to_ascii_uppercase())
        };
        Ok(Color::Rgb(hex))
    } else if node.has_attribute("indexed") {
        let index = node.attribute("indexed").unwrap().parse::<i32>()?;
        if index == 64 {
            // 64 is "transparent" in OOXML, but we don't have a good way to represent that, so we'll just ignore it
            return Ok(Color::None);
        }
        if index < 0 {
            // A malformed `indexed="-1"` would underflow `index as usize` inside
            // `get_indexed_color` and panic; treat it like any other out-of-range index and
            // fall back to the default palette entry.
            return Ok(Color::Rgb(get_indexed_color(0)));
        }
        // Prefer the file's `<indexedColors>` override entry (when the file supplies one and it
        // covers this index); otherwise fall back to the legacy default indexed palette.
        let rgb = indexed
            .and_then(|palette| {
                usize::try_from(index)
                    .ok()
                    .and_then(|i| palette.get(i))
                    .filter(|s| !s.is_empty())
                    .cloned()
            })
            .unwrap_or_else(|| get_indexed_color(index));
        Ok(Color::Rgb(rgb))
    } else if node.has_attribute("theme") {
        let theme_index = node.attribute("theme").unwrap().parse::<i32>()?;
        let tint = match node.attribute("tint") {
            Some(t) => t.parse::<f64>().unwrap_or(0.0),
            None => 0.0,
        };
        Ok(Color::Theme(theme_index, tint))
    } else if node.has_attribute("auto") {
        Ok(Color::None)
    } else {
        println!("Unexpected color node {node:?}");
        Ok(Color::None)
    }
}

/// Parse an `xsd:boolean` attribute value.
///
/// The SpreadsheetML attributes read through these helpers (`customHeight`, `customWidth`,
/// `customFormat`, `hidden`, `wrapText`, the `<xf>` `apply*` / `quotePrefix` flags, the
/// `<tableStyleInfo>` `show*` flags, ...) are all typed **`xsd:boolean`** in the ECMA-376 /
/// ISO 29500 SpreadsheetML schema. Per W3C XML Schema Part 2 §3.2.2 the lexical space of
/// `xsd:boolean` is *exactly* the four literals `true`, `false`, `1`, `0`. So all four are
/// spec-valid; the previous `"1"`/`"0"`-only code was the non-compliant one — it rejected the
/// valid `true`/`false` form that LibreOffice emits. Excel writes `"1"`/`"0"`; both must work.
///
/// We deliberately do **not** over-accept: only the four `xsd:boolean` literals flip the value,
/// and any other token (empty, `"yes"`, `"on"`, garbage) falls through to `default` rather than
/// being read as true. `xsd:boolean`'s canonical literals are lowercase; matching them
/// case-insensitively (and trimming surrounding whitespace) is a deliberate lenient-read
/// allowance for real-world files, not an invitation to accept non-`xsd:boolean` spellings.
///
/// `default` is returned when the attribute is absent or holds an unrecognised value — for
/// `xsd:boolean` attributes with a schema default of `false` (e.g. `customHeight`, `wrapText`),
/// callers pass `false`.
fn get_bool_with_default(node: Node, s: &str, default: bool) -> bool {
    match node.attribute(s) {
        Some(value) => {
            let value = value.trim();
            if value == "1" || value.eq_ignore_ascii_case("true") {
                true
            } else if value == "0" || value.eq_ignore_ascii_case("false") {
                false
            } else {
                // Not a valid xsd:boolean literal — fall back to the schema default rather than
                // guessing (do NOT treat arbitrary non-empty strings as true).
                default
            }
        }
        None => default,
    }
}

pub(super) fn get_bool(node: Node, s: &str) -> bool {
    // defaults to true
    get_bool_with_default(node, s, true)
}

pub(super) fn get_bool_false(node: Node, s: &str) -> bool {
    // defaults to false
    get_bool_with_default(node, s, false)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use roxmltree::Document;

    // The attributes these helpers read are typed `xsd:boolean` in ECMA-376 SpreadsheetML, whose
    // lexical space (W3C XML Schema Part 2 §3.2.2) is exactly {`true`, `false`, `1`, `0`}. So all
    // four forms must parse. Before the fix the helpers only recognised the Excel form (`"1"`/
    // `"0"`) and silently mishandled the equally-valid `"true"`/`"false"` form LibreOffice emits —
    // dropping `customHeight="true"`, `wrapText="true"`, etc. The helpers must ALSO not over-accept:
    // tokens outside the xsd:boolean lexical space (`"yes"`, `"on"`, garbage, empty) are not valid
    // and must fall back to the attribute's schema default, never be read as true.
    const XML: &str = r#"<a one="1" zero="0" t="true" f="false" tu="TRUE" fc="False" ws=" true " empty="" yes="yes" on="on" other="x"/>"#;

    #[test]
    fn get_bool_false_accepts_all_xsd_boolean_forms() {
        let doc = Document::parse(XML).unwrap();
        let node = doc.root_element();
        // Truthy xsd:boolean literals (Excel `1`, ECMA-376 `true`, case/whitespace lenient).
        assert!(get_bool_false(node, "one"));
        assert!(get_bool_false(node, "t"));
        assert!(get_bool_false(node, "tu"));
        assert!(get_bool_false(node, "ws"));
        // Falsy xsd:boolean literals.
        assert!(!get_bool_false(node, "zero"));
        assert!(!get_bool_false(node, "f"));
        assert!(!get_bool_false(node, "fc"));
        // Absent / empty fall back to the schema default (false).
        assert!(!get_bool_false(node, "missing"));
        assert!(!get_bool_false(node, "empty"));
        // NOT over-accepted: non-xsd:boolean tokens must not read as true.
        assert!(!get_bool_false(node, "yes"));
        assert!(!get_bool_false(node, "on"));
        assert!(!get_bool_false(node, "other"));
    }

    #[test]
    fn get_bool_accepts_all_xsd_boolean_forms() {
        let doc = Document::parse(XML).unwrap();
        let node = doc.root_element();
        // Truthy xsd:boolean literals.
        assert!(get_bool(node, "one"));
        assert!(get_bool(node, "t"));
        assert!(get_bool(node, "tu"));
        // Falsy xsd:boolean literals — including `false`, which the pre-fix code wrongly read as
        // true (anything but `"0"`).
        assert!(!get_bool(node, "zero"));
        assert!(!get_bool(node, "f"));
        assert!(!get_bool(node, "fc"));
        // Absent / empty / non-xsd:boolean tokens fall back to the schema default (true here).
        assert!(get_bool(node, "missing"));
        assert!(get_bool(node, "empty"));
        assert!(get_bool(node, "yes"));
        assert!(get_bool(node, "other"));
    }
}
