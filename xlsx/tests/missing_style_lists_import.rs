#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use std::io::{Cursor, Read, Write};

use ironcalc::import::load_from_xlsx_bytes;
use ironcalc_base::Model;

// Every list of a stylesheet (`fonts`, `fills`, `borders`, `cellStyleXfs`,
// `cellXfs`, `cellStyles`) is optional (`minOccurs="0"`), and files written by
// other tools leave some of them out. The importer used to index the first
// match of each and panicked when one was missing.

const LISTS: [&str; 6] = [
    "fonts",
    "fills",
    "borders",
    "cellStyleXfs",
    "cellXfs",
    "cellStyles",
];

/// tests/example.xlsx with the given lists removed from xl/styles.xml.
fn example_without(lists: &[&str]) -> Vec<u8> {
    let bytes = std::fs::read("tests/example.xlsx").unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut out = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(Cursor::new(&mut out));
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            let name = file.name().to_string();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            if name == "xl/styles.xml" {
                let mut text = String::from_utf8(data).unwrap();
                for list in lists {
                    let start = text.find(&format!("<{list} ")).unwrap();
                    let end_tag = format!("</{list}>");
                    let end = text[start..].find(&end_tag).unwrap() + start + end_tag.len();
                    text.replace_range(start..end, "");
                }
                data = text.into_bytes();
            }
            writer
                .start_file(name, zip::write::FileOptions::default())
                .unwrap();
            writer.write_all(&data).unwrap();
        }
        writer.finish().unwrap();
    }
    out
}

fn load(bytes: &[u8]) -> Model<'static> {
    let workbook = load_from_xlsx_bytes(bytes, "example", "en", "UTC").unwrap();
    Model::from_workbook(workbook, "en").unwrap()
}

/// Reads the style of every cell of the first sheet. Without `cellXfs` the cells
/// still point at the file's formats, which are gone: those are errors, not panics.
fn assert_styles_resolve(model: &Model, case: &str, cells_keep_formats: bool) {
    let dimension = model.workbook.worksheet(0).unwrap().dimension();
    for row in 1..=dimension.max_row {
        for column in 1..=dimension.max_column {
            let style = model.get_style_for_cell(0, row, column);
            if cells_keep_formats {
                assert!(style.is_ok(), "{case}: {row},{column}");
            }
        }
    }
}

#[test]
fn each_missing_style_list_loads() {
    for list in LISTS {
        let mut model = load(&example_without(&[list]));
        model.evaluate();
        assert_styles_resolve(&model, list, list != "cellXfs");
    }
}

#[test]
fn all_style_lists_missing_loads() {
    let mut model = load(&example_without(&LISTS));
    model.evaluate();
    let styles = &model.workbook.styles;
    assert!(!styles.fonts.is_empty());
    assert!(!styles.fills.is_empty());
    assert!(!styles.borders.is_empty());
    assert!(!styles.cell_style_xfs.is_empty());
    assert!(!styles.cell_xfs.is_empty());
    assert!(!styles.cell_styles.is_empty());
    assert_styles_resolve(&model, "all", false);
}
