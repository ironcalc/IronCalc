#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::export::save_to_xlsx;
use ironcalc::import::{load_from_xlsx, load_from_xlsx_bytes};
use ironcalc_base::types::{Color, HorizontalAlignment, Link, MergedCell, VerticalAlignment};
use ironcalc_base::{Model, UserModel, ROW_HEIGHT_FACTOR};
use std::fs;
use std::io::Read;

// This is a functional test.
// We check that the output of example.xlsx is what we expect.
#[test]
fn test_example() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    // We should use the API once it is in place
    let workbook = model.workbook;
    let ws = &workbook.worksheets;
    let expected_names = vec![
        "Sheet1".to_string(),
        "Second".to_string(),
        "Sheet4".to_string(),
        "shared".to_string(),
        "Table".to_string(),
        "Sheet2".to_string(),
        "Created fourth".to_string(),
        "Frozen".to_string(),
        "Split".to_string(),
        "Hidden".to_string(),
    ];
    let names: Vec<String> = ws.iter().map(|s| s.name.clone()).collect();

    // One is not not imported and one is hidden
    assert_eq!(expected_names, names);

    assert_eq!(workbook.views[&0].sheet, 7);

    // Test selection:
    // First sheet (Sheet1)
    // E13 and E13:N20
    assert_eq!(ws[0].frozen_rows, 0);
    assert_eq!(ws[0].frozen_columns, 0);
    assert_eq!(ws[0].views[&0].row, 13);
    assert_eq!(ws[0].views[&0].column, 5);
    assert_eq!(ws[0].views[&0].range, [13, 5, 20, 14]);
}

#[test]
fn test_load_from_xlsx_bytes() {
    let file_path = std::path::Path::new("tests/example.xlsx");
    let mut file = fs::File::open(file_path).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let workbook = load_from_xlsx_bytes(&bytes, "home", "en", "UTC").unwrap();
    assert_eq!(workbook.views[&0].sheet, 7);
}

#[test]
fn no_grid() {
    let model = load_from_xlsx("tests/NoGrid.xlsx", "en", "UTC", "en").unwrap();
    {
        let workbook = &model.workbook;
        let ws = &workbook.worksheets;

        // NoGrid does not show grid lines
        let no_grid_sheet = &ws[0];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(!no_grid_sheet.show_grid_lines);

        let sheet2 = &ws[1];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(sheet2.show_grid_lines);

        let no_grid_no_headers_sheet = &ws[2];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        // There is also no headers
        assert!(!no_grid_no_headers_sheet.show_grid_lines);
    }
    {
        // save it and check again
        let temp_file_name = "temp_file_no_grid.xlsx";
        save_to_xlsx(&model, temp_file_name).unwrap();
        let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
        let workbook = &model.workbook;
        let ws = &workbook.worksheets;

        // NoGrid does not show grid lines
        let no_grid_sheet = &ws[0];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(!no_grid_sheet.show_grid_lines);

        let sheet2 = &ws[1];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        assert!(sheet2.show_grid_lines);

        let no_grid_no_headers_sheet = &ws[2];
        assert_eq!(no_grid_sheet.name, "NoGrid".to_string());
        // There is also no headers
        assert!(!no_grid_no_headers_sheet.show_grid_lines);
        fs::remove_file(temp_file_name).unwrap();
    }
}

#[test]
fn test_save_to_xlsx() {
    let mut model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_example.xlsx";
    // test can safe
    save_to_xlsx(&model, temp_file_name).unwrap();
    // test can open
    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    let metadata = &model.workbook.metadata;
    assert_eq!(metadata.application, "IronCalc Sheets");
    // FIXME: This will need to be updated once we fix versioning
    assert_eq!(metadata.app_version, "10.0000");

    let workbook = model.workbook;
    let ws = &workbook.worksheets;

    assert_eq!(workbook.views[&0].sheet, 7);

    // Test selection:
    // First sheet (Sheet1)
    // E13 and E13:N20
    assert_eq!(ws[0].frozen_rows, 0);
    assert_eq!(ws[0].frozen_columns, 0);
    assert_eq!(ws[0].views[&0].row, 13);
    assert_eq!(ws[0].views[&0].column, 5);
    assert_eq!(ws[0].views[&0].range, [13, 5, 20, 14]);
    // TODO: can we show it is the 'same' model?
    fs::remove_file(temp_file_name).unwrap();
}

