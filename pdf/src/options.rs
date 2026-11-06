//! Configuration options for spreadsheet-to-PDF export.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Main export configuration controlling the PDF export pipeline.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfExportOptions {
    /// Which sheets or ranges to include in the output.
    pub scope: ExportScope,
    /// How workbook-persisted print properties interact with export settings.
    pub print_settings: PrintSettingsPolicy,
    /// Page geometry, dimensions, orientation, and margins.
    pub page: PageOptions,
    /// Pagination, scaling, and page order.
    pub layout: LayoutOptions,
    /// Cell range extraction and visibility.
    pub range: RangeOptions,
    /// Repeated headers/titles and frozen pane reproduction.
    pub repeat: RepeatOptions,
    /// Content rendering controls (gridlines, row/col headings, formulas, errors).
    pub content: ContentOptions,
    /// Header and footer layout and token replacement.
    pub header_footer: HeaderFooterOptions,
    /// PDF backend document properties, hyperlinks, and bookmarks.
    pub pdf: PdfDocumentOptions,
    /// Custom font data to embed into the PDF (e.g. from Google Fonts or local files).
    #[cfg_attr(feature = "serde", serde(default))]
    pub custom_fonts: Vec<CustomFontData>,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            scope: ExportScope::ActiveSheet,
            print_settings: PrintSettingsPolicy::HonorWorkbookSettings,

            page: PageOptions {
                paper_size: PaperSizePolicy::FromDocumentOr(PaperSize::A4),
                orientation: OrientationPolicy::FromDocumentOr(Orientation::Portrait),
                margins: MarginPolicy::FromDocumentOr(Margins::normal()),
                center_on_page: CenterOnPagePolicy::FromDocumentOr {
                    horizontal: false,
                    vertical: false,
                },
                first_page_number: FirstPageNumberPolicy::FromDocumentOrAuto,
                horizontal_alignment: PageHorizontalAlignment::Left,
                vertical_alignment: PageVerticalAlignment::Top,
            },

            layout: LayoutOptions {
                scaling: ScalingPolicy::FromDocumentOr(Scaling::FitToWidth {
                    pages_wide: 1,
                    pages_tall: None,
                }),
                manual_page_breaks: PageBreakPolicy::Honor,
                automatic_page_breaks: AutomaticPageBreakPolicy::Compute,
                page_order: PageOrderPolicy::FromDocumentOr(PageOrder::DownThenOver),
                allow_sheet_to_span_multiple_pages: true,
            },

            range: RangeOptions {
                print_area: PrintAreaPolicy::HonorDefinedPrintAreas,
                fallback_when_no_print_area: RangeFallback::UsedRange,
                multiple_print_areas: MultiplePrintAreasPolicy::SeparatePageBlocks,
                hidden_rows_columns: HiddenContentPolicy::Omit,
            },

            repeat: RepeatOptions {
                titles: RepeatTitlePolicy::HonorDefinedPrintTitles,
                frozen_panes: FrozenPanePrintPolicy::DoNotRepeat,
            },

            content: ContentOptions {
                gridlines: SheetBooleanPolicy::FromDocumentOr(true),
                row_column_headings: SheetBooleanPolicy::FromDocumentOr(false),
                charts: ObjectPrintPolicy::Render,
                images: ObjectPrintPolicy::Render,
                shapes: ObjectPrintPolicy::Render,
                comments: CommentPrintPolicy::FromDocumentOr(CommentPrintMode::None),
                cell_errors: CellErrorPolicy::FromDocumentOr(CellErrorMode::DisplayAsStored),
                formulas: FormulaPrintPolicy::CalculatedValues,
            },

            header_footer: HeaderFooterOptions {
                mode: HeaderFooterMode::None,
                scale_with_document: true,
                align_with_page_margins: true,
            },

            pdf: PdfDocumentOptions {
                profile: PdfProfile::StandardPdf,
                optimization: PdfOptimization::StandardQuality,
                font_embedding: FontEmbeddingPolicy::SubsetEmbeddableFonts,
                raster_image_dpi: 300,
                external_hyperlinks: HyperlinkPolicy::Preserve,
                internal_hyperlinks: HyperlinkPolicy::BestEffort,
                bookmarks: BookmarkPolicy::SheetBookmarksWhenExportingMultipleSheets,
                accessibility_tags: false,
                open_after_export: false,
            },
            custom_fonts: Vec::new(),
        }
    }
}

impl PdfExportOptions {
    /// Creates a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the export scope.
    pub fn with_scope(mut self, scope: ExportScope) -> Self {
        self.scope = scope;
        self
    }

