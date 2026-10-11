//! Rendering engine producing PDF documents.
//!
//! Transforms planned pages into PDF vector graphics, text, gridlines,
//! borders, styles, headers, and footers.

use std::collections::HashMap;

use hipdf::fonts::{Font, FontManager, StandardFont, TextBuilder};
use hipdf::lopdf::content::{Content, Operation};
use hipdf::lopdf::{dictionary, Document, Object, ObjectId, Stream};
use hipdf::ocg::{Layer, OCGManager};
use ironcalc_base::expressions::utils::number_to_column;
use ironcalc_base::types::{
    BorderStyle, CellType, HorizontalAlignment, MergedCell, VerticalAlignment,
};
use ironcalc_base::Model;

use crate::error::PdfError;
use crate::layout::PagePlan;
use crate::options::{
    BookmarkPolicy, CellErrorMode, CellErrorPolicy, FormulaPrintPolicy, PdfExportOptions,
};

pub fn render_pdf(
    model: &Model,
    options: &PdfExportOptions,
    plans: &[PagePlan],
) -> Result<Vec<u8>, PdfError> {
    if plans.is_empty() {
        return Err(PdfError::IronCalc("No pages to render".to_string()));
    }

    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    // Initialize font catalog with standard 12 fonts + any custom TrueType/OpenType fonts
    let font_catalog = FontCatalog::init(&mut doc, &mut font_manager, &options.custom_fonts)?;

    // Setup Optional Content Groups (layers)
    let mut ocg_manager = OCGManager::with_config(Default::default());
    let _layer_grid = ocg_manager.add_layer(Layer::new("Gridlines", true));
    let _layer_headings = ocg_manager.add_layer(Layer::new("Row & Column Headings", true));
    let _layer_content = ocg_manager.add_layer(Layer::new("Spreadsheet Content", true));
    let _layer_header_footer = ocg_manager.add_layer(Layer::new("Headers & Footers", true));
    ocg_manager.initialize(&mut doc);

    // Reserve pages object ID
    let pages_id = doc.new_object_id();

    let mut page_ids = Vec::with_capacity(plans.len());
    let mut sheet_first_page: Vec<(String, ObjectId)> = Vec::new();
    let mut current_sheet_idx: Option<u32> = None;

    for plan in plans {
        let mut resources = dictionary! {};

        // Add font resources to page dictionary
        font_catalog.add_to_resources(&mut font_manager, &mut resources);

        // Setup OCG layer tags in resources
        let layer_tags = ocg_manager.setup_page_resources(&mut resources);

        let grid_tag = layer_tags
            .get("Gridlines")
            .cloned()
            .unwrap_or_else(|| "L0".to_string());
        let headings_tag = layer_tags
            .get("Row & Column Headings")
            .cloned()
            .unwrap_or_else(|| "L1".to_string());
        let content_tag = layer_tags
            .get("Spreadsheet Content")
            .cloned()
            .unwrap_or_else(|| "L2".to_string());
        let hf_tag = layer_tags
            .get("Headers & Footers")
            .cloned()
            .unwrap_or_else(|| "L3".to_string());

        // Render page graphics and text operations
        let mut ops = Vec::new();

        // 1. Spreadsheet Content Layer (Cell backgrounds, values, and borders)
        ops.push(Operation::new(
            "BDC",
            vec![
                Object::Name(b"OC".to_vec()),
                Object::Name(content_tag.as_bytes().to_vec()),
            ],
        ));
        render_cell_backgrounds(model, plan, &mut ops);
        ops.push(Operation::new("EMC", vec![]));

        // 2. Gridlines Layer
        if plan.show_gridlines {
            ops.push(Operation::new(
                "BDC",
                vec![
                    Object::Name(b"OC".to_vec()),
                    Object::Name(grid_tag.as_bytes().to_vec()),
                ],
            ));
            render_gridlines(plan, &mut ops);
            ops.push(Operation::new("EMC", vec![]));
        }

        // 3. Cell Borders & Text (part of Content layer)
        ops.push(Operation::new(
            "BDC",
            vec![
                Object::Name(b"OC".to_vec()),
                Object::Name(content_tag.as_bytes().to_vec()),
            ],
        ));
        render_cell_borders(model, plan, &mut ops);
        render_cell_texts(model, options, plan, &font_catalog, &mut ops);
        ops.push(Operation::new("EMC", vec![]));

        // 4. Headings Layer (Row numbers and column letters)
        if plan.show_headings {
            ops.push(Operation::new(
                "BDC",
                vec![
                    Object::Name(b"OC".to_vec()),
                    Object::Name(headings_tag.as_bytes().to_vec()),
                ],
            ));
            let h_reg = &font_catalog.entries[font_catalog.helvetica_reg];
            let h_bold = &font_catalog.entries[font_catalog.helvetica_bold];
            render_headings(
                plan,
                &h_reg.font,
                &h_bold.font,
                &h_reg.resource_name,
                &h_bold.resource_name,
                &mut ops,
            );
            ops.push(Operation::new("EMC", vec![]));
        }

        // 5. Headers & Footers Layer
        ops.push(Operation::new(
            "BDC",
            vec![
                Object::Name(b"OC".to_vec()),
                Object::Name(hf_tag.as_bytes().to_vec()),
            ],
        ));
        let h_reg = &font_catalog.entries[font_catalog.helvetica_reg];
        render_headers_and_footers(plan, &h_reg.font, &h_reg.resource_name, &mut ops);
        ops.push(Operation::new("EMC", vec![]));

        // Create Content stream
        let content = Content { operations: ops };
        let encoded_bytes = content
            .encode()
            .map_err(|e| PdfError::Pdf(format!("Failed to encode PDF stream: {e}")))?;
        let content_stream = Stream::new(dictionary! {}, encoded_bytes);
        let content_id = doc.add_object(content_stream);

        // Create Page Object
        let page_dict = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), plan.page_width.into(), plan.page_height.into()],
            "Contents" => content_id,
            "Resources" => resources,
        };
        let page_id = doc.add_object(page_dict);
        page_ids.push(page_id);

        // Record first page of sheet for bookmarks
        if current_sheet_idx != Some(plan.sheet_index) {
            current_sheet_idx = Some(plan.sheet_index);
            sheet_first_page.push((plan.sheet_name.clone(), page_id));
        }
    }

    // Set Pages dictionary
    let pages_dict = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.iter().map(|id| Object::Reference(*id)).collect::<Vec<_>>(),
        "Count" => page_ids.len() as i32,
    };
    doc.set_object(pages_id, Object::Dictionary(pages_dict));

    // Create Outlines (Bookmarks) if requested
    let mut catalog_dict = dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    };

    let make_bookmarks = match options.pdf.bookmarks {
        BookmarkPolicy::None => false,
        BookmarkPolicy::AlwaysBySheet => true,
        BookmarkPolicy::SheetBookmarksWhenExportingMultipleSheets => sheet_first_page.len() > 1,
    };

    if make_bookmarks && !sheet_first_page.is_empty() {
        let outlines_id = doc.new_object_id();
        let mut outline_item_ids = Vec::with_capacity(sheet_first_page.len());

        for (sheet_name, target_page_id) in &sheet_first_page {
            let item_id = doc.new_object_id();
            outline_item_ids.push((item_id, sheet_name.clone(), *target_page_id));
        }

        let num_items = outline_item_ids.len();
        for i in 0..num_items {
            let (item_id, ref title, target_page) = outline_item_ids[i];
            let prev = if i > 0 {
                Some(outline_item_ids[i - 1].0)
            } else {
                None
            };
            let next = if i + 1 < num_items {
                Some(outline_item_ids[i + 1].0)
            } else {
                None
            };

            let mut item_dict = dictionary! {
                "Title" => Object::string_literal(title.as_bytes().to_vec()),
                "Parent" => Object::Reference(outlines_id),
                "Dest" => vec![Object::Reference(target_page), Object::Name(b"XYZ".to_vec()), Object::Null, Object::Null, Object::Null],
            };
            if let Some(p) = prev {
                item_dict.set("Prev", Object::Reference(p));
            }
            if let Some(n) = next {
                item_dict.set("Next", Object::Reference(n));
            }
            doc.set_object(item_id, Object::Dictionary(item_dict));
        }

        let first_id = outline_item_ids[0].0;
        let last_id = outline_item_ids[num_items - 1].0;

        let outlines_dict = dictionary! {
            "Type" => "Outlines",
            "Count" => num_items as i32,
            "First" => Object::Reference(first_id),
            "Last" => Object::Reference(last_id),
        };
        doc.set_object(outlines_id, Object::Dictionary(outlines_dict));
        catalog_dict.set("Outlines", Object::Reference(outlines_id));
    }

    let catalog_id = doc.add_object(catalog_dict);
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Update Catalog with OCG properties
    ocg_manager.update_catalog(&mut doc);

    // Save document to buffer
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|e| PdfError::Pdf(format!("Failed to serialize PDF: {e}")))?;

    Ok(buffer)
}

