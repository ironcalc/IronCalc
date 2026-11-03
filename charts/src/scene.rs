//! What a renderer draws: shapes placed in pixels, with no decisions left.
//!
//! The origin is the top left corner of the chart, y grows downwards. Every
//! color is already an `#RRGGBB` string, every length a pixel.

use serde::{Deserialize, Serialize};

/// A placed picture of a chart.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Scene {
    pub width: f64,
    pub height: f64,
    /// In painting order: later elements go on top.
    pub elements: Vec<Element>,
}

/// One shape and the part of the chart it belongs to.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Element {
    pub role: Role,
    pub shape: Shape,
}

/// The part of the chart an element belongs to, so a UI can tell what was
/// clicked without knowing how the chart was laid out.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(tag = "type")]
pub enum Role {
    ChartArea,
    Title,
    Legend,
    PlotArea,
    Gridlines,
    CategoryAxis,
    ValueAxis,
    /// The bars (or points) of the series with this index.
    Series(usize),
    /// The data labels of the series with this index.
    DataLabel(usize),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Shape {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        /// No fill when `None`.
        fill: Option<String>,
        stroke: Option<Stroke>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: Stroke,
    },
    Text {
        /// The anchor point: on the baseline, at the start, middle or end of
        /// the text depending on `anchor`.
        x: f64,
        y: f64,
        text: String,
        font: ResolvedFont,
        anchor: Anchor,
        /// Degrees, clockwise, around the anchor point. `-90` reads upwards.
        rotation: f64,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Stroke {
    pub color: String,
    pub width: f64,
    /// Lengths of dashes and gaps, empty for a solid line.
    pub dash: Vec<f64>,
}

/// A font with every value a renderer needs spelled out.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResolvedFont {
    pub family: String,
    /// In pixels.
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: String,
}

/// Which point of the text sits on the anchor, horizontally.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Anchor {
    Start,
    Middle,
    End,
}
