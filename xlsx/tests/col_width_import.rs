#![allow(clippy::unwrap_used)]

use ironcalc::import::load_from_xlsx;

const DEFAULT_COLUMN_WIDTH: f64 = 90.0;

// The `width` attribute of `<col>` is optional (ECMA-376 Part 1, 18.3.1.13), and
// third-party producers routinely emit a `<col>` that only carries a style or
// `hidden="1"`. Reading `width` with `get_attribute` hard-errored with
// `Missing "width" XML attribute` and failed the whole load; Excel opens these
// files and gives such a column the sheet's default width.
#[test]
fn col_without_width_loads() {
    let model = load_from_xlsx("tests/col_without_width.xlsx", "en", "UTC", "en").unwrap();
    let sheet = model.workbook.worksheet(0).unwrap();

    // The <col> without a width is not a custom width, so it renders at the default.
    // It is hidden, so `get_column_width` reports 0 and `get_actual_column_width` the size.
    assert!(
        (sheet.get_actual_column_width(2).unwrap() - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON
    );
    assert!(sheet.is_column_hidden(2).unwrap());
    // ... and it still carries the style it declared.
    assert_eq!(sheet.get_column_style(2).unwrap(), Some(1));
    assert!(sheet.is_column_hidden(54).unwrap());

    // The sibling <col> that does declare a width is unaffected.
    assert!((sheet.get_column_width(1).unwrap() - 5.25 * 9.0).abs() < f64::EPSILON);
    assert!(!sheet.is_column_hidden(1).unwrap());

    // Columns past the <cols> ranges fall back to the default too.
    assert!((sheet.get_column_width(100).unwrap() - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON);
}
