use hipdf::lopdf::Document;
use ironcalc_base::types::{
    Alignment, Border, BorderItem, BorderStyle, Color, Fill, Font, HorizontalAlignment, MergedCell,
    Style, VerticalAlignment,
};
use ironcalc_base::Model;
use ironcalc_pdf::{
    export_to_pdf, BookmarkPolicy, CellErrorMode, CellErrorPolicy, CellRange, ExportScope,
    FormulaPrintPolicy, HeaderFooterField, HeaderFooterLayout, PaperSize, PdfExportOptions,
};

fn create_sample_model() -> Model<'static> {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    model
        .set_user_input(0, 1, 1, "Quarterly Report".to_string())
        .expect("set A1");
    model
        .set_user_input(0, 2, 1, "Category".to_string())
        .expect("set A2");
    model
        .set_user_input(0, 2, 2, "Q1".to_string())
        .expect("set B2");
    model
        .set_user_input(0, 2, 3, "Q2".to_string())
        .expect("set C2");
    model
        .set_user_input(0, 2, 4, "Total".to_string())
        .expect("set D2");

    model
        .set_user_input(0, 3, 1, "Revenue".to_string())
        .expect("set A3");
    model
        .set_user_input(0, 3, 2, "12500".to_string())
        .expect("set B3");
    model
        .set_user_input(0, 3, 3, "15800".to_string())
        .expect("set C3");
    model
        .set_user_input(0, 3, 4, "=B3+C3".to_string())
        .expect("set D3");

    model
        .set_user_input(0, 4, 1, "Expenses".to_string())
        .expect("set A4");
    model
        .set_user_input(0, 4, 2, "8200".to_string())
        .expect("set B4");
    model
        .set_user_input(0, 4, 3, "9100".to_string())
        .expect("set C4");
    model
        .set_user_input(0, 4, 4, "=B4+C4".to_string())
        .expect("set D4");

    model
        .set_user_input(0, 5, 1, "Profit".to_string())
        .expect("set A5");
    model
        .set_user_input(0, 5, 2, "=B3-B4".to_string())
        .expect("set B5");
    model
        .set_user_input(0, 5, 3, "=C3-C4".to_string())
        .expect("set C5");
    model
        .set_user_input(0, 5, 4, "=D3-D4".to_string())
        .expect("set D5");

    model.evaluate();
    model
}

#[test]
fn test_export_empty_sheet() {
    let model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    let options = PdfExportOptions::default();
    let pdf_bytes = export_to_pdf(&model, &options).expect("export to pdf");

    assert!(!pdf_bytes.is_empty());
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    // Verify valid PDF structure by parsing with lopdf
    let doc = Document::load_mem(&pdf_bytes).expect("parse generated pdf");
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1);
}

#[test]
fn test_export_sample_spreadsheet() {
    let model = create_sample_model();
    let options = PdfExportOptions::default()
        .with_gridlines(true)
        .with_headings(true);

    let pdf_bytes = export_to_pdf(&model, &options).expect("export to pdf");
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    let doc = Document::load_mem(&pdf_bytes).expect("parse generated pdf");
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1);

    // Verify media box matches A4 portrait
    let first_page_id = *pages.values().next().unwrap();
    let page_obj = doc.get_object(first_page_id).unwrap().as_dict().unwrap();
    let media_box = page_obj.get(b"MediaBox").unwrap().as_array().unwrap();
    assert_eq!(media_box.len(), 4);
    let width = media_box[2]
        .as_f32()
        .or_else(|_| media_box[2].as_i64().map(|v| v as f32))
        .unwrap();
    let height = media_box[3]
        .as_f32()
        .or_else(|_| media_box[3].as_i64().map(|v| v as f32))
        .unwrap();
    assert!((width - 595.27).abs() < 2.0);
    assert!((height - 841.89).abs() < 2.0);
}

