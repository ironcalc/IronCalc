//! Layout and pagination calculation engine.
//!
//! Resolves workbook sheets, ranges, print areas, page setups, scaling,
//! and pagination into deterministic page plans ready for rendering.

use std::collections::HashMap;

use chrono::Local;
use ironcalc_base::types::{MergedCell, SheetState};
use ironcalc_base::Model;

use crate::error::PdfError;
use crate::options::*;

/// Represents the planned layout and content of a single PDF page.
#[derive(Debug, Clone)]
pub struct PagePlan {
    /// 0-indexed sheet in workbook.
    pub sheet_index: u32,
    /// Sheet name.
    pub sheet_name: String,
    /// 1-indexed page number within the exported document.
    pub page_number: u32,
    /// Total pages in the exported document.
    pub total_pages: u32,
    /// Page width in PDF points.
    pub page_width: f32,
    /// Page height in PDF points.
    pub page_height: f32,
    /// Margins in millimeters.
    pub margins: Margins,
    /// Top-left content X coordinate in PDF points (distance from left of page).
    pub content_x: f32,
    /// Top-left content Y coordinate in PDF points (distance from top of page).
    pub content_y: f32,
    /// Scaling factor applied to cells, fonts, and dimensions.
    pub scale: f32,
    /// Whether cell gridlines should be drawn.
    pub show_gridlines: bool,
    /// Whether row numbers (1, 2, ...) and column letters (A, B, ...) should be drawn.
    pub show_headings: bool,
    /// Scaled width of row headings column (0.0 if headings disabled).
    pub heading_col_width: f32,
    /// Scaled height of column headings row (0.0 if headings disabled).
    pub heading_row_height: f32,
    /// Repeated title columns on this page (anchored before slice columns).
    pub repeat_cols: Vec<i32>,
    /// Data columns on this page.
    pub slice_cols: Vec<i32>,
    /// Repeated title rows on this page (anchored before slice rows).
    pub repeat_rows: Vec<i32>,
    /// Data rows on this page.
    pub slice_rows: Vec<i32>,
    /// Scaled width for each column (column_index -> width_pt).
    pub col_widths: HashMap<i32, f32>,
    /// Scaled height for each row (row_index -> height_pt).
    pub row_heights: HashMap<i32, f32>,
    /// Merged cells relevant to this sheet.
    pub merged_cells: Vec<MergedCell>,
    /// Evaluated header text (left, center, right).
    pub header_text: (String, String, String),
    /// Evaluated footer text (left, center, right).
    pub footer_text: (String, String, String),
}

impl PagePlan {
    /// Returns all columns drawn on this page in left-to-right order.
    pub fn all_columns(&self) -> Vec<i32> {
        let mut cols = Vec::with_capacity(self.repeat_cols.len() + self.slice_cols.len());
        cols.extend_from_slice(&self.repeat_cols);
        cols.extend_from_slice(&self.slice_cols);
        cols
    }

    /// Returns all rows drawn on this page in top-to-bottom order.
    pub fn all_rows(&self) -> Vec<i32> {
        let mut rows = Vec::with_capacity(self.repeat_rows.len() + self.slice_rows.len());
        rows.extend_from_slice(&self.repeat_rows);
        rows.extend_from_slice(&self.slice_rows);
        rows
    }

    /// Total width of the table content on this page (including row headings if enabled).
    pub fn content_width(&self) -> f32 {
        let mut w = if self.show_headings {
            self.heading_col_width
        } else {
            0.0
        };
        for c in self.all_columns() {
            if let Some(cw) = self.col_widths.get(&c) {
                w += *cw;
            }
        }
        w
    }

    /// Total height of the table content on this page (including column headings if enabled).
    pub fn content_height(&self) -> f32 {
        let mut h = if self.show_headings {
            self.heading_row_height
        } else {
            0.0
        };
        for r in self.all_rows() {
            if let Some(rh) = self.row_heights.get(&r) {
                h += *rh;
            }
        }
        h
    }

