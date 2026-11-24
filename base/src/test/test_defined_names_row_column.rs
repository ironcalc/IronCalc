#![allow(clippy::unwrap_used)]

// Defined names follow the cells they point at when rows or columns are
// inserted, deleted or moved, as the references in a cell formula do.

use crate::model::{DefinedNameEntry, Model};
use crate::test::util::new_empty_model;

fn name_formula(model: &Model, name: &str, scope: Option<u32>) -> String {
    model
        .get_defined_name_list()
        .into_iter()
        .find(|entry| entry.name == name && entry.scope == scope)
        .unwrap()
        .formula
}

#[test]
fn insert_rows_above_the_target() {
    let mut model = new_empty_model();
    model._set("A1", "0.1");
    model._set("A2", "200");
    model._set("B2", "=A2*Rate");
    model._set("C1", "=Rate");
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$A$1"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B2"), "20");

    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();

    // The value is now in A2, and so is the name
    assert_eq!(model._get_text("A2"), "0.1");
    assert_eq!(name_formula(&model, "Rate", None), "Sheet1!$A$2");
    assert_eq!(model._get_formula("B3"), "=A3*Rate");
    assert_eq!(model._get_text("B3"), "20");
    assert_eq!(model._get_text("C2"), "0.1");

    // Rows inserted below the target leave the name alone
    model.insert_rows(0, 3, 5).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Rate", None), "Sheet1!$A$2");
    assert_eq!(model._get_text("B8"), "20");
}

