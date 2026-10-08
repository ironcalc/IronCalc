#![allow(clippy::unwrap_used)]

use ironcalc_base::expressions::parser::{
    new_parser_english, static_analysis::add_implicit_intersection, Parser,
};
use std::{
    collections::HashMap,
    io::{BufReader, Read},
    num::ParseIntError,
};

use super::xml::XmlNode;
use ironcalc_base::{
    expressions::{
        parser::{stringify::to_rc_format, DefinedNameS},
        token::{get_error_by_english_name, Error},
        types::CellReferenceRC,
        utils::{column_to_number, is_valid_column_number, is_valid_row, parse_reference_a1},
    },
    types::{
        ArrayKind, Cell, Col, Color, Comment, DefinedName, Dxf, FormulaValue, Link, MergedCell,
        SheetState, SpillValue, Table, Theme, Worksheet, WorksheetView,
    },
};
use thiserror::Error;

use crate::error::XlsxError;

use super::{
    conditional_formatting::load_conditional_formatting,
    shared_strings::{decode_xlsx_escapes, SharedStringTable},
    sheet_data::{read_sheet_data, SheetDataXml},
    tables::load_table,
    util::{get_attribute, get_bool_false, get_color, get_number},
};

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

impl WorkbookXML {
    fn get_defined_names_with_scope(&self) -> Vec<DefinedNameS> {
        let sheet_id_index: Vec<u32> = self.worksheets.iter().map(|s| s.sheet_id).collect();

        let defined_names = self
            .defined_names
            .iter()
            .map(|dn| {
                let index = dn
                    .sheet_id
                    .and_then(|sheet_id| {
                        // returns an Option<usize>
                        sheet_id_index.iter().position(|&x| x == sheet_id)
                    })
                    // convert Option<usize> to Option<u32>
                    .map(|pos| pos as u32);
                (dn.name.clone(), index, dn.formula.clone())
            })
            .collect::<Vec<_>>();
        defined_names
    }
}

/// The row and the column of a reference in its plain form, `B12`: up to three
/// capital letters and then a number. `None` for anything else, which is not
/// to say that it is wrong.
fn parse_plain_cell_reference(cell: &str) -> Option<(i32, i32)> {
    let bytes = cell.as_bytes();
    let letters = bytes.iter().take_while(|b| b.is_ascii_uppercase()).count();
    let digits = &bytes[letters..];
    if !(1..=3).contains(&letters)
        || !(1..=7).contains(&digits.len())
        || digits[0] == b'0'
        || !digits.iter().all(u8::is_ascii_digit)
    {
        return None;
    }
    let column = bytes[..letters]
        .iter()
        .fold(0, |column, b| column * 26 + (b - b'A' + 1) as i32);
    let row = digits.iter().fold(0, |row, b| row * 10 + (b - b'0') as i32);
    (is_valid_column_number(column) && is_valid_row(row)).then_some((row, column))
}

pub(super) fn parse_cell_reference(cell: &str) -> Result<(i32, i32), String> {
    // Every cell of a sheet says where it is, nearly always in the plain form.
    // That one is read without the strings the general reader builds on the
    // way, which with millions of cells were a good part of the import.
    if let Some(position) = parse_plain_cell_reference(cell) {
        return Ok(position);
    }
    if let Some(r) = parse_reference_a1(cell) {
        Ok((r.row, r.column))
    } else {
        Err(format!("Invalid cell reference: '{cell}'"))
    }
}

pub(super) fn parse_range(range: &str) -> Result<(i32, i32, i32, i32), String> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() == 1 {
        if let Some(r) = parse_reference_a1(parts[0]) {
            Ok((r.row, r.column, r.row, r.column))
        } else {
            Err(format!("Invalid range: '{range}'"))
        }
    } else if parts.len() == 2 {
        match (parse_reference_a1(parts[0]), parse_reference_a1(parts[1])) {
            (Some(left), Some(right)) => Ok((left.row, left.column, right.row, right.column)),
            _ => Err(format!("Invalid range: '{range}'")),
        }
    } else {
        Err(format!("Invalid range: '{range}'"))
    }
}

#[cfg(test)]
mod test {
    use super::{parse_plain_cell_reference, parse_reference_a1};
    use crate::import::worksheets::parse_range;

