use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::expressions::token::Error;
use crate::number_format::{
    DEFAULT_NUM_FMTS, SHORT_DATE_FMT_ID, SHORT_DATE_TIME_FMT_ID,
};

fn default_as_false() -> bool {
    false
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub application: String,
    pub app_version: String,
    pub creator: String,
    pub last_modified_by: String,
    pub created: String,       // "2020-08-06T21:20:53Z",
    pub last_modified: String, //"2020-11-20T16:24:35"
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct WorkbookSettings {
    pub tz: String,
    pub locale: String,
}

/// A Workbook View tracks of the selected sheet for each view
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct WorkbookView {
    /// The index of the currently selected sheet.
    pub sheet: u32,
    /// The current width of the window
    pub window_width: i64,
    /// The current height of the window
    pub window_height: i64,
}

/// An internal representation of an IronCalc Workbook
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Workbook {
    pub shared_strings: Vec<String>,
    pub defined_names: Vec<DefinedName>,
    pub worksheets: Vec<Worksheet>,
    pub styles: Styles,
    pub name: String,
    pub settings: WorkbookSettings,
    pub metadata: Metadata,
    pub tables: HashMap<String, Table>,
    pub views: HashMap<u32, WorkbookView>,
}

/// A defined name. The `sheet_id` is the sheet index in case the name is local
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DefinedName {
    pub name: String,
    pub formula: String,
    pub sheet_id: Option<u32>,
}

/// * state:
///   18.18.68 ST_SheetState (Sheet Visibility Types)
///   hidden, veryHidden, visible
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum SheetState {
    Visible,
    Hidden,
    VeryHidden,
}

impl Display for SheetState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SheetState::Visible => write!(formatter, "visible"),
            SheetState::Hidden => write!(formatter, "hidden"),
            SheetState::VeryHidden => write!(formatter, "veryHidden"),
        }
    }
}

/// Represents the state of the worksheet as seen by the user. This includes
/// details such as the currently selected cell, the visible range, and the
/// position of the viewport.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct WorksheetView {
    /// The row index of the currently selected cell.
    pub row: i32,
    /// The column index of the currently selected cell.
    pub column: i32,
    /// The selected range in the worksheet, specified as [start_row, start_column, end_row, end_column].
    pub range: [i32; 4],
    /// The row index of the topmost visible cell in the worksheet view.
    pub top_row: i32,
    /// The column index of the leftmost visible cell in the worksheet view.
    pub left_column: i32,
}

/// Internal representation of a worksheet Excel object
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Worksheet {
    pub dimension: String,
    pub cols: Vec<Col>,
    pub rows: Vec<Row>,
    pub name: String,
    pub sheet_data: SheetData,
    pub shared_formulas: Vec<String>,
    pub sheet_id: u32,
    pub state: SheetState,
    pub color: Option<String>,
    pub merge_cells: Vec<String>,
    pub comments: Vec<Comment>,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    pub views: HashMap<u32, WorksheetView>,
    /// Whether or not to show the grid lines in the worksheet
    pub show_grid_lines: bool,
}

/// Internal representation of Excel's sheet_data
/// It is row first and because of this all of our API's should be row first
pub type SheetData = HashMap<i32, HashMap<i32, Cell>>;

// ECMA-376-1:2016 section 18.3.1.73
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Row {
    /// Row index
    pub r: i32,
    pub height: f64,
    pub custom_format: bool,
    pub custom_height: bool,
    pub s: i32,
    pub hidden: bool,
}

// ECMA-376-1:2016 section 18.3.1.13
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct Col {
    // Column definitions are defined on ranges, unlike rows which store unique, per-row entries.
    /// First column affected by this record. Settings apply to column in \[min, max\] range.
    pub min: i32,
    /// Last column affected by this record. Settings apply to column in \[min, max\] range.
    pub max: i32,
    pub width: f64,
    pub custom_width: bool,
    pub hidden: bool,
    pub style: Option<i32>,
}