    /// Sets the paper size.
    pub fn with_paper_size(mut self, paper_size: PaperSize) -> Self {
        self.page.paper_size = PaperSizePolicy::Fixed(paper_size);
        self
    }

    /// Sets orientation to landscape.
    pub fn landscape(mut self) -> Self {
        self.page.orientation = OrientationPolicy::Fixed(Orientation::Landscape);
        self
    }

    /// Sets orientation to portrait.
    pub fn portrait(mut self) -> Self {
        self.page.orientation = OrientationPolicy::Fixed(Orientation::Portrait);
        self
    }

    /// Sets page margins.
    pub fn with_margins(mut self, margins: Margins) -> Self {
        self.page.margins = MarginPolicy::Fixed(margins);
        self
    }

    /// Sets whether gridlines should be shown.
    pub fn with_gridlines(mut self, show: bool) -> Self {
        self.content.gridlines = SheetBooleanPolicy::Fixed(show);
        self
    }

    /// Sets whether row numbers and column letters should be shown.
    pub fn with_headings(mut self, show: bool) -> Self {
        self.content.row_column_headings = SheetBooleanPolicy::Fixed(show);
        self
    }

    /// Sets scaling to fit the sheet on a single page.
    pub fn fit_to_page(mut self) -> Self {
        self.layout.scaling = ScalingPolicy::Fixed(Scaling::FitSheetOnOnePage);
        self
    }

    /// Sets scaling to fit within a specified number of pages wide.
    pub fn fit_to_width(mut self, pages_wide: u16) -> Self {
        self.layout.scaling = ScalingPolicy::Fixed(Scaling::FitToWidth {
            pages_wide,
            pages_tall: None,
        });
        self
    }

    /// Sets scaling percentage (e.g. 100 for 100%, 80 for 80%).
    pub fn with_scale_percent(mut self, percent: u16) -> Self {
        self.layout.scaling = ScalingPolicy::Fixed(Scaling::Percent(percent));
        self
    }

    /// Sets centering on page.
    pub fn with_centering(mut self, horizontal: bool, vertical: bool) -> Self {
        self.page.center_on_page = CenterOnPagePolicy::Fixed {
            horizontal,
            vertical,
        };
        self
    }

    /// Enables headers and footers with a custom layout.
    pub fn with_header_footer(
        mut self,
        header: HeaderFooterLayout,
        footer: HeaderFooterLayout,
    ) -> Self {
        self.header_footer.mode = HeaderFooterMode::Explicit { header, footer };
        self
    }

    /// Enables bookmarks for sheets.
    pub fn with_bookmarks(mut self, policy: BookmarkPolicy) -> Self {
        self.pdf.bookmarks = policy;
        self
    }

    /// Repeats specific row ranges on every printed page (1-indexed).
    pub fn with_repeat_rows(mut self, start_row: i32, end_row: i32) -> Self {
        self.repeat.titles = RepeatTitlePolicy::Explicit {
            rows: Some(RowRange { start_row, end_row }),
            columns: None,
        };
        self
    }

    /// Sets whether frozen panes should be repeated as print titles.
    pub fn with_repeat_frozen_panes(mut self, repeat: bool) -> Self {
        self.repeat.frozen_panes = if repeat {
            FrozenPanePrintPolicy::RepeatAsPrintTitles
        } else {
            FrozenPanePrintPolicy::DoNotRepeat
        };
        self
    }

    /// Adds a custom font (TrueType or OpenType) to be embedded in the PDF.
    pub fn with_custom_font(mut self, name: impl Into<String>, data: Vec<u8>) -> Self {
        self.custom_fonts.push(CustomFontData {
            name: name.into(),
            data,
        });
        self
    }

    /// Sets the formula printing policy (CalculatedValues or FormulaText).
    pub fn with_formulas(mut self, policy: FormulaPrintPolicy) -> Self {
        self.content.formulas = policy;
        self
    }

    /// Sets the cell error display policy (DisplayAsStored, Dash, Blank, NotAvailable).
    pub fn with_cell_errors(mut self, mode: CellErrorMode) -> Self {
        self.content.cell_errors = CellErrorPolicy::Fixed(mode);
        self
    }

    /// Sets the scaling strategy.
    pub fn with_scaling(mut self, scaling: Scaling) -> Self {
        self.layout.scaling = ScalingPolicy::Fixed(scaling);
        self
    }

    /// Sets scaling to fit within a specified number of pages tall.
    pub fn fit_to_height(mut self, pages_tall: u16) -> Self {
        self.layout.scaling = ScalingPolicy::Fixed(Scaling::FitToHeight {
            pages_wide: None,
            pages_tall,
        });
        self
    }

