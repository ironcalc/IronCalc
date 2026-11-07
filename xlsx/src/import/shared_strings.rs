use std::collections::HashMap;
use std::io::{BufReader, Read};

use ironcalc_base::types::PhoneticRun;

use super::xml::XmlNode;

use crate::error::XlsxError;

/// The workbook's shared strings while its sheets are read: the list, and an
/// index from each string to its first position in it. Inline-string and
/// formula-string (`t="str"`) cells look their text up here; a linear search
/// of the list per cell made such workbooks quadratic to load.
pub(crate) struct SharedStringTable {
    strings: Vec<String>,
    index: HashMap<String, i32>,
}

impl SharedStringTable {
    pub(crate) fn new(strings: Vec<String>) -> SharedStringTable {
        let mut index = HashMap::with_capacity(strings.len());
        for (i, s) in strings.iter().enumerate() {
            // The first occurrence wins, as a search of the list would find it.
            index.entry(s.clone()).or_insert(i as i32);
        }
        SharedStringTable { strings, index }
    }

    /// The position of `s` in the list, appending it if it is not there yet.
    pub(crate) fn index_of(&mut self, s: &str) -> i32 {
        if let Some(i) = self.index.get(s) {
            return *i;
        }
        let i = self.strings.len() as i32;
        self.strings.push(s.to_string());
        self.index.insert(s.to_string(), i);
        i
    }

    pub(crate) fn into_strings(self) -> Vec<String> {
        self.strings
    }
}

/// The shared strings and the furigana of those that have them (by index)
pub(crate) type SharedStrings = (Vec<String>, HashMap<i32, Vec<PhoneticRun>>);

/// Reads the list of shared strings in an Excel workbook
/// Note than in IronCalc we lose _internal_ styling of a string
/// See Section 18.4
pub(crate) fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<SharedStrings, XlsxError> {
    match archive.by_name("xl/sharedStrings.xml") {
        Ok(file) => read_shared_strings_from_xml(XmlNode::parse(BufReader::new(file))?),
        Err(_e) => Ok((Vec::new(), HashMap::new())),
    }
}

#[cfg(test)]
fn read_shared_strings_from_string(text: &str) -> Result<SharedStrings, XlsxError> {
    read_shared_strings_from_xml(XmlNode::parse(text.as_bytes())?)
}

fn text_of(node: &XmlNode) -> String {
    decode_xlsx_escapes(node.text().unwrap_or(""))
}

fn read_shared_strings_from_xml(doc: XmlNode) -> Result<SharedStrings, XlsxError> {
    let mut shared_strings = Vec::new();
    let mut phonetic_runs = HashMap::new();
    let nodes: Vec<&XmlNode> = doc.descendants().filter(|n| n.has_tag_name("si")).collect();
    for (index, node) in nodes.into_iter().enumerate() {
        // The text is in <si><t> or in the rich text runs <si><r><t>. The <t> inside the
        // phonetic runs <si><rPh><t> is the furigana, not part of the text.
        let mut text = String::new();
        let mut runs = Vec::new();
        for child in node.children() {
            if child.has_tag_name("t") {
                text.push_str(&text_of(child));
            } else if child.has_tag_name("r") {
                for t in child.children().filter(|n| n.has_tag_name("t")) {
                    text.push_str(&text_of(t));
                }
            } else if child.has_tag_name("rPh") {
                let position = |name| child.attribute(name).and_then(|v| v.parse::<u32>().ok());
                if let (Some(start), Some(end), Some(t)) = (
                    position("sb"),
                    position("eb"),
                    child.children().find(|n| n.has_tag_name("t")),
                ) {
                    runs.push(PhoneticRun {
                        start,
                        end,
                        text: text_of(t),
                    });
                }
            }
        }
        if !runs.is_empty() {
            phonetic_runs.insert(index as i32, runs);
        }
        shared_strings.push(text);
    }
    Ok((shared_strings, phonetic_runs))
}

