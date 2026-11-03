//! Chart definitions.
//!
//! A chart is the content of a [`crate::drawing_types::Drawing`]. Each chart
//! kind has its own module (`columnchart`, later `linechart`, ...) holding the
//! struct for that kind. Pieces shared by every kind (fonts, titles, legends,
//! axes, datasets) live in `common`, the `Chart` enum that ties them together
//! lives in `types`, and `data` resolves a data source into the values to plot.

pub mod columnchart;
pub mod common;
pub mod data;
pub mod types;

pub use columnchart::ColumnChart;
pub use common::{
    Alignment, Bars, CategoryAxis, DataLabelPosition, DataLabels, DataRange, DataSource,
    EmptyCells, Fill, Font, Gridlines, Legend, Line, LineStyle, Orientation, Position, Series,
    Stacking, Subtitle, TextDirection, TickMark, Title, ValueAxis, ValueFormat,
};
pub use data::{expand_data_range, ChartData, SeriesData};
pub use types::Chart;
