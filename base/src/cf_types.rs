use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum Operator {
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

    // Ratings
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
pub enum CfRule {
    ColorScale {
        cfvo: Vec<Cfvo>,
        colors: Vec<String>,
    },
    CellIs {
        operator: Operator,
        formula: String,
        // Only present for Between and NotBetween operators
        formula2: Option<String>,
        dxf_id: u32,
    },
    DuplicateValues {
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
    TimePeriod {
        dxf_id: u32,
        time_period: String,
    },
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct ConditionalFormatting {
    pub range: String,
    pub cf_rule: CfRule,
    pub priority: u32,
}
