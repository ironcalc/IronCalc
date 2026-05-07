use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::Style;

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
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

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum TextOperator {
    Contains, // NOT(ISERROR(SEARCH(value,A1)))
    DoesNotContain,
    BeginsWith,
    EndsWith, // RIGHT(E1,LEN(value))=
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
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

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum IconSet {
    // Directional
    Arrows3,
    ArrowsGray3,
    Arrows4,
    ArrowsGray4,
    Arrows5,
    ArrowsGray5,
    Triangles3,

    // Shapes
    TrafficLights3,
    TrafficLights3Rimmed,
    TrafficLights4,
    Signs3,
    RedToBlack4,

    // Indicators
    Symbols3Circled,
    Symbols3Uncircled,
    Flags3,

    // Ratings, they all
    Stars3,
    Quarters5,
    Boxes5,
    Ratings4,
    Ratings5,
}

// These are the threshold definitions for icon set and color scale conditional formatting.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum Cfvo {
    Min,
    Max,
    Number(f64),
    Percent(f64),
    Percentile(f64),
    Formula(String),
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
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
    Signal1,
    Signal2,
    Signal3,
    Signal4,
    Signal5,
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum CfRule {
    ColorScale {
        cfvo: Vec<Cfvo>,
        colors: Vec<String>,
    },
    CellIs {
        operator: ValueOperator,
        formula: String,
        // Only present for Between and NotBetween operators
        formula2: Option<String>,
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
        cfvo: Vec<Cfvo>,
        color: String,
        show_value: bool,
    },
    IconSet {
        set: IconSet,
        cfvo: Vec<Cfvo>,
        show_value: bool,
    },
    IconSetCustom2 {
        set: [Icon; 2],
        cfvo: [Cfvo; 2],
        color: [String; 2],
        show_value: bool,
    },
    IconSetCustom3 {
        set: [Icon; 3],
        cfvo: [Cfvo; 3],
        color: [String; 3],
        show_value: bool,
    },
    IconSetCustom4 {
        set: [Icon; 4],
        cfvo: [Cfvo; 4],
        color: [String; 4],
        show_value: bool,
    },
    IconSetCustom5 {
        set: [Icon; 5],
        cfvo: [Cfvo; 5],
        color: [String; 5],
        show_value: bool,
    },
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct ConditionalFormatting {
    pub range: String,
    pub cf_rule: CfRule,
    pub priority: u32,
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
    /// Data bar: proportion filled (0..1), color, and show_value flag.
    DataBar {
        color: String,
        value: f64,
        show_value: bool,
    },
    /// Icon set: which icon set and which icon index (0-indexed).
    Icon {
        set: IconSet,
        index: u32,
        show_value: bool,
    },
    /// Custom icon: explicit unicode character and color.
    CustomIcon {
        char: String,
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
    pub set: IconSet,
    /// 0-indexed position within the icon set.
    pub index: u32,
    pub show_value: bool,
}

/// Data bar decoration for a cell.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CfDataBar {
    /// Hex color for the bar fill.
    pub color: String,
    /// Proportion of the bar to fill, in \[0.0, 1.0\].
    pub value: f64,
    pub show_value: bool,
}

/// Custom icon decoration for a cell (from an IconSetCustom rule).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CfCustomIcon {
    /// Unicode character representing the icon.
    pub char: String,
    /// Hex color for the icon.
    pub color: String,
    pub show_value: bool,
}

/// The full visual description of a cell, including any conditional formatting overlay.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtendedStyle {
    /// The final cell style (base style with any CF dxf/color-scale overlay applied).
    pub style: Style,
    /// Set when a preset icon-set rule applies to the cell.
    pub icon: Option<CfIcon>,
    /// Set when a data-bar rule applies to the cell.
    pub data_bar: Option<CfDataBar>,
    /// Set when a custom icon-set rule applies to the cell.
    pub custom_icon: Option<CfCustomIcon>,
}
