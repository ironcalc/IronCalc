//! Reads a drawing part (`xl/drawings/drawingN.xml`): the objects floating
//! over a worksheet and where they sit.
//!
//! Every `twoCellAnchor` becomes a [`Drawing`]. A graphic frame holding a
//! column chart gets the chart as content; anything else (pictures, shapes,
//! other chart kinds) keeps its frame as `DrawingContent::Unsupported`.
//! One-cell and absolute anchors are not read yet.

use std::collections::HashMap;
use std::io::{BufReader, Read};

use ironcalc_base::charts::{Fill, Line, LineStyle};
use ironcalc_base::drawing_types::{Anchor, AnchorBehavior, AnchorPoint, Drawing, DrawingContent};
use ironcalc_base::types::Color;

use super::charts::load_chart;
use super::util::get_attribute;
use super::xml::XmlNode;
use crate::error::XlsxError;

const CHART_GRAPHIC_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
const RELATIONSHIPS_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Resolves a relationship target against the folder of the part that holds
/// the rels: `../charts/chart1.xml` from `xl/drawings` is `xl/charts/chart1.xml`.
fn resolve_target(base_dir: &str, target: &str) -> String {
    if let Some(absolute) = target.strip_prefix('/') {
        return absolute.to_string();
    }
    let mut parts: Vec<&str> = base_dir.split('/').filter(|p| !p.is_empty()).collect();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            name => parts.push(name),
        }
    }
    parts.join("/")
}

/// The rels of a part: relationship id -> resolved target path.
fn load_part_rels<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    part_path: &str,
) -> Result<HashMap<String, String>, XlsxError> {
    let (dir, file_name) = match part_path.rsplit_once('/') {
        Some((dir, name)) => (dir, name),
        None => ("", part_path),
    };
    let rels_path = format!("{dir}/_rels/{file_name}.rels");
    let mut rels = HashMap::new();
    let Ok(file) = archive.by_name(&rels_path) else {
        return Ok(rels);
    };
    let doc = XmlNode::parse(BufReader::new(file))?;
    for rel in doc.children().filter(|n| n.has_tag_name("Relationship")) {
        let id = get_attribute(rel, "Id")?.to_string();
        let target = get_attribute(rel, "Target")?;
        rels.insert(id, resolve_target(dir, target));
    }
    Ok(rels)
}

fn parse_anchor_point(node: &XmlNode) -> Result<AnchorPoint, XlsxError> {
    let number = |name: &str| -> Result<i64, XlsxError> {
        node.child(name)
            .and_then(|n| n.text())
            .ok_or_else(|| XlsxError::Xml(format!("Missing <{name}> in anchor")))?
            .trim()
            .parse::<i64>()
            .map_err(|_| XlsxError::Xml(format!("Invalid <{name}> in anchor")))
    };
    // The file is 0-based, the model is 1-based.
    Ok(AnchorPoint {
        row: number("row")? as i32 + 1,
        column: number("col")? as i32 + 1,
        row_offset: number("rowOff")?,
        column_offset: number("colOff")?,
    })
}

fn parse_anchor(node: &XmlNode) -> Result<Anchor, XlsxError> {
    let from = node
        .child("from")
        .ok_or_else(|| XlsxError::Xml("Missing <from> in anchor".to_string()))?;
    let to = node
        .child("to")
        .ok_or_else(|| XlsxError::Xml("Missing <to> in anchor".to_string()))?;
    let behavior = match node.attribute("editAs") {
        Some("oneCell") => AnchorBehavior::MoveOnly,
        Some("absolute") => AnchorBehavior::Fixed,
        _ => AnchorBehavior::MoveAndResize,
    };
    Ok(Anchor {
        from: parse_anchor_point(from)?,
        to: parse_anchor_point(to)?,
        behavior,
    })
}

/// The chart part a graphic frame points to, if it holds a chart.
fn chart_target<'a>(anchor: &XmlNode, rels: &'a HashMap<String, String>) -> Option<&'a String> {
    let graphic_data = anchor
        .child("graphicFrame")?
        .child("graphic")?
        .child("graphicData")?;
    if graphic_data.attribute("uri") != Some(CHART_GRAPHIC_URI) {
        return None;
    }
    let rel_id = graphic_data
        .child("chart")?
        .attribute_ns(RELATIONSHIPS_NS, "id")?;
    rels.get(rel_id)
}

