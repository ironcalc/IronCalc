#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! A `<sheet>` in `xl/workbook.xml` names its worksheet part through `r:id`,
//! which shall reference that part (ECMA-376 Part 1, §12.3.24). A file whose
//! `r:id` is empty or points at no relationship does not conform; reading it
//! returns an error naming the sheet instead of panicking.

use std::io::Write;

use ironcalc::error::XlsxError;
use ironcalc::import::load_from_xlsx_bytes;

const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";

/// A workbook with two sheets: "Data" (r:id="rId1", a valid relationship) and
/// "Summary" with the given `r:id`.
fn xlsx(summary_rid: &str) -> Vec<u8> {
    let parts = [
        (
            "[Content_Types].xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
             <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
             <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
             <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
             <Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
             <Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>\
             </Types>"
                .to_string(),
        ),
        (
            "_rels/.rels",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>\
             </Relationships>"
                .to_string(),
        ),
        (
            "xl/workbook.xml",
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <workbook xmlns=\"{MAIN}\" \
                 xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
                 <sheets>\
                 <sheet name=\"Data\" sheetId=\"1\" r:id=\"rId1\"/>\
                 <sheet name=\"Summary\" sheetId=\"2\" r:id=\"{summary_rid}\"/>\
                 </sheets></workbook>"
            ),
        ),
        (
            "xl/_rels/workbook.xml.rels",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>\
             <Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>\
             </Relationships>"
                .to_string(),
        ),
        (
            "xl/worksheets/sheet1.xml",
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <worksheet xmlns=\"{MAIN}\"><sheetData>\
                 <row r=\"1\"><c r=\"A1\"><v>42</v></c></row>\
                 </sheetData></worksheet>"
            ),
        ),
        (
            "xl/styles.xml",
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <styleSheet xmlns=\"{MAIN}\">\
                 <fonts count=\"1\"><font><sz val=\"11\"/><name val=\"Calibri\"/></font></fonts>\
                 <fills count=\"1\"><fill><patternFill patternType=\"none\"/></fill></fills>\
                 <borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>\
                 <cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>\
                 <cellXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/></cellXfs>\
                 </styleSheet>"
            ),
        ),
    ];
    let mut bytes = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut bytes);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in parts {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    bytes.into_inner()
}

#[test]
fn sheet_with_valid_relationship_loads() {
    // The same workbook with both sheets pointing at the one worksheet part.
    let workbook = load_from_xlsx_bytes(&xlsx("rId1"), "rels", "en", "UTC").unwrap();
    assert_eq!(workbook.worksheets.len(), 2);
}

#[test]
fn sheet_with_empty_or_unknown_relationship_is_an_error() {
    for rid in ["", "rId99"] {
        match load_from_xlsx_bytes(&xlsx(rid), "rels", "en", "UTC") {
            Err(XlsxError::Xml(message)) => {
                assert!(message.contains("Summary"), "r:id=\"{rid}\": {message}");
            }
            Err(error) => panic!("r:id=\"{rid}\": unexpected error {error}"),
            Ok(_) => panic!("r:id=\"{rid}\": loaded a sheet with no relationship"),
        }
    }
}
