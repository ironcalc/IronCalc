//! Errors stored as rich values.
//!
//! A `t="e"` cell only takes the errors of the xlsx standard. Excel writes the newer
//! ones (`#SPILL!`, `#CALC!`) as `#VALUE!`, with a `vm` (value metadata) attribute
//! pointing to the actual error (see `import/value_metadata.rs` for how it is read):
//!
//! ```text
//! <c r="A1" t="e" vm="1"><v>#VALUE!</v></c>
//! ```
//!
//! We write the same parts as Excel, with two rich values: `vm="1"` is `#SPILL!`
//! and `vm="2"` is `#CALC!`. They are only written if the workbook has one of those
//! errors.

use ironcalc_base::{
    expressions::token::Error,
    types::{Cell, FormulaValue, SpillValue, Workbook},
};

/// The `vm` attribute of a cell with an error, empty if the error is not one of those
/// kept in the value metadata.
pub(crate) fn get_value_metadata_attribute(ei: &Error) -> &'static str {
    match ei {
        Error::SPILL => " vm=\"1\"",
        Error::CALC => " vm=\"2\"",
        _ => "",
    }
}

/// Whether a cell of the workbook has an error kept in the value metadata.
pub(crate) fn has_value_metadata(workbook: &Workbook) -> bool {
    workbook.worksheets.iter().any(|worksheet| {
        worksheet
            .sheet_data
            .cells()
            .any(|(_, _, cell)| match &*cell {
                Cell::ErrorCell { ei, .. }
                | Cell::SpillCell {
                    v: SpillValue::Error(ei),
                    ..
                }
                | Cell::CellFormula {
                    v: FormulaValue::Error { ei, .. },
                    ..
                }
                | Cell::ArrayFormula {
                    v: FormulaValue::Error { ei, .. },
                    ..
                } => !get_value_metadata_attribute(ei).is_empty(),
                _ => false,
            })
    })
}

/// `xl/metadata.xml` of a workbook with errors in the value metadata: `metadata.xml`
/// plus the two rich values.
pub(crate) const METADATA_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<metadata xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:xlrd="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata" xmlns:xda="http://schemas.microsoft.com/office/spreadsheetml/2017/dynamicarray"><metadataTypes count="2"><metadataType name="XLDAPR" minSupportedVersion="120000" copy="1" pasteAll="1" pasteValues="1" merge="1" splitFirst="1" rowColShift="1" clearFormats="1" clearComments="1" assign="1" coerce="1" cellMeta="1"/><metadataType name="XLRICHVALUE" minSupportedVersion="120000" copy="1" pasteAll="1" pasteValues="1" merge="1" splitFirst="1" rowColShift="1" clearFormats="1" clearComments="1" assign="1" coerce="1"/></metadataTypes><futureMetadata name="XLDAPR" count="1"><bk><extLst><ext uri="{bdbb8cdc-fa1e-496e-a857-3c3f30c029c3}"><xda:dynamicArrayProperties fDynamic="1" fCollapsed="0"/></ext></extLst></bk></futureMetadata><futureMetadata name="XLRICHVALUE" count="2"><bk><extLst><ext uri="{3e2802c4-a4d2-4d8b-9148-e3be6c30e623}"><xlrd:rvb i="0"/></ext></extLst></bk><bk><extLst><ext uri="{3e2802c4-a4d2-4d8b-9148-e3be6c30e623}"><xlrd:rvb i="1"/></ext></extLst></bk></futureMetadata><cellMetadata count="1"><bk><rc t="1" v="0"/></bk></cellMetadata><valueMetadata count="2"><bk><rc t="2" v="0"/></bk><bk><rc t="2" v="1"/></bk></valueMetadata></metadata>"#;

/// `xl/richData/rdrichvalue.xml`: the `errorType` (8 is `#SPILL!`, 13 is `#CALC!`)
/// and the `subType` of each rich value.
pub(crate) const RICH_VALUE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rvData xmlns="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata" count="2"><rv s="0"><v>8</v><v>0</v></rv><rv s="0"><v>13</v><v>0</v></rv></rvData>"#;

/// `xl/richData/rdrichvaluestructure.xml`
pub(crate) const RICH_VALUE_STRUCTURE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rvStructures xmlns="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata" count="1"><s t="_error"><k n="errorType" t="i"/><k n="subType" t="i"/></s></rvStructures>"#;

/// `xl/richData/rdRichValueTypes.xml`, as Excel writes it.
pub(crate) const RICH_VALUE_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rvTypesInfo xmlns="http://schemas.microsoft.com/office/spreadsheetml/2017/richdata2" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="x" xmlns:x="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><global><keyFlags><key name="_Self"><flag name="ExcludeFromFile" value="1"/><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_DisplayString"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Flags"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Format"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_SubLabel"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Attribution"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Icon"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_Display"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_CanonicalPropertyNames"><flag name="ExcludeFromCalcComparison" value="1"/></key><key name="_ClassificationId"><flag name="ExcludeFromCalcComparison" value="1"/></key></keyFlags></global></rvTypesInfo>"#;

/// The parts in `xl/richData`: (file name, relationship type, content type, content).
/// The relationship types start with `http://schemas.microsoft.com/office/2017/06/relationships/`.
pub(crate) const RICH_DATA_PARTS: [(&str, &str, &str, &str); 3] = [
    (
        "rdrichvalue.xml",
        "rdRichValue",
        "application/vnd.ms-excel.rdrichvalue+xml",
        RICH_VALUE_XML,
    ),
    (
        "rdrichvaluestructure.xml",
        "rdRichValueStructure",
        "application/vnd.ms-excel.rdrichvaluestructure+xml",
        RICH_VALUE_STRUCTURE_XML,
    ),
    (
        "rdRichValueTypes.xml",
        "rdRichValueTypes",
        "application/vnd.ms-excel.rdrichvaluetypes+xml",
        RICH_VALUE_TYPES_XML,
    ),
];
