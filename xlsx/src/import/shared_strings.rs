use std::io::Read;

use roxmltree::Node;

use crate::error::XlsxError;

/// Reads the list of shared strings in an Excel workbook
/// Note than in IronCalc we lose _internal_ styling of a string
/// See Section 18.4
pub(crate) fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Vec<String>, XlsxError> {
    match archive.by_name("xl/sharedStrings.xml") {
        Ok(mut file) => {
            let mut text = String::new();
            file.read_to_string(&mut text)?;
            read_shared_strings_from_string(&text)
        }
        Err(_e) => Ok(Vec::new()),
    }
}

fn read_shared_strings_from_string(text: &str) -> Result<Vec<String>, XlsxError> {
    let doc = roxmltree::Document::parse(text)?;
    let mut shared_strings = Vec::new();
    let nodes: Vec<Node> = doc.descendants().filter(|n| n.has_tag_name("si")).collect();
    for node in nodes {
        let text = node
            .descendants()
            .filter(|n| n.has_tag_name("t"))
            .map(|n| n.text().unwrap_or("").to_string())
            .collect::<Vec<String>>()
            .join("");
        shared_strings.push(text);
    }
    Ok(shared_strings)
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
                <rFont val="Calibri"/>
                <family val="2"/>
                <scheme val="minor"/>
            </rPr>
            <t xml:space="preserve"> World</t>
        </r>
    </si>
</sst>"#;
        let shared_strings = read_shared_strings_from_string(xml_string.trim()).unwrap();
        assert_eq!(
            shared_strings,
            [
                "A string".to_string(),
                "A second String".to_string(),
                "Hello World".to_string()
            ]
        );
    }
}