/*
   Rendering Helpers
*/

fn parse_hex_color(hex: &str) -> Option<(f32, f32, f32)> {
    let clean = hex.trim().strip_prefix('#').unwrap_or(hex.trim());
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
}

fn sanitize_winansi_text(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for ch in text.chars() {
        let code = ch as u32;
        if code <= 0x7F {
            out.push(code as u8);
        } else {
            // Map common Unicode characters to WinAnsi (Windows-1252)
            let winansi = match ch {
                '€' => 0x80,
                '‚' => 0x82,
                'ƒ' => 0x83,
                '„' => 0x84,
                '…' => 0x85,
                '†' => 0x86,
                '‡' => 0x87,
                'ˆ' => 0x88,
                '‰' => 0x89,
                'Š' => 0x8A,
                '‹' => 0x8B,
                'Œ' => 0x8C,
                'Ž' => 0x8E,
                '‘' => 0x91,
                '’' => 0x92,
                '“' => 0x93,
                '”' => 0x94,
                '•' => 0x95,
                '–' => 0x96,
                '—' => 0x97,
                '˜' => 0x98,
                '™' => 0x99,
                'š' => 0x9A,
                '›' => 0x9B,
                'œ' => 0x9C,
                'ž' => 0x9E,
                'Ÿ' => 0x9F,
                _ if (0xA0..=0xFF).contains(&code) => code as u8,
                _ => b'?',
            };
            out.push(winansi);
        }
    }
    out
}

fn render_cell_backgrounds(model: &Model, plan: &PagePlan, ops: &mut Vec<Operation>) {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();

    let mut rendered_merges = HashMap::new();

    for &row in &all_rows {
        for &col in &all_cols {
            // Check if cell is covered by a merge
            if let Some(merge) = find_merge(row, col, &plan.merged_cells) {
                if rendered_merges.contains_key(&(merge.row, merge.column)) {
                    continue; // Already painted on this page
                }
                rendered_merges.insert((merge.row, merge.column), true);

                if let Some((x, y, w, h)) = get_merged_rect_on_page(merge, plan) {
                    if let Ok(style) =
                        model.get_style_for_cell(plan.sheet_index, merge.row, merge.column)
                    {
                        let hex = style.fill.color.to_rgb(&model.workbook.theme);
                        if let Some((r, g, b)) = parse_hex_color(&hex) {
                            let pdf_y = plan.page_height - y - h;
                            ops.push(Operation::new("q", vec![]));
                            ops.push(Operation::new("rg", vec![r.into(), g.into(), b.into()]));
                            ops.push(Operation::new(
                                "re",
                                vec![x.into(), pdf_y.into(), w.into(), h.into()],
                            ));
                            ops.push(Operation::new("f", vec![]));
                            ops.push(Operation::new("Q", vec![]));
                        }
                    }
                }
                continue;
            }

            // Normal cell
            if let (Some(x), Some(y)) = (plan.column_x(col), plan.row_y(row)) {
                let w = plan.col_widths.get(&col).copied().unwrap_or(0.0);
                let h = plan.row_heights.get(&row).copied().unwrap_or(0.0);
                if w <= 0.0 || h <= 0.0 {
                    continue;
                }

                if let Ok(style) = model.get_style_for_cell(plan.sheet_index, row, col) {
                    let hex = style.fill.color.to_rgb(&model.workbook.theme);
                    if let Some((r, g, b)) = parse_hex_color(&hex) {
                        let pdf_y = plan.page_height - y - h;
                        ops.push(Operation::new("q", vec![]));
                        ops.push(Operation::new("rg", vec![r.into(), g.into(), b.into()]));
                        ops.push(Operation::new(
                            "re",
                            vec![x.into(), pdf_y.into(), w.into(), h.into()],
                        ));
                        ops.push(Operation::new("f", vec![]));
                        ops.push(Operation::new("Q", vec![]));
                    }
                }
            }
        }
    }
}