#[test]
fn test_freeze() {
    // freeze has 3 frozen columns and 2 frozen rows
    let model = load_from_xlsx("tests/freeze.xlsx", "en", "UTC", "en")
        .unwrap()
        .workbook;
    assert_eq!(model.worksheets[0].frozen_rows, 2);
    assert_eq!(model.worksheets[0].frozen_columns, 3);
}

#[test]
fn test_split() {
    // We test that a workbook with split panes do not produce frozen rows and columns
    let model = load_from_xlsx("tests/split.xlsx", "en", "UTC", "en")
        .unwrap()
        .workbook;
    assert_eq!(model.worksheets[0].frozen_rows, 0);
    assert_eq!(model.worksheets[0].frozen_columns, 0);
}

fn test_model_has_correct_styles(model: &Model) {
    // A1 is bold
    let style_a1 = model.get_style_for_cell(0, 1, 1).unwrap();
    assert!(style_a1.font.b);
    assert!(!style_a1.font.i);
    assert!(!style_a1.font.u);

    // B1 is Italics
    let style_b1 = model.get_style_for_cell(0, 1, 2).unwrap();
    assert!(style_b1.font.i);
    assert!(!style_b1.font.b);
    assert!(!style_b1.font.u);

    // C1 Underlined
    let style_c1 = model.get_style_for_cell(0, 1, 3).unwrap();
    assert!(style_c1.font.u);
    assert!(!style_c1.font.b);
    assert!(!style_c1.font.i);

    // D1 Bold and Italics
    let style_d1 = model.get_style_for_cell(0, 1, 4).unwrap();
    assert!(style_d1.font.b);
    assert!(style_d1.font.i);
    assert!(!style_d1.font.u);

    // E1 Bold, italics and underlined
    let style_e1 = model.get_style_for_cell(0, 1, 5).unwrap();
    assert!(style_e1.font.b);
    assert!(style_e1.font.i);
    assert!(style_e1.font.u);
    assert!(!style_e1.font.strike);

    // F1 strikethrough
    let style_f1 = model.get_style_for_cell(0, 1, 6).unwrap();
    assert!(style_f1.font.strike);

    // G1 Double underlined just get simple underlined
    let style_g1 = model.get_style_for_cell(0, 1, 7).unwrap();
    assert!(style_g1.font.u);

    // Taken from the xlsx
    let ht_row3 = 68.0;
    let height_row_3 = model.workbook.worksheet(0).unwrap().row_height(3).unwrap();
    assert_eq!(height_row_3, ht_row3 * ROW_HEIGHT_FACTOR);

    // Taken from the xlsx
    let ht_row_5 = 31.0;
    let height_row_5 = model.workbook.worksheet(0).unwrap().row_height(5).unwrap();
    assert_eq!(height_row_5, ht_row_5 * ROW_HEIGHT_FACTOR);

    // Second sheet has alignment
    // Horizontal
    let alignment = model.get_style_for_cell(1, 2, 1).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model
        .get_style_for_cell(1, 3, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Left);

    let alignment = model
        .get_style_for_cell(1, 4, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Distributed);

    let alignment = model
        .get_style_for_cell(1, 5, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Right);

    let alignment = model
        .get_style_for_cell(1, 6, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Center);

    let alignment = model
        .get_style_for_cell(1, 7, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Fill);

    let alignment = model
        .get_style_for_cell(1, 8, 1)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.horizontal, HorizontalAlignment::Justify);

    // Vertical
    let alignment = model.get_style_for_cell(1, 2, 2).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model.get_style_for_cell(1, 3, 2).unwrap().alignment;
    assert_eq!(alignment, None);

    let alignment = model
        .get_style_for_cell(1, 4, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Top);

    let alignment = model
        .get_style_for_cell(1, 5, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Center);

    let alignment = model
        .get_style_for_cell(1, 6, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Justify);

    let alignment = model
        .get_style_for_cell(1, 7, 2)
        .unwrap()
        .alignment
        .unwrap();
    assert_eq!(alignment.vertical, VerticalAlignment::Distributed);
}