/// Decodes Excel's `_xXXXX_` escape sequences for characters that are invalid in XML 1.0.
/// For example, `_x0001_` → U+0001 (SOH).
pub(crate) fn decode_xlsx_escapes(s: &str) -> String {
    if !s.contains("_x") {
        return s.to_string();
    }

    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Look for _xHHHH_ pattern (7 bytes: '_', 'x', 4 hex digits, '_')
        if i + 6 < len && bytes[i] == b'_' && bytes[i + 1] == b'x' && bytes[i + 6] == b'_' {
            let hex = &s[i + 2..i + 6];

            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(code) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(code) {
                        result.push(c);
                        i += 7;
                        continue;
                    }
                }
            }
        }

        if let Some(c) = s[i..].chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_shared_strings() {
        let xml_string = r#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="3" uniqueCount="3">
    <si>
        <t>A string</t>
    </si>
    <si>
        <t>A second String</t>
    </si>
    <si>
        <r>
            <t>Hello</t>
        </r>
            <r>
            <rPr>
                <b/>
                <sz val="11"/>
                <color rgb="FFFF0000"/>
                <rFont val="Inter"/>
                <family val="2"/>
                <scheme val="minor"/>
            </rPr>
            <t xml:space="preserve"> World</t>
        </r>
    </si>
</sst>"#;
        let (shared_strings, phonetic_runs) =
            read_shared_strings_from_string(xml_string.trim()).unwrap();
        assert!(phonetic_runs.is_empty());
        assert_eq!(
            shared_strings,
            [
                "A string".to_string(),
                "A second String".to_string(),
                "Hello World".to_string()
            ]
        );
    }

    #[test]
    fn test_shared_strings_with_furigana() {
        // As written by Japanese Excel: the <t> inside <rPh> is the furigana
        let xml_string = r#"
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
    <si><t>Plain</t></si>
    <si>
        <r><t>山田</t></r><r><rPr><b/></rPr><t>太郎</t></r>
        <rPh sb="0" eb="2"><t>ヤマダ</t></rPh>
        <rPh sb="2" eb="4"><t>タロウ</t></rPh>
        <phoneticPr fontId="1"/>
    </si>
</sst>"#;
        let (shared_strings, phonetic_runs) =
            read_shared_strings_from_string(xml_string.trim()).unwrap();
        assert_eq!(shared_strings, ["Plain", "山田太郎"]);
        assert_eq!(phonetic_runs.len(), 1);
        assert_eq!(
            phonetic_runs[&1],
            [
                PhoneticRun {
                    start: 0,
                    end: 2,
                    text: "ヤマダ".to_string()
                },
                PhoneticRun {
                    start: 2,
                    end: 4,
                    text: "タロウ".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_decode_xlsx_escapes_control_chars() {
        assert_eq!(decode_xlsx_escapes("_x0001_"), "\x01");
        assert_eq!(decode_xlsx_escapes("_x001F_"), "\x1F");
        assert_eq!(decode_xlsx_escapes("_x000B_"), "\x0B");
        assert_eq!(decode_xlsx_escapes("hello_x0001_world"), "hello\x01world");
    }

    #[test]
    fn test_decode_xlsx_escapes_literal_underscore() {
        // _x005F_ decodes to underscore; the rest is literal
        assert_eq!(decode_xlsx_escapes("_x005F_x0001_"), "_x0001_");
        assert_eq!(decode_xlsx_escapes("_x005F_x005F_"), "_x005F_");
    }

    #[test]
    fn test_decode_xlsx_escapes_non_patterns() {
        // Strings that look similar but are not valid _xHHHH_ patterns
        assert_eq!(decode_xlsx_escapes("_xGGGG_"), "_xGGGG_"); // non-hex digits
        assert_eq!(decode_xlsx_escapes("_x001"), "_x001"); // too short
        assert_eq!(decode_xlsx_escapes("_x00001_"), "_x00001_"); // 5 hex digits
        assert_eq!(decode_xlsx_escapes("plain text"), "plain text");
        assert_eq!(decode_xlsx_escapes("_hello"), "_hello");
    }

    #[test]
    fn test_decode_xlsx_escapes_nul() {
        assert_eq!(decode_xlsx_escapes("_x0000_"), "\x00");
    }
}

#[cfg(test)]
mod table_tests {
    use super::SharedStringTable;

    #[test]
    fn finds_the_first_occurrence_and_appends_new_strings() {
        let mut table =
            SharedStringTable::new(vec!["a".to_string(), "b".to_string(), "a".to_string()]);
        assert_eq!(table.index_of("a"), 0);
        assert_eq!(table.index_of("b"), 1);
        assert_eq!(table.index_of("c"), 3);
        assert_eq!(table.index_of("c"), 3);
        assert_eq!(table.into_strings(), ["a", "b", "a", "c"]);
    }
}