    /// Calculates the top-left X coordinate of a column on this page (relative to left of page).
    pub fn column_x(&self, col: i32) -> Option<f32> {
        let all_cols = self.all_columns();
        if !all_cols.contains(&col) {
            return None;
        }

        let mut x = self.content_x;
        if self.show_headings {
            x += self.heading_col_width;
        }

        for c in all_cols {
            if c == col {
                return Some(x);
            }
            if let Some(cw) = self.col_widths.get(&c) {
                x += *cw;
            }
        }
        None
    }

    /// Calculates the top-left Y coordinate of a row on this page (relative to top of page).
    pub fn row_y(&self, row: i32) -> Option<f32> {
        let all_rows = self.all_rows();
        if !all_rows.contains(&row) {
            return None;
        }

        let mut y = self.content_y;
        if self.show_headings {
            y += self.heading_row_height;
        }

        for r in all_rows {
            if r == row {
                return Some(y);
            }
            if let Some(rh) = self.row_heights.get(&r) {
                y += *rh;
            }
        }
        None
    }
}

/// Points-per-pixel conversion factor (standard 72 pt/inch / 96 px/inch).
pub const PT_PER_PX: f32 = 72.0 / 96.0;

/// Default row heading column width in points.
pub const DEFAULT_HEADING_COL_WIDTH_PT: f32 = 30.0 * PT_PER_PX;

/// Default column heading row height in points.
pub const DEFAULT_HEADING_ROW_HEIGHT_PT: f32 = 24.0 * PT_PER_PX;

/// Plans the page layouts for exporting the model to PDF according to the given options.
pub fn plan_export_layout(
    model: &Model,
    options: &PdfExportOptions,
) -> Result<Vec<PagePlan>, PdfError> {
    let sheets_to_export = resolve_sheets_to_export(model, options)?;
    if sheets_to_export.is_empty() {
        return Err(PdfError::SheetNotFound(
            "No sheets found matching export scope".to_string(),
        ));
    }

    let mut unnumbered_plans = Vec::new();

    for (sheet_idx, explicit_range) in sheets_to_export {
        let worksheet = match model.workbook.worksheets.get(sheet_idx as usize) {
            Some(ws) => ws,
            None => {
                return Err(PdfError::SheetNotFound(format!(
                    "Sheet index {sheet_idx} out of range"
                )))
            }
        };

        let sheet_plans = plan_sheet_layout(model, worksheet, sheet_idx, explicit_range, options)?;
        unnumbered_plans.extend(sheet_plans);
    }

    if unnumbered_plans.is_empty() {
        return Err(PdfError::IronCalc(
            "Export produced no printable pages".to_string(),
        ));
    }

    // Number pages and resolve header/footer page count tokens
    let total_pages = unnumbered_plans.len() as u32;
    let mut final_plans = Vec::with_capacity(unnumbered_plans.len());

    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H:%M").to_string();

    let start_num = match options.page.first_page_number {
        FirstPageNumberPolicy::Number(n) => n,
        _ => 1,
    };

    for (i, mut plan) in unnumbered_plans.into_iter().enumerate() {
        let page_num = start_num + (i as u32);
        plan.page_number = page_num;
        plan.total_pages = total_pages;

        // Evaluate headers and footers
        plan.header_text = evaluate_header_footer_layout(
            &options.header_footer,
            &plan,
            model,
            page_num,
            total_pages,
            &date_str,
            &time_str,
            true,
        );

        plan.footer_text = evaluate_header_footer_layout(
            &options.header_footer,
            &plan,
            model,
            page_num,
            total_pages,
            &date_str,
            &time_str,
            false,
        );

        final_plans.push(plan);
    }

    Ok(final_plans)
}

fn resolve_sheets_to_export(
    model: &Model,
    options: &PdfExportOptions,
) -> Result<Vec<(u32, Option<CellRange>)>, PdfError> {
    match &options.scope {
        ExportScope::ActiveSheet => {
            let active = model
                .workbook
                .worksheets
                .iter()
                .position(|ws| ws.state == SheetState::Visible)
                .unwrap_or(0) as u32;
            Ok(vec![(active, None)])
        }
        ExportScope::SelectedSheets(indices) => {
            let mut list = Vec::new();
            for &idx in indices {
                if (idx as usize) < model.workbook.worksheets.len() {
                    list.push((idx, None));
                }
            }
            Ok(list)
        }
        ExportScope::EntireWorkbook => {
            let mut list = Vec::new();
            for (idx, ws) in model.workbook.worksheets.iter().enumerate() {
                if ws.state == SheetState::Visible {
                    list.push((idx as u32, None));
                }
            }
            Ok(list)
        }
        ExportScope::Selection { sheet, range } => Ok(vec![(*sheet, Some(*range))]),
        ExportScope::ExplicitRanges(ranges) => {
            Ok(ranges.iter().map(|sr| (sr.sheet, Some(sr.range))).collect())
        }
    }
}

