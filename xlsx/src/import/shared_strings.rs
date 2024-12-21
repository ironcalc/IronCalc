use std::io::{BufReader, Read};

use quick_xml::events::Event;

use crate::error::XlsxError;

struct SSParser {
    state: ParserState,
    current_string: String,
    strings: Vec<String>,
}

impl SSParser {
    fn new() -> Self {
        Self {
            state: ParserState::OutsideSi,
            current_string: String::new(),
            strings: vec![],
        }
    }

    fn process(&mut self, ev: Event) -> Result<(), XlsxError> {
        self.state = match self.state {
            ParserState::OutsideSi => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"si" => ParserState::InsideSi,
                _ => ParserState::OutsideSi,
            },
            ParserState::InsideSi => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"t" => ParserState::T,
                Event::End(e) if e.local_name().into_inner() == b"si" => {
                    self.strings.push(self.current_string.clone());
                    self.current_string.clear();
                    ParserState::OutsideSi
                }
                _ => ParserState::InsideSi,
            },
            ParserState::T => match ev {
                Event::Text(t) => {
                    self.current_string
                        .push_str(t.unescape().unwrap_or("".into()).as_ref());
                    ParserState::T
                }
                Event::End(e) if e.local_name().into_inner() == b"t" => ParserState::InsideSi,
                _ => ParserState::T,
            },
        };

        Ok(())
    }

    fn strings(self) -> Result<Vec<String>, XlsxError> {
        if self.state != ParserState::OutsideSi {
            return Err(XlsxError::Xml("Corrupt XML structure".to_string()));
        }
        Ok(self.strings)
    }
}

#[derive(Debug, PartialEq)]
enum ParserState {
    OutsideSi,
    InsideSi,
    T,
}

/// Reads the list of shared strings in an Excel workbook
/// Note than in IronCalc we lose _internal_ styling of a string
/// See Section 18.4
pub(crate) fn read_shared_strings<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Vec<String>, XlsxError> {
    match archive.by_name("xl/sharedStrings.xml") {
        Ok(mut file) => read_shared_strings_from_reader(&mut file),
        Err(_e) => Ok(Vec::new()),
    }
}

fn read_shared_strings_from_reader<R: Read>(reader: &mut R) -> Result<Vec<String>, XlsxError> {
    let mut parser = SSParser::new();

    let xmlfile = BufReader::new(reader);
    let mut xmlfile = quick_xml::Reader::from_reader(xmlfile);

    const BUF_SIZE: usize = 900;
    let mut buf = Vec::with_capacity(BUF_SIZE);
    loop {
        match xmlfile
            .read_event_into(&mut buf)
            .map_err(|e| XlsxError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            event => parser.process(event)?,
        };
        buf.clear();
    }

    parser.strings()
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
        let shared_strings =
            read_shared_strings_from_reader(&mut xml_string.trim().as_bytes()).unwrap();
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
