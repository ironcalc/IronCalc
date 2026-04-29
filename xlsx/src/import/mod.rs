mod colors;
mod metadata;
mod shared_strings;
mod styles;
mod tables;
mod theme;
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
    expressions::{
        parser::{new_parser_english, stringify::to_english_string},
        types::CellReferenceRC,
    },
    types::{Metadata, Workbook, WorkbookSettings, WorkbookView},
    Model,
};

use crate::error::XlsxError;

use shared_strings::read_shared_strings;

use metadata::load_metadata;
use styles::load_styles;
use theme::Theme;
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

fn resolve_theme_path(rels: &HashMap<String, Relationship>) -> Option<String> {
    let target = rels
        .values()
        .find(|r| r.rel_type.ends_with("/theme"))?
        .target
        .clone();
    Some(if let Some(absolute) = target.strip_prefix('/') {
        absolute.to_string()
    } else {
        format!("xl/{target}")
    })
}

fn reparse_formula(formula: &str, worksheets: &[String]) -> Result<String, XlsxError> {
    let defined_names = Vec::new();
    let tables = HashMap::new();
    let mut parser = new_parser_english(worksheets.to_owned(), defined_names, tables);
    let cell_reference = CellReferenceRC {
        sheet: worksheets[0].clone(),
        column: 1,
        row: 1,
    };
    let t = parser.parse(formula, &cell_reference);

    Ok(to_english_string(&t, &cell_reference))
}

fn load_xlsx_from_reader<R: Read + std::io::Seek>(
    name: String,
    reader: R,
    locale: &str,
    tz: &str,
) -> Result<Workbook, XlsxError> {
    let mut archive = zip::ZipArchive::new(reader)?;

    let mut shared_strings = read_shared_strings(&mut archive)?;
    let mut workbook = load_workbook(&mut archive)?;
    let rels = load_relationships(&mut archive)?;
    let theme_path = resolve_theme_path(&rels);
    let theme = Theme::load(&mut archive, theme_path.as_deref());
    let mut tables = HashMap::new();
    let (worksheets, selected_sheet) = load_sheets(
        &mut archive,
        &rels,
        &workbook,
        &mut tables,
        &mut shared_strings,
        &theme,
    )?;
    let styles = load_styles(&mut archive, &theme)?;
    // reparse formulas in defined names, since they may refer to sheets and tables that have been loaded
    let worksheet_names = worksheets
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<_>>();
    for dn in &mut workbook.defined_names {
        dn.formula = reparse_formula(&dn.formula, &worksheet_names)?;
    }
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
pub fn load_from_xlsx<'a>(
    file_name: &str,
    locale: &str,
    tz: &str,
    language: &'a str,
) -> Result<Model<'a>, XlsxError> {
    let workbook = load_from_excel(file_name, locale, tz)?;
    Model::from_workbook(workbook, language).map_err(XlsxError::Workbook)
}

/// Loads a [Model] from an `ic` file (a file in the IronCalc internal representation)
pub fn load_from_icalc<'a>(file_name: &str, language_id: &'a str) -> Result<Model<'a>, XlsxError> {
    let contents = fs::read(file_name)
        .map_err(|e| XlsxError::IO(format!("Could not extract workbook name: {e}")))?;
    let workbook: Workbook = bitcode::decode(&contents)
        .map_err(|e| XlsxError::IO(format!("Failed to decode file: {e}")))?;
    Model::from_workbook(workbook, language_id).map_err(XlsxError::Workbook)
}
