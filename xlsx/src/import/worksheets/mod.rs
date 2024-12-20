use core::str;
use std::{collections::HashMap, io::Read, num::ParseIntError};

use ironcalc_base::{
    expressions::{
        parser::{stringify::to_rc_format, Parser},
        token::{get_error_by_english_name, Error},
        types::CellReferenceRC,
        utils::{column_to_number, parse_reference_a1},
    },
    types::{Cell, Comment, DefinedName, SheetState, Table, Worksheet},
};
use roxmltree::Node;
use thiserror::Error;

use crate::error::XlsxError;

use super::{
    tables::load_table,
    util::{get_attribute, get_color},
};

mod worksheet_streaming;
mod worksheet_tree;

pub(crate) struct Sheet {
    pub(crate) name: String,
    pub(crate) sheet_id: u32,
    pub(crate) id: String,
    pub(crate) state: SheetState,
}

pub(crate) struct WorkbookXML {
    pub(crate) worksheets: Vec<Sheet>,
    pub(crate) defined_names: Vec<DefinedName>,
}

pub(crate) struct Relationship {
    pub(crate) target: String,
    pub(crate) rel_type: String,
}

pub(super) struct SheetSettings {
    pub id: u32,
    pub name: String,
    pub state: SheetState,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone)]
struct SheetView {
    is_selected: bool,
    selected_row: i32,
    selected_column: i32,
    frozen_columns: i32,
    frozen_rows: i32,
    range: [i32; 4],
    show_grid_lines: bool,
}

impl Default for SheetView {
    fn default() -> Self {
        Self {
            is_selected: false,
            selected_row: 1,
            selected_column: 1,
            frozen_rows: 0,
            frozen_columns: 0,
            range: [1, 1, 1, 1],
            show_grid_lines: true,
        }
    }
}

pub(super) fn load_sheets<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    rels: &HashMap<String, Relationship>,
    workbook: &WorkbookXML,
    tables: &mut HashMap<String, Table>,
    shared_strings: &mut Vec<String>,
) -> Result<(Vec<Worksheet>, u32), XlsxError> {
    // load comments and tables
    let mut comments = HashMap::new();
    for sheet in &workbook.worksheets {
        let rel = &rels[&sheet.id];
        if rel.rel_type.ends_with("worksheet") {
            let path = &rel.target;
            let path = if let Some(p) = path.strip_prefix('/') {
                p.to_string()
            } else {
                format!("xl/{path}")
            };
            comments.insert(
                &sheet.id,
                load_sheet_rels(archive, &path, tables, &sheet.name)?,
            );
        }
    }

    // load all sheets
    let worksheets: &Vec<String> = &workbook.worksheets.iter().map(|s| s.name.clone()).collect();
    let mut sheets = Vec::new();
    let mut selected_sheet = 0;
    let mut sheet_index = 0;
    for sheet in &workbook.worksheets {
        let sheet_name = &sheet.name;
        let rel_id = &sheet.id;
        let state = &sheet.state;
        let rel = &rels[rel_id];
        if rel.rel_type.ends_with("worksheet") {
            let path = &rel.target;
            let path = if let Some(p) = path.strip_prefix('/') {
                p.to_string()
            } else {
                format!("xl/{path}")
            };
            let settings = SheetSettings {
                name: sheet_name.to_string(),
                id: sheet.sheet_id,
                state: state.clone(),
                comments: comments
                    .get(rel_id)
                    .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?
                    .to_vec(),
            };

            let streaming = true;
            let (s, is_selected) = if streaming {
                worksheet_streaming::load_sheet(
                    archive,
                    &path,
                    settings,
                    worksheets,
                    tables,
                    shared_strings,
                )?
            } else {
                worksheet_tree::load_sheet(
                    archive,
                    &path,
                    settings,
                    worksheets,
                    tables,
                    shared_strings,
                )?
            };
            if is_selected {
                selected_sheet = sheet_index;
            }
            sheets.push(s);
            sheet_index += 1;
        }
    }
    Ok((sheets, selected_sheet))
}

