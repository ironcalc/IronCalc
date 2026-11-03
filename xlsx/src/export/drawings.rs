//! Writes the drawing part of a worksheet (`xl/drawings/drawingN.xml`): one
//! anchored graphic frame per chart. Drawings whose content we cannot write
//! (`DrawingContent::Unsupported`) are left out.

use ironcalc_base::drawing_types::{AnchorBehavior, AnchorPoint, Drawing};

use super::escape::escape_xml;
use super::xml_constants::XML_DECLARATION;

fn anchor_point_xml(point: &AnchorPoint) -> String {
    // The model is 1-based, the file is 0-based.
    format!(
        "<xdr:col>{}</xdr:col><xdr:colOff>{}</xdr:colOff><xdr:row>{}</xdr:row><xdr:rowOff>{}</xdr:rowOff>",
        point.column - 1,
        point.column_offset,
        point.row - 1,
        point.row_offset
    )
}

/// The drawing part holding `drawings`, every one of them a chart. The chart
/// of the i-th drawing is relationship `rId{i+1}` of the part, see
/// [`get_drawing_xml_rels`].
pub(crate) fn get_drawing_xml(drawings: &[&Drawing]) -> String {
    let mut anchors = Vec::new();
    for (index, drawing) in drawings.iter().enumerate() {
        let rel_id = index + 1;
        let edit_as = match drawing.anchor.behavior {
            AnchorBehavior::MoveAndResize => "",
            AnchorBehavior::MoveOnly => " editAs=\"oneCell\"",
            AnchorBehavior::Fixed => " editAs=\"absolute\"",
        };
        anchors.push(format!(
            "<xdr:twoCellAnchor{edit_as}><xdr:from>{}</xdr:from><xdr:to>{}</xdr:to><xdr:graphicFrame macro=\"\"><xdr:nvGraphicFramePr><xdr:cNvPr id=\"{}\" name=\"{}\"/><xdr:cNvGraphicFramePr/></xdr:nvGraphicFramePr><xdr:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/></xdr:xfrm><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" r:id=\"rId{rel_id}\"/></a:graphicData></a:graphic></xdr:graphicFrame><xdr:clientData/></xdr:twoCellAnchor>",
            anchor_point_xml(&drawing.anchor.from),
            anchor_point_xml(&drawing.anchor.to),
            drawing.id,
            escape_xml(&drawing.name)
        ));
    }
    format!(
        "{XML_DECLARATION}\n<xdr:wsDr xmlns:xdr=\"http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">{}</xdr:wsDr>",
        anchors.join("")
    )
}

/// The rels part of a drawing: `rId{i+1}` points to the i-th chart part.
pub(crate) fn get_drawing_xml_rels(chart_numbers: &[usize]) -> String {
    let relationships: Vec<String> = chart_numbers
        .iter()
        .enumerate()
        .map(|(index, chart_number)| {
            format!(
                "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" Target=\"../charts/chart{chart_number}.xml\"/>",
                index + 1
            )
        })
        .collect();
    format!(
        "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        relationships.join("")
    )
}