    #[test]
    fn plain_cell_references_are_read_like_any_other() {
        let references = [
            "A1",
            "B12",
            "Z9",
            "AA10",
            "ACN50257",
            "XFD1048576",
            "XFD1",
            "A1048576",
            // not in the sheet
            "XFE1",
            "A1048577",
            "ZZZ1",
            "A9999999",
            "A0",
            // not plain, or not a reference
            "A01",
            "$A$1",
            "$A1",
            "A$1",
            "a1",
            "AAAA1",
            "A",
            "1",
            "",
            "A1B",
            "A-1",
            "A1:B2",
            "Ä1",
        ];
        for reference in references {
            let general = parse_reference_a1(reference).map(|r| (r.row, r.column));
            // What the plain reader reads is what the general one reads
            if let Some(position) = parse_plain_cell_reference(reference) {
                assert_eq!(Some(position), general, "{reference}");
            }
        }
        assert_eq!(parse_plain_cell_reference("B12"), Some((12, 2)));
        assert_eq!(
            parse_plain_cell_reference("XFD1048576"),
            Some((1_048_576, 16_384))
        );
        assert_eq!(parse_plain_cell_reference("XFE1"), None);
        assert_eq!(parse_plain_cell_reference("$A$1"), None);
    }

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

fn load_dimension(ws: &XmlNode) -> String {
    // <dimension ref="A1:O18"/>
    let application_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("dimension"))
        .collect::<Vec<&XmlNode>>();
    if application_nodes.len() == 1 {
        application_nodes[0]
            .attribute("ref")
            .unwrap_or("A1")
            .to_string()
    } else {
        "A1".to_string()
    }
}

fn load_columns(ws: &XmlNode) -> Result<Vec<Col>, XlsxError> {
    // cols
    // <cols>
    //     <col min="5" max="5" width="38.26953125" customWidth="1"/>
    //     <col min="6" max="6" width="9.1796875" style="1"/>
    //     <col min="8" max="8" width="4" customWidth="1"/>
    // </cols>
    let mut cols = Vec::new();
    let columns = ws
        .children()
        .filter(|n| n.has_tag_name("cols"))
        .collect::<Vec<&XmlNode>>();
    if columns.len() == 1 {
        for col in columns[0].children() {
            let min = get_attribute(col, "min")?;
            let min = min.parse::<i32>()?;
            let max = get_attribute(col, "max")?;
            let max = max.parse::<i32>()?;
            let width = get_attribute(col, "width")?;
            let width = width.parse::<f64>()?;
            let custom_width = get_bool_false(col, "customWidth");
            let hidden = get_bool_false(col, "hidden");
            let style = col
                .attribute("style")
                .map(|s| s.parse::<i32>().unwrap_or(0));
            cols.push(Col {
                min,
                max,
                width,
                custom_width,
                style,
                hidden,
            })
        }
    }
    Ok(cols)
}

fn load_merge_cells(ws: &XmlNode) -> Result<Vec<MergedCell>, XlsxError> {
    // 18.3.1.55 Merge Cells
    // <mergeCells count="1">
    //    <mergeCell ref="K7:L10"/>
    // </mergeCells>
    // Malformed, single-cell and overlapping entries are skipped: the engine
    // invariants are that merged ranges span more than one cell and never
    // intersect each other.
    let mut merged_cells: Vec<MergedCell> = Vec::new();
    let merge_cells_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("mergeCells"))
        .collect::<Vec<&XmlNode>>();
    if merge_cells_nodes.len() == 1 {
        for merge_cell in merge_cells_nodes[0]
            .children()
            .filter(|n| n.has_tag_name("mergeCell"))
        {
            let reference = get_attribute(merge_cell, "ref")?;
            let Ok((row, column, last_row, last_column)) = parse_range(reference) else {
                continue;
            };
            let width = last_column - column + 1;
            let height = last_row - row + 1;
            if width < 1 || height < 1 || (width == 1 && height == 1) {
                continue;
            }
            if merged_cells
                .iter()
                .any(|m| m.intersects(row, column, width, height))
            {
                continue;
            }
            merged_cells.push(MergedCell {
                row,
                column,
                width,
                height,
            });
        }
    }
    Ok(merged_cells)
}

