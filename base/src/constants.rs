/// Excel compatibility values
/// COLUMN_WIDTH and ROW_HEIGHT are pixel values
/// A column width of Excel value `w` will result in `w * COLUMN_WIDTH_FACTOR` pixels
/// Note that these constants are inlined
pub(crate) const DEFAULT_COLUMN_WIDTH: f64 = 100.0;
pub(crate) const DEFAULT_ROW_HEIGHT: f64 = 21.0;
pub(crate) const COLUMN_WIDTH_FACTOR: f64 = 12.0;
pub(crate) const ROW_HEIGHT_FACTOR: f64 = 2.0;
pub(crate) const DEFAULT_WINDOW_HEIGH: i64 = 600;
pub(crate) const DEFAULT_WINDOW_WIDTH: i64 = 800;

pub(crate) const LAST_COLUMN: i32 = 16_384;
pub(crate) const LAST_ROW: i32 = 1_048_576;

// 693_594 is computed as:
// NaiveDate::from_ymd(1900, 1, 1).num_days_from_ce() - 2
// The 2 days offset is because of Excel 1900 bug
pub(crate) const EXCEL_DATE_BASE: i32 = 693_594;
