use ironcalc_base::types::Workbook;

use super::value_metadata::RICH_DATA_PARTS;
use super::xml_constants::{XML_DECLARATION, XML_WORKSHEET};

pub(crate) fn get_workbook_xml_rels(workbook: &Workbook, has_value_metadata: bool) -> String {
    let mut relationships_str: Vec<String> = vec![];
    let worksheet_count = workbook.worksheets.len() + 1;
    for id in 1..worksheet_count {
        relationships_str.push(format!(
            "<Relationship Id=\"rId{id}\" Type=\"{XML_WORKSHEET}\" Target=\"worksheets/sheet{id}.xml\"/>"
        ));
    }
    let mut id = worksheet_count;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata\" Target=\"metadata.xml\"/>")
    );
    id += 1;
    relationships_str.push(
        format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>")
    );
    if has_value_metadata {
        for (file_name, rel_type, _, _) in RICH_DATA_PARTS {
            id += 1;
            relationships_str.push(
                format!("<Relationship Id=\"rId{id}\" Type=\"http://schemas.microsoft.com/office/2017/06/relationships/{rel_type}\" Target=\"richData/{file_name}\"/>")
            );
        }
    }
    format!(
        "{XML_DECLARATION}\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{}</Relationships>",
        relationships_str.join("")
    )
}