#[test]
fn test_landscape_orientation() {
    let model = create_sample_model();
    let options = PdfExportOptions::default().landscape();

    let pdf_bytes = export_to_pdf(&model, &options).expect("export to pdf");
    let doc = Document::load_mem(&pdf_bytes).expect("parse generated pdf");
    let pages = doc.get_pages();
    let first_page_id = *pages.values().next().unwrap();
    let page_obj = doc.get_object(first_page_id).unwrap().as_dict().unwrap();
    let media_box = page_obj.get(b"MediaBox").unwrap().as_array().unwrap();
    let width = media_box[2]
        .as_f32()
        .or_else(|_| media_box[2].as_i64().map(|v| v as f32))
        .unwrap();
    let height = media_box[3]
        .as_f32()
        .or_else(|_| media_box[3].as_i64().map(|v| v as f32))
        .unwrap();

    // In landscape, width is greater than height
    assert!(width > height);
    assert!((width - 841.89).abs() < 2.0);
    assert!((height - 595.27).abs() < 2.0);
}

#[test]
fn test_paper_sizes() {
    let model = create_sample_model();

    // Letter size (612.0 x 792.0 pt)
    let options = PdfExportOptions::default().with_paper_size(PaperSize::Letter);
    let pdf_bytes = export_to_pdf(&model, &options).expect("export letter");
    let doc = Document::load_mem(&pdf_bytes).unwrap();
    let pages = doc.get_pages();
    let page_obj = doc
        .get_object(*pages.values().next().unwrap())
        .unwrap()
        .as_dict()
        .unwrap();
    let media_box = page_obj.get(b"MediaBox").unwrap().as_array().unwrap();
    let w = media_box[2]
        .as_f32()
        .or_else(|_| media_box[2].as_i64().map(|v| v as f32))
        .unwrap();
    let h = media_box[3]
        .as_f32()
        .or_else(|_| media_box[3].as_i64().map(|v| v as f32))
        .unwrap();
    assert!((w - 612.0).abs() < 2.0);
    assert!((h - 792.0).abs() < 2.0);

    // Custom size (150mm x 200mm)
    let options_custom = PdfExportOptions::default().with_paper_size(PaperSize::Custom {
        width_mm: 150.0,
        height_mm: 200.0,
    });
    let pdf_bytes_custom = export_to_pdf(&model, &options_custom).expect("export custom");
    let doc_custom = Document::load_mem(&pdf_bytes_custom).unwrap();
    let pages_custom = doc_custom.get_pages();
    let page_obj_custom = doc_custom
        .get_object(*pages_custom.values().next().unwrap())
        .unwrap()
        .as_dict()
        .unwrap();
    let media_box_custom = page_obj_custom
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap();
    let w_custom = media_box_custom[2]
        .as_f32()
        .or_else(|_| media_box_custom[2].as_i64().map(|v| v as f32))
        .unwrap();
    let h_custom = media_box_custom[3]
        .as_f32()
        .or_else(|_| media_box_custom[3].as_i64().map(|v| v as f32))
        .unwrap();
    assert!((w_custom - 150.0 * 72.0 / 25.4).abs() < 2.0);
    assert!((h_custom - 200.0 * 72.0 / 25.4).abs() < 2.0);
}

#[test]
fn test_pagination_multipage_and_fit_to_page() {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");

    // Populate a tall table (80 rows)
    for row in 1..=80 {
        model
            .set_user_input(0, row, 1, format!("Row {row}"))
            .unwrap();
        model
            .set_user_input(0, row, 2, format!("{}", row * 100))
            .unwrap();
    }
    model.evaluate();

    // Default: multipage allowed -> should produce multiple pages
    let default_options = PdfExportOptions::default();
    let pdf_bytes_multi = export_to_pdf(&model, &default_options).expect("multipage export");
    let doc_multi = Document::load_mem(&pdf_bytes_multi).unwrap();
    let page_count_multi = doc_multi.get_pages().len();
    assert!(
        page_count_multi > 1,
        "80 rows should span multiple pages by default, got {page_count_multi}"
    );

    // Fit to page: should force all content onto exactly 1 page
    let fit_options = PdfExportOptions::default().fit_to_page();
    let pdf_bytes_fit = export_to_pdf(&model, &fit_options).expect("fit to page export");
    let doc_fit = Document::load_mem(&pdf_bytes_fit).unwrap();
    let page_count_fit = doc_fit.get_pages().len();
    assert_eq!(
        page_count_fit, 1,
        "fit_to_page should produce exactly 1 page"
    );
}

