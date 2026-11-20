#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;
#[test]
fn test_fn_t_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("A4", "5");
    model._set("A5", "6");
    model._set("A6", "10");
    model._set("A7", "3");
    model._set("A8", "2");
    model._set("A9", "4");
    model._set("A10", "7");

    model._set("B2", "6");
    model._set("B3", "19");
    model._set("B4", "3");
    model._set("B5", "2");
    model._set("B6", "13");
    model._set("B7", "4");
    model._set("B8", "5");
    model._set("B9", "17");
    model._set("B10", "3");

    model._set("C1", "=T.TEST(A2:A10, B2:B10, 1, 1)");
    model._set("C2", "=T.TEST(A2:A10, B2:B10, 1, 2)");
    model._set("C3", "=T.TEST(A2:A10, B2:B10, 1, 3)");
    model._set("C4", "=T.TEST(A2:A10, B2:B10, 2, 1)");
    model._set("C5", "=T.TEST(A2:A10, B2:B10, 2, 2)");
    model._set("C6", "=T.TEST(A2:A10, B2:B10, 2, 3)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0.103836888");
    assert_eq!(model._get_text("C2"), *"0.100244599");
    assert_eq!(model._get_text("C3"), *"0.105360319");
    assert_eq!(model._get_text("C4"), *"0.207673777");
    assert_eq!(model._get_text("C5"), *"0.200489197");
    assert_eq!(model._get_text("C6"), *"0.210720639");
}

// Whole columns are read only as far as the used area: the cells past it are
// empty, and T.TEST skips empty cells, in pairs for the paired test.
#[test]
fn test_fn_t_test_whole_columns() {
    let mut model = new_empty_model();
    for (row, (x, y)) in [(3, 4), (5, 9), (2, 2), (8, 11), (6, 7)].iter().enumerate() {
        model._set(&format!("D{}", row + 1), &x.to_string());
        model._set(&format!("E{}", row + 1), &y.to_string());
    }
    model._set("E7", "text");
    for test_type in 1..=3 {
        model._set(
            &format!("A{test_type}"),
            &format!("=T.TEST(D:D,E:E,2,{test_type})=T.TEST(D1:D5,E1:E5,2,{test_type})"),
        );
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "TRUE");
}

// The paired test pairs the cells of the two samples in reading order. A
// range more than one row tall is read at its full width, so that a sample is
// not cut narrower on its own sheet than the other is: Sheet1 is two columns
// wide and Sheet2 three, and cutting each to its own width would pair 8 with
// 100 and 10 with 9.
#[test]
fn test_fn_t_test_paired_whole_rows_on_two_sheets() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "1");
    model._set("B1", "4");
    model._set("A2", "8");
    model._set("B2", "10");
    model._set("Sheet2!A1", "2");
    model._set("Sheet2!B1", "5");
    model._set("Sheet2!A2", "9");
    model._set("Sheet2!B2", "12");
    model._set("Sheet2!C1", "100");
    model._set("A5", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,1)");
    model._set("A6", "=T.TEST(Sheet1!A1:C2,Sheet2!A1:C2,2,1)");
    model._set("A7", "=TTEST(Sheet1!1:2,Sheet2!1:2,2,1)");
    // Against an array of another shape the pairs are the first cells of the
    // range in reading order: (1, 1) and (4, 2)
    model._set("A8", "=T.TEST(Sheet1!1:2,{1,2,3,4,5,6},2,1)");
    model._set("A9", "=T.TEST(Sheet1!A1:F1,{1,2,3,4,5,6},2,1)");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "0.015392438");
    assert_eq!(model._get_text("A6"), "0.015392438");
    assert_eq!(model._get_text("A7"), "0.015392438");
    assert_eq!(model._get_text("A8"), "0.5");
    assert_eq!(model._get_text("A9"), "0.5");
}

fn new_model_with_two_sheets<'a>() -> crate::model::Model<'a> {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "1");
    model._set("B1", "4");
    model._set("A2", "8");
    model._set("B2", "10");
    model._set("Sheet2!A1", "2");
    model._set("Sheet2!B1", "5");
    model._set("Sheet2!A2", "9");
    model._set("Sheet2!B2", "12");
    model._set("Sheet2!C1", "100");
    model
}

