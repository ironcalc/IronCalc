//! Legacy cell comments ("notes"): `xl/commentsN.xml` with the text, and
//! `xl/drawings/vmlDrawingN.vml` with the shapes Excel needs to show them.

use ironcalc_base::{expressions::utils::parse_reference_a1, types::Worksheet};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

/// The authors in order of first appearance. A comment refers to its author by index.
fn get_authors(worksheet: &Worksheet) -> Vec<&str> {
    let mut authors: Vec<&str> = Vec::new();
    for comment in &worksheet.comments {
        if !authors.contains(&comment.author_name.as_str()) {
            authors.push(&comment.author_name);
        }
    }
    authors
}

/// The comments part of a worksheet, or `None` if it has no comments.
pub(crate) fn get_comments_xml(worksheet: &Worksheet) -> Option<String> {
    if worksheet.comments.is_empty() {
        return None;
    }
    let authors = get_authors(worksheet);
    let authors_str: String = authors
        .iter()
        .map(|author| format!("<author>{}</author>", escape_xml(author)))
        .collect();
    let comments_str: String = worksheet
        .comments
        .iter()
        .map(|comment| {
            let author_id = authors
                .iter()
                .position(|a| *a == comment.author_name)
                .unwrap_or(0);
            format!(
                "<comment ref=\"{}\" authorId=\"{author_id}\"><text><t xml:space=\"preserve\">{}</t></text></comment>",
                escape_xml(&comment.cell_ref),
                escape_xml(&comment.text)
            )
        })
        .collect();
    Some(format!(
        "{XML_DECLARATION}\n<comments xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><authors>{authors_str}</authors><commentList>{comments_str}</commentList></comments>"
    ))
}

/// The VML drawing that anchors each comment to its cell. Excel shows no
/// comment without one. `sheet_id` keeps shape ids unique across sheets.
pub(crate) fn get_vml_drawing_xml(worksheet: &Worksheet, sheet_id: usize) -> Option<String> {
    if worksheet.comments.is_empty() {
        return None;
    }
    let shapes: String = worksheet
        .comments
        .iter()
        .enumerate()
        .filter_map(|(index, comment)| {
            let reference = parse_reference_a1(&comment.cell_ref)?;
            // Zero based, as VML wants them
            let row = reference.row - 1;
            let column = reference.column - 1;
            let shape_id = 1024 * sheet_id + index + 1;
            Some(format!(
                "<v:shape id=\"_x0000_s{shape_id}\" type=\"#_x0000_t202\" style=\"position:absolute;margin-left:59.25pt;margin-top:1.5pt;width:108pt;height:59.25pt;z-index:{z};visibility:hidden\" fillcolor=\"#ffffe1\" o:insetmode=\"auto\">\
<v:fill color2=\"#ffffe1\"/><v:shadow on=\"t\" color=\"black\" obscured=\"t\"/><v:path o:connecttype=\"none\"/>\
<v:textbox style=\"mso-direction-alt:auto\"><div style=\"text-align:left\"></div></v:textbox>\
<x:ClientData ObjectType=\"Note\"><x:MoveWithCells/><x:SizeWithCells/>\
<x:Anchor>{left}, 15, {top}, 2, {right}, 15, {bottom}, 16</x:Anchor><x:AutoFill>False</x:AutoFill>\
<x:Row>{row}</x:Row><x:Column>{column}</x:Column></x:ClientData></v:shape>",
                z = index + 1,
                left = column + 1,
                top = row.max(1) - 1,
                right = column + 3,
                bottom = row + 3,
            ))
        })
        .collect();
    Some(format!(
        "<xml xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:o=\"urn:schemas-microsoft-com:office:office\" xmlns:x=\"urn:schemas-microsoft-com:office:excel\">\
<o:shapelayout v:ext=\"edit\"><o:idmap v:ext=\"edit\" data=\"{sheet_id}\"/></o:shapelayout>\
<v:shapetype id=\"_x0000_t202\" coordsize=\"21600,21600\" o:spt=\"202\" path=\"m,l,21600r21600,l21600,xe\">\
<v:stroke joinstyle=\"miter\"/><v:path gradientshapeok=\"t\" o:connecttype=\"rect\"/></v:shapetype>\
{shapes}</xml>"
    ))
}