#[test]
fn test_multisheet_and_bookmarks() {
    let mut model = Model::new_empty("Summary", "en", "UTC", "en").expect("create model");
    model
        .set_user_input(0, 1, 1, "Summary Data".to_string())
        .unwrap();

    model.new_sheet();
    model
        .rename_sheet_by_index(1, "Details")
        .expect("rename sheet 2");
    model
        .set_user_input(1, 1, 1, "Detailed Breakdown".to_string())
        .unwrap();

    model.new_sheet();
    model
        .rename_sheet_by_index(2, "Charts")
        .expect("rename sheet 3");
    model
        .set_user_input(2, 1, 1, "Chart Metrics".to_string())
        .unwrap();
    model.evaluate();

    let options = PdfExportOptions::default()
        .with_scope(ExportScope::EntireWorkbook)
        .with_bookmarks(BookmarkPolicy::AlwaysBySheet);

    let pdf_bytes = export_to_pdf(&model, &options).expect("export entire workbook");
    let doc = Document::load_mem(&pdf_bytes).unwrap();
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 3, "Should have 3 pages for 3 sheets");

    // Verify Catalog contains Outlines (Bookmarks)
    let catalog = doc.catalog().unwrap();
    assert!(
        catalog.has(b"Outlines"),
        "Catalog must contain Outlines for bookmarks"
    );
}

#[test]
fn test_export_selection_scope() {
    let model = create_sample_model();
    // Select only range B2:D3
    let options = PdfExportOptions::default().with_scope(ExportScope::Selection {
        sheet: 0,
        range: CellRange::new(2, 2, 3, 4),
    });

    let pdf_bytes = export_to_pdf(&model, &options).expect("export selection");
    let doc = Document::load_mem(&pdf_bytes).unwrap();
    assert_eq!(doc.get_pages().len(), 1);
}

#[test]
fn test_headers_and_footers() {
    let model = create_sample_model();
    let header_layout = HeaderFooterLayout::new(
        HeaderFooterField::SheetName,
        HeaderFooterField::Text("Confidential".to_string()),
        HeaderFooterField::Date,
    );
    let footer_layout = HeaderFooterLayout::new(
        HeaderFooterField::WorkbookName,
        HeaderFooterField::PageNumberOfTotal,
        HeaderFooterField::Time,
    );

    let options = PdfExportOptions::default().with_header_footer(header_layout, footer_layout);
    let pdf_bytes = export_to_pdf(&model, &options).expect("export with headers and footers");
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    let doc = Document::load_mem(&pdf_bytes).unwrap();
    assert_eq!(doc.get_pages().len(), 1);
}

#[test]
fn test_ocg_layers_present() {
    let model = create_sample_model();
    let options = PdfExportOptions::default()
        .with_gridlines(true)
        .with_headings(true);

    let pdf_bytes = export_to_pdf(&model, &options).expect("export pdf");
    let doc = Document::load_mem(&pdf_bytes).unwrap();

    let catalog = doc.catalog().unwrap();
    assert!(
        catalog.has(b"OCProperties"),
        "Catalog must contain OCProperties dictionary for hipdf layers"
    );
}