fn render_gridlines(plan: &PagePlan, ops: &mut Vec<Operation>) {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();
    if all_cols.is_empty() || all_rows.is_empty() {
        return;
    }

    let mut start_x = plan.content_x;
    if plan.show_headings {
        start_x += plan.heading_col_width;
    }
    let mut start_y = plan.content_y;
    if plan.show_headings {
        start_y += plan.heading_row_height;
    }

    let total_table_w: f32 = all_cols.iter().filter_map(|c| plan.col_widths.get(c)).sum();
    let total_table_h: f32 = all_rows
        .iter()
        .filter_map(|r| plan.row_heights.get(r))
        .sum();
    let end_x = start_x + total_table_w;
    let end_y = start_y + total_table_h;

    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("w", vec![0.5.into()]));
    ops.push(Operation::new(
        "RG",
        vec![0.83.into(), 0.83.into(), 0.83.into()],
    )); // #D4D4D4

    // Horizontal grid lines
    let mut current_y = start_y;
    for (row_idx, &row) in all_rows.iter().enumerate() {
        let pdf_y = plan.page_height - current_y;

        let mut segment_start: Option<f32> = None;
        let mut segment_end: f32 = start_x;

        for &col in &all_cols {
            let cw = plan.col_widths.get(&col).copied().unwrap_or(0.0);
            if cw <= 0.0 {
                continue;
            }
            let col_x = plan.column_x(col).unwrap_or(segment_end);
            let is_interior = if row_idx == 0 {
                false
            } else {
                let row_above = all_rows[row_idx - 1];
                let row_below = row;
                let m_above = find_merge(row_above, col, &plan.merged_cells);
                let m_below = find_merge(row_below, col, &plan.merged_cells);
                m_above.is_some() && m_above == m_below
            };

            if !is_interior {
                if segment_start.is_none() {
                    segment_start = Some(col_x);
                }
                segment_end = col_x + cw;
            } else if let Some(sx) = segment_start.take() {
                ops.push(Operation::new("m", vec![sx.into(), pdf_y.into()]));
                ops.push(Operation::new("l", vec![segment_end.into(), pdf_y.into()]));
                ops.push(Operation::new("S", vec![]));
            }
        }
        if let Some(sx) = segment_start.take() {
            ops.push(Operation::new("m", vec![sx.into(), pdf_y.into()]));
            ops.push(Operation::new("l", vec![segment_end.into(), pdf_y.into()]));
            ops.push(Operation::new("S", vec![]));
        }

        if let Some(rh) = plan.row_heights.get(&row) {
            current_y += *rh;
        }
    }
    // Bottom border line
    let pdf_y_bottom = plan.page_height - end_y;
    ops.push(Operation::new(
        "m",
        vec![start_x.into(), pdf_y_bottom.into()],
    ));
    ops.push(Operation::new("l", vec![end_x.into(), pdf_y_bottom.into()]));
    ops.push(Operation::new("S", vec![]));

    // Vertical grid lines
    let mut current_x = start_x;
    for (col_idx, &col) in all_cols.iter().enumerate() {
        let mut segment_start: Option<f32> = None;
        let mut segment_end: f32 = start_y;

        for &row in &all_rows {
            let rh = plan.row_heights.get(&row).copied().unwrap_or(0.0);
            if rh <= 0.0 {
                continue;
            }
            let row_y = plan.row_y(row).unwrap_or(segment_end);
            let is_interior = if col_idx == 0 {
                false
            } else {
                let col_left = all_cols[col_idx - 1];
                let col_right = col;
                let m_left = find_merge(row, col_left, &plan.merged_cells);
                let m_right = find_merge(row, col_right, &plan.merged_cells);
                m_left.is_some() && m_left == m_right
            };

            if !is_interior {
                if segment_start.is_none() {
                    segment_start = Some(row_y);
                }
                segment_end = row_y + rh;
            } else if let Some(sy) = segment_start.take() {
                let pdf_y1 = plan.page_height - sy;
                let pdf_y2 = plan.page_height - segment_end;
                ops.push(Operation::new("m", vec![current_x.into(), pdf_y1.into()]));
                ops.push(Operation::new("l", vec![current_x.into(), pdf_y2.into()]));
                ops.push(Operation::new("S", vec![]));
            }
        }
        if let Some(sy) = segment_start.take() {
            let pdf_y1 = plan.page_height - sy;
            let pdf_y2 = plan.page_height - segment_end;
            ops.push(Operation::new("m", vec![current_x.into(), pdf_y1.into()]));
            ops.push(Operation::new("l", vec![current_x.into(), pdf_y2.into()]));
            ops.push(Operation::new("S", vec![]));
        }

        if let Some(cw) = plan.col_widths.get(&col) {
            current_x += *cw;
        }
    }
    // Right border line
    let pdf_y1 = plan.page_height - start_y;
    let pdf_y2 = plan.page_height - end_y;
    ops.push(Operation::new("m", vec![end_x.into(), pdf_y1.into()]));
    ops.push(Operation::new("l", vec![end_x.into(), pdf_y2.into()]));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("Q", vec![]));
}