/// Cell type enum matching Excel TYPE() function values.
#[derive(Debug, Eq, PartialEq)]
pub enum CellType {
    Number = 1,
    Text = 2,
    LogicalValue = 4,
    ErrorValue = 16,
    Array = 64,
    CompoundData = 128,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Cell {
    EmptyCell {
        s: i32,
    },

    BooleanCell {
        v: bool,
        s: i32,
    },

    NumberCell {
        v: f64,
        s: i32,
    },
    // Maybe we should not have this type. In Excel this is just a string
    ErrorCell {
        ei: Error,
        s: i32,
    },
    // Always a shared string
    SharedString {
        si: i32,
        s: i32,
    },
    // Non evaluated Formula
    CellFormula {
        f: i32,
        s: i32,
    },

    CellFormulaBoolean {
        f: i32,
        v: bool,
        s: i32,
    },

    CellFormulaNumber {
        f: i32,
        v: f64,
        s: i32,
    },
    // always inline string
    CellFormulaString {
        f: i32,
        v: String,
        s: i32,
    },

    CellFormulaError {
        f: i32,
        ei: Error,
        s: i32,
        // Origin: Sheet3!C4
        o: String,
        // Error Message: "Not implemented function"
        m: String,
    },
    // TODO: Array formulas
}

impl Default for Cell {
    fn default() -> Self {
        Cell::EmptyCell { s: 0 }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Comment {
    pub text: String,
    pub author_name: String,
    pub author_id: Option<String>,
    pub cell_ref: String,
}

// ECMA-376-1:2016 section 18.5.1.2
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Table {
    pub name: String,
    pub display_name: String,
    pub sheet_name: String,
    pub reference: String,
    pub totals_row_count: u32,
    pub header_row_count: u32,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub columns: Vec<TableColumn>,
    pub style_info: TableStyleInfo,
    pub has_filters: bool,
}

// totals_row_label vs totals_row_function might be mutually exclusive. Use an enum?
// the totals_row_function is an enum not String methinks
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct TableColumn {
    pub id: u32,
    pub name: String,
    pub totals_row_label: Option<String>,
    pub header_row_dxf_id: Option<u32>,
    pub data_dxf_id: Option<u32>,
    pub totals_row_dxf_id: Option<u32>,
    pub totals_row_function: Option<String>,
}

impl Default for TableColumn {
    fn default() -> Self {
        TableColumn {
            id: 0,
            name: "Column".to_string(),
            totals_row_label: None,
            totals_row_function: None,
            data_dxf_id: None,
            header_row_dxf_id: None,
            totals_row_dxf_id: None,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct TableStyleInfo {
    pub name: Option<String>,
    pub show_first_column: bool,
    pub show_last_column: bool,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Styles {
    pub num_fmts: Vec<NumFmt>,
    pub fonts: Vec<Font>,
    pub fills: Vec<Fill>,
    pub borders: Vec<Border>,
    pub cell_style_xfs: Vec<CellStyleXfs>,
    pub cell_xfs: Vec<CellXfs>,
    pub cell_styles: Vec<CellStyles>,
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            num_fmts: vec![],
            fonts: vec![Default::default()],
            fills: vec![
                Default::default(),
                Fill {
                    pattern_type: "gray125".to_string(),
                    fg_color: None,
                    bg_color: None,
                },
            ],
            borders: vec![Default::default()],
            cell_style_xfs: vec![Default::default()],
            cell_xfs: vec![Default::default()],
            cell_styles: vec![Default::default()],
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    pub num_fmt: NumFmt,
    pub fill: Fill,
    pub font: Font,
    pub border: Border,
    pub quote_prefix: bool,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct NumFmt {
    pub num_fmt_id: i32,
    pub format_code: String,
}

// Custom deserializer so that workbooks serialized before the `String` →
// `NumFmt` migration can still be loaded.  Old JSON has `"num_fmt": "mm/dd/yy"`;
// new JSON has `"num_fmt": {"num_fmt_id": 14, "format_code": "mm/dd/yy"}`.
impl<'de> Deserialize<'de> for NumFmt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct NumFmtVisitor;

        impl<'de> Visitor<'de> for NumFmtVisitor {
            type Value = NumFmt;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a format-code string (legacy) or a NumFmt object")
            }

            // Legacy path: `num_fmt` was serialized as a plain format-code string.
            fn visit_str<E: de::Error>(self, value: &str) -> Result<NumFmt, E> {
                Ok(NumFmt::from_format_code(value))
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<NumFmt, M::Error> {
                let mut num_fmt_id: Option<i32> = None;
                let mut format_code: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "num_fmt_id" => num_fmt_id = Some(map.next_value()?),
                        "format_code" => format_code = Some(map.next_value()?),
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
                    }
                }
                let format_code = format_code
                    .ok_or_else(|| de::Error::missing_field("format_code"))?;
                // Re-derive num_fmt_id from format_code so the two fields are always
                // consistent.  If the incoming id disagrees (e.g. a hand-edited file),
                // format_code is treated as the source of truth.
                let derived = NumFmt::from_format_code(&format_code);
                // Accept the stored id only if it matches what we'd derive — this lets
                // custom IDs (≥ 164) round-trip correctly when both fields are present.
                let stored_id = num_fmt_id.unwrap_or(derived.num_fmt_id);
                let fmt = if stored_id == derived.num_fmt_id || derived.num_fmt_id == -1 {
                    NumFmt { num_fmt_id: stored_id, format_code }
                } else {
                    // Mismatch — trust format_code, discard stale id.
                    derived
                };
                Ok(fmt)
            }
        }

        deserializer.deserialize_any(NumFmtVisitor)
    }
}

// Emit only the format_code string.  The Deserialize impl handles both this
// string form and the legacy struct form, so round-trips are lossless for
// built-in IDs (re-derived by from_format_code) and for custom IDs (sentinel
// -1, re-resolved at persist time by get_or_register).
impl Serialize for NumFmt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.format_code)
    }
}

impl Default for NumFmt {
    fn default() -> Self {
        NumFmt {
            num_fmt_id: 0,
            format_code: "general".to_string(),
        }
    }
}

impl NumFmt {
    /// Construct a `NumFmt` from a known `num_fmt_id` and `format_code`.
    ///
    /// Prefer [`NumFmt::from_id`], [`NumFmt::from_format_code`], or
    /// [`NumFmt::get_or_register`] when working within the engine.  This
    /// constructor exists for callers (e.g. XLSX import) that already hold a
    /// validated id/code pair from an external source and need to build the
    /// struct directly.
    pub fn new(num_fmt_id: i32, format_code: String) -> Self {
        NumFmt {
            num_fmt_id,
            format_code,
        }
    }

