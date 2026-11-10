#![allow(clippy::unwrap_used)]

use ironcalc::import::load_from_xlsx;

// `Table1[]` is the table's data body, the same as `Table1[#Data]` (#1606).
// Table1 in tables.xlsx is WithHeaders!A1:F5 with a header row and a totals row; Excel saved
// SUM(Table1[#Data]) in F17 as 274.
#[test]
fn whole_table_reference_is_the_data_body() {
    let mut model = load_from_xlsx(
        "tests/calc_test_no_export/tables.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    let sheet = 0;
    model
        .set_user_input(sheet, 20, 8, "=SUM(Table1[])".to_string())
        .unwrap();
    model
        .set_user_input(sheet, 21, 8, "=VLOOKUP(6,Table1[],2,FALSE)".to_string())
        .unwrap();
    model
        .set_user_input(sheet, 22, 8, "=ROWS(Table1[])".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_formatted_cell_value(sheet, 17, 6).unwrap(), "274");
    assert_eq!(model.get_formatted_cell_value(sheet, 20, 8).unwrap(), "274");
    assert_eq!(model.get_formatted_cell_value(sheet, 21, 8).unwrap(), "7");
    assert_eq!(model.get_formatted_cell_value(sheet, 22, 8).unwrap(), "3");
}