fn plan_sheet_layout(
    model: &Model,
    worksheet: &ironcalc_base::types::Worksheet,
    sheet_idx: u32,
    explicit_range: Option<CellRange>,
    options: &PdfExportOptions,
) -> Result<Vec<PagePlan>, PdfError> {
    // 1. Resolve Paper Size
    let (raw_w_pt, raw_h_pt) = match &options.page.paper_size {
        PaperSizePolicy::FromDocumentOr(sz) | PaperSizePolicy::Fixed(sz) => sz.dimensions_pt(),
        PaperSizePolicy::Custom {
            width_mm,
            height_mm,
        } => {
            const MM_TO_PT: f32 = 72.0 / 25.4;
            (width_mm * MM_TO_PT, height_mm * MM_TO_PT)
        }
    };

    // 2. Resolve Orientation
    let orientation = match options.page.orientation {
        OrientationPolicy::FromDocumentOr(o) | OrientationPolicy::Fixed(o) => o,
    };

    let (page_w, page_h) = match orientation {
        Orientation::Portrait => (raw_w_pt.min(raw_h_pt), raw_w_pt.max(raw_h_pt)),
        Orientation::Landscape => (raw_w_pt.max(raw_h_pt), raw_w_pt.min(raw_h_pt)),
    };

    // 3. Resolve Margins
    let margins = match options.page.margins {
        MarginPolicy::FromDocumentOr(m) | MarginPolicy::Fixed(m) => m,
    };

    let left_margin = margins.left_pt();
    let right_margin = margins.right_pt();
    let top_margin = margins.top_pt();
    let bottom_margin = margins.bottom_pt();

    // Reserve space for header / footer if present
    let has_header = !matches!(options.header_footer.mode, HeaderFooterMode::None);
    let has_footer = !matches!(options.header_footer.mode, HeaderFooterMode::None);

    let header_space = if has_header {
        (margins.header_pt() + 14.0).max(18.0)
    } else {
        0.0
    };
    let footer_space = if has_footer {
        (margins.footer_pt() + 14.0).max(18.0)
    } else {
        0.0
    };

    let printable_w = (page_w - left_margin - right_margin).max(10.0);
    let printable_h = (page_h - top_margin - bottom_margin - header_space - footer_space).max(10.0);

    // 4. Resolve Content Range
    let cell_range = if let Some(r) = explicit_range {
        r
    } else {
        match options.range.print_area {
            PrintAreaPolicy::OnlyExplicitRange(r) => r,
            PrintAreaPolicy::HonorDefinedPrintAreas | PrintAreaPolicy::IgnoreDefinedPrintAreas => {
                let dim = worksheet.dimension();
                CellRange::new(dim.min_row, dim.min_column, dim.max_row, dim.max_column)
            }
        }
    };

    // 5. Filter visible rows and columns
    let omit_hidden = matches!(options.range.hidden_rows_columns, HiddenContentPolicy::Omit);

    let mut visible_cols = Vec::new();
    for col in cell_range.start_col..=cell_range.end_col {
        if omit_hidden && model.is_column_hidden(sheet_idx, col).unwrap_or(false) {
            continue;
        }
        let w = model.get_column_width(sheet_idx, col).unwrap_or(90.0);
        if omit_hidden && w <= 0.0 {
            continue;
        }
        visible_cols.push(col);
    }
    if visible_cols.is_empty() {
        visible_cols.push(cell_range.start_col);
    }

    let mut visible_rows = Vec::new();
    for row in cell_range.start_row..=cell_range.end_row {
        if omit_hidden && model.is_row_hidden(sheet_idx, row).unwrap_or(false) {
            continue;
        }
        let h = model.get_row_height(sheet_idx, row).unwrap_or(25.0);
        if omit_hidden && h <= 0.0 {
            continue;
        }
        visible_rows.push(row);
    }
    if visible_rows.is_empty() {
        visible_rows.push(cell_range.start_row);
    }

    // 6. Repeated rows and columns (print titles or frozen panes)
    let (mut repeat_cols, mut repeat_rows) = match &options.repeat.titles {
        RepeatTitlePolicy::Explicit { rows, columns } => {
            let r_cols = if let Some(c) = columns {
                (c.start_col..=c.end_col)
                    .filter(|c| visible_cols.contains(c))
                    .collect()
            } else {
                Vec::new()
            };
            let r_rows = if let Some(r) = rows {
                (r.start_row..=r.end_row)
                    .filter(|r| visible_rows.contains(r))
                    .collect()
            } else {
                Vec::new()
            };
            (r_cols, r_rows)
        }
        _ => (Vec::new(), Vec::new()),
    };

    if repeat_cols.is_empty()
        && matches!(
            options.repeat.frozen_panes,
            FrozenPanePrintPolicy::RepeatAsPrintTitles
        )
    {
        let frozen_c = model.get_frozen_columns_count(sheet_idx).unwrap_or(0);
        if frozen_c > 0 {
            repeat_cols = (1..=frozen_c)
                .filter(|c| visible_cols.contains(c))
                .collect();
        }
    }

    if repeat_rows.is_empty()
        && matches!(
            options.repeat.frozen_panes,
            FrozenPanePrintPolicy::RepeatAsPrintTitles
        )
    {
        let frozen_r = model.get_frozen_rows_count(sheet_idx).unwrap_or(0);
        if frozen_r > 0 {
            repeat_rows = (1..=frozen_r)
                .filter(|r| visible_rows.contains(r))
                .collect();
        }
    }

    // Remove repeated cols/rows from data cols/rows so they don't appear twice
    let data_cols: Vec<i32> = visible_cols
        .iter()
        .copied()
        .filter(|c| !repeat_cols.contains(c))
        .collect();
    let data_rows: Vec<i32> = visible_rows
        .iter()
        .copied()
        .filter(|r| !repeat_rows.contains(r))
        .collect();

    // 7. Headings settings
    let show_headings = match options.content.row_column_headings {
        SheetBooleanPolicy::FromDocumentOr(b) | SheetBooleanPolicy::Fixed(b) => b,
    };
    let unscaled_heading_col_w = if show_headings {
        let max_row = *visible_rows.iter().max().unwrap_or(&1);
        let digits = format!("{max_row}").len();
        (DEFAULT_HEADING_COL_WIDTH_PT + (digits.saturating_sub(2) as f32) * 6.0).max(20.0)
    } else {
        0.0
    };
    let unscaled_heading_row_h = if show_headings {
        DEFAULT_HEADING_ROW_HEIGHT_PT
    } else {
        0.0
    };

    // 8. Unscaled widths and heights
    let mut unscaled_col_widths = HashMap::new();
    for &col in &visible_cols {
        let w_px = model.get_column_width(sheet_idx, col).unwrap_or(90.0) as f32;
        unscaled_col_widths.insert(col, w_px * PT_PER_PX);
    }

    let mut unscaled_row_heights = HashMap::new();
    for &row in &visible_rows {
        let h_px = model.get_row_height(sheet_idx, row).unwrap_or(25.0) as f32;
        unscaled_row_heights.insert(row, h_px * PT_PER_PX);
    }

    let total_unscaled_cols_w: f32 = visible_cols.iter().map(|c| unscaled_col_widths[c]).sum();
    let total_unscaled_rows_h: f32 = visible_rows.iter().map(|r| unscaled_row_heights[r]).sum();

    let full_table_w = unscaled_heading_col_w + total_unscaled_cols_w;
    let full_table_h = unscaled_heading_row_h + total_unscaled_rows_h;

    // 9. Calculate Scaling
    let scaling = match &options.layout.scaling {
        ScalingPolicy::FromDocumentOr(s) | ScalingPolicy::Fixed(s) => s.clone(),
    };

    let scale = match scaling {
        Scaling::ActualSize => 1.0,
        Scaling::Percent(pct) => (pct as f32) / 100.0,
        Scaling::FitSheetOnOnePage => {
            let sx = printable_w / full_table_w;
            let sy = printable_h / full_table_h;
            sx.min(sy).clamp(0.1, 1.0)
        }
        Scaling::FitToWidth {
            pages_wide,
            pages_tall,
        } => {
            let sx = (printable_w * pages_wide as f32) / full_table_w;
            if let Some(ptall) = pages_tall {
                let sy = (printable_h * ptall as f32) / full_table_h;
                sx.min(sy).clamp(0.1, 1.0)
            } else {
                sx.clamp(0.1, 1.0)
            }
        }
        Scaling::FitToHeight {
            pages_wide,
            pages_tall,
        } => {
            let sy = (printable_h * pages_tall as f32) / full_table_h;
            if let Some(pwide) = pages_wide {
                let sx = (printable_w * pwide as f32) / full_table_w;
                sx.min(sy).clamp(0.1, 1.0)
            } else {
                sy.clamp(0.1, 1.0)
            }
        }
        Scaling::FitToPages {
            pages_wide,
            pages_tall,
        } => {
            let sx = (printable_w * pages_wide as f32) / full_table_w;
            let sy = (printable_h * pages_tall as f32) / full_table_h;
            sx.min(sy).clamp(0.1, 1.0)
        }
    };

    // Scaled widths and heights
    let heading_col_width = unscaled_heading_col_w * scale;
    let heading_row_height = unscaled_heading_row_h * scale;

    let mut col_widths = HashMap::new();
    for (&c, &w) in &unscaled_col_widths {
        col_widths.insert(c, w * scale);
    }

    let mut row_heights = HashMap::new();
    for (&r, &h) in &unscaled_row_heights {
        row_heights.insert(r, h * scale);
    }

    // 10. Slice columns and rows into pages
    let repeat_cols_w: f32 = repeat_cols.iter().map(|c| col_widths[c]).sum();
    let avail_slice_w = (printable_w - heading_col_width - repeat_cols_w).max(20.0);

    let repeat_rows_h: f32 = repeat_rows.iter().map(|r| row_heights[r]).sum();
    let avail_slice_h = (printable_h - heading_row_height - repeat_rows_h).max(20.0);

    let col_slices = slice_items(&data_cols, &col_widths, avail_slice_w);
    let row_slices = slice_items(&data_rows, &row_heights, avail_slice_h);

    let (final_col_slices, final_row_slices) = if matches!(scaling, Scaling::FitSheetOnOnePage) {
        (vec![data_cols.clone()], vec![data_rows.clone()])
    } else if !options.layout.allow_sheet_to_span_multiple_pages {
        (
            vec![col_slices.first().cloned().unwrap_or_default()],
            vec![row_slices.first().cloned().unwrap_or_default()],
        )
    } else {
        (col_slices, row_slices)
    };

    // 11. Determine page order
    let page_order = match &options.layout.page_order {
        PageOrderPolicy::FromDocumentOr(po) | PageOrderPolicy::Fixed(po) => *po,
    };

    let doc_show_gridlines = model
        .workbook
        .worksheet(sheet_idx)
        .map(|ws| ws.show_grid_lines)
        .unwrap_or(true);

    let show_gridlines = match options.content.gridlines {
        SheetBooleanPolicy::FromDocumentOr(_) => doc_show_gridlines,
        SheetBooleanPolicy::Fixed(b) => b,
    };

    let (center_h, center_v) = match options.page.center_on_page {
        CenterOnPagePolicy::FromDocumentOr {
            horizontal,
            vertical,
        }
        | CenterOnPagePolicy::Fixed {
            horizontal,
            vertical,
        } => (horizontal, vertical),
    };

    let mut plans = Vec::new();

    let make_page = |c_slice: &[i32], r_slice: &[i32]| -> PagePlan {
        let mut total_page_content_w = heading_col_width + repeat_cols_w;
        for c in c_slice {
            total_page_content_w += col_widths[c];
        }

        let mut total_page_content_h = heading_row_height + repeat_rows_h;
        for r in r_slice {
            total_page_content_h += row_heights[r];
        }

        let content_x = if center_h && printable_w > total_page_content_w {
            left_margin + (printable_w - total_page_content_w) / 2.0
        } else {
            left_margin
        };

        let content_y = if center_v && printable_h > total_page_content_h {
            top_margin + header_space + (printable_h - total_page_content_h) / 2.0
        } else {
            top_margin + header_space
        };

        PagePlan {
            sheet_index: sheet_idx,
            sheet_name: worksheet.name.clone(),
            page_number: 1,
            total_pages: 1,
            page_width: page_w,
            page_height: page_h,
            margins,
            content_x,
            content_y,
            scale,
            show_gridlines,
            show_headings,
            heading_col_width,
            heading_row_height,
            repeat_cols: repeat_cols.clone(),
            slice_cols: c_slice.to_vec(),
            repeat_rows: repeat_rows.clone(),
            slice_rows: r_slice.to_vec(),
            col_widths: col_widths.clone(),
            row_heights: row_heights.clone(),
            merged_cells: worksheet.merged_cells.clone(),
            header_text: (String::new(), String::new(), String::new()),
            footer_text: (String::new(), String::new(), String::new()),
        }
    };

    match page_order {
        PageOrder::DownThenOver => {
            for c_slice in &final_col_slices {
                for r_slice in &final_row_slices {
                    plans.push(make_page(c_slice, r_slice));
                }
            }
        }
        PageOrder::OverThenDown => {
            for r_slice in &final_row_slices {
                for c_slice in &final_col_slices {
                    plans.push(make_page(c_slice, r_slice));
                }
            }
        }
    }

    Ok(plans)
}

