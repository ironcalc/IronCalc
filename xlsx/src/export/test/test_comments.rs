use std::fs;
use std::io::{Read, Write};

use ironcalc_base::types::{Comment, Link};
use ironcalc_base::Model;

use crate::export::save_to_xlsx;
use crate::import::load_from_xlsx;

fn comment(cell_ref: &str, author_name: &str, text: &str) -> Comment {
    Comment {
        text: text.to_string(),
        author_name: author_name.to_string(),
        author_id: None,
        cell_ref: cell_ref.to_string(),
    }
}

fn model_with_comments<'a>() -> Model<'a> {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.add_sheet("Second").unwrap();
    model.set_user_input(0, 1, 1, "42".to_string()).unwrap();
    model.workbook.worksheets[0].comments = vec![
        comment("A1", "Ann", "The answer"),
        comment("C7", "Bo", "Rent & <deposit> \"incl.\"\nsecond line"),
        comment("AA100", "Ann", "你好世界！"),
    ];
    model.workbook.worksheets[1].comments = vec![comment("B2", "", "No author")];
    model.evaluate();
    model
}

#[test]
fn comments_round_trip() {
    let model = model_with_comments();
    let temp_file_name = "temp_file_test_comments_round_trip.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let loaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert_eq!(
        loaded.workbook.worksheets[0].comments,
        model.workbook.worksheets[0].comments
    );
    assert_eq!(
        loaded.workbook.worksheets[1].comments,
        model.workbook.worksheets[1].comments
    );
}

#[test]
fn no_comments_writes_no_parts() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_test_no_comments.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let names = zip_entry_names(temp_file_name);
    fs::remove_file(temp_file_name).unwrap();

    assert!(!names
        .iter()
        .any(|n| n.contains("comments") || n.contains("vml")));
}

#[test]
fn comments_and_hyperlinks_get_distinct_relationship_ids() {
    let mut model = model_with_comments();
    let links = &mut model.workbook.worksheets[0].links;
    for (row, target) in [(2, "https://ironcalc.com/"), (3, "https://example.com/")] {
        links.insert(
            (row, 1),
            Link::External {
                target: target.to_string(),
                tooltip: None,
            },
        );
    }
    let temp_file_name = "temp_file_test_comments_and_links.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let rels = zip_entry(temp_file_name, "xl/worksheets/_rels/sheet1.xml.rels");
    let sheet = zip_entry(temp_file_name, "xl/worksheets/sheet1.xml");
    let loaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert!(rels.contains(r#"Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments""#));
    assert!(rels.contains(r#"Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/vmlDrawing""#));
    assert!(sheet.contains(r#"<legacyDrawing r:id="rId4"/>"#));
    assert_eq!(loaded.workbook.worksheets[0].links.len(), 2);
    assert_eq!(loaded.workbook.worksheets[0].comments.len(), 3);
}

#[test]
fn every_part_has_a_content_type() {
    let model = model_with_comments();
    let temp_file_name = "temp_file_test_comments_content_types.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let content_types = zip_entry(temp_file_name, "[Content_Types].xml");
    let names = zip_entry_names(temp_file_name);
    fs::remove_file(temp_file_name).unwrap();

    for part in [
        "xl/comments1.xml",
        "xl/comments2.xml",
        "xl/drawings/vmlDrawing1.vml",
        "xl/drawings/vmlDrawing2.vml",
    ] {
        assert!(names.iter().any(|n| n == part), "{part} missing");
    }
    assert!(content_types.contains(r#"<Override PartName="/xl/comments1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.comments+xml"/>"#));
    assert!(content_types.contains(r#"<Override PartName="/xl/comments2.xml""#));
    assert!(content_types.contains(r#"<Default Extension="vml" ContentType="application/vnd.openxmlformats-officedocument.vmlDrawing"/>"#));
}

/// openpyxl, among others, writes absolute targets such as `/xl/comments/comment1.xml`.
#[test]
fn comments_with_an_absolute_target_are_imported() {
    let model = model_with_comments();
    let temp_file_name = "temp_file_test_comments_absolute.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let rewritten_file_name = "temp_file_test_comments_absolute_rewritten.xlsx";
    rewrite_zip(temp_file_name, rewritten_file_name, |name, bytes| {
        if name == "xl/worksheets/_rels/sheet1.xml.rels" {
            String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "Target=\"../comments1.xml\"",
                    "Target=\"/xl/comments1.xml\"",
                )
                .into_bytes()
        } else {
            bytes
        }
    });
    let loaded = load_from_xlsx(rewritten_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    fs::remove_file(rewritten_file_name).unwrap();

    assert_eq!(
        loaded.workbook.worksheets[0].comments,
        model.workbook.worksheets[0].comments
    );
}

fn zip_entry_names(file_name: &str) -> Vec<String> {
    let archive = zip::ZipArchive::new(fs::File::open(file_name).unwrap()).unwrap();
    archive.file_names().map(str::to_string).collect()
}

fn zip_entry(file_name: &str, entry: &str) -> String {
    let mut archive = zip::ZipArchive::new(fs::File::open(file_name).unwrap()).unwrap();
    let mut content = String::new();
    archive
        .by_name(entry)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();
    content
}

fn rewrite_zip(from: &str, to: &str, change: impl Fn(&str, Vec<u8>) -> Vec<u8>) {
    let mut archive = zip::ZipArchive::new(fs::File::open(from).unwrap()).unwrap();
    let mut writer = zip::ZipWriter::new(fs::File::create(to).unwrap());
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let name = file.name().to_string();
        if file.is_dir() {
            writer
                .add_directory(name, zip::write::FileOptions::default())
                .unwrap();
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        writer
            .start_file(name.clone(), zip::write::FileOptions::default())
            .unwrap();
        writer.write_all(&change(&name, bytes)).unwrap();
    }
    writer.finish().unwrap();
}