#[test]
fn test_merged_cells_rendering() {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    // Merge A1:C2
    let merge = MergedCell {
        row: 1,
        column: 1,
        width: 3,
        height: 2,
    };
    model.workbook.worksheets[0].merged_cells.push(merge);
    model
        .set_user_input(0, 1, 1, "Big Merged Title".to_string())
        .unwrap();

    model
        .set_user_input(0, 3, 1, "Detail 1".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 2, "Detail 2".to_string())
        .unwrap();
    model
        .set_user_input(0, 3, 3, "Detail 3".to_string())
        .unwrap();
    model.evaluate();

    let options = PdfExportOptions::default()
        .with_gridlines(true)
        .fit_to_page();

    let pdf_bytes = export_to_pdf(&model, &options).expect("export with merged cells");
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    let doc = Document::load_mem(&pdf_bytes).unwrap();
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1);

    let page_id = *pages.get(&1).unwrap();
    let content_bytes = doc.get_page_content(page_id).unwrap();
    let content = hipdf::lopdf::content::Content::decode(&content_bytes).unwrap();

    // Verify that the horizontal grid line between row 1 and row 2 is omitted:
    // In a 3-row grid without merges, there are 4 horizontal lines:
    // row 1 top, row 2 top, row 3 top, row 3 bottom.
    // For A1:C2, row 2 top is interior to the merge across all 3 columns, so it is omitted.
    // Only 3 horizontal gridlines should be stroked.
    let mut horizontal_lines = 0;
    let mut _vertical_lines = 0;
    for i in 0..content.operations.len().saturating_sub(2) {
        if content.operations[i].operator == "m"
            && content.operations[i + 1].operator == "l"
            && content.operations[i + 2].operator == "S"
        {
            let m_ops = &content.operations[i].operands;
            let l_ops = &content.operations[i + 1].operands;
            let y1 = m_ops[1]
                .as_float()
                .or_else(|_| m_ops[1].as_i64().map(|v| v as f32))
                .unwrap();
            let y2 = l_ops[1]
                .as_float()
                .or_else(|_| l_ops[1].as_i64().map(|v| v as f32))
                .unwrap();
            let x1 = m_ops[0]
                .as_float()
                .or_else(|_| m_ops[0].as_i64().map(|v| v as f32))
                .unwrap();
            let x2 = l_ops[0]
                .as_float()
                .or_else(|_| l_ops[0].as_i64().map(|v| v as f32))
                .unwrap();

            if (y1 - y2).abs() < 0.01 && (x2 - x1).abs() > 1.0 {
                horizontal_lines += 1;
            } else if (x1 - x2).abs() < 0.01 && (y1 - y2).abs() > 1.0 {
                _vertical_lines += 1;
            }
        }
    }
    // Exactly 3 horizontal lines (top of row 1, bottom of merge / top of row 3, bottom of row 3)
    assert_eq!(
        horizontal_lines, 3,
        "Interior horizontal gridline between row 1 and 2 must be omitted"
    );
}

#[test]
fn test_merged_cells_borders_rendering() {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    // Merge B2:C3
    let merge = MergedCell {
        row: 2,
        column: 2,
        width: 2,
        height: 2,
    };
    model.workbook.worksheets[0].merged_cells.push(merge);
    model.set_user_input(0, 1, 1, "A1".to_string()).unwrap();
    model
        .set_user_input(0, 2, 2, "Merged B2:C3".to_string())
        .unwrap();
    model.set_user_input(0, 4, 4, "D4".to_string()).unwrap();

    // Set border on the merge anchor cell
    let mut style = Style::default();
    style.border = Border {
        left: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Color::Rgb("#FF0000".to_string()),
        }),
        right: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Color::Rgb("#FF0000".to_string()),
        }),
        top: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Color::Rgb("#FF0000".to_string()),
        }),
        bottom: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Color::Rgb("#FF0000".to_string()),
        }),
        diagonal: None,
        diagonal_up: false,
        diagonal_down: false,
    };
    model.set_cell_style(0, 2, 2, &style).unwrap();
    model.evaluate();

    let options = PdfExportOptions::default().with_gridlines(false);

    let pdf_bytes = export_to_pdf(&model, &options).expect("export merged cells with border");
    let doc = Document::load_mem(&pdf_bytes).unwrap();
    let pages = doc.get_pages();
    let page_id = *pages.get(&1).unwrap();
    let content_bytes = doc.get_page_content(page_id).unwrap();
    let content = hipdf::lopdf::content::Content::decode(&content_bytes).unwrap();

    // Verify exactly 4 thick border lines rendered around the perimeter of B2:C3 (top, bottom, left, right)
    // and 0 interior border lines between row 2 and 3 or col 2 and 3!
    let mut thick_border_lines = 0;
    for op in &content.operations {
        if op.operator == "w" {
            let width = op.operands[0]
                .as_float()
                .or_else(|_| op.operands[0].as_i64().map(|v| v as f32))
                .unwrap();
            if (width - 2.25).abs() < 0.1 {
                thick_border_lines += 1;
            }
        }
    }
    assert_eq!(
        thick_border_lines, 4,
        "Merged cell should have exactly 4 perimeter border lines (top, bottom, left, right)"
    );
}