    /// Sets page order (DownThenOver or OverThenDown).
    pub fn with_page_order(mut self, order: PageOrder) -> Self {
        self.layout.page_order = PageOrderPolicy::Fixed(order);
        self
    }

    /// Sets export scope to a specific rectangular cell selection on a sheet.
    pub fn with_selection(mut self, sheet: u32, range: CellRange) -> Self {
        self.scope = ExportScope::Selection { sheet, range };
        self
    }
}

/*
   Export Scope
*/

/// Defines which workbook content is included in the PDF export.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ExportScope {
    /// Exports the currently active sheet.
    ActiveSheet,
    /// Exports the specified sheet indices.
    SelectedSheets(Vec<u32>),
    /// Exports all visible sheets in the workbook.
    EntireWorkbook,
    /// Exports a specific rectangular selection on a sheet.
    Selection { sheet: u32, range: CellRange },
    /// Exports explicit ranges.
    ExplicitRanges(Vec<SheetRange>),
}

/// A 1-indexed rectangular cell range `[start_row, start_col, end_row, end_col]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CellRange {
    pub start_row: i32,
    pub start_col: i32,
    pub end_row: i32,
    pub end_col: i32,
}

impl CellRange {
    pub fn new(start_row: i32, start_col: i32, end_row: i32, end_col: i32) -> Self {
        Self {
            start_row: start_row.min(end_row),
            start_col: start_col.min(end_col),
            end_row: start_row.max(end_row),
            end_col: start_col.max(end_col),
        }
    }

    pub fn contains(&self, row: i32, col: i32) -> bool {
        row >= self.start_row && row <= self.end_row && col >= self.start_col && col <= self.end_col
    }

    pub fn num_rows(&self) -> i32 {
        (self.end_row - self.start_row + 1).max(0)
    }

    pub fn num_cols(&self) -> i32 {
        (self.end_col - self.start_col + 1).max(0)
    }
}

impl From<[i32; 4]> for CellRange {
    fn from(arr: [i32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<(i32, i32, i32, i32)> for CellRange {
    fn from(t: (i32, i32, i32, i32)) -> Self {
        Self::new(t.0, t.1, t.2, t.3)
    }
}

/// A cell range anchored to a specific sheet index.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SheetRange {
    pub sheet: u32,
    pub range: CellRange,
}

/// Row range (1-indexed, inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RowRange {
    pub start_row: i32,
    pub end_row: i32,
}

/// Column range (1-indexed, inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColumnRange {
    pub start_col: i32,
    pub end_col: i32,
}

/// Controls how workbook print metadata interacts with export options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PrintSettingsPolicy {
    /// Existing workbook settings are used if present; explicit options override them.
    HonorWorkbookSettings,
    /// Ignore persisted print metadata and use only the export configuration.
    IgnoreWorkbookSettings,
    /// Fail when relevant workbook print settings cannot be represented faithfully.
    RequireFaithfulWorkbookSettings,
}