#[test]
fn test_simple_text() {
    let model = load_from_xlsx("tests/basic_text.xlsx", "en", "UTC", "en").unwrap();

    test_model_has_correct_styles(&model);

    let temp_file_name = "temp_file_test_named_styles.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();

    let model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    test_model_has_correct_styles(&model);
}

#[test]
fn test_defined_names_casing() {
    let test_file_path = "tests/calc_tests/defined_names_for_unit_test.xlsx";
    let loaded_workbook = load_from_xlsx(test_file_path, "en", "UTC", "en")
        .unwrap()
        .workbook;
    let mut model = Model::from_bytes(&bitcode::encode(&loaded_workbook), "en").unwrap();

    let (row, column) = (2, 13); // B13
    let test_cases = [
        ("=named1", "11"),
        ("=NAMED1", "11"),
        ("=NaMeD1", "11"),
        ("=named2", "22"),
        ("=NAMED2", "22"),
        ("=NaMeD2", "22"),
        ("=named3", "33"),
        ("=NAMED3", "33"),
        ("=NaMeD3", "33"),
    ];
    for (formula, expected_value) in test_cases {
        model
            .set_user_input(0, row, column, formula.to_string())
            .unwrap();
        model.evaluate();
        assert_eq!(
            model.get_formatted_cell_value(0, row, column).unwrap(),
            expected_value
        );
    }
}

// This test verifies whether exporting the merged cells functionality is happening properly or not.
// It first loads the Excel having the merged cell and exports it to another xlsx and verifies whether merged
// cell node is same in both of the xlsx file or not.
#[test]
fn test_exporting_merged_cells() {
    let temp_file_name = "temp_file_test_export_merged_cells.xlsx";
    let expected_merged_cells = {
        // loading the xlsx file containing merged cells
        let example_file_name = "tests/example.xlsx";
        let mut model = load_from_xlsx(example_file_name, "en", "UTC", "en").unwrap();
        let expected_merged_cells = model
            .workbook
            .worksheets
            .first()
            .unwrap()
            .merged_cells
            .clone();
        // example.xlsx has K7:L10 and H18:J20 merged in the first sheet
        assert_eq!(
            expected_merged_cells,
            vec![
                MergedCell {
                    row: 7,
                    column: 11,
                    width: 2,
                    height: 4
                },
                MergedCell {
                    row: 18,
                    column: 8,
                    width: 3,
                    height: 3
                },
            ]
        );
        // exporting and saving it in another xlsx
        model.evaluate();
        save_to_xlsx(&model, temp_file_name).unwrap();
        expected_merged_cells
    };
    {
        let mut temp_model = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
        {
            // loading the previous file back and verifying whether
            // merged cells got exported properly or not
            let got_merged_cells = &temp_model
                .workbook
                .worksheets
                .first()
                .unwrap()
                .merged_cells
                .clone();
            assert_eq!(expected_merged_cells, *got_merged_cells);
            fs::remove_file(temp_file_name).unwrap();
        }
        {
            // this block is to verify that if there are no
            // merged cells, exported xml should not have the
            // <mergeCells/> xml node
            temp_model
                .workbook
                .worksheets
                .get_mut(0)
                .unwrap()
                .merged_cells
                .clear();

            save_to_xlsx(&temp_model, temp_file_name).unwrap();
            let temp_model2 = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
            let got_merged_cells_count = &temp_model2
                .workbook
                .worksheets
                .first()
                .unwrap()
                .merged_cells
                .len();
            assert!(*got_merged_cells_count == 0);
        }
    }

    fs::remove_file(temp_file_name).unwrap();
}