#[test]
fn test_formula_text_and_error_modes() {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    model.set_user_input(0, 1, 1, "=1/0".to_string()).unwrap();
    model.set_user_input(0, 1, 2, "=10+20".to_string()).unwrap();
    model.evaluate();

    // Mode 1: Display formula text
    let mut options_formula = PdfExportOptions::default();
    options_formula.content.formulas = FormulaPrintPolicy::FormulaText;
    let pdf_formula = export_to_pdf(&model, &options_formula).expect("formula text export");
    assert!(pdf_formula.starts_with(b"%PDF-"));

    // Mode 2: Cell errors as dash
    let mut options_error = PdfExportOptions::default();
    options_error.content.cell_errors = CellErrorPolicy::Fixed(CellErrorMode::Dash);
    let pdf_error = export_to_pdf(&model, &options_error).expect("error as dash export");
    assert!(pdf_error.starts_with(b"%PDF-"));
}

#[test]
fn test_cell_styles_and_borders() {
    let mut model = Model::new_empty("Sheet1", "en", "UTC", "en").expect("create model");
    model
        .set_user_input(0, 1, 1, "Styled Cell".to_string())
        .unwrap();

    // Add custom style with fill, bold font, and borders
    let style = Style {
        alignment: Some(Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Center,
            wrap_text: true,
        }),
        num_fmt: "general".to_string(),
        fill: Fill {
            color: Color::Rgb("#4472C4".to_string()),
        },
        font: Font {
            strike: false,
            u: true,
            b: true,
            i: false,
            sz: 14,
            color: Color::Rgb("#FFFFFF".to_string()),
            name: "Helvetica".to_string(),
            family: 2,
            scheme: ironcalc_base::types::FontScheme::Minor,
        },
        border: Border {
            diagonal_up: false,
            diagonal_down: false,
            left: Some(BorderItem {
                style: BorderStyle::Medium,
                color: Color::Rgb("#000000".to_string()),
            }),
            right: Some(BorderItem {
                style: BorderStyle::Medium,
                color: Color::Rgb("#000000".to_string()),
            }),
            top: Some(BorderItem {
                style: BorderStyle::Thick,
                color: Color::Rgb("#000000".to_string()),
            }),
            bottom: Some(BorderItem {
                style: BorderStyle::Double,
                color: Color::Rgb("#000000".to_string()),
            }),
            diagonal: None,
        },
        quote_prefix: false,
    };

    model.set_cell_style(0, 1, 1, &style).expect("set style");
    model.evaluate();

    let options = PdfExportOptions::default().fit_to_page();
    let pdf_bytes = export_to_pdf(&model, &options).expect("styled cell export");
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_respects_document_gridlines_visibility() {
    let mut model = Model::new_empty("NoGridSheet", "en", "UTC", "en").unwrap();
    model.set_show_grid_lines(0, false).unwrap();
    model.set_user_input(0, 1, 1, "Hello".to_string()).unwrap();

    let options = PdfExportOptions::default();
    let plans = ironcalc_pdf::plan_export_layout(&model, &options).unwrap();
    assert_eq!(plans.len(), 1);
    assert!(
        !plans[0].show_gridlines,
        "Gridlines should be hidden when sheet has show_grid_lines = false"
    );

    // But if explicitly overridden with Fixed(true)
    let explicit_options = PdfExportOptions::default().with_gridlines(true);
    let explicit_plans = ironcalc_pdf::plan_export_layout(&model, &explicit_options).unwrap();
    assert!(
        explicit_plans[0].show_gridlines,
        "Gridlines should be shown when explicitly set to true"
    );
}

