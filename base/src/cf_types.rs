use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct ColorScale {}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum Operator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    NotEqual,
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

// These are the rules for the icon set and color scale conditional formatting types.
// They specify the thresholds for the icons and colors.
// For instance, if we have three rules, we need 4 icons/colors.
#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub enum Cfvo {
    Min,
    Max,
    Number(f64),
    Percent(f64),
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
    },
    IconSet {
        set: IconSet,
        cfvo: Vec<Cfvo>,
    },
}

#[derive(Encode, Decode, Debug, PartialEq, Clone)]
pub struct ConditionalFormatting {
    // sqref is the range of cells that the conditional formatting applies to.
    range: String,
    cf_rule: CfRule,
    show_value: bool,
    priority: u32,
}