fn load_sheet_rels<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    tables: &mut HashMap<String, Table>,
    sheet_name: &str,
) -> Result<Vec<Comment>, XlsxError> {
    // ...xl/worksheets/sheet6.xml -> xl/worksheets/_rels/sheet6.xml.rels
    let mut comments = Vec::new();
    let v: Vec<&str> = path.split("/worksheets/").collect();
    let mut path = v[0].to_string();
    path.push_str("/worksheets/_rels/");
    path.push_str(v[1]);
    path.push_str(".rels");
    let file = archive.by_name(&path);
    if file.is_err() {
        return Ok(comments);
    }
    let mut text = String::new();
    file.unwrap().read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;

    let rels = doc
        .root()
        .first_child()
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?
        .children()
        .collect::<Vec<Node>>();
    for rel in rels {
        let t = get_attribute(&rel, "Type")?.to_string();
        if t.ends_with("comments") {
            let mut target = get_attribute(&rel, "Target")?.to_string();
            // Target="../comments1.xlsx"
            target.replace_range(..2, v[0]);
            comments = load_comments(archive, &target)?;
        } else if t.ends_with("table") {
            let mut target = get_attribute(&rel, "Target")?.to_string();

            let path = if let Some(p) = target.strip_prefix('/') {
                p.to_string()
            } else {
                // Target="../table1.xlsx"
                target.replace_range(..2, v[0]);
                target
            };

            let table = load_table(archive, &path, sheet_name)?;
            tables.insert(table.name.clone(), table);
        }
    }
    Ok(comments)
}

fn load_comments<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
) -> Result<Vec<Comment>, XlsxError> {
    let mut comments = Vec::new();
    let mut file = archive.by_name(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;
    let ws = doc
        .root()
        .first_child()
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?;
    let comment_list = ws
        .children()
        .filter(|n| n.has_tag_name("commentList"))
        .collect::<Vec<Node>>();
    if comment_list.len() == 1 {
        for comment in comment_list[0].children() {
            let text = comment
                .descendants()
                .filter(|n| n.has_tag_name("t"))
                .map(|n| n.text().unwrap().to_string())
                .collect::<Vec<String>>()
                .join("");
            let cell_ref = get_attribute(&comment, "ref")?.to_string();
            // TODO: Read author_name from the list of authors
            let author_name = "".to_string();
            comments.push(Comment {
                text,
                author_name,
                author_id: None,
                cell_ref,
            });
        }
    }

    Ok(comments)
}

fn get_formula_index(formula: &str, shared_formulas: &[String]) -> Option<i32> {
    for (index, f) in shared_formulas.iter().enumerate() {
        if f == formula {
            return Some(index as i32);
        }
    }
    None
}

fn from_a1_to_rc(
    formula: String,
    worksheets: &[String],
    context: String,
    tables: HashMap<String, Table>,
) -> Result<String, XlsxError> {
    let mut parser = Parser::new(worksheets.to_owned(), tables);
    let cell_reference =
        parse_reference(&context).map_err(|error| XlsxError::Xml(error.to_string()))?;
    let t = parser.parse(&formula, &Some(cell_reference));
    Ok(to_rc_format(&t))
}

#[derive(Error, Debug, PartialEq)]
enum ParseReferenceError {
    #[error("RowError: {0}")]
    RowError(ParseIntError),
    #[error("ColumnError: {0}")]
    ColumnError(String),
}

// This parses Sheet1!AS23 into sheet, column and row
// FIXME: This is buggy. Does not check that is a valid sheet name
// There is a similar named function in ironcalc_base. We probably should fix both at the same time.
// NB: Maybe use regexes for this?
fn parse_reference(s: &str) -> Result<CellReferenceRC, ParseReferenceError> {
    let bytes = s.as_bytes();
    let mut sheet_name = "".to_string();
    let mut column = "".to_string();
    let mut row = "".to_string();
    let mut state = "sheet"; // "sheet", "col", "row"
    for &byte in bytes {
        match state {
            "sheet" => {
                if byte == b'!' {
                    state = "col"
                } else {
                    sheet_name.push(byte as char);
                }
            }
            "col" => {
                if byte.is_ascii_alphabetic() {
                    column.push(byte as char);
                } else {
                    state = "row";
                    row.push(byte as char);
                }
            }
            _ => {
                row.push(byte as char);
            }
        }
    }
    Ok(CellReferenceRC {
        sheet: sheet_name,
        row: row.parse::<i32>().map_err(ParseReferenceError::RowError)?,
        column: column_to_number(&column).map_err(ParseReferenceError::ColumnError)?,
    })
}

fn get_column_from_ref(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_digit()).collect()
}

