#![allow(clippy::unwrap_used)]
//! <sheet name="Sheet1" sheetId="1" r:id="rId1"/>

//! A workbook is composed of workbook-level properties and a collection of 1 or more sheets.
//! The workbook part and corresponding properties comprise data
//! used to set application and workbook-level operational state. The workbook also serves to bind all the sheets
//! and child elements into an organized single file. The workbook XML attributes and elements include information
//! about what application last saved the file, where and how the windows of the workbook were positioned, and
//! an enumeration of the worksheets in the workbook.
//! This is the XML for the smallest possible (blank) workbook:
//!
//! <workbook>
//!   <sheets>
//!     <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
//!   </sheets>
//! </workbook>
//!
//! Note that this workbook has a single sheet, named Sheet1. An Id for the sheet is required, and a relationship Id
//! pointing to the location of the sheet definition is also required.
//!
//!
//!
//! The most important objet of this part is a collection of all the sheets and all the defined names
//! of the workbook.
//!
//! It also may hold state properties like the selected tab

//! # bookViews
//!

use std::collections::HashMap;

use ironcalc_base::types::{SheetState, Workbook};

use super::escape::escape_xml;
use super::xml_constants::XML_DECLARATION;

pub(crate) fn get_workbook_xml(workbook: &Workbook, selected_sheet: u32) -> String {
    // sheets
    // <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
    let mut sheets_str: Vec<String> = vec![];
    let mut sheet_id_to_sheet_index: HashMap<u32, u32> = HashMap::new();
    for (sheet_index, worksheet) in workbook.worksheets.iter().enumerate() {
        let name = &worksheet.name;
        let name = escape_xml(name);
        let sheet_id = worksheet.sheet_id;
        let state_str = match &worksheet.state {
            SheetState::Visible => "",
            SheetState::Hidden => " state=\"hidden\"",
            SheetState::VeryHidden => " state=\"veryHidden\"",
        };

        sheets_str.push(format!(
            "<sheet name=\"{name}\" sheetId=\"{sheet_id}\" r:id=\"rId{}\"{state_str}/>",
            sheet_index + 1
        ));
        sheet_id_to_sheet_index.insert(sheet_id, sheet_index as u32);
    }

    // defined names
    // <definedName localSheetId="4" name="answer">shared!$G$5</definedName>
    // <definedName name="numbers">Sheet1!$A$16:$A$18</definedName>
    let mut defined_names_str: Vec<String> = vec![];
    for defined_name in &workbook.defined_names {
        let name = &defined_name.name;
        let name = escape_xml(name);
        let local_sheet_id = if let Some(sheet_id) = defined_name.sheet_id {
            // In Excel the localSheetId is actually the index of the sheet.
            let excel_local_sheet_id = sheet_id_to_sheet_index.get(&sheet_id).unwrap();
            format!(" localSheetId=\"{excel_local_sheet_id}\"")
        } else {
            "".to_string()
        };
        let formula = escape_xml(&defined_name.formula);
        defined_names_str.push(format!(
            "<definedName name=\"{name}\"{local_sheet_id}>{formula}</definedName>"
        ))
    }

    let sheets = sheets_str.join("");
    let defined_names = defined_names_str.join("");
    format!("{XML_DECLARATION}\n\
    <workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
    <bookViews>
    <workbookView activeTab=\"{selected_sheet}\"/>\
    </bookViews>
      <sheets>\
        {sheets}\
      </sheets>\
      <definedNames>\
        {defined_names}\
      </definedNames>\
      <calcPr/>\
    </workbook>")
}