fn load_sheet_color(ws: &XmlNode, theme: &Theme) -> Result<Color, XlsxError> {
    // <sheetPr>
    //     <tabColor theme="5" tint="-0.249977111117893"/>
    // </sheetPr>
    let mut color = Color::None;
    let sheet_pr = ws
        .children()
        .filter(|n| n.has_tag_name("sheetPr"))
        .collect::<Vec<&XmlNode>>();
    if sheet_pr.len() == 1 {
        let tabs = sheet_pr[0]
            .children()
            .filter(|n| n.has_tag_name("tabColor"))
            .collect::<Vec<&XmlNode>>();
        if tabs.len() == 1 {
            color = get_color(tabs[0], theme)?;
        }
    }
    Ok(color)
}

fn load_comments<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
) -> Result<Vec<Comment>, XlsxError> {
    let mut comments = Vec::new();
    let file = archive.by_name(path)?;
    let doc = XmlNode::parse(BufReader::new(file))?;
    let ws = &doc;
    let comment_list = ws
        .children()
        .filter(|n| n.has_tag_name("commentList"))
        .collect::<Vec<&XmlNode>>();
    if comment_list.len() == 1 {
        for comment in comment_list[0].children() {
            let text = comment
                .descendants()
                .filter(|n| n.has_tag_name("t"))
                .map(|n| n.text().unwrap().to_string())
                .collect::<Vec<String>>()
                .join("");
            let cell_ref = get_attribute(comment, "ref")?.to_string();
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
    let mut sheet_name = "".to_string();
    let mut column = "".to_string();
    let mut row = "".to_string();
    let mut state = "sheet"; // "sheet", "col", "row"
    for ch in s.chars() {
        match state {
            "sheet" => {
                if ch == '!' {
                    state = "col"
                } else {
                    sheet_name.push(ch);
                }
            }
            "col" => {
                if ch.is_ascii_alphabetic() {
                    column.push(ch);
                } else {
                    state = "row";
                    row.push(ch);
                }
            }
            _ => {
                row.push(ch);
            }
        }
    }
    Ok(CellReferenceRC {
        sheet: sheet_name,
        row: row.parse::<i32>().map_err(ParseReferenceError::RowError)?,
        column: column_to_number(&column).map_err(ParseReferenceError::ColumnError)?,
    })
}

/// The number in the `<v>` of a cell. One that cannot be read is 0, and so is
/// one that is not finite ("NaN", "inf", "1e400"): no cell holds such a number.
fn parse_cell_number(cell_value: Option<&str>) -> f64 {
    cell_value
        .unwrap_or("0")
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .unwrap_or(0.0)
}

pub(super) fn from_a1_to_rc(
    formula: String,
    parser: &mut Parser,
    context: String,
    is_array_formula: bool,
) -> Result<String, XlsxError> {
    let cell_reference =
        parse_reference(&context).map_err(|error| XlsxError::Xml(error.to_string()))?;
    let mut t = parser.parse(&formula, &cell_reference);
    if !is_array_formula {
        add_implicit_intersection(&mut t, true);
    }

    Ok(to_rc_format(&t))
}

/// The index of a formula among the shared formulas of the sheet being read,
/// adding it to them if it is new. `lookup` maps each formula to its index, so
/// that a sheet with many different formulas is not searched from the start
/// for every one of them; placeholders are not in it.
pub(super) fn find_or_add_formula(
    formula: String,
    shared_formulas: &mut Vec<String>,
    lookup: &mut HashMap<String, i32>,
) -> i32 {
    if let Some(index) = lookup.get(&formula) {
        return *index;
    }
    let index = shared_formulas.len() as i32;
    lookup.insert(formula.clone(), index);
    shared_formulas.push(formula);
    index
}

pub(super) enum CellArrayKind {
    None,
    DynamicArray(i32, i32),
    ArrayFormula(i32, i32),
}

// FIXME
#[allow(clippy::too_many_arguments)]
pub(super) fn get_cell_from_excel(
    cell_value: Option<&str>,
    value_metadata: Option<&str>,
    cell_type: &str,
    cell_style: i32,
    formula_index: i32,
    sheet_name: &str,
    cell_ref: &str,
    shared_strings: &mut SharedStringTable,
    rich_text_inline: Option<String>,
    anchor_cell: Option<(i32, i32)>,
    array_kind: CellArrayKind,
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
            "b" => {
                if let Some(anchor) = anchor_cell {
                    Cell::SpillCell {
                        v: SpillValue::Boolean(cell_value == Some("1")),
                        s: cell_style,
                        a: anchor,
                    }
                } else {
                    Cell::BooleanCell {
                        v: cell_value == Some("1"),
                        s: cell_style,
                    }
                }
            }
            "n" => {
                if let Some(anchor) = anchor_cell {
                    Cell::SpillCell {
                        v: SpillValue::Number(parse_cell_number(cell_value)),
                        s: cell_style,
                        a: anchor,
                    }
                } else {
                    Cell::NumberCell {
                        v: parse_cell_number(cell_value),
                        s: cell_style,
                    }
                }
            }
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
                if let Some(anchor) = anchor_cell {
                    Cell::SpillCell {
                        v: SpillValue::Error(
                            get_error_by_english_name(error_name).unwrap_or(Error::ERROR),
                        ),
                        s: cell_style,
                        a: anchor,
                    }
                } else {
                    Cell::ErrorCell {
                        ei: get_error_by_english_name(error_name).unwrap_or(Error::ERROR),
                        s: cell_style,
                    }
                }
            }
            "s" => Cell::SharedString {
                si: cell_value.unwrap_or("0").parse::<i32>().unwrap_or(0),
                s: cell_style,
            },
            "str" => {
                let s = decode_xlsx_escapes(cell_value.unwrap_or(""));
                let si = shared_strings.index_of(&s);

                if let Some(anchor) = anchor_cell {
                    Cell::SpillCell {
                        v: SpillValue::Text(s),
                        s: cell_style,
                        a: anchor,
                    }
                } else {
                    Cell::SharedString { si, s: cell_style }
                }
            }
            "d" => {
                // Not implemented
                println!("Invalid type (d) in {sheet_name}!{cell_ref}");
                Cell::ErrorCell {
                    ei: Error::NIMPL,
                    s: cell_style,
                }
            }
            "inlineStr" => {
                let s = rich_text_inline.unwrap_or_default();
                let si = shared_strings.index_of(&s);

                Cell::SharedString { si, s: cell_style }
            }
            "empty" => Cell::EmptyCell { s: cell_style },
            _ => {
                // error
                println!("Unexpected type ({cell_type}) in {sheet_name}!{cell_ref}");
                Cell::ErrorCell {
                    ei: Error::ERROR,
                    s: cell_style,
                }
            }
        }
    } else {
        let make_cell = |fv: FormulaValue| match array_kind {
            CellArrayKind::None => Cell::CellFormula {
                f: formula_index,
                s: cell_style,
                v: fv,
            },
            CellArrayKind::DynamicArray(width, height) => Cell::ArrayFormula {
                f: formula_index,
                s: cell_style,
                r: (width, height),
                kind: ArrayKind::Dynamic,
                v: fv,
            },
            CellArrayKind::ArrayFormula(width, height) => Cell::ArrayFormula {
                f: formula_index,
                s: cell_style,
                r: (width, height),
                kind: ArrayKind::Cse,
                v: fv,
            },
        };
        match cell_type {
            "b" => make_cell(FormulaValue::Boolean(cell_value == Some("1"))),
            "n" => make_cell(FormulaValue::Number(parse_cell_number(cell_value))),
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
                make_cell(FormulaValue::new_error(
                    get_error_by_english_name(error_name).unwrap_or(Error::ERROR),
                    format!("{sheet_name}!{cell_ref}"),
                    cell_value.unwrap_or("#ERROR!").to_string(),
                ))
            }
            "s" => {
                // Not implemented
                println!("Invalid type (s) in {sheet_name}!{cell_ref}");
                make_cell(FormulaValue::new_error(
                    Error::NIMPL,
                    format!("{sheet_name}!{cell_ref}"),
                    Error::NIMPL.to_string(),
                ))
            }
            "str" => {
                // In Excel and in IronCalc all strings in cells result of a formula are *not* shared strings.
                make_cell(FormulaValue::Text(decode_xlsx_escapes(
                    cell_value.unwrap_or(""),
                )))
            }
            "d" => {
                // Not implemented
                println!("Invalid type (d) in {sheet_name}!{cell_ref}");
                make_cell(FormulaValue::new_error(
                    Error::NIMPL,
                    format!("{sheet_name}!{cell_ref}"),
                    Error::NIMPL.to_string(),
                ))
            }
            "inlineStr" => {
                // NB: This is untested, I don't know of any engine that uses inline strings in formulas
                make_cell(FormulaValue::Text(
                    rich_text_inline.unwrap_or("".to_string()),
                ))
            }
            _ => {
                // error
                println!("Unexpected type ({cell_type}) in {sheet_name}!{cell_ref}");
                make_cell(FormulaValue::new_error(
                    Error::ERROR,
                    format!("{sheet_name}!{cell_ref}"),
                    Error::ERROR.to_string(),
                ))
            }
        }
    }
}

