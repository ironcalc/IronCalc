use std::io::{BufReader, Read};

use super::xml::XmlNode;
use ironcalc_base::types::{DefinedName, IterativeCalculation, SheetState};

use crate::error::XlsxError;

use super::{
    util::{get_attribute, parse_bool_with_default},
    worksheets::{Sheet, WorkbookXML},
};

pub(super) fn load_workbook<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<WorkbookXML, XlsxError> {
    let file = archive.by_name("xl/workbook.xml")?;
    let doc = XmlNode::parse(BufReader::new(file))?;
    let mut defined_names = Vec::new();
    let mut sheets = Vec::new();
    // Get the sheets
    let sheet_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("sheet"))
        .collect();
    for sheet in sheet_nodes {
        let name = get_attribute(sheet, "name")?.to_string();
        let sheet_id = get_attribute(sheet, "sheetId")?.to_string();
        let sheet_id = sheet_id.parse::<u32>()?;
        let id = sheet
            .attribute_ns(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
                "id",
            )
            .ok_or_else(|| XlsxError::Xml("Missing \"r:id\" XML attribute".to_string()))?
            .to_string();
        let state = match sheet.attribute("state") {
            Some("visible") | None => SheetState::Visible,
            Some("hidden") => SheetState::Hidden,
            Some("veryHidden") => SheetState::VeryHidden,
            Some(state) => return Err(XlsxError::Xml(format!("Unknown sheet state: {state}"))),
        };
        sheets.push(Sheet {
            name,
            sheet_id,
            id,
            state,
        });
    }
    // Get the defined names
    let name_nodes: Vec<&XmlNode> = doc
        .descendants()
        .filter(|n| n.has_tag_name("definedName"))
        .collect();
    for node in name_nodes {
        let name = get_attribute(node, "name")?.to_string();
        let formula = node.text().unwrap_or("").to_string();
        // NOTE: In Excel the `localSheetId` is just the index of the worksheet and unrelated to the sheetId
        let sheet_id = match node.attribute("localSheetId") {
            Some(s) => {
                let index = s.parse::<usize>()?;
                Some(sheets[index].sheet_id)
            }
            None => None,
        };
        defined_names.push(DefinedName {
            name,
            formula,
            sheet_id,
        })
    }
    // The calculation properties
    let mut iterative_calculation = IterativeCalculation::default();
    if let Some(calc_pr) = doc.descendants().find(|n| n.has_tag_name("calcPr")) {
        iterative_calculation.enabled =
            parse_bool_with_default(calc_pr.attribute("iterate"), false);
        if let Some(count) = calc_pr
            .attribute("iterateCount")
            .and_then(|s| s.trim().parse::<u32>().ok())
        {
            iterative_calculation.maximum_iterations = count;
        }
        if let Some(delta) = calc_pr
            .attribute("iterateDelta")
            .and_then(|s| s.trim().parse::<f64>().ok())
        {
            iterative_calculation.maximum_change = delta;
        }
    }
    // read the relationships file
    Ok(WorkbookXML {
        worksheets: sheets,
        defined_names,
        iterative_calculation,
    })
}