fn render_cell_borders(model: &Model, plan: &PagePlan, ops: &mut Vec<Operation>) {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();

    for &row in &all_rows {
        for &col in &all_cols {
            // If this cell is part of a merge, its borders are handled separately as a unified rectangle
            if find_merge(row, col, &plan.merged_cells).is_some() {
                continue;
            }

            let (Some(x), Some(y)) = (plan.column_x(col), plan.row_y(row)) else {
                continue;
            };
            let w = plan.col_widths.get(&col).copied().unwrap_or(0.0);
            let h = plan.row_heights.get(&row).copied().unwrap_or(0.0);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }

            let Ok(style) = model.get_style_for_cell(plan.sheet_index, row, col) else {
                continue;
            };

            // Top Border
            if let Some(ref top) = style.border.top {
                draw_border_line(
                    ops,
                    top.style.clone(),
                    top.color.to_rgb(&model.workbook.theme).as_str(),
                    x,
                    plan.page_height - y,
                    x + w,
                    plan.page_height - y,
                );
            }
            // Bottom Border
            if let Some(ref bottom) = style.border.bottom {
                draw_border_line(
                    ops,
                    bottom.style.clone(),
                    bottom.color.to_rgb(&model.workbook.theme).as_str(),
                    x,
                    plan.page_height - (y + h),
                    x + w,
                    plan.page_height - (y + h),
                );
            }
            // Left Border
            if let Some(ref left) = style.border.left {
                draw_border_line(
                    ops,
                    left.style.clone(),
                    left.color.to_rgb(&model.workbook.theme).as_str(),
                    x,
                    plan.page_height - y,
                    x,
                    plan.page_height - (y + h),
                );
            }
            // Right Border
            if let Some(ref right) = style.border.right {
                draw_border_line(
                    ops,
                    right.style.clone(),
                    right.color.to_rgb(&model.workbook.theme).as_str(),
                    x + w,
                    plan.page_height - y,
                    x + w,
                    plan.page_height - (y + h),
                );
            }
        }
    }

    // Now render perimeter borders for merged cells as unified rectangles
    let mut rendered_merges = HashMap::new();
    for merge in &plan.merged_cells {
        if rendered_merges.contains_key(&(merge.row, merge.column)) {
            continue;
        }
        rendered_merges.insert((merge.row, merge.column), true);

        let Some((x, y, w, h)) = get_merged_rect_on_page(merge, plan) else {
            continue;
        };

        let page_cols: Vec<i32> = all_cols
            .iter()
            .copied()
            .filter(|&c| c >= merge.column && c <= merge.last_column())
            .collect();
        let page_rows: Vec<i32> = all_rows
            .iter()
            .copied()
            .filter(|&r| r >= merge.row && r <= merge.last_row())
            .collect();

        let anchor_style = model
            .get_style_for_cell(plan.sheet_index, merge.row, merge.column)
            .ok();

        // 1. Top border
        if all_rows.contains(&merge.row) {
            let top_y_pdf = plan.page_height - y;
            let mut current_border: Option<(BorderStyle, String)> = None;
            let mut seg_start = x;
            let mut seg_end = x;

            for &c in &page_cols {
                let cw = plan.col_widths.get(&c).copied().unwrap_or(0.0);
                if cw <= 0.0 {
                    continue;
                }
                let cx = plan.column_x(c).unwrap_or(seg_end);
                let border = model
                    .get_style_for_cell(plan.sheet_index, merge.row, c)
                    .ok()
                    .and_then(|s| s.border.top)
                    .or_else(|| anchor_style.as_ref().and_then(|s| s.border.top.clone()));

                let b_key = border.map(|b| (b.style, b.color.to_rgb(&model.workbook.theme)));
                if b_key == current_border {
                    seg_end = cx + cw;
                } else {
                    if let Some((b_style, ref b_color)) = current_border {
                        draw_border_line(
                            ops, b_style, b_color, seg_start, top_y_pdf, seg_end, top_y_pdf,
                        );
                    }
                    current_border = b_key;
                    seg_start = cx;
                    seg_end = cx + cw;
                }
            }
            if let Some((b_style, ref b_color)) = current_border {
                draw_border_line(
                    ops, b_style, b_color, seg_start, top_y_pdf, seg_end, top_y_pdf,
                );
            }
        }

        // 2. Bottom border
        let last_row = merge.last_row();
        if all_rows.contains(&last_row) {
            let bottom_y_pdf = plan.page_height - (y + h);
            let mut current_border: Option<(BorderStyle, String)> = None;
            let mut seg_start = x;
            let mut seg_end = x;

            for &c in &page_cols {
                let cw = plan.col_widths.get(&c).copied().unwrap_or(0.0);
                if cw <= 0.0 {
                    continue;
                }
                let cx = plan.column_x(c).unwrap_or(seg_end);
                let border = model
                    .get_style_for_cell(plan.sheet_index, last_row, c)
                    .ok()
                    .and_then(|s| s.border.bottom)
                    .or_else(|| anchor_style.as_ref().and_then(|s| s.border.bottom.clone()));

                let b_key = border.map(|b| (b.style, b.color.to_rgb(&model.workbook.theme)));
                if b_key == current_border {
                    seg_end = cx + cw;
                } else {
                    if let Some((b_style, ref b_color)) = current_border {
                        draw_border_line(
                            ops, b_style, b_color, seg_start, bottom_y_pdf, seg_end, bottom_y_pdf,
                        );
                    }
                    current_border = b_key;
                    seg_start = cx;
                    seg_end = cx + cw;
                }
            }
            if let Some((b_style, ref b_color)) = current_border {
                draw_border_line(
                    ops, b_style, b_color, seg_start, bottom_y_pdf, seg_end, bottom_y_pdf,
                );
            }
        }

        // 3. Left border
        if all_cols.contains(&merge.column) {
            let mut current_border: Option<(BorderStyle, String)> = None;
            let mut seg_start_y = y;
            let mut seg_end_y = y;

            for &r in &page_rows {
                let rh = plan.row_heights.get(&r).copied().unwrap_or(0.0);
                if rh <= 0.0 {
                    continue;
                }
                let ry = plan.row_y(r).unwrap_or(seg_end_y);
                let border = model
                    .get_style_for_cell(plan.sheet_index, r, merge.column)
                    .ok()
                    .and_then(|s| s.border.left)
                    .or_else(|| anchor_style.as_ref().and_then(|s| s.border.left.clone()));

                let b_key = border.map(|b| (b.style, b.color.to_rgb(&model.workbook.theme)));
                if b_key == current_border {
                    seg_end_y = ry + rh;
                } else {
                    if let Some((b_style, ref b_color)) = current_border {
                        let y1 = plan.page_height - seg_start_y;
                        let y2 = plan.page_height - seg_end_y;
                        draw_border_line(ops, b_style, b_color, x, y1, x, y2);
                    }
                    current_border = b_key;
                    seg_start_y = ry;
                    seg_end_y = ry + rh;
                }
            }
            if let Some((b_style, ref b_color)) = current_border {
                let y1 = plan.page_height - seg_start_y;
                let y2 = plan.page_height - seg_end_y;
                draw_border_line(ops, b_style, b_color, x, y1, x, y2);
            }
        }

        // 4. Right border
        let last_col = merge.last_column();
        if all_cols.contains(&last_col) {
            let right_x = x + w;
            let mut current_border: Option<(BorderStyle, String)> = None;
            let mut seg_start_y = y;
            let mut seg_end_y = y;

            for &r in &page_rows {
                let rh = plan.row_heights.get(&r).copied().unwrap_or(0.0);
                if rh <= 0.0 {
                    continue;
                }
                let ry = plan.row_y(r).unwrap_or(seg_end_y);
                let border = model
                    .get_style_for_cell(plan.sheet_index, r, last_col)
                    .ok()
                    .and_then(|s| s.border.right)
                    .or_else(|| anchor_style.as_ref().and_then(|s| s.border.right.clone()));

                let b_key = border.map(|b| (b.style, b.color.to_rgb(&model.workbook.theme)));
                if b_key == current_border {
                    seg_end_y = ry + rh;
                } else {
                    if let Some((b_style, ref b_color)) = current_border {
                        let y1 = plan.page_height - seg_start_y;
                        let y2 = plan.page_height - seg_end_y;
                        draw_border_line(ops, b_style, b_color, right_x, y1, right_x, y2);
                    }
                    current_border = b_key;
                    seg_start_y = ry;
                    seg_end_y = ry + rh;
                }
            }
            if let Some((b_style, ref b_color)) = current_border {
                let y1 = plan.page_height - seg_start_y;
                let y2 = plan.page_height - seg_end_y;
                draw_border_line(ops, b_style, b_color, right_x, y1, right_x, y2);
            }
        }
    }
}

fn draw_border_line(
    ops: &mut Vec<Operation>,
    style: BorderStyle,
    color_hex: &str,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) {
    let (r, g, b) = parse_hex_color(color_hex).unwrap_or((0.0, 0.0, 0.0));
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new("RG", vec![r.into(), g.into(), b.into()]));

    let (width, dash): (f32, Option<Vec<Object>>) = match style {
        BorderStyle::Thin => (0.75, None),
        BorderStyle::Medium => (1.5, None),
        BorderStyle::Thick => (2.25, None),
        BorderStyle::Double => (0.75, None), // simplified double
        BorderStyle::Dotted => (0.75, Some(vec![1.into(), 2.into()])),
        BorderStyle::MediumDashed | BorderStyle::SlantDashDot => {
            (1.5, Some(vec![4.into(), 2.into()]))
        }
        BorderStyle::MediumDashDot => (1.5, Some(vec![4.into(), 2.into(), 1.into(), 2.into()])),
        BorderStyle::MediumDashDotDot => (
            1.5,
            Some(vec![
                4.into(),
                2.into(),
                1.into(),
                2.into(),
                1.into(),
                2.into(),
            ]),
        ),
    };

    ops.push(Operation::new("w", vec![width.into()]));
    if let Some(d) = dash {
        ops.push(Operation::new("d", vec![Object::Array(d), 0.into()]));
    }

    ops.push(Operation::new("m", vec![x1.into(), y1.into()]));
    ops.push(Operation::new("l", vec![x2.into(), y2.into()]));
    ops.push(Operation::new("S", vec![]));

    ops.push(Operation::new("Q", vec![]));
}

struct FontEntry {
    font: Font,
    resource_name: String,
    obj_id: ObjectId,
    is_bold: bool,
    is_italic: bool,
    family_normalized: String,
}

