#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;

#[test]
fn test_gridlines_issue_1269() {
    let model = load_from_xlsx("tests/gridlines_issue_1269.xlsx", "en", "UTC", "en").unwrap();

    // sheet 0 has no gridlines
    let sheet0 = model.workbook.worksheet(0).unwrap();
    assert!(!sheet0.show_grid_lines);

    // sheet 1 has no gridlines
    let sheet1 = model.workbook.worksheet(1).unwrap();
    assert!(!sheet1.show_grid_lines);

    let sheet2 = model.workbook.worksheet(2).unwrap();
    assert!(sheet2.show_grid_lines);
}
