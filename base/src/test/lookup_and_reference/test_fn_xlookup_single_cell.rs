#![allow(clippy::unwrap_used)]

use crate::model::DefinedNameEntry;
use crate::test::util::new_empty_model;

// Excel treats a reference to a single cell, given as the lookup array or the
// return array of XLOOKUP, as a list of one item: with A1 = "x" and B1 = 10,
// XLOOKUP("x",A1,B1) is 10, the same as XLOOKUP("x",A1:A1,B1:B1).

#[test]
fn a_single_cell_is_a_list_of_one_item() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "x");
    model._set("B1", "10");
    model._set("Sheet2!A1", "y");
    model._set("Sheet2!B1", "20");

    model._set("D1", "=XLOOKUP(\"x\",A1,B1)");
    model._set("D2", "=XLOOKUP(\"x\",$A$1,$B$1)");
    model._set("D3", "=XLOOKUP(\"y\",Sheet2!A1,Sheet2!B1)");
    model._set("D4", "=XLOOKUP(\"x\",A1:A1,B1:B1)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("D2"), "10");
    assert_eq!(model._get_text("D3"), "20");
    assert_eq!(model._get_text("D4"), "10");
}

#[test]
fn a_value_not_in_the_cell_is_not_found() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "10");

    model._set("D1", "=XLOOKUP(\"z\",A1,B1)");
    model._set("D2", "=XLOOKUP(\"z\",A1,B1,\"none\")");

    model.evaluate();

    assert_eq!(model._get_text("D1"), "#N/A");
    assert_eq!(model._get_text("D2"), "none");
}

#[test]
fn a_name_for_one_cell_is_a_list_of_one_item() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "10");
    model
        .new_defined_name(DefinedNameEntry::new("Key", "Sheet1!$A$1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Amount", "Sheet1!$B$1"))
        .unwrap();

    model._set("D1", "=XLOOKUP(\"x\",Key,Amount)");
    model._set("D2", "=XLOOKUP(\"x\",Key,B1)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("D2"), "10");
}

// The match and search modes over one cell give what they give over the same
// cell written as a range.
#[test]
fn the_modes_over_one_cell_agree_with_the_one_cell_range() {
    let mut model = new_empty_model();
    model._set("A1", "3");
    model._set("B1", "three");

    let tails = [
        "",        // exact match
        ",,-1",    // exact match or next smaller
        ",,1",     // exact match or next larger
        ",,0,-1",  // search from the last item
        ",,0,2",   // binary search, ascending
        ",,-1,-2", // binary search, descending, next smaller
    ];
    for (row, tail) in tails.iter().enumerate() {
        let row = row + 1;
        for (lookup_value, column) in [("3", 'D'), ("2", 'E'), ("4", 'F')] {
            model._set(
                &format!("{column}{row}"),
                &format!("=XLOOKUP({lookup_value},A1,B1{tail})"),
            );
            let range_column = (column as u8 + 3) as char;
            model._set(
                &format!("{range_column}{row}"),
                &format!("=XLOOKUP({lookup_value},A1:A1,B1:B1{tail})"),
            );
        }
    }

    model.evaluate();

    for row in 1..=tails.len() {
        for column in ['D', 'E', 'F'] {
            let range_column = (column as u8 + 3) as char;
            assert_eq!(
                model._get_text(&format!("{column}{row}")),
                model._get_text(&format!("{range_column}{row}")),
                "{column}{row} against {range_column}{row}"
            );
        }
    }
    // Spot checks: an exact match, and the next smaller value.
    assert_eq!(model._get_text("D1"), "three");
    assert_eq!(model._get_text("F2"), "three");
}

// Two cells to look in and one to return from are lists of different sizes.
#[test]
fn lists_of_different_sizes_are_still_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "y");
    model._set("B1", "10");

    model._set("D1", "=XLOOKUP(\"x\",A1:A2,B1)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), "#VALUE!");
}

// A formula parsed at one cell and evaluated at another can give a range whose
// corners are swapped. XLOOKUP still reads it from the top.
#[test]
fn a_range_with_swapped_corners_is_read_from_the_top() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model._set(&format!("A{row}"), "k");
        model._set(&format!("B{row}"), &format!("{}", row * 10));
    }

    // In D5 and E5 the ranges are A1:$A$3 and B1:$B$3.
    model._set("D5", "=XLOOKUP(\"k\",A1:$A$3,B1:$B$3)");
    model._set("E5", "=XLOOKUP(\"k\",A1:$A$3,B1:$B$3,,0,-1)");

    // As in an xlsx shared formula, D10 and E10 hold the same formulas without
    // parsing them again: there the ranges are A6:$A$3 and B6:$B$3.
    for column in 4..=5 {
        let index = model
            .workbook
            .worksheet(0)
            .unwrap()
            .cell(5, column)
            .unwrap()
            .get_formula()
            .unwrap();
        model
            .workbook
            .worksheet_mut(0)
            .unwrap()
            .set_cell_with_formula(10, column, index, 0)
            .unwrap();
    }

    model.evaluate();

    // The first and the last "k"
    assert_eq!(model._get_text("D5"), "10");
    assert_eq!(model._get_text("E5"), "30");
    assert_eq!(model._get_text("D10"), "30");
    assert_eq!(model._get_text("E10"), "60");
}