fn slice_items(items: &[i32], sizes: &HashMap<i32, f32>, max_size: f32) -> Vec<Vec<i32>> {
    if items.is_empty() {
        return vec![Vec::new()];
    }

    let mut slices = Vec::new();
    let mut current_slice = Vec::new();
    let mut current_size = 0.0;
    let threshold = max_size + 0.5;

    for &item in items {
        let item_size = sizes.get(&item).copied().unwrap_or(0.0);
        if !current_slice.is_empty() && (current_size + item_size > threshold) {
            slices.push(current_slice);
            current_slice = vec![item];
            current_size = item_size;
        } else {
            current_slice.push(item);
            current_size += item_size;
        }
    }

    if !current_slice.is_empty() {
        slices.push(current_slice);
    }

    slices
}

#[allow(clippy::too_many_arguments)]
fn evaluate_header_footer_layout(
    options: &HeaderFooterOptions,
    plan: &PagePlan,
    model: &Model,
    page_num: u32,
    total_pages: u32,
    date_str: &str,
    time_str: &str,
    is_header: bool,
) -> (String, String, String) {
    let layout = match &options.mode {
        HeaderFooterMode::Explicit { header, footer } => {
            if is_header {
                header
            } else {
                footer
            }
        }
        _ => return (String::new(), String::new(), String::new()),
    };

    let eval_field = |field: &HeaderFooterField| -> String {
        match field {
            HeaderFooterField::Empty => String::new(),
            HeaderFooterField::Text(s) => s.clone(),
            HeaderFooterField::PageNumber => format!("{page_num}"),
            HeaderFooterField::PageNumberOfTotal => format!("{page_num} / {total_pages}"),
            HeaderFooterField::SheetName => plan.sheet_name.clone(),
            HeaderFooterField::WorkbookName => model.workbook.name.clone(),
            HeaderFooterField::Date => date_str.to_string(),
            HeaderFooterField::Time => time_str.to_string(),
            HeaderFooterField::Custom(tokens) => {
                let mut out = String::new();
                for t in tokens {
                    match t {
                        HeaderFooterToken::Text(s) => out.push_str(s),
                        HeaderFooterToken::PageNumber => out.push_str(&format!("{page_num}")),
                        HeaderFooterToken::TotalPages => out.push_str(&format!("{total_pages}")),
                        HeaderFooterToken::SheetName => out.push_str(&plan.sheet_name),
                        HeaderFooterToken::WorkbookName => out.push_str(&model.workbook.name),
                        HeaderFooterToken::Date => out.push_str(date_str),
                        HeaderFooterToken::Time => out.push_str(time_str),
                        _ => {}
                    }
                }
                out
            }
        }
    };

    (
        eval_field(&layout.left),
        eval_field(&layout.center),
        eval_field(&layout.right),
    )
}