#[test]
fn test_pdf_export_payload_json_deserialization() {
    let json_str = r#"{
        "scope": "CurrentSheet",
        "paper_size": "Letter",
        "orientation": "Landscape",
        "scale": "FitToWidth",
        "margins": "Narrow",
        "show_grid_lines": false,
        "show_headings": true,
        "page_order": "DownThenOver",
        "horizontal_alignment": "Center",
        "vertical_alignment": "Middle",
        "page_numbers": true,
        "sheet_name": true,
        "workbook_title": true,
        "repeat_frozen_rows": true
    }"#;

    let payload: ironcalc_pdf::PdfExportOptionsPayload =
        serde_json::from_str(json_str).expect("deserialize payload");
    let opts = payload.into_options();

    assert_eq!(opts.scope, ironcalc_pdf::ExportScope::ActiveSheet);
    assert_eq!(
        opts.page.paper_size,
        ironcalc_pdf::PaperSizePolicy::Fixed(ironcalc_pdf::PaperSize::Letter)
    );
    assert_eq!(
        opts.page.orientation,
        ironcalc_pdf::OrientationPolicy::Fixed(ironcalc_pdf::Orientation::Landscape)
    );
    assert_eq!(
        opts.content.gridlines,
        ironcalc_pdf::SheetBooleanPolicy::Fixed(false)
    );
    assert_eq!(
        opts.content.row_column_headings,
        ironcalc_pdf::SheetBooleanPolicy::Fixed(true)
    );
    assert_eq!(
        opts.repeat.frozen_panes,
        ironcalc_pdf::FrozenPanePrintPolicy::RepeatAsPrintTitles
    );
    assert_eq!(
        opts.page.center_on_page,
        ironcalc_pdf::CenterOnPagePolicy::Fixed {
            horizontal: true,
            vertical: true
        }
    );
}