// bad_merge_cells.xlsx has a pretty-printed (whitespace-indented) mergeCells
// section with the entries B2:C3 (valid), C3:D4 (overlaps the previous one),
// A10:A10 (single cell), FOO (garbage) and E5:F6 (valid). The anchor B2 holds
// the number 42 and the covered cell C2 the number 43. Import must keep only
// the two valid ranges, keep the anchor content and clear the content left in
// covered cells.
#[test]
fn test_import_sanitizes_merged_cells() {
    let model = load_from_xlsx("tests/bad_merge_cells.xlsx", "en", "UTC", "en").unwrap();
    let merged_cells = &model.workbook.worksheets.first().unwrap().merged_cells;
    assert_eq!(
        merged_cells,
        &vec![
            MergedCell {
                row: 2,
                column: 2,
                width: 2,
                height: 2
            },
            MergedCell {
                row: 5,
                column: 5,
                width: 2,
                height: 2
            },
        ]
    );
    // the anchor keeps its content
    assert_eq!(model.is_empty_cell(0, 2, 2), Ok(false));
    // the covered cell is cleared
    assert_eq!(model.is_empty_cell(0, 2, 3), Ok(true));
}

#[test]
fn test_user_model() {
    let temp_file_name = "temp_file_test_user_model.xlsx";
    let mut model = UserModel::new_empty("my_model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "=1+1").unwrap();

    // test we can use `get_model` to save the model
    save_to_xlsx(model.get_model(), temp_file_name).unwrap();
    fs::remove_file(temp_file_name).unwrap();

    // we can still use the model afterwards
    model.set_rows_height(0, 1, 1, 100.0).unwrap();
}

// This is produced with:
// from openpyxl import Workbook

// # Create new workbook
// wb = Workbook()
// ws = wb.active

// # Write text and formula
// ws['A1'] = 'Hello, World!'
// ws['A2'] = '=1+1'

// ws['B1'] = '=CONCAT("It is", " what it is")'

// # Save
// wb.save('openpyxl_example.xlsx')
#[test]
fn test_pyopenxl_example() {
    let mut model = load_from_xlsx("tests/openpyxl_example.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();

    let a1 = model.get_formatted_cell_value(0, 1, 1).unwrap();
    assert_eq!(a1, "Hello, World!");

    let a2 = model.get_formatted_cell_value(0, 2, 1).unwrap();
    assert_eq!(a2, "2");

    let b1 = model.get_formatted_cell_value(0, 1, 2).unwrap();
    assert_eq!(b1, "It is what it is");
}

fn assert_eq_ignoring_metadata_and_name(
    workbook1: ironcalc_base::types::Workbook,
    workbook2: ironcalc_base::types::Workbook,
) {
    let mut w2 = workbook2.clone();
    w2.metadata = workbook1.metadata.clone();
    w2.name = workbook1.name.clone();
    assert_eq!(workbook1, w2);
}

#[test]
fn test_dynamic_arrays() {
    let model = load_from_xlsx("tests/dynamic_arrays.xlsx", "en", "UTC", "en").unwrap();
    let temp_file_name = "temp_file_test_dynamic_arrays.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model2 = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    assert_eq_ignoring_metadata_and_name(model.workbook, model2.workbook);
}

#[test]
// This tests the `xl/worksheets/_rels/sheet*` are parsed correctly
// libreoffice sometimes exports .xlsx file with whitespace in the <Relationships> element
fn test_relationship_whitespace_example() {
    let mut model =
        load_from_xlsx("tests/libreoffice_888_example.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
}

#[test]
fn test_missing_r_on_row() {
    let mut model = load_from_xlsx("tests/missing_r_on_row.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
}

fn external_link(target: &str) -> Link {
    Link::External {
        target: target.to_string(),
        tooltip: None,
    }
}

fn internal_link(location: &str) -> Link {
    Link::Internal {
        location: location.to_string(),
        tooltip: None,
    }
}

// link_test.xlsx has two sheets, "Sheet1" and "Target". Sheet1 has 13 hyperlinks in
// column B: external links of every flavour (https, ftp, mailto, file) in B2:B9 and
// B29, and internal links (cell references and a defined name) in B10:B12 and B33.
#[test]
fn test_hyperlinks_import() {
    let model = load_from_xlsx("tests/link_test.xlsx", "en", "UTC", "en").unwrap();
    let links = &model.workbook.worksheets[0].links;

    assert_eq!(links.len(), 13);
    assert_eq!(
        links.get(&(2, 2)),
        Some(&external_link("http://www.ironcalc.com/"))
    );
    assert_eq!(
        links.get(&(3, 2)),
        Some(&external_link("https://www.microsoft.com/"))
    );
    // B4 has a tooltip (called ScreenTip in Excel)
    assert_eq!(
        links.get(&(4, 2)),
        Some(&Link::External {
            target: "https://support.microsoft.com/".to_string(),
            tooltip: Some("This is a ScreenTip / tooltip".to_string()),
        })
    );
    assert_eq!(
        links.get(&(5, 2)),
        Some(&external_link("ftp://ftp.gnu.org/"))
    );
    assert_eq!(
        links.get(&(6, 2)),
        Some(&external_link("mailto:someone@example.com"))
    );
    // the &amp; in the rels part is XML-decoded, the percent-encoding is kept
    assert_eq!(
        links.get(&(7, 2)),
        Some(&external_link(
            "mailto:someone@example.com?subject=Test%20Subject&body=Hello%20there"
        ))
    );
    assert_eq!(
        links.get(&(8, 2)),
        Some(&external_link("file:///C:/Temp/test.xlsx"))
    );
    assert_eq!(
        links.get(&(9, 2)),
        Some(&external_link("file:///share/report.xlsx"))
    );
    assert_eq!(links.get(&(10, 2)), Some(&internal_link("Sheet1!A30")));
    assert_eq!(links.get(&(11, 2)), Some(&internal_link("Target!A1")));
    assert_eq!(links.get(&(12, 2)), Some(&internal_link("NamedTarget")));
    assert_eq!(
        links.get(&(29, 2)),
        Some(&external_link(
            "mailto:daniel@ironcalc.com?subject=hola%20que%20tal"
        ))
    );
    // B33 is an internal link with a tooltip
    assert_eq!(
        links.get(&(33, 2)),
        Some(&Link::Internal {
            location: "Target!A1".to_string(),
            tooltip: Some("Pronto sucedera".to_string()),
        })
    );

    // The second sheet has no links
    assert!(model.workbook.worksheets[1].links.is_empty());
}

#[test]
fn test_hyperlinks_export_roundtrip() {
    let mut model = load_from_xlsx("tests/link_test.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
    let temp_file_name = "temp_file_link_test.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let model2 = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();

    assert_eq!(
        model.workbook.worksheets[0].links,
        model2.workbook.worksheets[0].links
    );
    assert_eq!(
        model.workbook.worksheets[1].links,
        model2.workbook.worksheets[1].links
    );
}

#[test]
// This tests theme color resolution against the workbook's `xl/theme/theme1.xml`
// rather than the hardcoded Office 2013 palette. custom_theme_colors.xlsx ships
// a custom theme where accent6 = #C9211E; B15 uses <fgColor theme="9"/>, which
// must resolve to that red.
fn test_workbook_theme_colors() {
    let model = load_from_xlsx("tests/custom_theme_colors.xlsx", "en", "UTC", "en").unwrap();
    let style_b15 = model.get_style_for_cell(0, 15, 2).unwrap();
    // Theme index 9 = accent6 in OOXML; the custom theme sets accent6 = #C9211E
    assert_eq!(style_b15.fill.color, Color::Theme(9, 0.0));
    assert_eq!(
        style_b15.fill.color.to_rgb(&model.workbook.theme),
        "#C9211E".to_string()
    );
}
