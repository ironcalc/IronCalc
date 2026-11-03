//! Building blocks shared by every chart kind.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::types::Color;

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Font {
    /// Font size in points.
    pub size: u32,
    /// Font family name, e.g. "Inter". None means the default font of the chart.
    pub family: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Alignment {
    Left,
    Middle, // Center or Centre
    Right,
}

/// A second line under a title, with its own font. Excel writes it as a
/// line break inside the title's text.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Subtitle {
    pub text: String,
    pub font: Font,
}

/// A piece of text with its own formatting: the chart title or an axis title.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Title {
    pub text: String,
    pub font: Font,
    pub alignment: Alignment,
    /// Behind the text and the subtitle; `None` is no fill.
    pub fill: Color,
    /// Around the text and the subtitle; `Automatic` is no border, as in
    /// Excel's default style.
    pub border: Line,
    pub subtitle: Option<Subtitle>,
}

/// Where an element (so far only the legend) sits relative to the plot area.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Position {
    None,
    Top,
    Right,
    Bottom,
    Left,
    /// Free position: (x, y) as fractions in [0, 1] of the chart width and
    /// height, measured from the top-left corner of the chart.
    Float(f64, f64),
}

/// How an area is painted.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Fill {
    /// The theme decides. In Excel's default chart style this is white.
    Automatic,
    None,
    Solid(Color),
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum LineStyle {
    /// The theme decides the style and the color (the color below is ignored).
    /// In Excel's default chart style this is a thin line in text color at 15%.
    Automatic,
    /// The line is not drawn.
    None,
    Thin,
    Thick,
    Dashed,
}

/// A line: the border of a box, an axis or a gridline.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Line {
    pub style: LineStyle,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Legend {
    pub position: Position,
    /// Whether the entries are laid out vertically (one per line) or horizontally.
    pub vertical: bool,
    /// Padding inside the legend box: (top, right, bottom, left) in pixels.
    pub padding: Option<(i32, i32, i32, i32)>,
    pub fill: Color,
    pub border: Line,
    pub font: Font,
    /// Whether the legend is drawn over the plot area. When false the plot
    /// area shrinks to make room for it.
    pub overlay: bool,
}

/// How several datasets are combined in charts that support it (column, bar, line, area).
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Stacking {
    /// Datasets side by side.
    None,
    /// Datasets stacked on top of each other.
    Standard,
    /// Datasets stacked and normalized so that every category adds up to 100%.
    Full,
}

/// How values are rendered: as axis labels or as data labels.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct ValueFormat {
    /// A number format, e.g. "#,##0.00". None means the format of the source cells.
    pub number_format: Option<String>,
    pub font: Font,
}

/// The lines across the plot area at the ticks of an axis, and the tick
/// marks on the axis line itself.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Gridlines {
    /// At every major tick. `LineStyle::None` means no gridlines;
    /// `Automatic` is Excel's thin line in text color at 85%.
    pub major: Line,
    /// Between the major ticks. `LineStyle::None` means no minor gridlines;
    /// `Automatic` is Excel's thin line in text color at 95%.
    pub minor: Line,
    /// The tick marks on the axis line at the major ticks.
    pub major_ticks: TickMark,
    /// The tick marks at the minor ticks.
    pub minor_ticks: TickMark,
}

/// A tick mark on an axis line: on which side of the line it is drawn. Tick
/// marks take the axis line's style, so an axis without a line has none.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum TickMark {
    None,
    /// Into the plot area.
    Inside,
    /// Away from the plot area, towards the labels.
    Outside,
    /// Both ways, across the axis line.
    Cross,
}

/// The axis that holds the categories (the x axis in a column chart).
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct CategoryAxis {
    pub title: Option<Title>,
    /// Font of the category labels.
    pub labels: Font,
    /// Angle of the category labels in degrees, as in Excel's "Custom angle":
    /// positive turns the text counter-clockwise, in -90..=90.
    /// None is Excel's automatic choice.
    pub rotation: Option<i32>,
    /// The axis line itself.
    pub line: Line,
    /// The vertical lines across the plot area.
    pub gridlines: Gridlines,
}

