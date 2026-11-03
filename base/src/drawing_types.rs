use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::charts::common::{Fill, Line};
use crate::charts::types::Chart;

/// A floating object drawn over the grid: a frame anchored to cells plus its content.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Drawing {
    /// Unique within the worksheet. The UI and the API address drawings by id.
    pub id: u32,
    /// Shown in the UI and written to the xlsx as the object name, e.g. "Chart 1".
    pub name: String,
    pub anchor: Anchor,
    /// Background of the frame, under the content.
    pub fill: Fill,
    /// Outline of the frame.
    pub border: Line,
    pub content: DrawingContent,
}

/// The rectangle a drawing occupies, given by two cell corners.
/// Mirrors xdr:twoCellAnchor. Rows and columns are 1-based.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Anchor {
    pub from: AnchorPoint,
    pub to: AnchorPoint,
    pub behavior: AnchorBehavior,
}

/// A point inside a cell. Offsets are in EMU (914400 per inch, 9525 per CSS pixel),
/// measured from the top-left corner of the cell.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct AnchorPoint {
    pub row: i32,
    pub column: i32,
    pub row_offset: i64,
    pub column_offset: i64,
}

/// How the drawing reacts when rows or columns are resized, inserted or deleted.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum AnchorBehavior {
    /// Both corners follow their cells (editAs="twoCell", the default for charts).
    MoveAndResize,
    /// The top-left corner follows its cell, the size is kept (editAs="oneCell").
    MoveOnly,
    /// Stays at the same pixel position on the sheet (editAs="absolute").
    Fixed,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum DrawingContent {
    Chart(Box<Chart>),
    // Image(Image)
    /// Placeholder for the drawings we read but do not support (shapes, SmartArt, ...).
    /// Keeps the frame so the user can see something is there.
    Unsupported,
}
