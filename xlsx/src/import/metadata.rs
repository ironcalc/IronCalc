use std::io::{BufReader, Read};

use ironcalc_base::types::Metadata;
use quick_xml::events::Event;

use crate::error::XlsxError;

struct CoreData {
    creator: String,
    last_modified_by: String,
    created: String,
    last_modified: String,
}

impl Default for CoreData {
    fn default() -> Self {
        Self {
            creator: "Anonymous User".to_string(),
            last_modified_by: "Anonymous User".to_string(),
            created: Default::default(),
            last_modified: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum CoreDataParserState {
    Before,
    Inside,
    After,
}

struct CoreParser {
    state: CoreDataParserState,
    core_data: CoreData,
    current_element: Vec<u8>,
}

impl CoreParser {
    fn new() -> Self {
        Self {
            state: CoreDataParserState::Before,
            core_data: Default::default(),
            current_element: Default::default(),
        }
    }

    fn process(&mut self, ev: Event) {
        self.state = match self.state {
            CoreDataParserState::Before => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"coreProperties" => {
                    CoreDataParserState::Inside
                }
                _ => CoreDataParserState::Before,
            },
            CoreDataParserState::Inside => match ev {
                Event::Start(e) => {
                    self.current_element = e.local_name().into_inner().to_vec();
                    CoreDataParserState::Inside
                }
                Event::Text(t) => {
                    if let Ok(t) = t.unescape() {
                        match &self.current_element[..] {
                            b"creator" => self.core_data.creator = t.to_string(),
                            b"lastModifiedBy" => self.core_data.last_modified_by = t.to_string(),
                            b"created" => self.core_data.created = t.to_string(),
                            b"modified" => self.core_data.last_modified = t.to_string(),
                            _ => {}
                        };
                    }
                    CoreDataParserState::Inside
                }
                Event::End(e) if e.local_name().into_inner() == b"coreProperties" => {
                    CoreDataParserState::After
                }
                _ => CoreDataParserState::Inside,
            },
            CoreDataParserState::After => CoreDataParserState::After,
        }
    }

    fn core_data(self) -> Result<CoreData, XlsxError> {
        if self.state != CoreDataParserState::After {
            return Err(XlsxError::Xml("Corrupt XML structure".to_string()));
        }
        Ok(self.core_data)
    }
}

fn load_core<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<CoreData, XlsxError> {
    let file = archive.by_name("docProps/core.xml")?;

    let mut parser = CoreParser::new();

    let xmlfile = BufReader::new(file);
    let mut xmlfile = quick_xml::Reader::from_reader(xmlfile);

    const BUF_SIZE: usize = 300;
    let mut buf = Vec::with_capacity(BUF_SIZE);
    loop {
        match xmlfile
            .read_event_into(&mut buf)
            .map_err(|e| XlsxError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            event => parser.process(event),
        };
        buf.clear();
    }

    parser.core_data()
}

struct AppData {
    application: String,
    app_version: String,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            application: "Unknown application".to_string(),
            app_version: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AppDataParserState {
    Before,
    Inside,
    After,
}

struct AppParser {
    state: AppDataParserState,
    app_data: AppData,
    current_element: Vec<u8>,
}

impl AppParser {
    fn new() -> Self {
        Self {
            state: AppDataParserState::Before,
            app_data: Default::default(),
            current_element: Default::default(),
        }
    }

    fn process(&mut self, ev: Event) {
        self.state = match self.state {
            AppDataParserState::Before => match ev {
                Event::Start(e) if e.local_name().into_inner() == b"Properties" => {
                    AppDataParserState::Inside
                }
                _ => AppDataParserState::Before,
            },
            AppDataParserState::Inside => match ev {
                Event::Start(e) => {
                    self.current_element = e.local_name().into_inner().to_vec();
                    AppDataParserState::Inside
                }
                Event::Text(t) => {
                    if let Ok(t) = t.unescape() {
                        match &self.current_element[..] {
                            b"Application" => self.app_data.application = t.to_string(),
                            b"AppVersion" => self.app_data.app_version = t.to_string(),
                            _ => {}
                        };
                    }
                    AppDataParserState::Inside
                }
                Event::End(e) if e.local_name().into_inner() == b"Properties" => {
                    AppDataParserState::After
                }
                _ => AppDataParserState::Inside,
            },
            AppDataParserState::After => AppDataParserState::After,
        }
    }

    fn app_data(self) -> Result<AppData, XlsxError> {
        if self.state != AppDataParserState::After {
            return Err(XlsxError::Xml("Corrupt XML structure".to_string()));
        }
        Ok(self.app_data)
    }
}

fn load_app<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<AppData, XlsxError> {
    let file = archive.by_name("docProps/app.xml")?;

    let mut parser = AppParser::new();

    let xmlfile = BufReader::new(file);
    let mut xmlfile = quick_xml::Reader::from_reader(xmlfile);

    const BUF_SIZE: usize = 200;
    let mut buf = Vec::with_capacity(BUF_SIZE);
    loop {
        match xmlfile
            .read_event_into(&mut buf)
            .map_err(|e| XlsxError::Xml(e.to_string()))?
        {
            Event::Eof => break,
            event => parser.process(event),
        };
        buf.clear();
    }

    parser.app_data()
}

pub(super) fn load_metadata<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Metadata, XlsxError> {
    let app_data = load_app(archive)?;
    let core_data = load_core(archive)?;
    Ok(Metadata {
        application: app_data.application,
        app_version: app_data.app_version,
        creator: core_data.creator,
        last_modified_by: core_data.last_modified_by,
        created: core_data.created,
        last_modified: core_data.last_modified,
    })
}
