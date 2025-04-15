use std::io::Read;

use ironcalc_base::types::{Table, TableColumn, TableStyleInfo};
use roxmltree::Node;

use crate::error::XlsxError;

use super::util::{get_bool, get_bool_false};

// <table name="Table" displayName="Table" totalsRowCount ref="A1:D6">
//   <autoFilter ref="A1:D6">
//       <filterColumn colId="0">
//            <customFilters><customFilter operator="greaterThan" val=20></customFilter></customFilters>
//       </filterColumn>
//   </autoFilter>
//   <tableColumns count="5">
//      <tableColumn name="Monday" totalsRowFunction="sum" />
//      ...
//   </tableColumns>
//   <tableStyleInfo name="TableStyle5"/>
// </table>

/// Reads a table in an Excel workbook
pub(crate) fn load_table<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
    path: &str,
    sheet_name: &str,
) -> Result<Table, XlsxError> {
    let mut file = archive.by_name(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let document = roxmltree::Document::parse(&text)?;

    // table
    let table = document
        .root()
        .first_child()
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?;

    // Name and display name are normally the same and are unique in a workbook
    // They also need to be different from any defined name
    let name = table
        .attribute("name")
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure: missing table name".to_string()))?
        .to_string();

    let display_name = table
        .attribute("name")
        .ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing table display name".to_string())
        })?
        .to_string();

    // Range of the table, including the totals if any and headers.
    let reference = table
        .attribute("ref")
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure: missing table ref".to_string()))?
        .to_string();

    // Either 0 or 1, indicates if the table has a formula for totals at the bottom of the table
    let totals_row_count = match table.attribute("totalsRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid totalsRowCount".to_string())
        })?,
        None => 0,
    };

    // Either 0 or 1, indicates if the table has headers at the top of the table
    let header_row_count = match table.attribute("headerRowCount") {
        Some(s) => s.parse::<u32>().map_err(|_| {
            XlsxError::Xml("Corrupt XML structure: Invalid headerRowCount".to_string())
        })?,
        None => 1,
    };

    // style index of the header row of the table
    let header_row_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the header row of the table
    let data_dxf_id = if let Some(index_str) = table.attribute("headerRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // style index of the totals row of the table
    let totals_row_dxf_id = if let Some(index_str) = table.attribute("totalsRowDxfId") {
        index_str.parse::<u32>().ok()
    } else {
        None
    };

    // Missing in Calc: styles can also be defined via a name:
    // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

    // Missing in Calc: styles can also be applied to the borders:
    // headerRowBorderDxfId, tableBorderDxfId, totalsRowBorderDxfId

    // TODO: Conformant implementations should panic if header_row_dxf_id or data_dxf_id are out of bounds.

    // Note that filters are non dynamic
    // The only thing important for us is whether or not it has filters
    let auto_filter = table
        .descendants()
        .filter(|n| n.has_tag_name("autoFilter"))
        .collect::<Vec<Node>>();

    let has_filters = if let Some(filter) = auto_filter.first() {
        filter.children().count() > 0
    } else {
        false
    };

    // tableColumn
    let table_column = table
        .descendants()
        .filter(|n| n.has_tag_name("tableColumn"))
        .collect::<Vec<Node>>();
    let mut columns = Vec::new();
    for table_column in table_column {
        let column_name = table_column.attribute("name").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column name".to_string())
        })?;
        let id = table_column.attribute("id").ok_or_else(|| {
            XlsxError::Xml("Corrupt XML structure: missing column id".to_string())
        })?;
        let id = id
            .parse::<u32>()
            .map_err(|_| XlsxError::Xml("Corrupt XML structure: invalid id".to_string()))?;

        // style index of the header row of the table
        let header_row_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the header row of the table column
        let data_dxf_id = if let Some(index_str) = table_column.attribute("headerRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // style index of the totals row of the table column
        let totals_row_dxf_id = if let Some(index_str) = table_column.attribute("totalsRowDxfId") {
            index_str.parse::<u32>().ok()
        } else {
            None
        };

        // NOTE: Same as before, we should panic if indices to differential formatting records are out of bounds
        // Missing in Calc: styles can also be defined via a name:
        // headerRowCellStyle, dataCellStyle, totalsRowCellStyle

        columns.push(TableColumn {
            id,
            name: column_name.to_string(),
            totals_row_label: None,
            header_row_dxf_id,
            data_dxf_id,
            totals_row_function: None,
            totals_row_dxf_id,
        });
    }

    // tableInfo
    let table_info = table
        .descendants()
        .filter(|n| n.has_tag_name("tableInfo"))
        .collect::<Vec<Node>>();
    let style_info = match table_info.first() {
        Some(node) => {
            let name = node.attribute("name").map(|s| s.to_string());
            TableStyleInfo {
                name,
                show_first_column: get_bool_false(*node, "showFirstColumn"),
                show_last_column: get_bool_false(*node, "showLastColumn"),
                show_row_stripes: get_bool(*node, "showRowStripes"),
                show_column_stripes: get_bool_false(*node, "showColumnStripes"),
            }
        }
        None => TableStyleInfo {
            name: None,
            show_first_column: false,
            show_last_column: false,
            show_row_stripes: true,
            show_column_stripes: false,
        },
    };
    Ok(Table {
        name,
        display_name,
        reference,
        totals_row_count,
        header_row_count,
        header_row_dxf_id,
        data_dxf_id,
        totals_row_dxf_id,
        columns,
        style_info,
        has_filters,
        sheet_name: sheet_name.to_string(),
    })
}