// FIXME
#[allow(clippy::too_many_arguments)]
fn get_cell_from_excel(
    cell_value: Option<&str>,
    value_metadata: Option<&str>,
    cell_type: &str,
    cell_style: i32,
    formula_index: i32,
    sheet_name: &str,
    cell_ref: &str,
    shared_strings: &mut Vec<String>,
) -> Cell {
    // Possible cell types:
    // 18.18.11 ST_CellType (Cell Type)
    //   b (Boolean)
    //   d (Date)
    //   e (Error)
    //   inlineStr (Inline String)
    //   n (Number)
    //   s (Shared String)
    //   str (String)

    if formula_index == -1 {
        match cell_type {
            "b" => Cell::BooleanCell {
                v: cell_value == Some("1"),
                s: cell_style,
            },
            "n" => Cell::NumberCell {
                v: cell_value.unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                s: cell_style,
            },
            "e" => {
                // For compatibility reasons Excel does not put the value #SPILL! but adds it as a metadata
                // Older engines would just import #VALUE!
                let mut error_name = cell_value.unwrap_or("#ERROR!");
                if error_name == "#VALUE!" && value_metadata.is_some() {
                    error_name = match value_metadata {
                        Some("1") => "#CALC!",
                        Some("2") => "#SPILL!",
                        _ => error_name,
                    }
                }
                Cell::ErrorCell {
                    ei: get_error_by_english_name(error_name).unwrap_or(Error::ERROR),
                    s: cell_style,
                }
            }
            "s" => Cell::SharedString {
                si: cell_value.unwrap_or("0").parse::<i32>().unwrap_or(0),
                s: cell_style,
            },
            "str" => {
                let s = cell_value.unwrap_or("");
                let si = if let Some(i) = shared_strings.iter().position(|r| r == s) {
                    i
                } else {
                    shared_strings.push(s.to_string());
                    shared_strings.len() - 1
                } as i32;

                Cell::SharedString { si, s: cell_style }
            }
            "d" => {
                // Not implemented
                println!("Invalid type (d) in {}!{}", sheet_name, cell_ref);
                Cell::ErrorCell {
                    ei: Error::NIMPL,
                    s: cell_style,
                }
            }
            "inlineStr" => {
                // Not implemented
                println!("Invalid type (inlineStr) in {}!{}", sheet_name, cell_ref);
                Cell::ErrorCell {
                    ei: Error::NIMPL,
                    s: cell_style,
                }
            }
            "empty" => Cell::EmptyCell { s: cell_style },
            _ => {
                // error
                println!(
                    "Unexpected type ({}) in {}!{}",
                    cell_type, sheet_name, cell_ref
                );
                Cell::ErrorCell {
                    ei: Error::ERROR,
                    s: cell_style,
                }
            }
        }
    } else {
        match cell_type {
            "b" => Cell::CellFormulaBoolean {
                f: formula_index,
                v: cell_value == Some("1"),
                s: cell_style,
            },
            "n" => Cell::CellFormulaNumber {
                f: formula_index,
                v: cell_value.unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                s: cell_style,
            },
            "e" => {
                // For compatibility reasons Excel does not put the value #SPILL! but adds it as a metadata
                // Older engines would just import #VALUE!
                let mut error_name = cell_value.unwrap_or("#ERROR!");
                if error_name == "#VALUE!" && value_metadata.is_some() {
                    error_name = match value_metadata {
                        Some("1") => "#CALC!",
                        Some("2") => "#SPILL!",
                        _ => error_name,
                    }
                }
                Cell::CellFormulaError {
                    f: formula_index,
                    ei: get_error_by_english_name(error_name).unwrap_or(Error::ERROR),
                    s: cell_style,
                    o: format!("{}!{}", sheet_name, cell_ref),
                    m: cell_value.unwrap_or("#ERROR!").to_string(),
                }
            }
            "s" => {
                // Not implemented
                let o = format!("{}!{}", sheet_name, cell_ref);
                let m = Error::NIMPL.to_string();
                println!("Invalid type (s) in {}!{}", sheet_name, cell_ref);
                Cell::CellFormulaError {
                    f: formula_index,
                    ei: Error::NIMPL,
                    s: cell_style,
                    o,
                    m,
                }
            }
            "str" => {
                // In Excel and in IronCalc all strings in cells result of a formula are *not* shared strings.
                Cell::CellFormulaString {
                    f: formula_index,
                    v: cell_value.unwrap_or("").to_string(),
                    s: cell_style,
                }
            }
            "d" => {
                // Not implemented
                println!("Invalid type (d) in {}!{}", sheet_name, cell_ref);
                let o = format!("{}!{}", sheet_name, cell_ref);
                let m = Error::NIMPL.to_string();
                Cell::CellFormulaError {
                    f: formula_index,
                    ei: Error::NIMPL,
                    s: cell_style,
                    o,
                    m,
                }
            }
            "inlineStr" => {
                // Not implemented
                let o = format!("{}!{}", sheet_name, cell_ref);
                let m = Error::NIMPL.to_string();
                println!("Invalid type (inlineStr) in {}!{}", sheet_name, cell_ref);
                Cell::CellFormulaError {
                    f: formula_index,
                    ei: Error::NIMPL,
                    s: cell_style,
                    o,
                    m,
                }
            }
            _ => {
                // error
                println!(
                    "Unexpected type ({}) in {}!{}",
                    cell_type, sheet_name, cell_ref
                );
                let o = format!("{}!{}", sheet_name, cell_ref);
                let m = Error::ERROR.to_string();
                Cell::CellFormulaError {
                    f: formula_index,
                    ei: Error::ERROR,
                    s: cell_style,
                    o,
                    m,
                }
            }
        }
    }
}

