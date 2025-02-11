use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

use quick_xml::events::Event;

use crate::error::XlsxError;

use super::util::get_required_attribute;

#[derive(Debug)]
pub(crate) struct Relationship {
    pub(crate) target: String,
    pub(crate) rel_type: String,
}

struct RelationshipParser {
    relationships: HashMap<String, Relationship>,
}

impl RelationshipParser {
    fn new() -> Self {
        Self {
            relationships: Default::default(),
        }
    }

    fn process(&mut self, ev: Event) -> Result<(), XlsxError> {
        match ev {
            Event::Start(e) if e.local_name().into_inner() == b"Relationship" => {
                self.relationships.insert(
                    get_required_attribute(&e, "Id")?.to_string(),
                    Relationship {
                        rel_type: get_required_attribute(&e, "Type")?.to_string(),
                        target: get_required_attribute(&e, "Target")?.to_string(),
                    },
                );
            }
            _ => {}
        }

        Ok(())
    }
}

pub(super) fn load_relationships<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<HashMap<String, Relationship>, XlsxError> {
    let file = archive.by_name("xl/_rels/workbook.xml.rels")?;

    let mut parser = RelationshipParser::new();

    let xmlfile = BufReader::new(file);
    let mut xmlfile = quick_xml::Reader::from_reader(xmlfile);
    xmlfile.config_mut().expand_empty_elements = true;

    const BUF_SIZE: usize = 200;
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

    Ok(parser.relationships)
}
