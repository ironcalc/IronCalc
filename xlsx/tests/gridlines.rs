#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;
use ironcalc_base::types::WorksheetView;

#[test]
fn test_gridlines_issue_1269() {
    let model = load_from_xlsx("tests/gridlines_issue_1269.xlsx", "en", "UTC", "en").unwrap();

    // sheet 0 has no gridlines and default selection
    let sheet0 = model.workbook.worksheet(0).unwrap();
    assert!(!sheet0.show_grid_lines);
    let view0 = sheet0.views.get(&0).unwrap();
    assert_eq!(
        view0,
        &WorksheetView {
            row: 1,
            column: 1,
            range: [1, 1, 1, 1],
            top_row: 1,
            left_column: 1
        }
    );

    // sheet 1 has no gridlines
    let sheet1 = model.workbook.worksheet(1).unwrap();
    assert!(!sheet1.show_grid_lines);
    assert_eq!(sheet1.views.len(), 1);

    let sheet2 = model.workbook.worksheet(2).unwrap();
    assert!(sheet2.show_grid_lines);
    assert_eq!(sheet2.views.len(), 1);
}