/*
   Page Setup
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PageOptions {
    pub paper_size: PaperSizePolicy,
    pub orientation: OrientationPolicy,
    pub margins: MarginPolicy,
    pub center_on_page: CenterOnPagePolicy,
    pub first_page_number: FirstPageNumberPolicy,
    pub horizontal_alignment: PageHorizontalAlignment,
    pub vertical_alignment: PageVerticalAlignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PageHorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PageVerticalAlignment {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaperSizePolicy {
    FromDocumentOr(PaperSize),
    Fixed(PaperSize),
    Custom { width_mm: f32, height_mm: f32 },
}

/// Standard paper sizes
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
    A3,
    A5,
    Tabloid,
    Statement,
    Executive,
    Folio,
    B4,
    B5,
    Custom { width_mm: f32, height_mm: f32 },
}

impl PaperSize {
    /// Dimensions in millimeters (width, height) in portrait mode.
    pub fn dimensions_mm(&self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A5 => (148.0, 210.0),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Statement => (139.7, 215.9),
            PaperSize::Executive => (184.15, 266.7),
            PaperSize::Folio => (215.9, 330.2),
            PaperSize::B4 => (250.0, 353.0),
            PaperSize::B5 => (176.0, 250.0),
            PaperSize::Custom {
                width_mm,
                height_mm,
            } => (*width_mm, *height_mm),
        }
    }

    /// Dimensions in PDF points (1 pt = 1/72 inch, 25.4 mm = 72 pt) in portrait mode.
    pub fn dimensions_pt(&self) -> (f32, f32) {
        let (w_mm, h_mm) = self.dimensions_mm();
        const MM_TO_PT: f32 = 72.0 / 25.4;
        (w_mm * MM_TO_PT, h_mm * MM_TO_PT)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrientationPolicy {
    FromDocumentOr(Orientation),
    Fixed(Orientation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MarginPolicy {
    FromDocumentOr(Margins),
    Fixed(Margins),
}

/// Page margins in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Margins {
    pub top_mm: f32,
    pub right_mm: f32,
    pub bottom_mm: f32,
    pub left_mm: f32,
    pub header_mm: f32,
    pub footer_mm: f32,
}

impl Margins {
    /// Normal margins (top/bottom: 19.0mm, left/right: 18.0mm, header/footer: 8.0mm).
    pub fn normal() -> Self {
        Self {
            top_mm: 19.0,
            right_mm: 18.0,
            bottom_mm: 19.0,
            left_mm: 18.0,
            header_mm: 8.0,
            footer_mm: 8.0,
        }
    }

    /// Narrow margins (6.35mm all around, 8.0mm header/footer).
    pub fn narrow() -> Self {
        Self {
            top_mm: 6.35,
            right_mm: 6.35,
            bottom_mm: 6.35,
            left_mm: 6.35,
            header_mm: 8.0,
            footer_mm: 8.0,
        }
    }

    /// Wide margins (25.4mm all around, 12.7mm header/footer).
    pub fn wide() -> Self {
        Self {
            top_mm: 25.4,
            right_mm: 25.4,
            bottom_mm: 25.4,
            left_mm: 25.4,
            header_mm: 12.7,
            footer_mm: 12.7,
        }
    }

    /// Zero margins.
    pub fn none() -> Self {
        Self {
            top_mm: 0.0,
            right_mm: 0.0,
            bottom_mm: 0.0,
            left_mm: 0.0,
            header_mm: 0.0,
            footer_mm: 0.0,
        }
    }

    /// Custom margins in millimeters.
    pub fn custom(
        top_mm: f32,
        bottom_mm: f32,
        left_mm: f32,
        right_mm: f32,
        header_mm: f32,
        footer_mm: f32,
    ) -> Self {
        Self {
            top_mm,
            right_mm,
            bottom_mm,
            left_mm,
            header_mm,
            footer_mm,
        }
    }

    const MM_TO_PT: f32 = 72.0 / 25.4;

    pub fn top_pt(&self) -> f32 {
        self.top_mm * Self::MM_TO_PT
    }

    pub fn right_pt(&self) -> f32 {
        self.right_mm * Self::MM_TO_PT
    }

    pub fn bottom_pt(&self) -> f32 {
        self.bottom_mm * Self::MM_TO_PT
    }

    pub fn left_pt(&self) -> f32 {
        self.left_mm * Self::MM_TO_PT
    }

    pub fn header_pt(&self) -> f32 {
        self.header_mm * Self::MM_TO_PT
    }

    pub fn footer_pt(&self) -> f32 {
        self.footer_mm * Self::MM_TO_PT
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CenterOnPagePolicy {
    FromDocumentOr { horizontal: bool, vertical: bool },
    Fixed { horizontal: bool, vertical: bool },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FirstPageNumberPolicy {
    FromDocumentOrAuto,
    Auto,
    Number(u32),
}

/*
   Layout and Pagination
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LayoutOptions {
    pub scaling: ScalingPolicy,
    pub manual_page_breaks: PageBreakPolicy,
    pub automatic_page_breaks: AutomaticPageBreakPolicy,
    pub page_order: PageOrderPolicy,
    pub allow_sheet_to_span_multiple_pages: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScalingPolicy {
    FromDocumentOr(Scaling),
    Fixed(Scaling),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scaling {
    /// 100% normal size.
    ActualSize,
    /// Explicit scaling percentage (e.g. 80 for 80%).
    Percent(u16),
    /// Fits horizontally across `pages_wide` pages while allowing vertical pagination.
    FitToWidth {
        pages_wide: u16,
        pages_tall: Option<u16>,
    },
    /// Fits vertically across `pages_tall` pages.
    FitToHeight {
        pages_wide: Option<u16>,
        pages_tall: u16,
    },
    /// Fits across an exact grid of `pages_wide` x `pages_tall` pages.
    FitToPages { pages_wide: u16, pages_tall: u16 },
    /// Fits the entire sheet on a single page.
    FitSheetOnOnePage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PageBreakPolicy {
    Honor,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AutomaticPageBreakPolicy {
    Compute,
    Disable,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PageOrderPolicy {
    FromDocumentOr(PageOrder),
    Fixed(PageOrder),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PageOrder {
    DownThenOver,
    OverThenDown,
}

/*
   Ranges and Visible Content
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangeOptions {
    pub print_area: PrintAreaPolicy,
    pub fallback_when_no_print_area: RangeFallback,
    pub multiple_print_areas: MultiplePrintAreasPolicy,
    pub hidden_rows_columns: HiddenContentPolicy,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PrintAreaPolicy {
    HonorDefinedPrintAreas,
    IgnoreDefinedPrintAreas,
    OnlyExplicitRange(CellRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RangeFallback {
    UsedRange,
    EntireSheet,
    CurrentSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MultiplePrintAreasPolicy {
    SeparatePageBlocks,
    MergeIfAdjacent,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HiddenContentPolicy {
    Omit,
    Include,
}

/*
   Repeated Rows and Columns
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RepeatOptions {
    pub titles: RepeatTitlePolicy,
    pub frozen_panes: FrozenPanePrintPolicy,
}

impl Default for RepeatOptions {
    fn default() -> Self {
        Self {
            titles: RepeatTitlePolicy::HonorDefinedPrintTitles,
            frozen_panes: FrozenPanePrintPolicy::DoNotRepeat,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RepeatTitlePolicy {
    HonorDefinedPrintTitles,
    None,
    Explicit {
        rows: Option<RowRange>,
        columns: Option<ColumnRange>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FrozenPanePrintPolicy {
    DoNotRepeat,
    RepeatAsPrintTitles,
    PerAxis { rows: bool, columns: bool },
}

impl FrozenPanePrintPolicy {
    pub fn repeat_rows(&self) -> bool {
        match self {
            Self::RepeatAsPrintTitles => true,
            Self::PerAxis { rows, .. } => *rows,
            Self::DoNotRepeat => false,
        }
    }

    pub fn repeat_columns(&self) -> bool {
        match self {
            Self::RepeatAsPrintTitles => true,
            Self::PerAxis { columns, .. } => *columns,
            Self::DoNotRepeat => false,
        }
    }
}

/*
   Spreadsheet Content Rendering
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContentOptions {
    pub gridlines: SheetBooleanPolicy,
    pub row_column_headings: SheetBooleanPolicy,
    pub charts: ObjectPrintPolicy,
    pub images: ObjectPrintPolicy,
    pub shapes: ObjectPrintPolicy,
    pub comments: CommentPrintPolicy,
    pub cell_errors: CellErrorPolicy,
    pub formulas: FormulaPrintPolicy,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SheetBooleanPolicy {
    FromDocumentOr(bool),
    Fixed(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ObjectPrintPolicy {
    Render,
    Skip,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CommentPrintPolicy {
    FromDocumentOr(CommentPrintMode),
    Fixed(CommentPrintMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CommentPrintMode {
    None,
    AsDisplayed,
    AtEndOfSheet,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CellErrorPolicy {
    FromDocumentOr(CellErrorMode),
    Fixed(CellErrorMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CellErrorMode {
    DisplayAsStored,
    Blank,
    Dash,
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FormulaPrintPolicy {
    CalculatedValues,
    FormulaText,
}

/*
   Headers and Footers
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFooterOptions {
    pub mode: HeaderFooterMode,
    pub scale_with_document: bool,
    pub align_with_page_margins: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HeaderFooterMode {
    FromDocumentOrNone,
    None,
    Explicit {
        header: HeaderFooterLayout,
        footer: HeaderFooterLayout,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFooterLayout {
    pub left: HeaderFooterField,
    pub center: HeaderFooterField,
    pub right: HeaderFooterField,
}

impl HeaderFooterLayout {
    pub fn new(
        left: HeaderFooterField,
        center: HeaderFooterField,
        right: HeaderFooterField,
    ) -> Self {
        Self {
            left,
            center,
            right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HeaderFooterField {
    #[default]
    Empty,
    Text(String),
    PageNumber,
    PageNumberOfTotal,
    SheetName,
    WorkbookName,
    Date,
    Time,
    Custom(Vec<HeaderFooterToken>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HeaderFooterToken {
    Text(String),
    PageNumber,
    TotalPages,
    SheetName,
    WorkbookName,
    Date,
    Time,
    FontName(String),
    FontSize(u16),
    Bold,
    Italic,
    Underline,
}

/*
   PDF Document Behavior
*/

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfDocumentOptions {
    pub profile: PdfProfile,
    pub optimization: PdfOptimization,
    pub font_embedding: FontEmbeddingPolicy,
    pub raster_image_dpi: u16,
    pub external_hyperlinks: HyperlinkPolicy,
    pub internal_hyperlinks: HyperlinkPolicy,
    pub bookmarks: BookmarkPolicy,
    pub accessibility_tags: bool,
    pub open_after_export: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PdfProfile {
    StandardPdf,
    PdfA1,
    PdfA2,
    PdfUa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PdfOptimization {
    StandardQuality,
    MinimumSize,
    PrintQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FontEmbeddingPolicy {
    None,
    SubsetEmbeddableFonts,
    EmbedAllEmbeddableFonts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HyperlinkPolicy {
    Preserve,
    Drop,
    BestEffort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BookmarkPolicy {
    None,
    SheetBookmarksWhenExportingMultipleSheets,
    AlwaysBySheet,
}

/// Custom font data containing raw font bytes (TrueType or OpenType).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CustomFontData {
    /// Family name or PostScript name matching spreadsheet font (e.g. "Inter", "Roboto", "Calibri").
    pub name: String,
    /// Raw TrueType (.ttf) or OpenType (.otf) font file bytes.
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "deserialize_bytes_flexible")
    )]
    pub data: Vec<u8>,
}

impl CustomFontData {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

/// Decodes standard RFC 4648 base64 strings into raw byte vectors without external dependencies.
pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let clean = input.trim();
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in clean.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\r' | b'\n' | b' ' | b'\t' => continue,
            _ => return Err(format!("Invalid base64 character: {}", b as char)),
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

#[cfg(feature = "serde")]
fn deserialize_bytes_flexible<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct BytesVisitor;
    impl<'de> Visitor<'de> for BytesVisitor {
        type Value = Vec<u8>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a base64 string or a byte sequence")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            decode_base64(v).map_err(de::Error::custom)
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(byte) = seq.next_element()? {
                vec.push(byte);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_any(BytesVisitor)
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PdfExportOptionsPayload {
    /// Export scope: "CurrentSheet", "Workbook", or "SelectedCells"
    #[cfg_attr(feature = "serde", serde(default))]
    pub scope: Option<String>,
    /// Selected range string if scope is "SelectedCells" (e.g. "A1:D10")
    #[cfg_attr(feature = "serde", serde(default))]
    pub selection_range: Option<String>,
    /// Selected sheet index (0-based)
    #[cfg_attr(feature = "serde", serde(default))]
    pub sheet_index: Option<u32>,

    /// Paper size: "Letter", "Tabloid", "Legal", "Statement", "Executive", "Folio", "A3", "A4", "A5", "B4", "B5", "Custom"
    #[cfg_attr(feature = "serde", serde(default))]
    pub paper_size: Option<String>,
    /// Custom paper width in points (if paper_size is "Custom")
    #[cfg_attr(feature = "serde", serde(default))]
    pub custom_width_pt: Option<f32>,
    /// Custom paper height in points (if paper_size is "Custom")
    #[cfg_attr(feature = "serde", serde(default))]
    pub custom_height_pt: Option<f32>,

    /// Orientation: "Landscape" or "Portrait"
    #[cfg_attr(feature = "serde", serde(default))]
    pub orientation: Option<String>,

    /// Scaling mode: "Normal", "FitToWidth", "FitToHeight", "FitToPage", "Custom"
    #[cfg_attr(feature = "serde", serde(default))]
    pub scale: Option<String>,
    /// Custom scale percentage (e.g. 100)
    #[cfg_attr(feature = "serde", serde(default))]
    pub scale_percentage: Option<f32>,

    /// Margins: "Normal", "Narrow", "Wide", "Custom"
    #[cfg_attr(feature = "serde", serde(default))]
    pub margins: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub margin_top_pt: Option<f32>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub margin_right_pt: Option<f32>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub margin_bottom_pt: Option<f32>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub margin_left_pt: Option<f32>,

    /// Gridlines override
    #[cfg_attr(feature = "serde", serde(default))]
    pub show_grid_lines: Option<bool>,
    /// Row & column headings override
    #[cfg_attr(feature = "serde", serde(default))]
    pub show_headings: Option<bool>,
    /// Page order: "DownThenOver" or "OverThenDown"
    #[cfg_attr(feature = "serde", serde(default))]
    pub page_order: Option<String>,
    /// Horizontal alignment: "Left", "Center", "Right"
    #[cfg_attr(feature = "serde", serde(default))]
    pub horizontal_alignment: Option<String>,
    /// Vertical alignment: "Top", "Middle", "Bottom"
    #[cfg_attr(feature = "serde", serde(default))]
    pub vertical_alignment: Option<String>,

    /// Header / Footer options
    #[cfg_attr(feature = "serde", serde(default))]
    pub page_numbers: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub workbook_title: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sheet_name: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub current_date: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub current_time: Option<bool>,

    /// Frozen rows / columns
    #[cfg_attr(feature = "serde", serde(default))]
    pub repeat_frozen_rows: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub repeat_frozen_columns: Option<bool>,

    /// Custom font files to embed
    #[cfg_attr(feature = "serde", serde(default))]
    pub custom_fonts: Option<Vec<CustomFontData>>,
}

impl PdfExportOptionsPayload {
    pub fn into_options(self) -> PdfExportOptions {
        let mut opts = PdfExportOptions::default();

        // Scope
        if let Some(scope_str) = self.scope.as_deref() {
            match scope_str.to_lowercase().as_str() {
                "workbook" | "entireworkbook" => {
                    opts.scope = ExportScope::EntireWorkbook;
                }
                "selectedcells" | "selection" => {
                    let sheet = self.sheet_index.unwrap_or(0);
                    let range_str = self.selection_range.unwrap_or_else(|| "A1".to_string());
                    if let Some(range) = parse_a1_range(&range_str) {
                        opts.scope = ExportScope::Selection { sheet, range };
                    } else {
                        opts.scope = ExportScope::Selection {
                            sheet,
                            range: CellRange::new(0, 0, 0, 0),
                        };
                    }
                }
                _ => {
                    opts.scope = ExportScope::ActiveSheet;
                }
            }
        }

        // Paper size
        if let Some(size_str) = self.paper_size.as_deref() {
            let paper = match size_str.to_lowercase().as_str() {
                "letter" => PaperSize::Letter,
                "tabloid" => PaperSize::Tabloid,
                "legal" => PaperSize::Legal,
                "statement" => PaperSize::Statement,
                "executive" => PaperSize::Executive,
                "folio" => PaperSize::Folio,
                "a3" => PaperSize::A3,
                "a4" => PaperSize::A4,
                "a5" => PaperSize::A5,
                "b4" => PaperSize::B4,
                "b5" => PaperSize::B5,
                "custom" => PaperSize::Custom {
                    width_mm: self.custom_width_pt.unwrap_or(595.28) / 72.0 * 25.4,
                    height_mm: self.custom_height_pt.unwrap_or(841.89) / 72.0 * 25.4,
                },
                _ => PaperSize::A4,
            };
            opts.page.paper_size = PaperSizePolicy::Fixed(paper);
        }

        // Orientation
        if let Some(orient_str) = self.orientation.as_deref() {
            match orient_str.to_lowercase().as_str() {
                "landscape" => {
                    opts.page.orientation = OrientationPolicy::Fixed(Orientation::Landscape);
                }
                "portrait" => {
                    opts.page.orientation = OrientationPolicy::Fixed(Orientation::Portrait);
                }
                _ => {}
            }
        }

        // Scale
        if let Some(scale_str) = self.scale.as_deref() {
            let scaling = match scale_str.to_lowercase().as_str() {
                "normal" => Scaling::ActualSize,
                "fittowidth" => Scaling::FitToWidth {
                    pages_wide: 1,
                    pages_tall: None,
                },
                "fittoheight" => Scaling::FitToHeight {
                    pages_wide: None,
                    pages_tall: 1,
                },
                "fittopage" => Scaling::FitSheetOnOnePage,
                "custom" => {
                    let pct = self.scale_percentage.unwrap_or(100.0);
                    Scaling::Percent((pct as u16).max(10))
                }
                _ => Scaling::FitToWidth {
                    pages_wide: 1,
                    pages_tall: None,
                },
            };
            opts.layout.scaling = ScalingPolicy::Fixed(scaling);
        }

        // Margins
        if let Some(margin_str) = self.margins.as_deref() {
            let margins = match margin_str.to_lowercase().as_str() {
                "narrow" => Margins::narrow(),
                "wide" => Margins::wide(),
                "custom" => Margins {
                    top_mm: self.margin_top_pt.unwrap_or(54.0) / 72.0 * 25.4,
                    right_mm: self.margin_right_pt.unwrap_or(54.0) / 72.0 * 25.4,
                    bottom_mm: self.margin_bottom_pt.unwrap_or(54.0) / 72.0 * 25.4,
                    left_mm: self.margin_left_pt.unwrap_or(54.0) / 72.0 * 25.4,
                    header_mm: 8.0,
                    footer_mm: 8.0,
                },
                _ => Margins::normal(),
            };
            opts.page.margins = MarginPolicy::Fixed(margins);
        }

        // Grid lines
        if let Some(show) = self.show_grid_lines {
            opts.content.gridlines = SheetBooleanPolicy::Fixed(show);
        }

        // Row/col headings
        if let Some(headings) = self.show_headings {
            opts.content.row_column_headings = SheetBooleanPolicy::Fixed(headings);
        }

        // Page order
        if let Some(order_str) = self.page_order.as_deref() {
            match order_str.to_lowercase().as_str() {
                "overthendown" => {
                    opts.layout.page_order = PageOrderPolicy::Fixed(PageOrder::OverThenDown);
                }
                "downthenover" => {
                    opts.layout.page_order = PageOrderPolicy::Fixed(PageOrder::DownThenOver);
                }
                _ => {}
            }
        }

        // Alignment (Center on page & general page alignment)
        let center_h = self
            .horizontal_alignment
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("center"))
            .unwrap_or(false);
        let center_v = self
            .vertical_alignment
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("middle") || s.eq_ignore_ascii_case("center"))
            .unwrap_or(false);
        if center_h || center_v {
            opts.page.center_on_page = CenterOnPagePolicy::Fixed {
                horizontal: center_h,
                vertical: center_v,
            };
        }
        opts.page.horizontal_alignment = match self
            .horizontal_alignment
            .as_deref()
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("center") => PageHorizontalAlignment::Center,
            Some("right") => PageHorizontalAlignment::Right,
            _ => {
                if center_h {
                    PageHorizontalAlignment::Center
                } else {
                    PageHorizontalAlignment::Left
                }
            }
        };
        opts.page.vertical_alignment = match self
            .vertical_alignment
            .as_deref()
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("middle") | Some("center") => PageVerticalAlignment::Middle,
            Some("bottom") => PageVerticalAlignment::Bottom,
            _ => {
                if center_v {
                    PageVerticalAlignment::Middle
                } else {
                    PageVerticalAlignment::Top
                }
            }
        };

        // Headers and Footers
        let has_hf = self.page_numbers.unwrap_or(false)
            || self.workbook_title.unwrap_or(false)
            || self.sheet_name.unwrap_or(false)
            || self.current_date.unwrap_or(false)
            || self.current_time.unwrap_or(false);

        if has_hf {
            let mut header = HeaderFooterLayout::default();
            let mut footer = HeaderFooterLayout::default();

            if self.sheet_name.unwrap_or(false) {
                header.left = HeaderFooterField::SheetName;
            }
            if self.workbook_title.unwrap_or(false) {
                header.center = HeaderFooterField::WorkbookName;
            }
            if self.current_date.unwrap_or(false) {
                footer.left = HeaderFooterField::Date;
            }
            if self.current_time.unwrap_or(false) {
                footer.center = HeaderFooterField::Time;
            }
            if self.page_numbers.unwrap_or(false) {
                footer.right = HeaderFooterField::PageNumberOfTotal;
            }

            opts.header_footer.mode = HeaderFooterMode::Explicit { header, footer };
        }

        // Repeat frozen panes
        let rep_rows = self.repeat_frozen_rows.unwrap_or(false);
        let rep_cols = self.repeat_frozen_columns.unwrap_or(false);
        opts.repeat.frozen_panes = match (rep_rows, rep_cols) {
            (true, true) => FrozenPanePrintPolicy::RepeatAsPrintTitles,
            (false, false) => FrozenPanePrintPolicy::DoNotRepeat,
            (r, c) => FrozenPanePrintPolicy::PerAxis {
                rows: r,
                columns: c,
            },
        };

        // Custom fonts
        if let Some(fonts) = self.custom_fonts {
            opts.custom_fonts = fonts;
        }

        opts
    }
}