// Types 2 and 3 do not pair cells: over whole rows they give what the rows
// written out give.
#[test]
fn test_fn_t_test_unpaired_whole_rows_on_two_sheets() {
    let mut model = new_model_with_two_sheets();
    model._set("A5", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,2)");
    model._set("A6", "=T.TEST(Sheet1!A1:C2,Sheet2!A1:C2,2,2)");
    model._set("A7", "=T.TEST(Sheet1!1:2,Sheet2!1:2,1,3)");
    model._set("A8", "=T.TEST(Sheet1!A1:C2,Sheet2!A1:C2,1,3)");
    model.evaluate();
    assert_ne!(model._get_text("A5"), "#DIV/0!");
    assert_eq!(model._get_text("A5"), model._get_text("A6"));
    assert_ne!(model._get_text("A7"), "#DIV/0!");
    assert_eq!(model._get_text("A7"), model._get_text("A8"));
}

// Types 2 and 3 read each sample only as far as the used area of its sheet,
// also over whole rows. With a dynamic array in the workbook every cell read
// goes on record, so the record counts the cells read: a whole row read at
// its full width would be 16,384 cells.
#[test]
fn test_fn_t_test_unpaired_whole_rows_read_only_the_used_area() {
    let mut model = new_model_with_two_sheets();
    model._set("A5", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,2)");
    model._set("A6", "=SEQUENCE(1)");
    model.evaluate();
    assert_ne!(model._get_text("A5"), "#DIV/0!");
    let cells_read = model.evaluation.seen.len();
    assert!(cells_read < 100, "{cells_read} cells read");
}

// The type can come from a cell or a formula, and it is truncated: 1.9 is the
// paired test.
#[test]
fn test_fn_t_test_type_from_a_cell_or_a_formula() {
    let mut model = new_model_with_two_sheets();
    model._set("A10", "1");
    model._set("A11", "2");
    model._set("A5", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,A10)");
    model._set("A6", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,1+0.9)");
    model._set("A7", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,1.9)");
    model._set("A8", "=T.TEST(Sheet1!1:2,Sheet2!1:2,2,A11)");
    model._set("A9", "=T.TEST(Sheet1!A1:C2,Sheet2!A1:C2,2,2)");
    model.evaluate();
    assert_eq!(model._get_text("A5"), "0.015392438");
    assert_eq!(model._get_text("A6"), "0.015392438");
    assert_eq!(model._get_text("A7"), "0.015392438");
    assert_eq!(model._get_text("A8"), model._get_text("A9"));
}

// The errors of the type come after those of the samples and of the tails.
#[test]
fn test_fn_t_test_error_order() {
    let mut model = new_empty_model();
    for row in 1..=3 {
        model._set(&format!("A{row}"), &row.to_string());
        model._set(&format!("B{row}"), &(row * 2).to_string());
        model._set(&format!("C{row}"), &row.to_string());
    }
    model._set("C2", "=NA()");
    model._set("E1", "=T.TEST(C1:C3,B1:B3,2,1/0)");
    model._set("E2", "=T.TEST(A1:A3,C1:C3,2,1/0)");
    model._set("E3", "=T.TEST(A1:A3,B1:B3,\"x\",1/0)");
    model._set("E4", "=T.TEST(A1:A3,B1:B3,3,1/0)");
    model._set("E5", "=T.TEST(A1:A3,B1:B3,2,1/0)");
    model._set("E6", "=T.TEST(A1:A3,B1:B3,2,4)");
    model._set("E7", "=T.TEST(A1:A3,B1:B3,2,\"x\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#N/A");
    assert_eq!(model._get_text("E2"), "#N/A");
    assert_eq!(model._get_text("E3"), "#VALUE!");
    assert_eq!(model._get_text("E4"), "#NUM!");
    assert_eq!(model._get_text("E5"), "#DIV/0!");
    assert_eq!(model._get_text("E6"), "#NUM!");
    assert_eq!(model._get_text("E7"), "#VALUE!");
}