    /// ECMA-376 numFmtId for the locale-derived short date (e.g. "m/d/yy" in en-US).
    pub(crate) const SHORT_DATE_ID: i32 = SHORT_DATE_FMT_ID;
    /// ECMA-376 numFmtId for the locale-derived short date+time (e.g. "m/d/yy h:mm").
    pub(crate) const SHORT_DATETIME_ID: i32 = SHORT_DATE_TIME_FMT_ID;
    
    /// The ECMA-376 built-in format strings, indexed by numFmtId.
    ///
    /// Prefer this accessor over importing `DEFAULT_NUM_FMTS` directly —
    /// callers get `.iter()`, `.len()`, and `.get(i)` without depending on
    /// the private constant.
    fn builtins() -> &'static [&'static str] {
        DEFAULT_NUM_FMTS
    }

    /// Return the ECMA-376 built-in `numFmtId` for `code`, or `None` if the
    /// code is not in the built-in table.
    fn builtin_id(code: &str) -> Option<i32> {
        Self::builtins()
            .iter()
            .position(|&s| s == code)
            .map(|i| i as i32)
    }

    /// Return `true` if `id` is a valid ECMA-376 built-in numFmtId.
    fn is_builtin_id(id: i32) -> bool {
        usize::try_from(id).is_ok_and(|i| i < Self::builtins().len())
    }
    
    /// Return `true` if `id` is one of the two locale-derived format IDs (14 or 22).
    pub(crate) fn is_locale_date_id(id: i32) -> bool {
        id == Self::SHORT_DATE_ID || id == Self::SHORT_DATETIME_ID
    }

    /// Return `true` if `id` is either a built-in ECMA-376 ID or registered
    /// in `custom_fmts`.  Use this to validate an ID before storing it in a
    /// `CellXfs` entry.
    pub(crate) fn is_known_id(id: i32, custom_fmts: &[NumFmt]) -> bool {
        Self::is_builtin_id(id) || custom_fmts.iter().any(|f| f.num_fmt_id == id)
    }