#[test]
fn test_pdf_export_payload_selected_cells() {
    let json_str = r#"{
        "scope": "SelectedCells",
        "sheet_index": 0,
        "selection_range": "Sheet1!B2:D5"
    }"#;

    let payload: ironcalc_pdf::PdfExportOptionsPayload =
        serde_json::from_str(json_str).expect("deserialize payload");
    let opts = payload.into_options();

    assert_eq!(
        opts.scope,
        ironcalc_pdf::ExportScope::Selection {
            sheet: 0,
            range: ironcalc_pdf::CellRange::new(2, 2, 5, 4)
        }
    );

    let model = create_sample_model();
    let pdf_bytes = ironcalc_pdf::export_to_pdf(&model, &opts).expect("export selected cells from payload");
    assert!(pdf_bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_font_family_resolution_and_text_spill() {
    let mut model = Model::new_empty("SpillSheet", "en", "UTC", "en").unwrap();
    // A long text in A1 with empty B1, C1, D1
    model
        .set_user_input(
            0,
            1,
            1,
            "This is a long header title that should spill into adjacent empty columns"
                .to_string(),
        )
        .unwrap();
    model.evaluate();

    let mut style = Style::default();
    style.font.name = "Times New Roman".to_string();
    style.font.family = 1; // Serif
    model.set_cell_style(0, 1, 1, &style).unwrap();

    let options = PdfExportOptions::default();
    let pdf_bytes = export_to_pdf(&model, &options).expect("export text spill with Times");
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    // Check that Times font is embedded in PDF
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(
        pdf_str.contains("Times"),
        "Expected Times font resource to be embedded in PDF"
    );
}

#[test]
fn test_export_excel_fixtures_multiple_fonts() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 1. Fixture 1: libreoffice_888_example.xlsx
    // Contains cells formatted with both 'Times New Roman' (Serif) and 'Arial' (Sans-Serif)
    let fixture1 = manifest_dir.join("../xlsx/tests/libreoffice_888_example.xlsx");
    let model1 = ironcalc::import::load_from_xlsx(fixture1.to_str().unwrap(), "en", "UTC", "en")
        .expect("load libreoffice_888_example.xlsx");

    let options = PdfExportOptions::default().fit_to_page();
    let pdf1 = export_to_pdf(&model1, &options).expect("export libreoffice_888_example to pdf");
    assert!(pdf1.starts_with(b"%PDF-"));

    let out_dir = manifest_dir.join("tests/output");
    let _ = std::fs::create_dir_all(&out_dir);
    std::fs::write(out_dir.join("multiple_fonts_libreoffice.pdf"), &pdf1)
        .expect("write multiple_fonts_libreoffice.pdf");

    let doc1 = Document::load_mem(&pdf1).expect("parse doc1");
    let pages1 = doc1.get_pages();
    assert!(!pages1.is_empty(), "Exported PDF must have pages");

    let page1_id = *pages1.get(&1).unwrap();
    let content1_bytes = doc1.get_page_content(page1_id).unwrap();
    let content1 = hipdf::lopdf::content::Content::decode(&content1_bytes).unwrap();

    // Verify multiple fonts used via Tf operations (Times for Times New Roman and Helvetica for Arial)
    let mut fonts_used_1 = std::collections::HashSet::new();
    for op in &content1.operations {
        if op.operator == "Tf" && !op.operands.is_empty() {
            if let hipdf::lopdf::Object::Name(name_bytes) = &op.operands[0] {
                fonts_used_1.insert(String::from_utf8_lossy(name_bytes).to_string());
            }
        }
    }
    assert!(
        fonts_used_1.len() >= 2,
        "libreoffice_888_example should render with at least 2 distinct font resources, got {:?}",
        fonts_used_1
    );

    // 2. Fixture 2: LOGICAL/IFERROR.xlsx
    // Contains cells formatted with both 'Aptos Narrow' (Sans-serif) and 'Consolas' (Monospace)
    let fixture2 = manifest_dir.join("../xlsx/tests/calc_tests/LOGICAL/IFERROR.xlsx");
    let model2 = ironcalc::import::load_from_xlsx(fixture2.to_str().unwrap(), "en", "UTC", "en")
        .expect("load IFERROR.xlsx");

    let pdf2 = export_to_pdf(&model2, &options).expect("export IFERROR to pdf");
    assert!(pdf2.starts_with(b"%PDF-"));

    std::fs::write(out_dir.join("multiple_fonts_iferror.pdf"), &pdf2)
        .expect("write multiple_fonts_iferror.pdf");

    let doc2 = Document::load_mem(&pdf2).expect("parse doc2");
    let pages2 = doc2.get_pages();
    let page2_id = *pages2.get(&1).unwrap();
    let content2_bytes = doc2.get_page_content(page2_id).unwrap();
    let content2 = hipdf::lopdf::content::Content::decode(&content2_bytes).unwrap();

    let mut fonts_used_2 = std::collections::HashSet::new();
    for op in &content2.operations {
        if op.operator == "Tf" && !op.operands.is_empty() {
            if let hipdf::lopdf::Object::Name(name_bytes) = &op.operands[0] {
                fonts_used_2.insert(String::from_utf8_lossy(name_bytes).to_string());
            }
        }
    }
    assert!(
        fonts_used_2.len() >= 2,
        "IFERROR.xlsx should render with at least 2 distinct font resources (Helvetica & Courier), got {:?}",
        fonts_used_2
    );
}