#[test]
fn insert_columns_left_of_the_target() {
    let mut model = new_empty_model();
    model._set("H14", "5000");
    model._set("A1", "=Budget-1");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4999");

    model.insert_columns(0, 3, 2).unwrap();
    model.evaluate();

    assert_eq!(model._get_text("J14"), "5000");
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!$J$14");
    assert_eq!(model._get_text("A1"), "4999");

    // Columns inserted to the right of the target leave the name alone
    model.insert_columns(0, 11, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!$J$14");
    assert_eq!(model._get_text("A1"), "4999");
}

#[test]
fn delete_rows_and_columns_before_the_target() {
    let mut model = new_empty_model();
    model._set("H14", "5000");
    model._set("A1", "=Budget-1");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model.evaluate();

    model.delete_rows(0, 5, 3).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!$H$11");
    assert_eq!(model._get_text("A1"), "4999");

    model.delete_columns(0, 2, 4).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!$D$11");
    assert_eq!(model._get_text("D11"), "5000");
    assert_eq!(model._get_text("A1"), "4999");

    // Rows and columns deleted after the target leave the name alone
    model.delete_rows(0, 12, 2).unwrap();
    model.delete_columns(0, 5, 2).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!$D$11");
    assert_eq!(model._get_text("A1"), "4999");
}

#[test]
fn delete_the_row_of_the_target() {
    let mut model = new_empty_model();
    model._set("H14", "5000");
    model._set("A1", "=Budget-1");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4999");

    model.delete_rows(0, 13, 3).unwrap();
    model.evaluate();

    // The cell is gone: the name does not silently point at what was below
    assert_eq!(name_formula(&model, "Budget", None), "#REF!");
    assert_eq!(model._get_formula("A1"), "=Budget-1");
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn delete_the_column_of_the_target() {
    let mut model = new_empty_model();
    model._set("H14", "5000");
    model._set("A1", "=Budget-1");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model.evaluate();

    model.delete_columns(0, 8, 1).unwrap();
    model.evaluate();

    assert_eq!(name_formula(&model, "Budget", None), "#REF!");
    assert_eq!(model._get_text("A1"), "#REF!");
}

#[test]
fn a_range_grows_and_shrinks() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model._set("K2", "20");
    model._set("K3", "30");
    model._set("A1", "=SUM(Items)");
    model
        .new_defined_name(DefinedNameEntry::new("Items", "Sheet1!$K$1:$K$3"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "60");

    // A row inserted inside the range makes it one row longer
    model.insert_rows(0, 2, 1).unwrap();
    model._set("K2", "5");
    model.evaluate();
    assert_eq!(name_formula(&model, "Items", None), "Sheet1!$K$1:$K$4");
    assert_eq!(model._get_text("A1"), "65");

    // and a row deleted inside it one row shorter
    model.delete_rows(0, 3, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Items", None), "Sheet1!$K$1:$K$3");
    assert_eq!(model._get_text("A1"), "45");

    // Same with columns
    model
        .new_defined_name(DefinedNameEntry::new("Block", "Sheet1!$J$1:$L$3"))
        .unwrap();
    model._set("A2", "=SUM(Block)");
    model.insert_columns(0, 11, 2).unwrap();
    model._set("K1", "1");
    model.evaluate();
    assert_eq!(name_formula(&model, "Block", None), "Sheet1!$J$1:$N$3");
    assert_eq!(name_formula(&model, "Items", None), "Sheet1!$M$1:$M$3");
    assert_eq!(model._get_text("A1"), "45");
    assert_eq!(model._get_text("A2"), "46");

    model.delete_columns(0, 11, 2).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Block", None), "Sheet1!$J$1:$L$3");
    assert_eq!(name_formula(&model, "Items", None), "Sheet1!$K$1:$K$3");
    assert_eq!(model._get_text("A2"), "45");
}

#[test]
fn a_name_local_to_a_sheet() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("Sheet1!C5", "7");
    model._set("Sheet2!C5", "9");
    model
        .new_defined_name(DefinedNameEntry::new("Local", "Sheet1!$C$5").scope(0))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Local", "Sheet2!$C$5").scope(1))
        .unwrap();
    model._set("Sheet1!A1", "=Local");
    model._set("Sheet2!A1", "=Local");
    model.evaluate();

    model.insert_rows(0, 2, 2).unwrap();
    model.insert_columns(0, 1, 1).unwrap();
    model.evaluate();

    assert_eq!(name_formula(&model, "Local", Some(0)), "Sheet1!$D$7");
    assert_eq!(model._get_text("Sheet1!B1"), "7");
    // The name of the other sheet did not move
    assert_eq!(name_formula(&model, "Local", Some(1)), "Sheet2!$C$5");
    assert_eq!(model._get_text("Sheet2!A1"), "9");
}

#[test]
fn a_name_that_is_an_expression() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model._set("K2", "20");
    model._set("K3", "30");
    model
        .new_defined_name(DefinedNameEntry::new("Double", "Sheet1!$K$1*2"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Total", "SUM(Sheet1!$K$1:$K$3)"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("TotalPlus", "Total+Double"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("AddFirst", "LAMBDA(x,x+Sheet1!$K$1)"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Pi", "3.14159"))
        .unwrap();
    model._set("A10", "=Double");
    model._set("A11", "=Total");
    model._set("A12", "=TotalPlus");
    model._set("A13", "=AddFirst(1)");
    model._set("A14", "=Pi");
    model.evaluate();
    assert_eq!(model._get_text("A10"), "20");
    assert_eq!(model._get_text("A11"), "60");
    assert_eq!(model._get_text("A12"), "80");
    assert_eq!(model._get_text("A13"), "11");

    model.insert_columns(0, 5, 1).unwrap();
    model.insert_rows(0, 1, 2).unwrap();
    model.evaluate();

    assert_eq!(name_formula(&model, "Double", None), "Sheet1!$L$3*2");
    assert_eq!(name_formula(&model, "Total", None), "SUM(Sheet1!$L$3:$L$5)");
    assert_eq!(name_formula(&model, "TotalPlus", None), "Total+Double");
    assert_eq!(
        name_formula(&model, "AddFirst", None),
        "LAMBDA(x,x+Sheet1!$L$3)"
    );
    assert_eq!(name_formula(&model, "Pi", None), "3.14159");
    assert_eq!(model._get_text("A12"), "20");
    assert_eq!(model._get_text("A13"), "60");
    assert_eq!(model._get_text("A14"), "80");
    assert_eq!(model._get_text("A15"), "11");
    assert_eq!(model._get_text("A16"), "3.14159");

    // Deleting a cell the expression reads makes that reference #REF!
    model.delete_rows(0, 3, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Double", None), "#REF!*2");
    assert_eq!(model._get_formula("A11"), "=Double");
    assert_eq!(model._get_text("A11"), "#REF!");
}

#[test]
fn names_that_point_at_another_sheet_do_not_move() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("Sheet2!H14", "5000");
    model._set("Sheet2!K1", "1");
    model._set("Sheet2!K2", "2");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet2!$H$14"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Items", "Sheet2!$K$1:$K$2"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Total", "SUM(Sheet2!$K$1:$K$2)"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("OnSheet1", "Sheet2!$H$14").scope(0))
        .unwrap();
    model._set("Sheet1!A20", "=Budget+SUM(Items)+Total+OnSheet1");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!A20"), "10006");

    // Everything happens in Sheet1
    model.insert_rows(0, 1, 3).unwrap();
    model.insert_columns(0, 1, 3).unwrap();
    model.delete_rows(0, 14, 1).unwrap();
    model.delete_columns(0, 8, 1).unwrap();
    model.move_rows_action(0, 1, 1, 13).unwrap();
    model.move_columns_action(0, 1, 1, 10).unwrap();
    model.evaluate();

    assert_eq!(name_formula(&model, "Budget", None), "Sheet2!$H$14");
    assert_eq!(name_formula(&model, "Items", None), "Sheet2!$K$1:$K$2");
    assert_eq!(name_formula(&model, "Total", None), "SUM(Sheet2!$K$1:$K$2)");
    assert_eq!(name_formula(&model, "OnSheet1", Some(0)), "Sheet2!$H$14");
    assert_eq!(model._get_text("Sheet1!C22"), "10006");

    // A change in Sheet2 moves them, also the one that is local to Sheet1
    model.insert_rows(1, 1, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet2!$H$15");
    assert_eq!(name_formula(&model, "Items", None), "Sheet2!$K$2:$K$3");
    assert_eq!(name_formula(&model, "Total", None), "SUM(Sheet2!$K$2:$K$3)");
    assert_eq!(name_formula(&model, "OnSheet1", Some(0)), "Sheet2!$H$15");
    assert_eq!(model._get_text("Sheet1!C22"), "10006");
}

#[test]
fn moving_rows_and_columns() {
    let mut model = new_empty_model();
    model._set("C5", "42");
    model._set("C6", "1");
    model._set("C7", "2");
    model._set("A1", "=Answer");
    model._set("A2", "=SUM(Pair)");
    model
        .new_defined_name(DefinedNameEntry::new("Answer", "Sheet1!$C$5"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Pair", "Sheet1!$C$6:$C$7"))
        .unwrap();
    model.evaluate();

    // Row 5 goes three rows down: rows 6 to 8 come one up
    model.move_rows_action(0, 5, 1, 3).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("C8"), "42");
    assert_eq!(name_formula(&model, "Answer", None), "Sheet1!$C$8");
    assert_eq!(name_formula(&model, "Pair", None), "Sheet1!$C$5:$C$6");
    assert_eq!(model._get_text("A1"), "42");
    assert_eq!(model._get_text("A2"), "3");

    // Column C goes one column to the left
    model.move_columns_action(0, 3, 1, -1).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B8"), "42");
    assert_eq!(name_formula(&model, "Answer", None), "Sheet1!$B$8");
    assert_eq!(name_formula(&model, "Pair", None), "Sheet1!$B$5:$B$6");
    assert_eq!(model._get_text("A1"), "42");
    assert_eq!(model._get_text("A2"), "3");
}

#[test]
fn names_that_do_not_move_are_left_as_they_are() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model
        .new_defined_name(DefinedNameEntry::new("WholeColumn", "Sheet1!$K:$K"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("WholeRow", "Sheet1!$3:$3"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("First", "Sheet1!$A$1"))
        .unwrap();
    model._set("A2", "=SUM(WholeColumn)");
    model.evaluate();

    // Rows do not change a whole column, nor columns a whole row
    model.insert_rows(0, 2, 4).unwrap();
    model.insert_columns(0, 12, 4).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "WholeColumn", None), "Sheet1!$K:$K");
    assert_eq!(name_formula(&model, "WholeRow", None), "Sheet1!$7:$7");
    assert_eq!(name_formula(&model, "First", None), "Sheet1!$A$1");
    assert_eq!(model._get_text("A6"), "10");

    model.insert_columns(0, 2, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "WholeColumn", None), "Sheet1!$L:$L");
    assert_eq!(name_formula(&model, "WholeRow", None), "Sheet1!$7:$7");
    assert_eq!(name_formula(&model, "First", None), "Sheet1!$A$1");
    assert_eq!(model._get_text("A6"), "10");
}

// A formula written after the change sees the name where it is now
#[test]
fn formulas_written_afterwards_use_the_new_target() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model._set("K2", "20");
    model
        .new_defined_name(DefinedNameEntry::new("Items", "Sheet1!$K$1:$K$2"))
        .unwrap();
    model.insert_rows(0, 1, 1).unwrap();
    model._set("A1", "=SUM(Items)");
    model._set("K1", "1000");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "30");

    // and the workbook keeps it when it is saved and opened again
    let bytes = model.to_bytes();
    let mut model = Model::from_bytes(&bytes, "en").unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Items", None), "Sheet1!$K$2:$K$3");
    assert_eq!(model._get_text("A1"), "30");
}

