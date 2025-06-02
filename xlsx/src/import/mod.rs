mod colors;
mod metadata;
mod shared_strings;
mod styles;
mod tables;
mod util;
mod workbook;
mod worksheets;

use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Cursor, Read},
};

use roxmltree::Node;

use ironcalc_base::{
    types::{Metadata, Workbook, WorkbookSettings, WorkbookView},
    Model,
};

use crate::error::XlsxError;

use shared_strings::read_shared_strings;

use metadata::load_metadata;
use styles::load_styles;
use util::get_attribute;
use workbook::load_workbook;
use worksheets::{load_sheets, Relationship};

fn load_relationships<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<HashMap<String, Relationship>, XlsxError> {
    let mut file = archive.by_name("xl/_rels/workbook.xml.rels")?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;
    let nodes: Vec<Node> = doc
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .collect();
    let mut rels = HashMap::new();
    for node in nodes {
        rels.insert(
            get_attribute(&node, "Id")?.to_string(),
            Relationship {
                rel_type: get_attribute(&node, "Type")?.to_string(),
                target: get_attribute(&node, "Target")?.to_string(),
            },
        );
    }
    Ok(rels)
}

fn load_xlsx_from_reader<R: Read + std::io::Seek>(
    name: String,
    reader: R,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut shared_strings = read_shared_strings(&mut archive)?;
    let workbook = load_workbook(&mut archive)?;
    let rels = load_relationships(&mut archive)?;
    let mut tables = HashMap::new();
    let (worksheets, selected_sheet) = load_sheets(
        &mut archive,
        &rels,
        &workbook,
        &mut tables,
        &mut shared_strings,
    )?;
    let styles = load_styles(&mut archive)?;
    let metadata = match load_metadata(&mut archive) {
        Ok(metadata) => metadata,
        Err(_) => {
            // In case there is no metadata, add some
            Metadata {
                application: "Unknown application".to_string(),
                app_version: "".to_string(),
                creator: "".to_string(),
                last_modified_by: "".to_string(),
                created: "".to_string(),
                last_modified: "".to_string(),
            }
        }
    };
    let mut views = HashMap::new();
    views.insert(
        0,
        WorkbookView {
            sheet: selected_sheet,
            window_width: 800,
            window_height: 600,
        },
    );
    Ok(Workbook {
        shared_strings,
        defined_names: workbook.defined_names,
        worksheets,
        styles,
        name,
        settings: WorkbookSettings {
            tz: tz.to_string(),
            locale: locale.to_string(),
        },
        metadata,
        tables,
        views,
        users: Vec::new(),
    })
}

// Imports a file from disk into an internal representation
fn load_from_excel(file_name: &str, locale: &str, tz: &str) -> Result<Workbook, XlsxError> {
    let file_path = std::path::Path::new(file_name);
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let name = file_path
        .file_stem()
        .ok_or_else(|| XlsxError::IO("Could not extract workbook name".to_string()))?
        .to_string_lossy()
        .to_string();
    load_xlsx_from_reader(name, reader, locale, tz)
}

/// Loads a [Workbook] from the bytes of an xlsx file.
/// This is useful, for instance, when bytes are transferred over the network
pub fn load_from_xlsx_bytes(
    bytes: &[u8],
    name: &str,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let cursor = Cursor::new(bytes);
    let reader = BufReader::new(cursor);
    load_xlsx_from_reader(name.to_string(), reader, locale, tz)
}

/// Loads a [Model] from an xlsx file
pub fn load_from_xlsx(file_name: &str, locale: &str, tz: &str) -> Result<Model, XlsxError> {
    let workbook = load_from_excel(file_name, locale, tz)?;
    Model::from_workbook(workbook).map_err(XlsxError::Workbook)
}

/// Loads a [Model] from an `ic` file (a file in the IronCalc internal representation)
pub fn load_from_icalc(file_name: &str) -> Result<Model, XlsxError> {
    let contents = fs::read(file_name)
        .map_err(|e| XlsxError::IO(format!("Could not extract workbook name: {}", e)))?;
    let workbook: Workbook = bitcode::decode(&contents)
        .map_err(|e| XlsxError::IO(format!("Failed to decode file: {}", e)))?;
    Model::from_workbook(workbook).map_err(XlsxError::Workbook)
}
