#![allow(clippy::unwrap_used)]

//! Whitespace at the ends of the text of a string, as desktop Excel reads it:
//! without `xml:space="preserve"` on the `<t>` or around it, the XML
//! whitespace (space, tab, line feed, carriage return) at the two ends of each
//! `<t>` is dropped; inside the text it stays, and so do other spaces such as
//! the no-break space. Every string here is made up; each answer is desktop
//! Excel's (Microsoft 365), measured on files of this shape.

use ironcalc::import::load_from_xlsx_bytes;
use ironcalc_base::Model;
use std::io::Write;

const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";

/// Attributes added to the opening tags of the file's elements (`c` to the
/// cells of the inline strings).
#[derive(Default)]
struct On<'a> {
    worksheet: &'a str,
    sheet_data: &'a str,
    row: &'a str,
    c: &'a str,
    is: &'a str,
    sst: &'a str,
    si: &'a str,
}

const PRESERVE: &str = " xml:space=\"preserve\"";

/// The bytes of an xlsx file with one sheet. Row n holds in A the shared
/// string whose `<si>` has the inner XML `shared[n - 1]`, and in B the inline
/// string whose `<is>` has the inner XML `inline[n - 1]`.
fn xlsx(shared: &[&str], inline: &[&str], on: &On) -> Vec<u8> {
    let On {
        worksheet,
        sheet_data,
        row: row_attributes,
        c,
        is,
        sst,
        si,
    } = on;
    let mut data = String::new();
    for row in 1..=shared.len().max(inline.len()) {
        data.push_str(&format!("<row r=\"{row}\"{row_attributes}>"));
        if row <= shared.len() {
            data.push_str(&format!("<c r=\"A{row}\" t=\"s\"><v>{}</v></c>", row - 1));
        }
        if let Some(inner) = inline.get(row - 1) {
            data.push_str(&format!(
                "<c r=\"B{row}\" t=\"inlineStr\"{c}><is{is}>{inner}</is></c>"
            ));
        }
        data.push_str("</row>");
    }
    let strings: String = shared
        .iter()
        .map(|inner| format!("<si{si}>{inner}</si>"))
        .collect();
    let parts = [
        (
            "[Content_Types].xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
             <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
             <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
             <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
             <Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\
             <Override PartName=\"/xl/sharedStrings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml\"/>\
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
                 <sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>"
            ),
        ),
        (
            "xl/_rels/workbook.xml.rels",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/>\
             <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings\" Target=\"sharedStrings.xml\"/>\
             <Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>\
             </Relationships>"
                .to_string(),
        ),
        (
            "xl/worksheets/sheet1.xml",
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <worksheet xmlns=\"{MAIN}\"{worksheet}><sheetData{sheet_data}>{data}</sheetData></worksheet>"
            ),
        ),
        (
            "xl/sharedStrings.xml",
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <sst xmlns=\"{MAIN}\"{sst}>{strings}</sst>"
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

/// The text of each cell of `column` in rows 1 to `rows` of the imported file.
fn texts(bytes: &[u8], column: i32, rows: usize) -> Vec<String> {
    let workbook = load_from_xlsx_bytes(bytes, "made-up", "en", "UTC").unwrap();
    let model = Model::from_workbook(workbook, "en").unwrap();
    (1..=rows as i32)
        .map(|row| model.get_formatted_cell_value(0, row, column).unwrap())
        .collect()
}

/// `cases` (the inner XML of an `<si>` or `<is>`, the text Excel reads) as
/// shared strings in column A and as inline strings in column B.
fn assert_read_as_excel(cases: &[(&str, &str)], inline_too: bool) {
    let xml: Vec<&str> = cases.iter().map(|(xml, _)| *xml).collect();
    let expected: Vec<String> = cases.iter().map(|(_, text)| text.to_string()).collect();
    let inline: &[&str] = if inline_too { &xml } else { &[] };
    let bytes = xlsx(&xml, inline, &On::default());
    assert_eq!(texts(&bytes, 1, cases.len()), expected, "shared strings");
    if inline_too {
        assert_eq!(texts(&bytes, 2, cases.len()), expected, "inline strings");
    }
}

#[test]
fn whitespace_at_the_ends_of_t_is_dropped() {
    assert_read_as_excel(
        &[
            ("<t> lead</t>", "lead"),
            ("<t>trail </t>", "trail"),
            ("<t>  both  </t>", "both"),
            ("<t>\ttab\t</t>", "tab"),
            ("<t>\nlf\n</t>", "lf"),
            ("<t>\r\ncrlf\r\n</t>", "crlf"),
            ("<t>&#9;tab&#10;</t>", "tab"),
            ("<t>&#13;cr&#13;</t>", "cr"),
            ("<t>   </t>", ""),
            ("<t>\n\t \n</t>", ""),
            ("<t>plain</t>", "plain"),
            ("<t></t>", ""),
        ],
        true,
    );
}

#[test]
fn whitespace_inside_the_text_and_other_spaces_stay() {
    assert_read_as_excel(
        &[
            ("<t> in  ner </t>", "in  ner"),
            ("<t>line1\nline2</t>", "line1\nline2"),
            ("<t>\u{a0}nbsp\u{a0}</t>", "\u{a0}nbsp\u{a0}"),
            ("<t>\u{3000}ideo\u{3000}</t>", "\u{3000}ideo\u{3000}"),
        ],
        true,
    );
}

#[test]
fn preserve_keeps_the_whitespace() {
    assert_read_as_excel(
        &[
            ("<t xml:space=\"preserve\"> kept </t>", " kept "),
            ("<t xml:space=\"default\"> dropped </t>", "dropped"),
        ],
        true,
    );
}

#[test]
fn each_run_of_rich_text_is_read_on_its_own() {
    assert_read_as_excel(
        &[
            ("<r><t> run1 </t></r><r><t> run2 </t></r>", "run1run2"),
            (
                "<r><t>Hello </t></r><r><t xml:space=\"preserve\"> World</t></r>",
                "Hello World",
            ),
            (
                "<r><t xml:space=\"preserve\">Hello </t></r><r><t xml:space=\"preserve\"> World</t></r>",
                "Hello  World",
            ),
            ("<r><t>A</t></r><r><t> </t></r><r><t>B</t></r>", "AB"),
            (
                "<r><rPr><b/><sz val=\"11\"/></rPr><t> bold </t></r><r><t>plain</t></r>",
                "boldplain",
            ),
        ],
        true,
    );
}

#[test]
fn escapes_are_decoded_after_the_ends_are_dropped() {
    // Inline strings do not decode `_xHHHH_` escapes yet, so shared strings only.
    assert_read_as_excel(
        &[
            ("<t>_x0020_esc_x0020_</t>", " esc "),
            ("<t> cr_x000D_ </t>", "cr\r"),
        ],
        false,
    );
}

#[test]
fn preserve_around_the_t_is_inherited() {
    let shared = ["<t> kept </t>", "<r><t> run1 </t></r><r><t> run2 </t></r>"];
    let expected = vec![" kept ".to_string(), " run1  run2 ".to_string()];
    let on_si = xlsx(
        &shared,
        &[],
        &On {
            si: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_si, 1, 2), expected, "on <si>");
    let on_sst = xlsx(
        &shared,
        &[],
        &On {
            sst: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_sst, 1, 2), expected, "on <sst>");
    let on_is = xlsx(
        &[],
        &shared,
        &On {
            is: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_is, 2, 2), expected, "on <is>");
    let on_c = xlsx(
        &[],
        &shared,
        &On {
            c: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_c, 2, 2), expected, "on <c>");
}

/// Above the cell the rule is XML's (Excel was measured on `<c>` and below):
/// `preserve` on the row, on `sheetData` or on the worksheet is inherited by
/// the inline strings under it, and `xml:space="default"` nearer the text
/// turns it off again.
#[test]
fn preserve_above_the_cell_is_inherited_and_default_turns_it_off() {
    let inline = ["<t> kept </t>", "<r><t> run1 </t></r><r><t> run2 </t></r>"];
    let kept = vec![" kept ".to_string(), " run1  run2 ".to_string()];
    let dropped = vec!["kept".to_string(), "run1run2".to_string()];
    let default = " xml:space=\"default\"";
    let on_row = xlsx(
        &[],
        &inline,
        &On {
            row: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_row, 2, 2), kept, "on <row>");
    let on_sheet_data = xlsx(
        &[],
        &inline,
        &On {
            sheet_data: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_sheet_data, 2, 2), kept, "on <sheetData>");
    let on_worksheet = xlsx(
        &[],
        &inline,
        &On {
            worksheet: PRESERVE,
            ..On::default()
        },
    );
    assert_eq!(texts(&on_worksheet, 2, 2), kept, "on <worksheet>");
    let off_on_c = xlsx(
        &[],
        &inline,
        &On {
            row: PRESERVE,
            c: default,
            ..On::default()
        },
    );
    assert_eq!(texts(&off_on_c, 2, 2), dropped, "on <row>, default on <c>");
    let off_on_row = xlsx(
        &[],
        &inline,
        &On {
            worksheet: PRESERVE,
            row: default,
            ..On::default()
        },
    );
    assert_eq!(
        texts(&off_on_row, 2, 2),
        dropped,
        "on <worksheet>, default on <row>"
    );
}