struct FontCatalog {
    entries: Vec<FontEntry>,
    helvetica_reg: usize,
    helvetica_bold: usize,
    helvetica_italic: usize,
    helvetica_bi: usize,
    times_reg: usize,
    times_bold: usize,
    times_italic: usize,
    times_bi: usize,
    courier_reg: usize,
    courier_bold: usize,
    courier_italic: usize,
    courier_bi: usize,
}

fn normalize_font_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn is_monospace_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("mono")
        || lower.contains("courier")
        || lower.contains("consolas")
        || lower.contains("menlo")
        || lower.contains("code")
}

fn is_serif_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("serif")
        || lower.contains("times")
        || lower.contains("georgia")
        || lower.contains("cambria")
        || lower.contains("garamond")
        || lower.contains("palatino")
        || lower.contains("bookman")
}

impl FontCatalog {
    pub fn init(
        doc: &mut Document,
        font_manager: &mut FontManager,
        custom_fonts: &[crate::options::CustomFontData],
    ) -> Result<Self, PdfError> {
        let mut entries = Vec::new();

        // 1. Standard Helvetica
        let h_reg = Font::standard(StandardFont::Helvetica);
        let h_bold = Font::standard(StandardFont::HelveticaBold);
        let h_italic = Font::standard(StandardFont::HelveticaOblique);
        let h_bi = Font::standard(StandardFont::HelveticaBoldOblique);

        let (h_reg_id, h_reg_res) = font_manager
            .embed_font(doc, h_reg.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed Helvetica: {e}")))?;
        let helvetica_reg = entries.len();
        entries.push(FontEntry {
            font: h_reg,
            resource_name: h_reg_res,
            obj_id: h_reg_id,
            is_bold: false,
            is_italic: false,
            family_normalized: "helvetica".to_string(),
        });

        let (h_bold_id, h_bold_res) = font_manager
            .embed_font(doc, h_bold.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed HelveticaBold: {e}")))?;
        let helvetica_bold = entries.len();
        entries.push(FontEntry {
            font: h_bold,
            resource_name: h_bold_res,
            obj_id: h_bold_id,
            is_bold: true,
            is_italic: false,
            family_normalized: "helvetica".to_string(),
        });

        let (h_it_id, h_it_res) = font_manager
            .embed_font(doc, h_italic.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed HelveticaOblique: {e}")))?;
        let helvetica_italic = entries.len();
        entries.push(FontEntry {
            font: h_italic,
            resource_name: h_it_res,
            obj_id: h_it_id,
            is_bold: false,
            is_italic: true,
            family_normalized: "helvetica".to_string(),
        });

        let (h_bi_id, h_bi_res) = font_manager
            .embed_font(doc, h_bi.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed HelveticaBoldOblique: {e}")))?;
        let helvetica_bi = entries.len();
        entries.push(FontEntry {
            font: h_bi,
            resource_name: h_bi_res,
            obj_id: h_bi_id,
            is_bold: true,
            is_italic: true,
            family_normalized: "helvetica".to_string(),
        });

        // 2. Standard Times
        let t_reg = Font::standard(StandardFont::TimesRoman);
        let t_bold = Font::standard(StandardFont::TimesBold);
        let t_italic = Font::standard(StandardFont::TimesItalic);
        let t_bi = Font::standard(StandardFont::TimesBoldItalic);

        let (t_reg_id, t_reg_res) = font_manager
            .embed_font(doc, t_reg.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed Times: {e}")))?;
        let times_reg = entries.len();
        entries.push(FontEntry {
            font: t_reg,
            resource_name: t_reg_res,
            obj_id: t_reg_id,
            is_bold: false,
            is_italic: false,
            family_normalized: "times".to_string(),
        });

        let (t_bold_id, t_bold_res) = font_manager
            .embed_font(doc, t_bold.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed TimesBold: {e}")))?;
        let times_bold = entries.len();
        entries.push(FontEntry {
            font: t_bold,
            resource_name: t_bold_res,
            obj_id: t_bold_id,
            is_bold: true,
            is_italic: false,
            family_normalized: "times".to_string(),
        });

        let (t_it_id, t_it_res) = font_manager
            .embed_font(doc, t_italic.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed TimesItalic: {e}")))?;
        let times_italic = entries.len();
        entries.push(FontEntry {
            font: t_italic,
            resource_name: t_it_res,
            obj_id: t_it_id,
            is_bold: false,
            is_italic: true,
            family_normalized: "times".to_string(),
        });

        let (t_bi_id, t_bi_res) = font_manager
            .embed_font(doc, t_bi.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed TimesBoldItalic: {e}")))?;
        let times_bi = entries.len();
        entries.push(FontEntry {
            font: t_bi,
            resource_name: t_bi_res,
            obj_id: t_bi_id,
            is_bold: true,
            is_italic: true,
            family_normalized: "times".to_string(),
        });

        // 3. Standard Courier
        let c_reg = Font::standard(StandardFont::Courier);
        let c_bold = Font::standard(StandardFont::CourierBold);
        let c_italic = Font::standard(StandardFont::CourierOblique);
        let c_bi = Font::standard(StandardFont::CourierBoldOblique);

        let (c_reg_id, c_reg_res) = font_manager
            .embed_font(doc, c_reg.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed Courier: {e}")))?;
        let courier_reg = entries.len();
        entries.push(FontEntry {
            font: c_reg,
            resource_name: c_reg_res,
            obj_id: c_reg_id,
            is_bold: false,
            is_italic: false,
            family_normalized: "courier".to_string(),
        });

        let (c_bold_id, c_bold_res) = font_manager
            .embed_font(doc, c_bold.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed CourierBold: {e}")))?;
        let courier_bold = entries.len();
        entries.push(FontEntry {
            font: c_bold,
            resource_name: c_bold_res,
            obj_id: c_bold_id,
            is_bold: true,
            is_italic: false,
            family_normalized: "courier".to_string(),
        });

        let (c_it_id, c_it_res) = font_manager
            .embed_font(doc, c_italic.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed CourierItalic: {e}")))?;
        let courier_italic = entries.len();
        entries.push(FontEntry {
            font: c_italic,
            resource_name: c_it_res,
            obj_id: c_it_id,
            is_bold: false,
            is_italic: true,
            family_normalized: "courier".to_string(),
        });

        let (c_bi_id, c_bi_res) = font_manager
            .embed_font(doc, c_bi.clone())
            .map_err(|e| PdfError::Pdf(format!("Failed to embed CourierBoldItalic: {e}")))?;
        let courier_bi = entries.len();
        entries.push(FontEntry {
            font: c_bi,
            resource_name: c_bi_res,
            obj_id: c_bi_id,
            is_bold: true,
            is_italic: true,
            family_normalized: "courier".to_string(),
        });

        // 4. Custom TrueType / OpenType fonts
        for cf in custom_fonts {
            if cf.data.is_empty() {
                continue;
            }
            if let Ok(font) = Font::from_bytes(cf.data.clone(), Some(cf.name.clone())) {
                let is_bold = font.is_bold();
                let is_italic = font.is_italic();
                let family_norm = normalize_font_name(&cf.name);
                if let Ok((obj_id, resource_name)) = font_manager.embed_font(doc, font.clone()) {
                    entries.push(FontEntry {
                        font,
                        resource_name,
                        obj_id,
                        is_bold,
                        is_italic,
                        family_normalized: family_norm,
                    });
                }
            }
        }

        Ok(Self {
            entries,
            helvetica_reg,
            helvetica_bold,
            helvetica_italic,
            helvetica_bi,
            times_reg,
            times_bold,
            times_italic,
            times_bi,
            courier_reg,
            courier_bold,
            courier_italic,
            courier_bi,
        })
    }

    pub fn add_to_resources(
        &self,
        font_manager: &mut FontManager,
        resources: &mut hipdf::lopdf::Dictionary,
    ) {
        for entry in &self.entries {
            font_manager.add_to_resources(resources, entry.obj_id, &entry.resource_name);
        }
    }

    pub fn resolve(
        &self,
        font_name: &str,
        is_bold: bool,
        is_italic: bool,
        family_id: i32,
    ) -> &FontEntry {
        let norm = normalize_font_name(font_name);
        let custom_candidates: Vec<&FontEntry> = self
            .entries
            .iter()
            .skip(12)
            .filter(|e| {
                !e.family_normalized.is_empty()
                    && (e.family_normalized == norm
                        || norm.contains(&e.family_normalized)
                        || e.family_normalized.contains(&norm))
            })
            .collect();

        if !custom_candidates.is_empty() {
            if let Some(entry) = custom_candidates
                .iter()
                .find(|e| e.is_bold == is_bold && e.is_italic == is_italic)
            {
                return entry;
            }
            if let Some(entry) = custom_candidates.iter().find(|e| e.is_bold == is_bold) {
                return entry;
            }
            return custom_candidates[0];
        }

        if family_id == 3 || is_monospace_name(font_name) {
            match (is_bold, is_italic) {
                (true, true) => &self.entries[self.courier_bi],
                (true, false) => &self.entries[self.courier_bold],
                (false, true) => &self.entries[self.courier_italic],
                (false, false) => &self.entries[self.courier_reg],
            }
        } else if family_id == 1 || is_serif_name(font_name) {
            match (is_bold, is_italic) {
                (true, true) => &self.entries[self.times_bi],
                (true, false) => &self.entries[self.times_bold],
                (false, true) => &self.entries[self.times_italic],
                (false, false) => &self.entries[self.times_reg],
            }
        } else {
            match (is_bold, is_italic) {
                (true, true) => &self.entries[self.helvetica_bi],
                (true, false) => &self.entries[self.helvetica_bold],
                (false, true) => &self.entries[self.helvetica_italic],
                (false, false) => &self.entries[self.helvetica_reg],
            }
        }
    }
}

fn render_cell_texts(
    model: &Model,
    options: &PdfExportOptions,
    plan: &PagePlan,
    fonts: &FontCatalog,
    ops: &mut Vec<Operation>,
) {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();
    let mut rendered_merges = HashMap::new();

    for &row in &all_rows {
        for &col in &all_cols {
            let (_is_merged, cell_x, cell_y, cell_w, cell_h, anchor_row, anchor_col) =
                if let Some(merge) = find_merge(row, col, &plan.merged_cells) {
                    if rendered_merges.contains_key(&(merge.row, merge.column)) {
                        continue; // Already processed for this page
                    }
                    rendered_merges.insert((merge.row, merge.column), true);

                    let Some((x, y, w, h)) = get_merged_rect_on_page(merge, plan) else {
                        continue;
                    };
                    (true, x, y, w, h, merge.row, merge.column)
                } else {
                    let (Some(x), Some(y)) = (plan.column_x(col), plan.row_y(row)) else {
                        continue;
                    };
                    let w = plan.col_widths.get(&col).copied().unwrap_or(0.0);
                    let h = plan.row_heights.get(&row).copied().unwrap_or(0.0);
                    (false, x, y, w, h, row, col)
                };

            if cell_w <= 0.0 || cell_h <= 0.0 {
                continue;
            }

            // Cell value resolution
            let text = if matches!(options.content.formulas, FormulaPrintPolicy::FormulaText) {
                model
                    .get_cell_formula(plan.sheet_index, anchor_row, anchor_col)
                    .unwrap_or(None)
                    .unwrap_or_else(|| {
                        model
                            .get_formatted_cell_value(plan.sheet_index, anchor_row, anchor_col)
                            .unwrap_or_default()
                    })
            } else {
                model
                    .get_formatted_cell_value(plan.sheet_index, anchor_row, anchor_col)
                    .unwrap_or_default()
            };

            let text = text.trim();
            if text.is_empty() {
                continue;
            }

            // Cell type and error detection
            let cell_type = model
                .get_cell_type(plan.sheet_index, anchor_row, anchor_col)
                .ok();

            // Error display policy
            let is_error = cell_type == Some(CellType::ErrorValue)
                || text == "#N/A"
                || text == "#N/A!"
                || text == "#N/IMPL"
                || (text.starts_with('#') && (text.ends_with('!') || text.ends_with('?')));
            let display_text = if is_error {
                match options.content.cell_errors {
                    CellErrorPolicy::Fixed(CellErrorMode::Blank)
                    | CellErrorPolicy::FromDocumentOr(CellErrorMode::Blank) => continue,
                    CellErrorPolicy::Fixed(CellErrorMode::Dash)
                    | CellErrorPolicy::FromDocumentOr(CellErrorMode::Dash) => "--".to_string(),
                    CellErrorPolicy::Fixed(CellErrorMode::NotAvailable)
                    | CellErrorPolicy::FromDocumentOr(CellErrorMode::NotAvailable) => {
                        "#N/A".to_string()
                    }
                    _ => text.to_string(),
                }
            } else {
                text.to_string()
            };

            let Ok(style) = model.get_style_for_cell(plan.sheet_index, anchor_row, anchor_col)
            else {
                continue;
            };

            // Font & style
            let font_size = ((style.font.sz as f32).max(6.0) * plan.scale).max(4.0);
            let is_b = style.font.b;
            let is_i = style.font.i;

            let active_font_entry =
                fonts.resolve(&style.font.name, is_b, is_i, style.font.family);
            let active_font = &active_font_entry.font;
            let active_res = &active_font_entry.resource_name;

            let (tr, tg, tb) = parse_hex_color(&style.font.color.to_rgb(&model.workbook.theme))
                .unwrap_or((0.0, 0.0, 0.0));

            // Alignment
            let h_align = match style
                .alignment
                .as_ref()
                .map(|a| a.horizontal.clone())
                .unwrap_or(HorizontalAlignment::General)
            {
                HorizontalAlignment::General => match cell_type {
                    Some(CellType::Number) => HorizontalAlignment::Right,
                    Some(CellType::LogicalValue) | Some(CellType::ErrorValue) => {
                        HorizontalAlignment::Center
                    }
                    _ => HorizontalAlignment::Left,
                },
                a => a,
            };

            let v_align = style
                .alignment
                .as_ref()
                .map(|a| a.vertical.clone())
                .unwrap_or(VerticalAlignment::Bottom);

            let wrap = style
                .alignment
                .as_ref()
                .map(|a| a.wrap_text)
                .unwrap_or(false);

            let padding = 3.0 * plan.scale;
            let avail_w = (cell_w - padding * 2.0).max(1.0);

            // Compute lines
            let lines: Vec<String> = if wrap {
                hipdf::fonts::utils::wrap_text(
                    active_font,
                    &display_text,
                    avail_w,
                    font_size,
                    hipdf::fonts::utils::WrapStrategy::Word,
                )
            } else {
                display_text.split('\n').map(|s| s.to_string()).collect()
            };

            if lines.is_empty() {
                continue;
            }

            let line_h = font_size * 1.25;
            let total_text_h = lines.len() as f32 * line_h;

            let text_top_y = match v_align {
                VerticalAlignment::Top => cell_y + padding,
                VerticalAlignment::Center => cell_y + (cell_h - total_text_h) / 2.0,
                VerticalAlignment::Bottom
                | VerticalAlignment::Justify
                | VerticalAlignment::Distributed => cell_y + cell_h - padding - total_text_h,
            };

            // Compute text spilling and clipping rectangle
            let mut clip_x = cell_x;
            let mut clip_w = cell_w;

            if !wrap && !_is_merged {
                let can_spill_right = matches!(
                    h_align,
                    HorizontalAlignment::Left
                        | HorizontalAlignment::General
                        | HorizontalAlignment::Justify
                        | HorizontalAlignment::Distributed
                        | HorizontalAlignment::Center
                        | HorizontalAlignment::CenterContinuous
                );
                let can_spill_left = matches!(
                    h_align,
                    HorizontalAlignment::Right
                        | HorizontalAlignment::Fill
                        | HorizontalAlignment::Center
                        | HorizontalAlignment::CenterContinuous
                );

                if can_spill_right {
                    let mut next_col = col + 1;
                    while let Some(&w) = plan.col_widths.get(&next_col) {
                        let is_covered =
                            plan.merged_cells.iter().any(|m| m.contains(row, next_col));
                        let val = model
                            .get_formatted_cell_value(plan.sheet_index, row, next_col)
                            .unwrap_or_default();
                        if !is_covered && val.trim().is_empty() {
                            clip_w += w;
                            next_col += 1;
                        } else {
                            break;
                        }
                    }
                }

                if can_spill_left {
                    let mut prev_col = col - 1;
                    while let Some(&w) = plan.col_widths.get(&prev_col) {
                        let is_covered =
                            plan.merged_cells.iter().any(|m| m.contains(row, prev_col));
                        let val = model
                            .get_formatted_cell_value(plan.sheet_index, row, prev_col)
                            .unwrap_or_default();
                        if !is_covered && val.trim().is_empty() {
                            clip_x -= w;
                            clip_w += w;
                            prev_col -= 1;
                        } else {
                            break;
                        }
                    }
                }
            }

            // Clip text to computed boundary (with spill)
            let pdf_cell_y = plan.page_height - cell_y - cell_h;
            ops.push(Operation::new("q", vec![]));
            ops.push(Operation::new(
                "re",
                vec![
                    clip_x.into(),
                    pdf_cell_y.into(),
                    clip_w.into(),
                    cell_h.into(),
                ],
            ));
            ops.push(Operation::new("W", vec![]));
            ops.push(Operation::new("n", vec![]));

            for (idx, line) in lines.iter().enumerate() {
                let text_w = active_font.text_width(line, font_size);

                let line_x = match h_align {
                    HorizontalAlignment::Left
                    | HorizontalAlignment::General
                    | HorizontalAlignment::Justify
                    | HorizontalAlignment::Distributed => cell_x + padding,
                    HorizontalAlignment::Center | HorizontalAlignment::CenterContinuous => {
                        cell_x + (cell_w - text_w) / 2.0
                    }
                    HorizontalAlignment::Right | HorizontalAlignment::Fill => {
                        cell_x + cell_w - padding - text_w
                    }
                };

                let line_y_from_top = text_top_y + (idx as f32) * line_h;
                let baseline_pdf_y = plan.page_height - (line_y_from_top + font_size * 0.85);

                let encoded_bytes = if active_font.needs_utf16_encoding() {
                    active_font.encode_text(line)
                } else {
                    sanitize_winansi_text(line)
                };

                let text_ops = TextBuilder::new()
                    .begin_text()
                    .set_fill_color(tr, tg, tb)
                    .set_font(active_res, font_size)
                    .position(line_x, baseline_pdf_y)
                    .show_encoded(encoded_bytes)
                    .end_text()
                    .build();

                ops.extend(text_ops);

                // Underline
                if style.font.u {
                    ops.push(Operation::new("q", vec![]));
                    ops.push(Operation::new("w", vec![0.75.into()]));
                    ops.push(Operation::new("RG", vec![tr.into(), tg.into(), tb.into()]));
                    ops.push(Operation::new(
                        "m",
                        vec![line_x.into(), (baseline_pdf_y - 1.5).into()],
                    ));
                    ops.push(Operation::new(
                        "l",
                        vec![(line_x + text_w).into(), (baseline_pdf_y - 1.5).into()],
                    ));
                    ops.push(Operation::new("S", vec![]));
                    ops.push(Operation::new("Q", vec![]));
                }

                // Strikethrough
                if style.font.strike {
                    ops.push(Operation::new("q", vec![]));
                    ops.push(Operation::new("w", vec![0.75.into()]));
                    ops.push(Operation::new("RG", vec![tr.into(), tg.into(), tb.into()]));
                    ops.push(Operation::new(
                        "m",
                        vec![line_x.into(), (baseline_pdf_y + font_size * 0.35).into()],
                    ));
                    ops.push(Operation::new(
                        "l",
                        vec![
                            (line_x + text_w).into(),
                            (baseline_pdf_y + font_size * 0.35).into(),
                        ],
                    ));
                    ops.push(Operation::new("S", vec![]));
                    ops.push(Operation::new("Q", vec![]));
                }
            }

            ops.push(Operation::new("Q", vec![]));
        }
    }
}

fn render_headings(
    plan: &PagePlan,
    font_regular: &Font,
    font_bold: &Font,
    res_reg: &str,
    res_bold: &str,
    ops: &mut Vec<Operation>,
) {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();
    if all_cols.is_empty() || all_rows.is_empty() {
        return;
    }

    let h_col_w = plan.heading_col_width;
    let h_row_h = plan.heading_row_height;
    let x0 = plan.content_x;
    let y0 = plan.content_y;

    // Top-left corner box
    let pdf_corner_y = plan.page_height - y0 - h_row_h;
    ops.push(Operation::new("q", vec![]));
    ops.push(Operation::new(
        "rg",
        vec![0.92.into(), 0.93.into(), 0.94.into()],
    )); // #EAECEF
    ops.push(Operation::new(
        "re",
        vec![
            x0.into(),
            pdf_corner_y.into(),
            h_col_w.into(),
            h_row_h.into(),
        ],
    ));
    ops.push(Operation::new("f", vec![]));
    ops.push(Operation::new("w", vec![0.75.into()]));
    ops.push(Operation::new(
        "RG",
        vec![0.75.into(), 0.77.into(), 0.80.into()],
    ));
    ops.push(Operation::new(
        "re",
        vec![
            x0.into(),
            pdf_corner_y.into(),
            h_col_w.into(),
            h_row_h.into(),
        ],
    ));
    ops.push(Operation::new("S", vec![]));
    ops.push(Operation::new("Q", vec![]));

    // Column Headings (A, B, C...)
    let font_size = (9.0 * plan.scale).max(6.0);
    let mut current_x = x0 + h_col_w;
    let col_h_pdf_y = plan.page_height - y0 - h_row_h;

    for &col in &all_cols {
        let cw = plan.col_widths.get(&col).copied().unwrap_or(0.0);
        if cw <= 0.0 {
            continue;
        }

        // Background & border
        ops.push(Operation::new("q", vec![]));
        ops.push(Operation::new(
            "rg",
            vec![0.96.into(), 0.96.into(), 0.97.into()],
        )); // #F4F5F7
        ops.push(Operation::new(
            "re",
            vec![
                current_x.into(),
                col_h_pdf_y.into(),
                cw.into(),
                h_row_h.into(),
            ],
        ));
        ops.push(Operation::new("f", vec![]));
        ops.push(Operation::new("w", vec![0.5.into()]));
        ops.push(Operation::new(
            "RG",
            vec![0.80.into(), 0.82.into(), 0.85.into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![
                current_x.into(),
                col_h_pdf_y.into(),
                cw.into(),
                h_row_h.into(),
            ],
        ));
        ops.push(Operation::new("S", vec![]));
        ops.push(Operation::new("Q", vec![]));

        // Text
        let label = number_to_column(col).unwrap_or_else(|| format!("{col}"));
        let tw = font_bold.text_width(&label, font_size);
        let tx = current_x + (cw - tw) / 2.0;
        let ty = col_h_pdf_y + (h_row_h - font_size) / 2.0;

        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(0.35, 0.40, 0.48) // #5E6C84
            .set_font(res_bold, font_size)
            .position(tx, ty)
            .show(&label)
            .end_text()
            .build();
        ops.extend(text_ops);

        current_x += cw;
    }

    // Row Headings (1, 2, 3...)
    let mut current_y = y0 + h_row_h;
    for &row in &all_rows {
        let rh = plan.row_heights.get(&row).copied().unwrap_or(0.0);
        if rh <= 0.0 {
            continue;
        }
        let row_h_pdf_y = plan.page_height - current_y - rh;

        // Background & border
        ops.push(Operation::new("q", vec![]));
        ops.push(Operation::new(
            "rg",
            vec![0.96.into(), 0.96.into(), 0.97.into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![x0.into(), row_h_pdf_y.into(), h_col_w.into(), rh.into()],
        ));
        ops.push(Operation::new("f", vec![]));
        ops.push(Operation::new("w", vec![0.5.into()]));
        ops.push(Operation::new(
            "RG",
            vec![0.80.into(), 0.82.into(), 0.85.into()],
        ));
        ops.push(Operation::new(
            "re",
            vec![x0.into(), row_h_pdf_y.into(), h_col_w.into(), rh.into()],
        ));
        ops.push(Operation::new("S", vec![]));
        ops.push(Operation::new("Q", vec![]));

        // Text
        let label = format!("{row}");
        let tw = font_regular.text_width(&label, font_size);
        let tx = x0 + h_col_w - tw - 4.0;
        let ty = row_h_pdf_y + (rh - font_size) / 2.0;

        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(0.35, 0.40, 0.48)
            .set_font(res_reg, font_size)
            .position(tx, ty)
            .show(&label)
            .end_text()
            .build();
        ops.extend(text_ops);

        current_y += rh;
    }
}

fn render_headers_and_footers(
    plan: &PagePlan,
    font_regular: &Font,
    res_reg: &str,
    ops: &mut Vec<Operation>,
) {
    let font_size = 9.0;
    let color = (0.4, 0.4, 0.4);

    let left_x = plan.margins.left_pt();
    let right_x = plan.page_width - plan.margins.right_pt();
    let center_x = (left_x + right_x) / 2.0;

    // Header (at top)
    let header_y_pdf = plan.page_height - plan.margins.header_pt() - 10.0;
    let (ref hl, ref hc, ref hr) = plan.header_text;

    if !hl.is_empty() {
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(left_x, header_y_pdf)
            .show(hl)
            .end_text()
            .build();
        ops.extend(text_ops);
    }
    if !hc.is_empty() {
        let tw = font_regular.text_width(hc, font_size);
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(center_x - tw / 2.0, header_y_pdf)
            .show(hc)
            .end_text()
            .build();
        ops.extend(text_ops);
    }
    if !hr.is_empty() {
        let tw = font_regular.text_width(hr, font_size);
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(right_x - tw, header_y_pdf)
            .show(hr)
            .end_text()
            .build();
        ops.extend(text_ops);
    }

    // Footer (at bottom)
    let footer_y_pdf = plan.margins.footer_pt() + 6.0;
    let (ref fl, ref fc, ref fr) = plan.footer_text;

    if !fl.is_empty() {
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(left_x, footer_y_pdf)
            .show(fl)
            .end_text()
            .build();
        ops.extend(text_ops);
    }
    if !fc.is_empty() {
        let tw = font_regular.text_width(fc, font_size);
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(center_x - tw / 2.0, footer_y_pdf)
            .show(fc)
            .end_text()
            .build();
        ops.extend(text_ops);
    }
    if !fr.is_empty() {
        let tw = font_regular.text_width(fr, font_size);
        let text_ops = TextBuilder::new()
            .begin_text()
            .set_fill_color(color.0, color.1, color.2)
            .set_font(res_reg, font_size)
            .position(right_x - tw, footer_y_pdf)
            .show(fr)
            .end_text()
            .build();
        ops.extend(text_ops);
    }
}

fn find_merge(row: i32, col: i32, merges: &[MergedCell]) -> Option<&MergedCell> {
    merges.iter().find(|m| m.contains(row, col))
}

fn get_merged_rect_on_page(merge: &MergedCell, plan: &PagePlan) -> Option<(f32, f32, f32, f32)> {
    let all_cols = plan.all_columns();
    let all_rows = plan.all_rows();

    let page_cols: Vec<i32> = all_cols
        .into_iter()
        .filter(|&c| c >= merge.column && c < merge.column + merge.width)
        .collect();
    let page_rows: Vec<i32> = all_rows
        .into_iter()
        .filter(|&r| r >= merge.row && r < merge.row + merge.height)
        .collect();

    if page_cols.is_empty() || page_rows.is_empty() {
        return None;
    }

    let first_col = *page_cols.first()?;
    let first_row = *page_rows.first()?;

    let x = plan.column_x(first_col)?;
    let y = plan.row_y(first_row)?;

    let mut w = 0.0;
    for &c in &page_cols {
        if let Some(cw) = plan.col_widths.get(&c) {
            w += *cw;
        }
    }

    let mut h = 0.0;
    for &r in &page_rows {
        if let Some(rh) = plan.row_heights.get(&r) {
            h += *rh;
        }
    }

    if w > 0.0 && h > 0.0 {
        Some((x, y, w, h))
    } else {
        None
    }
}