/// What a drawing shows and how its frame is painted.
struct Content {
    content: DrawingContent,
    fill: Fill,
    border: Line,
}

fn unsupported_content() -> Content {
    Content {
        content: DrawingContent::Unsupported,
        fill: Fill::Automatic,
        border: Line {
            style: LineStyle::Automatic,
            color: Color::None,
        },
    }
}

fn load_content<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    anchor: &XmlNode,
    rels: &HashMap<String, String>,
) -> Result<Content, XlsxError> {
    let Some(chart_path) = chart_target(anchor, rels) else {
        return Ok(unsupported_content());
    };
    let Ok(file) = archive.by_name(chart_path) else {
        return Ok(unsupported_content());
    };
    let doc = XmlNode::parse(BufReader::new(file))?;
    Ok(match load_chart(&doc) {
        Some(part) => Content {
            content: DrawingContent::Chart(Box::new(part.chart)),
            fill: part.fill,
            border: part.border,
        },
        None => unsupported_content(),
    })
}

/// Loads the drawings of the drawing part at `path` (e.g. `xl/drawings/drawing1.xml`).
pub(super) fn load_drawings<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
) -> Result<Vec<Drawing>, XlsxError> {
    let rels = load_part_rels(archive, path)?;
    let doc = {
        let file = archive.by_name(path)?;
        XmlNode::parse(BufReader::new(file))?
    };
    let mut drawings = Vec::new();
    for (index, node) in doc
        .children()
        .filter(|n| n.has_tag_name("twoCellAnchor"))
        .enumerate()
    {
        let anchor = parse_anchor(node)?;
        // Every object has a cNvPr with its id and name, a few levels down.
        let properties = node.descendants().find(|n| n.has_tag_name("cNvPr"));
        let id = properties
            .and_then(|p| p.attribute("id"))
            .and_then(|id| id.parse::<u32>().ok())
            .unwrap_or(index as u32 + 1);
        let name = properties
            .and_then(|p| p.attribute("name"))
            .unwrap_or("")
            .to_string();
        let Content {
            content,
            fill,
            border,
        } = load_content(archive, node, &rels)?;
        drawings.push(Drawing {
            id,
            name,
            anchor,
            fill,
            border,
            content,
        });
    }
    Ok(drawings)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_targets() {
        assert_eq!(
            resolve_target("xl/drawings", "../charts/chart1.xml"),
            "xl/charts/chart1.xml"
        );
        assert_eq!(
            resolve_target("xl/worksheets", "../drawings/drawing2.xml"),
            "xl/drawings/drawing2.xml"
        );
        assert_eq!(
            resolve_target("xl/drawings", "/xl/charts/chart3.xml"),
            "xl/charts/chart3.xml"
        );
    }

    #[test]
    fn parses_a_two_cell_anchor() {
        let xml = r#"<xdr:twoCellAnchor xmlns:xdr="http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing" editAs="oneCell"><xdr:from><xdr:col>1</xdr:col><xdr:colOff>501650</xdr:colOff><xdr:row>12</xdr:row><xdr:rowOff>184150</xdr:rowOff></xdr:from><xdr:to><xdr:col>7</xdr:col><xdr:colOff>120650</xdr:colOff><xdr:row>26</xdr:row><xdr:rowOff>82550</xdr:rowOff></xdr:to></xdr:twoCellAnchor>"#;
        let node = XmlNode::parse_str(xml).unwrap();
        let anchor = parse_anchor(&node).unwrap();
        assert_eq!(
            anchor.from,
            AnchorPoint {
                row: 13,
                column: 2,
                row_offset: 184150,
                column_offset: 501650
            }
        );
        assert_eq!(
            anchor.to,
            AnchorPoint {
                row: 27,
                column: 8,
                row_offset: 82550,
                column_offset: 120650
            }
        );
        assert_eq!(anchor.behavior, AnchorBehavior::MoveOnly);
    }
}
