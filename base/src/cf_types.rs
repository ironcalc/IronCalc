use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::{Dxf, Style};

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum ValueOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    NotEqual,
    Between,
    NotBetween,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum TextOperator {
    Contains, // NOT(ISERROR(SEARCH(value,A1)))
    DoesNotContain,
    BeginsWith,
    EndsWith, // RIGHT(E1,LEN(value))=
    Equals,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum PeriodType {
    Between,
    NotBetween,
    Yesterday,
    Today,
    Tomorrow,
    Last7Days,
    Next7Days,
    LastWeek,
    ThisWeek,
    NextWeek,
    LastMonth,
    ThisMonth,
    NextMonth,
    LastYear,
    ThisYear,
    NextYear,
}

// These are the threshold definitions for icon set and color scale conditional formatting.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Cfvo {
    Min,
    Max,
    Number(f64),
    Percent(f64),
    Percentile(f64),
    Formula(String),
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Icon {
    ArrowUp,
    ArrowRight,
    ArrowDown,
    ArrowAngleUp,
    ArrowAngleDown,
    Circle,
    TriangleUp,
    TriangleDown,
    FlatRectangle,
    Rhombus,
    Flag,
    Check,
    Cross,
    Exclamation,
    Star,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct ColorScaleThreshold {
    pub cfvo: Cfvo,
    pub color: String,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct IconThreshold {
    pub icon: Icon,
    pub cfvo: Cfvo,
    pub color: String,
    // If true, the threshold is "strict":
    // the icon applies only if the value is strictly greater than (for ">=" operator) the threshold value.
    pub is_strict: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum CfRule {
    ColorScale {
        thresholds: Vec<ColorScaleThreshold>,
    },
    CellIs {
        operator: ValueOperator,
        formula: String,
        // Only present for Between and NotBetween operators
        formula2: Option<String>,
        dxf_id: u32,
    },
    Formula {
        formula: String,
        dxf_id: u32,
    },
    Text {
        operator: TextOperator,
        value: String,
        dxf_id: u32,
    },
    TimePeriod {
        time_period: PeriodType,
        date1: Option<String>,
        date2: Option<String>,
        dxf_id: u32,
    },
    DuplicateValues {
        dxf_id: u32,
    },
    UniqueValues {
        dxf_id: u32,
    },
    Blanks {
        dxf_id: u32,
    },
    NotBlanks {
        dxf_id: u32,
    },
    Errors {
        dxf_id: u32,
    },
    NoErrors {
        dxf_id: u32,
    },
    AboveAverage {
        dxf_id: u32,
    },
    BelowAverage {
        dxf_id: u32,
    },
    Top10 {
        rank: u32,
        percent: bool,
        dxf_id: u32,
    },
    Bottom10 {
        rank: u32,
        percent: bool,
        dxf_id: u32,
    },
    DataBar {
        // If Options are None, they default to Automatic:
        // min is Min(0, values in the range), max is Max(0, values in the range).
        min: Option<Cfvo>,
        max: Option<Cfvo>,
        positive_color: String,
        negative_color: String,
        is_gradient: bool,
        // missing:
        // has_border: bool,
        // border_color_positive: String,
        // border_color_negative: String,
        // axis_position: DataBarAxisPosition, (automatic, none, cell_midpoint)
        // axis_color: String,
        show_value: bool,
    },
    IconSet {
        // The icon thresholds from highest to lowest value.
        // In a set with 5 icons, we have 4 thresholds:
        //  * the first applies to values >= threshold1,
        //  * the second applies to values >= threshold2 and < threshold1,
        //  * the third applies to values >= threshold3 and < threshold2,
        //  * the fourth applies to values >= threshold4 and < threshold3,
        //  * the fifth applies to values < threshold4.
        thresholds: Vec<IconThreshold>,
        show_value: bool,
    },
    IconRating {
        // In a rating an icon is repeated `max` times, with `count` of them filled in.
        icon: Icon,
        color: String,
        // thresholds from highest to lowest value. There are `max-1` thresholds.
        // (threshold, is_strict)
        thresholds: Vec<(Cfvo, bool)>,
        show_value: bool,
    },
}

/// User-facing input type for creating or updating a CF rule.
/// Mirrors `CfRule` but dxf-based variants carry an optional `Dxf` format
/// instead of a `dxf_id` index.  Non-dxf variants (ColorScale, DataBar,
/// IconSet, IconRating) are identical to their `CfRule` counterparts.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum CfRuleInput {
    ColorScale {
        thresholds: Vec<ColorScaleThreshold>,
    },
    CellIs {
        operator: ValueOperator,
        formula: String,
        formula2: Option<String>,
        format: Dxf,
    },
    Text {
        operator: TextOperator,
        value: String,
        format: Dxf,
    },
    Formula {
        formula: String,
        format: Dxf,
    },
    TimePeriod {
        time_period: PeriodType,
        date1: Option<String>,
        date2: Option<String>,
        format: Dxf,
    },
    DuplicateValues {
        format: Dxf,
    },
    UniqueValues {
        format: Dxf,
    },
    Blanks {
        format: Dxf,
    },
    NotBlanks {
        format: Dxf,
    },
    Errors {
        format: Dxf,
    },
    NoErrors {
        format: Dxf,
    },
    AboveAverage {
        format: Dxf,
    },
    BelowAverage {
        format: Dxf,
    },
    Top10 {
        rank: u32,
        percent: bool,
        format: Dxf,
    },
    Bottom10 {
        rank: u32,
        percent: bool,
        format: Dxf,
    },
    DataBar {
        min: Option<Cfvo>,
        max: Option<Cfvo>,
        positive_color: String,
        negative_color: String,
        is_gradient: bool,
        show_value: bool,
    },
    IconSet {
        thresholds: Vec<IconThreshold>,
        show_value: bool,
    },
    IconRating {
        icon: Icon,
        color: String,
        thresholds: Vec<(Cfvo, bool)>,
        show_value: bool,
    },
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct ConditionalFormatting {
    pub range: String,
    pub cf_rule: CfRule,
    pub priority: u32,
    pub stop_if_true: bool,
}

// ---------------------------------------------------------------------------
// Evaluated CF result for a single cell (transient, not stored in Workbook).
// ---------------------------------------------------------------------------

/// The winning CF result for a cell, stored in Model::cf_cache after evaluate().
#[derive(Clone, Debug)]
pub(crate) enum CfCellResult {
    /// A dxf-based rule matched; dxf_id indexes into styles.dxfs.
    Dxf(u32),
    /// Color scale: the pre-computed interpolated fill color (hex).
    ColorScale(String),
    /// Data bar: proportion filled (0..1), colors, gradient flag, and show_value flag.
    DataBar {
        positive_color: String,
        negative_color: String,
        is_gradient: bool,
        value: f64,
        /// Proportion [0,1] at which the zero axis falls within the cell width.
        axis_position: f64,
        show_value: bool,
    },
    /// Custom icon: icon name (Icon enum variant) and color.
    Icon {
        icon: Icon,
        color: String,
        show_value: bool,
    },
    /// Rating: show `count` copies of `icon` out of `max` possible.
    Rating {
        icon: Icon,
        count: u32,
        max: u32,
        color: String,
        show_value: bool,
    },
}

// ---------------------------------------------------------------------------
// Public output types returned by get_cell_style().
// ---------------------------------------------------------------------------

/// Icon set decoration for a cell.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CfIcon {
    pub icon: Icon,
    pub color: String,
    pub show_value: bool,
}

/// Data bar decoration for a cell.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CfDataBar {
    pub positive_color: String,
    pub negative_color: String,
    pub is_gradient: bool,
    /// Proportion of the bar to fill, in \[0.0, 1.0\].
    pub value: f64,
    /// Proportion [0,1] at which the zero axis falls within the cell width.
    pub axis_position: f64,
    pub show_value: bool,
}

/// Rating decoration for a cell: show `count` copies of `icon` out of `max` possible.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CfRating {
    /// Icon to show for each rating point (e.g. star, circle, etc.).
    pub icon: Icon,
    /// Number of filled icons to show (1..=max).
    pub count: u32,
    /// Maximum number of icons in the scale (3, 4, or 5).
    pub max: u32,
    pub color: String,
    pub show_value: bool,
}

/// Returns the ordered (Icon, color) pairs for a named Excel icon-set preset,
/// ordered from lowest to highest value bucket.
/// `name` is the XLSX `iconSetType` attribute value (e.g. `"3TrafficLights2"`).
/// Returns `None` for unknown names.
pub fn icon_set_icons(name: &str) -> Option<Vec<(Icon, String)>> {
    let s = |c: &'static str| c.to_string();
    match name {
        "3Arrows" => Some(vec![
            (Icon::ArrowDown, s("#e43400")),
            (Icon::ArrowRight, s("#ffeb84")),
            (Icon::ArrowUp, s("#84cb1f")),
        ]),
        "3ArrowsGray" => Some(vec![
            (Icon::ArrowDown, s("#808080")),
            (Icon::ArrowRight, s("#808080")),
            (Icon::ArrowUp, s("#808080")),
        ]),
        "4Arrows" => Some(vec![
            (Icon::ArrowDown, s("#e43400")),
            (Icon::ArrowAngleDown, s("#ffeb84")),
            (Icon::ArrowAngleUp, s("#ffeb84")),
            (Icon::ArrowUp, s("#84cb1f")),
        ]),
        "4ArrowsGray" => Some(vec![
            (Icon::ArrowDown, s("#808080")),
            (Icon::ArrowAngleDown, s("#808080")),
            (Icon::ArrowAngleUp, s("#808080")),
            (Icon::ArrowUp, s("#808080")),
        ]),
        "5Arrows" => Some(vec![
            (Icon::ArrowDown, s("#e43400")),
            (Icon::ArrowAngleDown, s("#ffeb84")),
            (Icon::ArrowRight, s("#ffeb84")),
            (Icon::ArrowAngleUp, s("#ffeb84")),
            (Icon::ArrowUp, s("#84cb1f")),
        ]),
        "5ArrowsGray" => Some(vec![
            (Icon::ArrowDown, s("#808080")),
            (Icon::ArrowAngleDown, s("#808080")),
            (Icon::ArrowRight, s("#808080")),
            (Icon::ArrowAngleUp, s("#808080")),
            (Icon::ArrowUp, s("#808080")),
        ]),
        "3Triangles" => Some(vec![
            (Icon::TriangleDown, s("#f8696b")),
            (Icon::FlatRectangle, s("#ffeb84")),
            (Icon::TriangleUp, s("#63be7b")),
        ]),
        "3TrafficLights1" | "3TrafficLights" | "3TrafficLights2" => Some(vec![
            (Icon::Circle, s("#f8696b")),
            (Icon::Circle, s("#ffeb84")),
            (Icon::Circle, s("#63be7b")),
        ]),
        "4TrafficLights" => Some(vec![
            (Icon::Circle, s("#000000")),
            (Icon::Circle, s("#f8696b")),
            (Icon::Circle, s("#ffeb84")),
            (Icon::Circle, s("#63be7b")),
        ]),
        "3Signs" => Some(vec![
            (Icon::Cross, s("#f8696b")),
            (Icon::TriangleUp, s("#ffeb84")),
            (Icon::Circle, s("#63be7b")),
        ]),
        "4RedToBlack" => Some(vec![
            (Icon::Circle, s("#000000")),
            (Icon::Circle, s("#808080")),
            (Icon::Circle, s("#f66f00")),
            (Icon::Circle, s("#e43400")),
        ]),
        "3Symbols" => Some(vec![
            (Icon::Cross, s("#f8696b")),
            (Icon::Exclamation, s("#ffeb84")),
            (Icon::Check, s("#63be7b")),
        ]),
        "3Symbols2" => Some(vec![
            (Icon::Cross, s("#f8696b")),
            (Icon::Exclamation, s("#ffeb84")),
            (Icon::Check, s("#63be7b")),
        ]),
        "3Flags" => Some(vec![
            (Icon::Flag, s("#f8696b")),
            (Icon::Flag, s("#ffeb84")),
            (Icon::Flag, s("#63be7b")),
        ]),
        _ => None,
    }
}

/// The full visual description of a cell, including any conditional formatting overlay.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtendedStyle {
    /// The final cell style (base style with any CF dxf/color-scale overlay applied).
    pub style: Style,
    /// Set when a icon-set rule applies to the cell.
    pub icon: Option<CfIcon>,
    /// Set when a data-bar rule applies to the cell.
    pub data_bar: Option<CfDataBar>,
    /// Set when a rating rule (IconSetRating3/4/5) applies to the cell.
    pub rating: Option<CfRating>,
}