// A reference without `$` in a name is displaced as an absolute one: a name
// has no cell of its own for it to be relative to
#[test]
fn a_name_with_a_relative_reference() {
    let mut model = new_empty_model();
    model._set("H14", "5000");
    model._set("A1", "=Budget-1");
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!H14"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "4999");

    model.insert_rows(0, 2, 2).unwrap();
    model.insert_columns(0, 2, 1).unwrap();
    model.evaluate();
    assert_eq!(name_formula(&model, "Budget", None), "Sheet1!I16");
    assert_eq!(model._get_text("A1"), "4999");
}

#[test]
fn a_sheet_name_that_needs_quotes() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "11월 지출내역").unwrap();
    // B16, H14 and B23
    for (row, column, value) in [
        (16, 2, "5000000"),
        (14, 8, "2804775"),
        (23, 2, "=Budget-H14"),
    ] {
        model
            .set_user_input(0, row, column, value.to_string())
            .unwrap();
    }
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "'11월 지출내역'!$B$16"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text_at(0, 23, 2), "2195225");

    model.insert_rows(0, 10, 1).unwrap();
    model.evaluate();
    assert_eq!(
        name_formula(&model, "Budget", None),
        "'11월 지출내역'!$B$17"
    );
    assert_eq!(
        model.get_cell_formula(0, 24, 2).unwrap(),
        Some("=Budget-H15".to_string())
    );
    assert_eq!(model._get_text_at(0, 24, 2), "2195225");
}