/// The axis that holds the values (the y axis in a column chart).
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct ValueAxis {
    pub title: Option<Title>,
    /// Format of the value labels along the axis.
    pub labels: ValueFormat,
    /// The axis line itself.
    pub line: Line,
    /// The horizontal lines across the plot area.
    pub gridlines: Gridlines,
}

/// Which way the series run inside a single-range data source.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum Orientation {
    /// Each column is a series: categories in the first column, names in the first row.
    Columns,
    /// Each row is a series: categories in the first row, names in the first column.
    Rows,
}

/// One series of data: a set of values plus a name for the legend.
///
/// Both are formulas in A1 notation with a sheet prefix, exactly as the user
/// would type them in the series dialog: `Sheet1!$C$1` or `"Population"` for the
/// name, `Sheet1!$C$2:$C$6` or `{1,2,3}` for the values. Values are evaluated
/// against the workbook when the chart is drawn, never stored here.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Series {
    pub name: String,
    pub values: String,
}

/// Which cells a chart reads.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum DataRange {
    /// One rectangular range, e.g. `Sheet1!$B$1:$E$6`. With `Orientation::Columns`
    /// the categories are B2:B6, the names are C1:E1 and each of the columns
    /// C, D and E is one series. `Orientation::Rows` transposes that.
    Range {
        range: String,
        orientation: Orientation,
    },
    /// Every piece chosen by hand.
    Manual {
        /// Category labels, e.g. `Sheet1!$B$2:$B$6`.
        categories: String,
        series: Vec<Series>,
    },
}

/// How an empty cell in the values is plotted.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum EmptyCells {
    /// Leave a hole in the plot.
    Gaps,
    /// Plot it as zero.
    Zero,
}

/// Where a chart gets its data and how it treats the cells it finds there.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct DataSource {
    pub range: DataRange,
    pub empty_cells: EmptyCells,
    /// Treat a `#N/A` value as an empty cell instead of plotting it.
    pub na_as_empty: bool,
    /// Plot values that sit in hidden rows or columns.
    pub show_hidden_data: bool,
}

/// Where the label of a column sits.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum DataLabelPosition {
    Center,
    InsideEnd,
    InsideBase,
    /// Above the column (below a negative one). Only for clustered columns:
    /// a stacked chart shows it as `Center`.
    OutsideEnd,
}

/// The spacing of the columns, in percent of a column's width.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct Bars {
    /// Between the columns of one category and the next, 0 to 500.
    pub gap: u32,
    /// Between neighbouring columns of a category, -100 to 100: negative
    /// leaves room between them, 0 makes them touch and 100 puts them on
    /// the same spot. Stacked columns are always at 100.
    pub overlap: i32,
}

impl Default for Bars {
    /// Excel's defaults for a new clustered column chart.
    fn default() -> Self {
        Bars {
            gap: 219,
            overlap: -27,
        }
    }
}

impl Bars {
    /// Excel's defaults for a new stacked column chart.
    pub fn stacked() -> Self {
        Bars {
            gap: 150,
            overlap: 100,
        }
    }
}

/// Which way the text of the data labels runs: Excel's "text direction".
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub enum TextDirection {
    Horizontal,
    /// A quarter turn clockwise: the text reads from top to bottom.
    Rotate90,
    /// A quarter turn counter-clockwise: the text reads from bottom to top.
    Rotate270,
}

/// The labels of one series: the value of every column, next to it.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Clone)]
pub struct DataLabels {
    pub position: DataLabelPosition,
    pub direction: TextDirection,
    /// A number format for the values; `None` uses the format of the cells.
    pub number_format: Option<String>,
    /// Color `None` is the automatic gray.
    pub font: Font,
    /// Behind the text; `None` is no fill.
    pub fill: Color,
    /// Around the text; `Automatic` is no border.
    pub border: Line,
}