    /// Build a `NumFmt` from a known `num_fmt_id`.
    ///
    /// Looks up the format code from the workbook's custom list first,
    /// then falls back to the ECMA-376 built-in table. Unknown IDs resolve
    /// to General (ID 0).
    pub fn from_id(id: i32, custom_fmts: &[NumFmt]) -> Self {
        if let Some(fmt) = custom_fmts.iter().find(|f| f.num_fmt_id == id) {
            return fmt.clone();
        }
        // Clamp out-of-range / negative IDs to 0 (General).
        let resolved = usize::try_from(id)
            .ok()
            .filter(|&i| i < Self::builtins().len())
            .map(|i| i as i32)
            .unwrap_or(0);
        // IDs in the ECMA-376 gap (49–163) or any positive ID not registered in
        // custom_fmts will silently fall back to General.  Flag this in debug
        // builds — it most likely indicates a misconfigured xlsx import.
        debug_assert!(
            id < 0 || resolved == id,
            "num_fmt_id {id} is unknown (not a built-in ECMA-376 ID and not in custom_fmts); \
             silently falling back to General (0)"
        );
        NumFmt {
            num_fmt_id: resolved,
            format_code: Self::format_code_for_id(resolved, &[]).to_string(),
        }
    }

    /// Build a `NumFmt` from a format code string.
    ///
    /// For ECMA-376 built-in codes the canonical ID is resolved immediately.
    /// For custom codes (not in the built-in table) `num_fmt_id` is `-1` — an
    /// unambiguous sentinel distinct from all valid IDs (which are ≥ 0).
    /// `Styles::get_style_index_or_create` re-derives the real ID from
    /// `format_code` at persist time, and `Styles::get_style_index` compares
    /// styles by `format_code`, so deduplication is correct regardless.
    pub fn from_format_code(code: &str) -> Self {
        let num_fmt_id = Self::builtin_id(code).unwrap_or(-1);
        NumFmt {
            num_fmt_id,
            format_code: code.to_string(),
        }
    }

    /// Resolve a `num_fmt_id` to its format code string.
    ///
    /// Checks the ECMA-376 built-in table first (O(1) bounds check), then
    /// falls back to a linear scan of `custom_fmts` (workbook-specific entries).
    /// Unknown or negative IDs — including the `-1` sentinel used by
    /// `from_format_code` — return `"general"` (ID 0).
    pub(crate) fn format_code_for_id<'a>(id: i32, custom_fmts: &'a [NumFmt]) -> &'a str {
        // Fast path: most IDs are ECMA-376 builtins.  A single bounds check
        // avoids the custom_fmts scan for the common case.
        if let Some(i) = usize::try_from(id).ok().filter(|&i| i < Self::builtins().len()) {
            return Self::builtins()[i];
        }
        custom_fmts
            .iter()
            .find(|f| f.num_fmt_id == id)
            .map(|f| f.format_code.as_str())
            .unwrap_or(Self::builtins()[0])
    }

    /// Covers functions `get_num_fmt()` and `get_new_num_fmt_index()` previously in file number_format.rs
    /// `Styles` methods .
    pub fn get_or_register(code: &str, num_fmts: &mut Vec<NumFmt>) -> Self {
        if let Some(id) = Self::builtin_id(code) {
            return NumFmt {
                num_fmt_id: id,
                format_code: code.to_string(),
            };
        }
        if let Some(existing) = num_fmts.iter().find(|f| f.format_code == code) {
            return existing.clone();
        }
        // Find the lowest custom ID (≥ built-in table length) not already in use.
        let mut new_id = Self::builtins().len() as i32;
        while num_fmts.iter().any(|f| f.num_fmt_id == new_id) {
            new_id += 1;
        }
        let fmt = NumFmt {
            num_fmt_id: new_id,
            format_code: code.to_string(),
        };
        num_fmts.push(fmt.clone());
        fmt
    }
}

// ST_FontScheme simple type (§18.18.33).
// Usually major fonts are used for styles like headings,
// and minor fonts are used for body and paragraph text.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum FontScheme {
    #[default]
    Minor,
    Major,
    None,
}

impl Display for FontScheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FontScheme::Minor => write!(formatter, "minor"),
            FontScheme::Major => write!(formatter, "major"),
            FontScheme::None => write!(formatter, "none"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Font {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub strike: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub u: bool, // seems that Excel supports a bit more - double underline / account underline etc.
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub b: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub i: bool,
    pub sz: i32,
    pub color: Option<String>,
    pub name: String,
    // This is the font family fallback
    // 1 -> serif
    // 2 -> sans serif
    // 3 -> monospaced
    // ...
    pub family: i32,
    pub scheme: FontScheme,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            strike: false,
            u: false,
            b: false,
            i: false,
            sz: 13,
            color: Some("#000000".to_string()),
            name: "Calibri".to_string(),
            family: 2,
            scheme: FontScheme::Minor,
        }
    }
}

// TODO: Maybe use an enum for the pattern_type values here?
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Fill {
    pub pattern_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
}