fn load_sheet_rels<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    tables: &mut HashMap<String, Table>,
    sheet_name: &str,
) -> Result<(Vec<Comment>, HashMap<String, String>), XlsxError> {
    // ...xl/worksheets/sheet6.xml -> xl/worksheets/_rels/sheet6.xml.rels
    let mut comments = Vec::new();
    // relationship id ("rId4") -> target of the hyperlink
    let mut hyperlinks = HashMap::new();
    let v: Vec<&str> = path.split("/worksheets/").collect();
    let mut path = v[0].to_string();
    path.push_str("/worksheets/_rels/");
    path.push_str(v[1]);
    path.push_str(".rels");
    let file = archive.by_name(&path);
    if file.is_err() {
        return Ok((comments, hyperlinks));
    }
    let doc = XmlNode::parse(BufReader::new(file?))?;

    let rels = doc
        .children()
        .filter(|n| n.has_tag_name("Relationship"))
        .collect::<Vec<&XmlNode>>();
    for rel in rels {
        let t = get_attribute(rel, "Type")?.to_string();
        if t.ends_with("comments") {
            let mut target = get_attribute(rel, "Target")?.to_string();
            // Target="../comments1.xlsx"
            target.replace_range(..2, v[0]);
            comments = load_comments(archive, &target)?;
        } else if t.ends_with("hyperlink") {
            let id = get_attribute(rel, "Id")?.to_string();
            let target = get_attribute(rel, "Target")?.to_string();
            hyperlinks.insert(id, target);
        } else if t.ends_with("table") {
            let mut target = get_attribute(rel, "Target")?.to_string();

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
    Ok((comments, hyperlinks))
}

/// Maximum number of cells a single `<hyperlink>` range is expanded to
const MAX_HYPERLINK_RANGE_CELLS: i64 = 10_000;

/// Loads the `<hyperlinks>` element of a worksheet:
/// ```xml
/// <hyperlinks>
///   <hyperlink ref="B2" r:id="rId1"/>
///   <hyperlink ref="B4" r:id="rId3" tooltip="This is a tooltip"/>
///   <hyperlink ref="B10" location="Sheet1!A30" display="Jump to A30 (this sheet)"/>
/// </hyperlinks>
/// ```
/// External links have an `r:id` attribute pointing to a relationship in the sheet rels
/// (`hyperlink_rels`), internal links have a `location` attribute instead.
/// The `display` attribute is skipped: the displayed text is the content of the cell.
fn load_hyperlinks(
    ws: &XmlNode,
    hyperlink_rels: &HashMap<String, String>,
) -> Result<HashMap<(i32, i32), Link>, XlsxError> {
    let mut links = HashMap::new();
    let hyperlink_nodes = ws
        .children()
        .filter(|n| n.has_tag_name("hyperlinks"))
        .flat_map(|n| n.children().filter(|n| n.has_tag_name("hyperlink")))
        .collect::<Vec<&XmlNode>>();
    for node in hyperlink_nodes {
        let cell_ref = get_attribute(node, "ref")?;
        // Although it is normally a single cell, the ref can be a range like "B2:C3"
        let (row_start, column_start, row_end, column_end) =
            parse_range(cell_ref).map_err(XlsxError::Xml)?;
        let tooltip = node.attribute("tooltip").map(str::to_string);
        let rel_id = node.attribute_ns(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
            "id",
        );
        let link = match rel_id {
            Some(rel_id) => {
                let target = match hyperlink_rels.get(rel_id) {
                    Some(target) => target.clone(),
                    // dangling relationship id, skip the hyperlink
                    None => continue,
                };
                // An external link may also point to a location inside the target
                // document. We keep it as a fragment of the target.
                let target = match node.attribute("location") {
                    Some(location) => format!("{target}#{location}"),
                    None => target,
                };
                Link::External { target, tooltip }
            }
            None => {
                let location = match node.attribute("location") {
                    Some(location) if !location.is_empty() => location.to_string(),
                    // a hyperlink with neither r:id nor location is malformed, skip it
                    _ => continue,
                };
                Link::Internal { location, tooltip }
            }
        };
        // The range is expanded to one link per cell. A corrupt or malicious file
        // could use an enormous range (up to the whole sheet); in that case only
        // the top-left cell gets the link instead of exhausting memory.
        let cell_count = (row_end - row_start + 1) as i64 * (column_end - column_start + 1) as i64;
        if cell_count > MAX_HYPERLINK_RANGE_CELLS {
            links.insert((row_start, column_start), link);
            continue;
        }
        for row in row_start..=row_end {
            for column in column_start..=column_end {
                links.insert((row, column), link.clone());
            }
        }
    }
    Ok(links)
}

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

fn get_sheet_view(ws: &XmlNode) -> SheetView {
    // <sheetViews>
    //   <sheetView workbookViewId="0">
    //     <selection activeCell="E10" sqref="E10"/>
    //   </sheetView>
    // </sheetViews>
    // <sheetFormatPr defaultRowHeight="14.5" x14ac:dyDescent="0.35"/>

    // If we have frozen rows and columns:

    // <sheetView tabSelected="1" workbookViewId="0">
    //   <pane xSplit="3" ySplit="2" topLeftCell="D3" activePane="bottomRight" state="frozen"/>
    //   <selection pane="topRight" activeCell="D1" sqref="D1"/>
    //   <selection pane="bottomLeft" activeCell="A3" sqref="A3"/>
    //   <selection pane="bottomRight" activeCell="K16" sqref="K16"/>
    // </sheetView>

    // 18.18.52 ST_Pane (Pane Types)
    // bottomLeft, bottomRight, topLeft, topRight

    // NB: bottomLeft is used when only rows are frozen, etc
    // IronCalc ignores all those.

    let mut frozen_rows = 0;
    let mut frozen_columns = 0;

    // In IronCalc there can only be one sheetView
    let sheet_views = ws
        .children()
        .filter(|n| n.has_tag_name("sheetViews"))
        .collect::<Vec<&XmlNode>>();

    // We are only expecting one `sheetViews` element. Otherwise return a default
    if sheet_views.len() != 1 {
        return SheetView::default();
    }

    let sheet_view = sheet_views[0]
        .children()
        .filter(|n| n.has_tag_name("sheetView"))
        .collect::<Vec<&XmlNode>>();

    // We are only expecting one `sheetView` element. Otherwise return a default
    if sheet_view.len() != 1 {
        return SheetView::default();
    }

    let sheet_view = sheet_view[0];
    let is_selected = sheet_view.attribute("tabSelected").unwrap_or("0") == "1";
    let show_grid_lines = sheet_view.attribute("showGridLines").unwrap_or("1") == "1";

    let pane = sheet_view
        .children()
        .filter(|n| n.has_tag_name("pane"))
        .collect::<Vec<&XmlNode>>();

    // 18.18.53 ST_PaneState (Pane State)
    // frozen, frozenSplit, split
    if pane.len() == 1 {
        if let Some("frozen") = pane[0].attribute("state") {
            // TODO: Should we assert that topLeft is consistent?
            // let top_left_cell = pane[0].attribute("topLeftCell").unwrap_or("A1").to_string();

            frozen_columns = get_number(pane[0], "xSplit");
            frozen_rows = get_number(pane[0], "ySplit");
        }
    }
    let selections = sheet_view
        .children()
        .filter(|n| n.has_tag_name("selection"))
        .collect::<Vec<&XmlNode>>();

    if let Some(selection) = selections.last() {
        let active_cell = match selection.attribute("activeCell").map(parse_cell_reference) {
            Some(Ok(s)) => Some(s),
            _ => None,
        };
        let sqref = match selection.attribute("sqref").map(parse_range) {
            Some(Ok(s)) => Some(s),
            _ => None,
        };

        let (selected_row, selected_column, row1, column1, row2, column2) =
            match (active_cell, sqref) {
                (Some(cell), Some(range)) => (cell.0, cell.1, range.0, range.1, range.2, range.3),
                (Some(cell), None) => (cell.0, cell.1, cell.0, cell.1, cell.0, cell.1),
                (None, Some(range)) => (range.0, range.1, range.0, range.1, range.2, range.3),
                _ => (1, 1, 1, 1, 1, 1),
            };

        SheetView {
            frozen_rows,
            frozen_columns,
            selected_row,
            selected_column,
            is_selected,
            show_grid_lines,
            range: [row1, column1, row2, column2],
        }
    } else {
        SheetView {
            frozen_rows,
            frozen_columns,
            is_selected,
            show_grid_lines,
            ..Default::default()
        }
    }
}

pub(super) struct SheetSettings {
    pub id: u32,
    pub name: String,
    pub state: SheetState,
    pub comments: Vec<Comment>,
    /// hyperlink relationships in the sheet rels: relationship id -> target
    pub hyperlink_rels: HashMap<String, String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn load_sheet<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    settings: SheetSettings,
    worksheets: &[String],
    tables: &HashMap<String, Table>,
    shared_strings: &mut SharedStringTable,
    defined_names: Vec<DefinedNameS>,
    theme: &Theme,
    dxfs: &mut Vec<Dxf>,
) -> Result<(Worksheet, bool), XlsxError> {
    let sheet_name = &settings.name;
    let sheet_id = settings.id;
    let state = &settings.state;

    let file = archive.by_name(path)?;
    // One parser for the whole sheet: building one clones the defined names
    // and tables, which is far too much to do once per formula cell.
    let mut parser = new_parser_english(worksheets.to_owned(), defined_names, tables.clone());
    // The cells are streamed; what is left of the worksheet is small and is
    // read as a tree.
    let SheetDataXml {
        worksheet,
        mut sheet_data,
        rows,
        shared_formulas,
    } = read_sheet_data(
        BufReader::new(file),
        sheet_name,
        &mut parser,
        shared_strings,
    )?;
    let ws = &worksheet;

    let dimension = load_dimension(ws);

    let sheet_view = get_sheet_view(ws);

    let cols = load_columns(ws)?;
    let color = load_sheet_color(ws, theme)?;

    let merged_cells = load_merge_cells(ws)?;

    // Covered cells of a merged range must not hold content (an engine
    // invariant); Excel leaves them empty but third-party producers sometimes
    // leave values behind, so we clear them here (keeping their styles).
    for merged_cell in &merged_cells {
        for row in merged_cell.row..=merged_cell.last_row() {
            for column in merged_cell.column..=merged_cell.last_column() {
                if row == merged_cell.row && column == merged_cell.column {
                    continue;
                }
                if let Some(cell) = sheet_data.cell_mut(row, column) {
                    *cell = Cell::EmptyCell {
                        s: cell.get_style(),
                    };
                }
            }
        }
    }

    let links = load_hyperlinks(ws, &settings.hyperlink_rels)?;

    let conditional_formatting = load_conditional_formatting(ws, theme, dxfs)?;
    // pageSetup
    // <pageSetup orientation="portrait" r:id="rId1"/>

    let mut views = HashMap::new();
    // The focus (the moving corner of the selection) is not in the file: use
    // the corner of the range opposite the selected cell on each axis.
    let [range_start_row, range_start_column, range_end_row, range_end_column] = sheet_view.range;
    let focus_row = if sheet_view.selected_row == range_end_row {
        range_start_row
    } else {
        range_end_row
    };
    let focus_column = if sheet_view.selected_column == range_end_column {
        range_start_column
    } else {
        range_end_column
    };
    views.insert(
        0,
        WorksheetView {
            row: sheet_view.selected_row,
            column: sheet_view.selected_column,
            range: sheet_view.range,
            focus_row,
            focus_column,
            top_row: 1,
            left_column: 1,
        },
    );

    Ok((
        Worksheet {
            dimension,
            cols,
            rows,
            shared_formulas,
            sheet_data,
            name: sheet_name.to_string(),
            sheet_id,
            state: state.to_owned(),
            color,
            merged_cells,
            comments: settings.comments,
            frozen_rows: sheet_view.frozen_rows,
            frozen_columns: sheet_view.frozen_columns,
            show_grid_lines: sheet_view.show_grid_lines,
            views,
            conditional_formatting,
            links,
        },
        sheet_view.is_selected,
    ))
}

pub(super) fn load_sheets<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    rels: &HashMap<String, Relationship>,
    workbook: &WorkbookXML,
    tables: &mut HashMap<String, Table>,
    shared_strings: &mut SharedStringTable,
    theme: &Theme,
    dxfs: &mut Vec<Dxf>,
) -> Result<(Vec<Worksheet>, u32), XlsxError> {
    // load comments, tables and hyperlink relationships
    let mut sheet_rels = HashMap::new();
    for sheet in &workbook.worksheets {
        let rel = &rels[&sheet.id];
        if rel.rel_type.ends_with("worksheet") {
            let path = &rel.target;
            let path = if let Some(p) = path.strip_prefix('/') {
                p.to_string()
            } else {
                format!("xl/{path}")
            };
            sheet_rels.insert(
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

    let defined_names = workbook.get_defined_names_with_scope();

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
            let (comments, hyperlink_rels) = sheet_rels
                .get(rel_id)
                .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?;
            let settings = SheetSettings {
                name: sheet_name.to_string(),
                id: sheet.sheet_id,
                state: state.clone(),
                comments: comments.to_vec(),
                hyperlink_rels: hyperlink_rels.clone(),
            };
            let (s, is_selected) = load_sheet(
                archive,
                &path,
                settings,
                worksheets,
                tables,
                shared_strings,
                defined_names.clone(),
                theme,
                dxfs,
            )?;
            if is_selected {
                selected_sheet = sheet_index;
            }
            sheets.push(s);
            sheet_index += 1;
        }
    }
    Ok((sheets, selected_sheet))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ironcalc_base::types::Link;

    use crate::import::xml::XmlNode;

    use crate::import::worksheets::{load_hyperlinks, parse_cell_number, parse_reference};

    #[test]
    fn cell_numbers_are_finite() {
        assert_eq!(parse_cell_number(Some("12.5")), 12.5);
        assert_eq!(parse_cell_number(Some("-1e308")), -1e308);
        assert_eq!(parse_cell_number(None), 0.0);
        // What cannot be read is 0, and so is what is not a finite number
        for value in ["", "abc", "NaN", "nan", "inf", "-inf", "Infinity", "1e400"] {
            assert_eq!(parse_cell_number(Some(value)), 0.0, "{value}");
        }
    }

    #[test]
    fn parse_reference_works() {
        let cell_reference = parse_reference("📈 Overview!B2");
        assert!(cell_reference.is_ok());
        let cell_reference = cell_reference.unwrap();
        assert_eq!(cell_reference.sheet, "📈 Overview");
    }

    #[test]
    fn load_hyperlinks_skips_malformed_and_caps_huge_ranges() {
        let xml = r#"<worksheet xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
            <hyperlinks>
                <hyperlink ref="B2" location="Target!A1"/>
                <hyperlink ref="B3"/>
                <hyperlink ref="B4" location=""/>
                <hyperlink ref="D1:E2" location="Target!A1"/>
                <hyperlink ref="G1:XFD1048576" location="Target!A1"/>
                <hyperlink ref="F1" r:id="rId9"/>
            </hyperlinks>
        </worksheet>"#;
        let doc = XmlNode::parse_str(xml).unwrap();
        let ws = &doc;
        // no relationships: the r:id hyperlink is dangling
        let rels = HashMap::new();

        let links = load_hyperlinks(ws, &rels).unwrap();

        let internal = Link::Internal {
            location: "Target!A1".to_string(),
            tooltip: None,
        };
        // B2 plus the four cells of D1:E2 plus the top-left of the huge range.
        // The hyperlinks with no location, an empty location or a dangling
        // relationship are skipped.
        assert_eq!(links.len(), 6);
        assert_eq!(links.get(&(2, 2)), Some(&internal));
        for (row, column) in [(1, 4), (1, 5), (2, 4), (2, 5)] {
            assert_eq!(links.get(&(row, column)), Some(&internal));
        }
        // the huge range is not expanded: only its top-left cell gets the link
        assert_eq!(links.get(&(1, 7)), Some(&internal));
        assert_eq!(links.get(&(2, 7)), None);
        assert_eq!(links.get(&(3, 2)), None);
        assert_eq!(links.get(&(4, 2)), None);
        assert_eq!(links.get(&(1, 6)), None);
    }
}