fn col_str_to_num(col: &str) -> Option<i32> {
    let mut num = 0i32;
    for ch in col.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        num = num * 26 + (ch.to_ascii_uppercase() as i32 - 'A' as i32 + 1);
    }
    if num > 0 {
        Some(num)
    } else {
        None
    }
}

fn parse_a1_cell(s: &str) -> Option<(i32, i32)> {
    let trimmed = s.trim().trim_start_matches('$');
    let col_end = trimmed.find(|c: char| c.is_ascii_digit())?;
    let col_str = &trimmed[..col_end];
    let row_str = trimmed[col_end..].trim_start_matches('$');
    let col = col_str_to_num(col_str)?;
    let row = row_str.parse::<i32>().ok()?;
    if !(1..=16_384).contains(&col) || !(1..=1_048_576).contains(&row) {
        return None;
    }
    Some((row, col))
}

fn parse_a1_range(s: &str) -> Option<CellRange> {
    let clean = s.split('!').next_back().unwrap_or(s).trim();
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() == 1 {
        let (r, c) = parse_a1_cell(parts[0])?;
        Some(CellRange::new(r, c, r, c))
    } else if parts.len() == 2 {
        let (r1, c1) = parse_a1_cell(parts[0])?;
        let (r2, c2) = parse_a1_cell(parts[1])?;
        Some(CellRange::new(r1, c1, r2, c2))
    } else {
        None
    }
}