fn parse_cell_reference(cell: &str) -> Result<(i32, i32), String> {
    if let Some(r) = parse_reference_a1(cell) {
        Ok((r.row, r.column))
    } else {
        Err(format!("Invalid cell reference: '{}'", cell))
    }
}

fn parse_range(range: &str) -> Result<(i32, i32, i32, i32), String> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() == 1 {
        if let Some(r) = parse_reference_a1(parts[0]) {
            Ok((r.row, r.column, r.row, r.column))
        } else {
            Err(format!("Invalid range: '{}'", range))
        }
    } else if parts.len() == 2 {
        match (parse_reference_a1(parts[0]), parse_reference_a1(parts[1])) {
            (Some(left), Some(right)) => {
                return Ok((left.row, left.column, right.row, right.column));
            }
            _ => return Err(format!("Invalid range: '{}'", range)),
        }
    } else {
        return Err(format!("Invalid range: '{}'", range));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_range() {
        assert!(parse_range("3Aw").is_err());
        assert_eq!(parse_range("A1"), Ok((1, 1, 1, 1)));
        assert_eq!(parse_range("B5:C6"), Ok((5, 2, 6, 3)));
        assert!(parse_range("A1:A2:A3").is_err());
        assert!(parse_range("A1:34").is_err());
        assert!(parse_range("A").is_err());
        assert!(parse_range("12").is_err());
    }
}