#[test]
fn test_calculation_and_formula_errors_printed_on_pdf() {
    let mut model = Model::new_empty("ErrorsSheet", "en", "UTC", "en").unwrap();
    // Setup various formula calculation errors
    model.set_user_input(0, 1, 1, "=100/0".to_string()).unwrap(); // #DIV/0!
    model.set_user_input(0, 2, 1, "=1+\"invalid\"".to_string()).unwrap(); // #VALUE!
    model.set_user_input(0, 3, 1, "=NA()".to_string()).unwrap(); // #N/A
    model.set_user_input(0, 4, 1, "=SQRT(-1)".to_string()).unwrap(); // #NUM!
    model.set_user_input(0, 5, 1, "=NONEXISTENT_FN()".to_string()).unwrap(); // #NAME?
    model.evaluate();

    // Verify cell evaluations
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#DIV/0!");
    assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "#VALUE!");
    assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "#N/A");
    assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "#NUM!");
    assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "#NAME?");

    // 1. Default mode: DisplayAsStored -> must print all error strings into the PDF
    let options = PdfExportOptions::default().fit_to_page();
    let pdf_bytes = export_to_pdf(&model, &options).expect("export errors to pdf");
    assert!(pdf_bytes.starts_with(b"%PDF-"));

    let doc = Document::load_mem(&pdf_bytes).unwrap();
    let page_id = *doc.get_pages().get(&1).unwrap();
    let content_bytes = doc.get_page_content(page_id).unwrap();
    let content = hipdf::lopdf::content::Content::decode(&content_bytes).unwrap();

    let mut printed_strings = Vec::new();
    for op in &content.operations {
        if op.operator == "Tj" {
            if let hipdf::lopdf::Object::String(bytes, _) = &op.operands[0] {
                printed_strings.push(String::from_utf8_lossy(bytes).to_string());
            }
        }
    }

    assert!(
        printed_strings.iter().any(|s| s.contains("#DIV/0!")),
        "Expected #DIV/0! to be printed on PDF, got {printed_strings:?}"
    );
    assert!(
        printed_strings.iter().any(|s| s.contains("#VALUE!")),
        "Expected #VALUE! to be printed on PDF, got {printed_strings:?}"
    );
    assert!(
        printed_strings.iter().any(|s| s.contains("#N/A")),
        "Expected #N/A to be printed on PDF, got {printed_strings:?}"
    );
    assert!(
        printed_strings.iter().any(|s| s.contains("#NUM!")),
        "Expected #NUM! to be printed on PDF, got {printed_strings:?}"
    );
    assert!(
        printed_strings.iter().any(|s| s.contains("#NAME?")),
        "Expected #NAME? to be printed on PDF, got {printed_strings:?}"
    );

    // 2. Dash mode: with_cell_errors(CellErrorMode::Dash) -> prints "--"
    let options_dash = PdfExportOptions::default()
        .with_cell_errors(CellErrorMode::Dash)
        .fit_to_page();
    let pdf_dash = export_to_pdf(&model, &options_dash).expect("export dash errors");
    let doc_dash = Document::load_mem(&pdf_dash).unwrap();
    let page_dash_id = *doc_dash.get_pages().get(&1).unwrap();
    let content_dash = hipdf::lopdf::content::Content::decode(
        &doc_dash.get_page_content(page_dash_id).unwrap(),
    )
    .unwrap();
    let mut dash_strings = Vec::new();
    for op in &content_dash.operations {
        if op.operator == "Tj" {
            if let hipdf::lopdf::Object::String(bytes, _) = &op.operands[0] {
                dash_strings.push(String::from_utf8_lossy(bytes).to_string());
            }
        }
    }
    assert!(
        dash_strings.iter().filter(|s| s.as_str() == "--").count() >= 5,
        "Expected at least 5 dashes printed for the 5 error cells, got {dash_strings:?}"
    );
}