impl Default for Fill {
    fn default() -> Self {
        Fill {
            pattern_type: "none".to_string(),
            fg_color: Default::default(),
            bg_color: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum HorizontalAlignment {
    Center,
    CenterContinuous,
    Distributed,
    Fill,
    #[default]
    General,
    Justify,
    Left,
    Right,
}

// Note that alignment in "General" depends on type

impl HorizontalAlignment {
    fn is_default(&self) -> bool {
        self == &HorizontalAlignment::default()
    }
}

// FIXME: Is there a way to generate this automatically?
impl Display for HorizontalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HorizontalAlignment::Center => write!(formatter, "center"),
            HorizontalAlignment::CenterContinuous => write!(formatter, "centerContinuous"),
            HorizontalAlignment::Distributed => write!(formatter, "distributed"),
            HorizontalAlignment::Fill => write!(formatter, "fill"),
            HorizontalAlignment::General => write!(formatter, "general"),
            HorizontalAlignment::Justify => write!(formatter, "justify"),
            HorizontalAlignment::Left => write!(formatter, "left"),
            HorizontalAlignment::Right => write!(formatter, "right"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum VerticalAlignment {
    #[default]
    Bottom,
    Center,
    Distributed,
    Justify,
    Top,
}

impl VerticalAlignment {
    fn is_default(&self) -> bool {
        self == &VerticalAlignment::default()
    }
}

impl Display for VerticalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VerticalAlignment::Bottom => write!(formatter, "bottom"),
            VerticalAlignment::Center => write!(formatter, "center"),
            VerticalAlignment::Distributed => write!(formatter, "distributed"),
            VerticalAlignment::Justify => write!(formatter, "justify"),
            VerticalAlignment::Top => write!(formatter, "top"),
        }
    }
}

// 1762
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct Alignment {
    #[serde(default)]
    #[serde(skip_serializing_if = "HorizontalAlignment::is_default")]
    pub horizontal: HorizontalAlignment,
    #[serde(skip_serializing_if = "VerticalAlignment::is_default")]
    #[serde(default)]
    pub vertical: VerticalAlignment,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub wrap_text: bool,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CellStyleXfs {
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    pub apply_number_format: bool,
    pub apply_border: bool,
    pub apply_alignment: bool,
    pub apply_protection: bool,
    pub apply_font: bool,
    pub apply_fill: bool,
}

impl Default for CellStyleXfs {
    fn default() -> Self {
        CellStyleXfs {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
            apply_number_format: true,
            apply_border: true,
            apply_alignment: true,
            apply_protection: true,
            apply_font: true,
            apply_fill: true,
        }
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct CellXfs {
    pub xf_id: i32,
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    pub apply_number_format: bool,
    pub apply_border: bool,
    pub apply_alignment: bool,
    pub apply_protection: bool,
    pub apply_font: bool,
    pub apply_fill: bool,
    pub quote_prefix: bool,
    pub alignment: Option<Alignment>,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct CellStyles {
    pub name: String,
    pub xf_id: i32,
    pub builtin_id: i32,
}

impl Default for CellStyles {
    fn default() -> Self {
        CellStyles {
            name: "normal".to_string(),
            xf_id: 0,
            builtin_id: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, PartialOrd, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    Thin,
    Medium,
    Thick,
    Double,
    Dotted,
    SlantDashDot,
    MediumDashed,
    MediumDashDotDot,
    MediumDashDot,
}

impl Display for BorderStyle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BorderStyle::Thin => write!(formatter, "thin"),
            BorderStyle::Thick => write!(formatter, "thick"),
            BorderStyle::SlantDashDot => write!(formatter, "slantdashdot"),
            BorderStyle::MediumDashed => write!(formatter, "mediumdashed"),
            BorderStyle::MediumDashDotDot => write!(formatter, "mediumdashdotdot"),
            BorderStyle::MediumDashDot => write!(formatter, "mediumdashdot"),
            BorderStyle::Medium => write!(formatter, "medium"),
            BorderStyle::Double => write!(formatter, "double"),
            BorderStyle::Dotted => write!(formatter, "dotted"),
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct BorderItem {
    pub style: BorderStyle,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, Default)]
pub struct Border {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_up: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_down: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagonal: Option<BorderItem>,
}

/// Information need to show a sheet tab in the UI
/// The color is serialized only if it is not Color::None
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SheetProperties {
    pub name: String,
    pub state: String,
    pub sheet_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
