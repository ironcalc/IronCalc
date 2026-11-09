#![allow(clippy::unwrap_used)]

// Excel tables in the user model. What Excel does is from Microsoft's
// "Overview of Excel tables", "Using structured references with Excel
// tables", "Total the data in an Excel table" and "Resize a table by adding
// or removing rows and columns".

use crate::{expressions::types::Area, test::util::new_empty_model, DefinedNameEntry, UserModel};

fn area(row: i32, column: i32, width: i32, height: i32) -> Area {
    Area {
        sheet: 0,
        row,
        column,
        width,
        height,
    }
}

/// A1:C4: Item, Qty, Price and three rows of data
fn sales() -> UserModel<'static> {
    let mut model = UserModel::from_model(new_empty_model());
    let cells = [
        ["Item", "Qty", "Price"],
        ["Pens", "2", "1.5"],
        ["Pads", "3", "4"],
        ["Ink", "1", "10"],
    ];
    for (r, row) in cells.iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            model
                .set_user_input(0, r as i32 + 1, c as i32 + 1, value)
                .unwrap();
        }
    }
    model
}

fn text(model: &UserModel, cell: (i32, i32)) -> String {
    model.get_formatted_cell_value(0, cell.0, cell.1).unwrap()
}

fn content(model: &UserModel, cell: (i32, i32)) -> String {
    model.get_cell_content(0, cell.0, cell.1).unwrap()
}

#[test]
fn create_from_a_range_with_headers() {
    let mut model = sales();
    let name = model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    assert_eq!(name, "Table1");
    let tables = model.get_tables(Some(0)).unwrap();
    assert_eq!(tables.len(), 1);
    let t = &tables[0];
    assert_eq!((t.row, t.column, t.width, t.height), (1, 1, 3, 4));
    assert!(t.header_row && !t.total_row && t.banded_rows && t.filter_button);
    assert_eq!(t.style.as_deref(), Some("TableStyleMedium2"));
    let names: Vec<&str> = t.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Item", "Qty", "Price"]);
    assert_eq!(
        model.get_table_at(0, 4, 3).unwrap().as_deref(),
        Some("Table1")
    );
    assert_eq!(model.get_table_at(0, 5, 3).unwrap(), None);

    // One undo step
    model.undo().unwrap();
    assert!(model.get_tables(None).unwrap().is_empty());
    model.redo().unwrap();
    assert_eq!(model.get_tables(None).unwrap().len(), 1);
}

#[test]
fn the_next_table_is_table2() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 6, "Name").unwrap();
    model.set_user_input(0, 2, 6, "Ann").unwrap();
    let name = model.create_table(&area(1, 6, 1, 2), true, None).unwrap();
    assert_eq!(name, "Table2");
}

#[test]
fn tables_cannot_overlap() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    assert!(model.create_table(&area(2, 2, 3, 4), true, None).is_err());
}

#[test]
fn headers_are_made_unique_and_text() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "Sales").unwrap();
    model.set_user_input(0, 1, 2, "Sales").unwrap();
    model.set_user_input(0, 1, 4, "2024").unwrap();
    model.set_user_input(0, 2, 1, "1").unwrap();
    model.create_table(&area(1, 1, 4, 2), true, None).unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    let names: Vec<&str> = t.columns.iter().map(|c| c.name.as_str()).collect();
    // A blank header takes the first free ColumnN, as a new column does
    assert_eq!(names, ["Sales", "Sales2", "Column1", "2024"]);
    assert_eq!(text(&model, (1, 2)), "Sales2");
    assert_eq!(text(&model, (1, 3)), "Column1");
    // A number header becomes text
    assert_eq!(
        model.get_cell_type(0, 1, 4).unwrap(),
        crate::types::CellType::Text
    );
}

#[test]
fn create_without_headers_inserts_a_header_row() {
    let mut model = UserModel::from_model(new_empty_model());
    model.set_user_input(0, 1, 1, "5").unwrap();
    model.set_user_input(0, 2, 1, "7").unwrap();
    model.set_user_input(0, 3, 1, "=SUM(A1:A2)").unwrap();
    model.create_table(&area(1, 1, 1, 2), false, None).unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!((t.row, t.height), (1, 3));
    assert_eq!(t.columns[0].name, "Column1");
    assert_eq!(text(&model, (1, 1)), "Column1");
    assert_eq!(text(&model, (2, 1)), "5");
    assert_eq!(text(&model, (3, 1)), "7");
    // The formula below moved down with its cells and still sums them
    assert_eq!(content(&model, (4, 1)), "=SUM(A2:A3)");
    assert_eq!(text(&model, (4, 1)), "12");

    model.undo().unwrap();
    assert_eq!(text(&model, (1, 1)), "5");
    assert_eq!(content(&model, (3, 1)), "=SUM(A1:A2)");
    assert!(model.get_tables(None).unwrap().is_empty());
}

#[test]
fn structured_references_evaluate() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 6, 1, "=SUM(Table1[Qty])").unwrap();
    model
        .set_user_input(0, 7, 1, "=ROWS(Table1[#All])")
        .unwrap();
    model
        .set_user_input(0, 8, 1, "=Table1[[#Headers],[Price]]")
        .unwrap();
    model
        .set_user_input(0, 9, 1, "=SUMPRODUCT(Table1[Qty],Table1[Price])")
        .unwrap();
    model.set_user_input(0, 10, 1, "=COUNTA(Table1)").unwrap();
    assert_eq!(text(&model, (6, 1)), "6");
    assert_eq!(text(&model, (7, 1)), "4");
    assert_eq!(text(&model, (8, 1)), "Price");
    assert_eq!(text(&model, (9, 1)), "25");
    assert_eq!(text(&model, (10, 1)), "9");
    // [@...] outside the table's rows is #VALUE!
    model.set_user_input(0, 11, 1, "=Table1[@Qty]").unwrap();
    assert_eq!(text(&model, (11, 1)), "#VALUE!");
    // ...but works on the same row, as Excel's implicit row
    model.set_user_input(0, 3, 6, "=Table1[@Qty]*2").unwrap();
    assert_eq!(text(&model, (3, 6)), "6");
    // #Totals with no total row is #REF!
    model
        .set_user_input(0, 12, 1, "=SUM(Table1[#Totals])")
        .unwrap();
    assert_eq!(text(&model, (12, 1)), "#REF!");
}

#[test]
fn a_defined_name_can_refer_to_a_table() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Prices", "Table1[Price]"))
        .unwrap();
    model.set_user_input(0, 8, 1, "=SUM(Prices)").unwrap();
    assert_eq!(text(&model, (8, 1)), "15.5");
    // The name follows the table as it grows
    model.set_user_input(0, 5, 3, "4.5").unwrap();
    assert_eq!(text(&model, (8, 1)), "20");
}

#[test]
fn calculated_column_fills_and_shows_short_form() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    // A new column by typing a header right of the table
    model.set_user_input(0, 1, 4, "Total").unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!(t.width, 4);
    assert_eq!(t.columns[3].name, "Total");
    // A formula in the empty column fills it
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    assert_eq!(text(&model, (2, 4)), "3");
    assert_eq!(text(&model, (3, 4)), "12");
    assert_eq!(text(&model, (4, 4)), "10");
    // Shown the way Excel shows it inside the table
    assert_eq!(content(&model, (3, 4)), "=[@Qty]*[@Price]");
    // and with the table name outside it
    model.set_user_input(0, 8, 1, "=Table1[@Qty]").unwrap();
    assert_eq!(content(&model, (8, 1)), "=Table1[@Qty]");
    // The fill is part of the same undo step
    model.undo().unwrap();
    assert_eq!(text(&model, (8, 1)), "");
    model.undo().unwrap();
    assert_eq!(text(&model, (2, 4)), "");
    assert_eq!(text(&model, (3, 4)), "");
}

#[test]
fn a_calculated_column_is_rewritten_together() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 4, "Total").unwrap();
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    model.set_user_input(0, 3, 4, "=[@Qty]*[@Price]*2").unwrap();
    assert_eq!(text(&model, (2, 4)), "6");
    assert_eq!(text(&model, (4, 4)), "20");
}

#[test]
fn a_typed_value_in_a_calculated_column_is_an_exception() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 4, "Total").unwrap();
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    model.set_user_input(0, 3, 4, "99").unwrap();
    assert_eq!(text(&model, (2, 4)), "3");
    assert_eq!(text(&model, (3, 4)), "99");
    assert_eq!(text(&model, (4, 4)), "10");
}

#[test]
fn grows_when_typing_below() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 4, "Total").unwrap();
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    model
        .set_user_input(0, 6, 1, "=SUM(Table1[Total])")
        .unwrap();
    assert_eq!(text(&model, (6, 1)), "25");
    // Typing in the row just below makes it a table row...
    model.set_user_input(0, 5, 2, "4").unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!(t.height, 5);
    // ...which gets the calculated column
    assert_eq!(content(&model, (5, 4)), "=[@Qty]*[@Price]");
    model.set_user_input(0, 5, 3, "2.5").unwrap();
    assert_eq!(text(&model, (5, 4)), "10");
    // and references to the column follow
    assert_eq!(text(&model, (6, 1)), "35");

    // One undo for the typed cell and the growth
    model.undo().unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].height, 4);
    assert_eq!(text(&model, (5, 4)), "");
    assert_eq!(text(&model, (6, 1)), "25");
}

#[test]
fn grows_when_typing_right() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 3, 4, "note").unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!(t.width, 4);
    assert_eq!(t.columns[3].name, "Column1");
    assert_eq!(text(&model, (1, 4)), "Column1");
    assert_eq!(text(&model, (3, 4)), "note");
}

#[test]
fn renaming_a_header_renames_references() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 6, 1, "=SUM(Table1[Qty])").unwrap();
    model.set_user_input(0, 1, 2, "Units").unwrap();
    assert_eq!(content(&model, (6, 1)), "=SUM(Table1[Units])");
    assert_eq!(text(&model, (6, 1)), "6");
    assert_eq!(model.get_tables(None).unwrap()[0].columns[1].name, "Units");
    model.undo().unwrap();
    assert_eq!(content(&model, (6, 1)), "=SUM(Table1[Qty])");
    assert_eq!(text(&model, (1, 2)), "Qty");
}

#[test]
fn rename_table() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 6, 1, "=SUM(Table1[Qty])").unwrap();
    assert!(model.rename_table("Table1", "A1").is_err());
    assert!(model.rename_table("Table1", "has space").is_err());
    model.rename_table("table1", "Sales").unwrap();
    assert_eq!(content(&model, (6, 1)), "=SUM(Sales[Qty])");
    assert_eq!(text(&model, (6, 1)), "6");
    model.undo().unwrap();
    assert_eq!(content(&model, (6, 1)), "=SUM(Table1[Qty])");
    model.redo().unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].name, "Sales");
}

#[test]
fn total_row() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model
        .set_user_input(0, 8, 1, "=SUM(Table1[Price])")
        .unwrap();
    model.set_table_total_row("Table1", true).unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert!(t.total_row);
    assert_eq!(t.height, 5);
    assert_eq!(t.columns[2].total_function.as_deref(), Some("sum"));
    assert_eq!(text(&model, (5, 1)), "Total");
    assert_eq!(content(&model, (5, 3)), "=SUBTOTAL(109,[Price])");
    assert_eq!(text(&model, (5, 3)), "15.5");
    // A column reference leaves the total row out
    assert_eq!(text(&model, (8, 1)), "15.5");

    model
        .set_table_column_total("Table1", "Qty", "average")
        .unwrap();
    assert_eq!(content(&model, (5, 2)), "=SUBTOTAL(101,[Qty])");
    assert_eq!(text(&model, (5, 2)), "2");

    // Typing below a table with a total row doesn't grow it
    model.set_user_input(0, 6, 2, "1").unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].height, 5);

    model.set_table_total_row("Table1", false).unwrap();
    assert_eq!(text(&model, (5, 1)), "");
    assert_eq!(text(&model, (5, 3)), "");
    assert_eq!(model.get_tables(None).unwrap()[0].height, 4);
    model.undo().unwrap();
    assert_eq!(text(&model, (5, 3)), "15.5");
}

#[test]
fn total_row_needs_an_empty_row() {
    let mut model = sales();
    model.set_user_input(0, 5, 3, "x").unwrap();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    assert!(model.set_table_total_row("Table1", true).is_err());
    model.set_user_input(0, 5, 3, "").unwrap();
    assert!(model.set_table_total_row("Table1", true).is_ok());
}

#[test]
fn insert_and_delete_rows_keep_references_right() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 4, "Total").unwrap();
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    model.set_user_input(0, 1, 7, "=SUM(Table1[Qty])").unwrap();
    // Insert a row inside the table: it grows and the new row gets the formula
    model.insert_rows(0, 3, 1).unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!(t.height, 5);
    assert_eq!(content(&model, (3, 4)), "=[@Qty]*[@Price]");
    model.set_user_input(0, 3, 2, "10").unwrap();
    assert_eq!(text(&model, (1, 7)), "16");
    assert_eq!(content(&model, (1, 7)), "=SUM(Table1[Qty])");
    // Insert above: the table moves
    model.insert_rows(0, 1, 2).unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].row, 3);
    assert_eq!(text(&model, (3, 7)), "16");
    // Delete a data row: it shrinks
    model.delete_rows(0, 5, 1).unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].height, 4);
    assert_eq!(text(&model, (3, 7)), "6");
    // Undo restores the exact table
    model.undo().unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].height, 5);
    assert_eq!(text(&model, (3, 7)), "16");
}

#[test]
fn insert_and_delete_columns_keep_references_right() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model
        .set_user_input(0, 1, 7, "=SUM(Table1[Price])")
        .unwrap();
    // A column inserted inside the table is a new table column
    model.insert_columns(0, 2, 1).unwrap();
    let t = &model.get_tables(None).unwrap()[0];
    assert_eq!(t.width, 4);
    assert_eq!(t.columns[1].name, "Column1");
    assert_eq!(text(&model, (1, 2)), "Column1");
    assert_eq!(text(&model, (1, 8)), "15.5");
    // Deleting a referenced column makes the reference #REF!
    model.delete_columns(0, 4, 1).unwrap();
    assert_eq!(model.get_tables(None).unwrap()[0].width, 3);
    assert_eq!(text(&model, (1, 7)), "#REF!");
    model.undo().unwrap();
    assert_eq!(text(&model, (1, 8)), "15.5");
}

#[test]
fn convert_to_range() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 1, 4, "Total").unwrap();
    model.set_user_input(0, 2, 4, "=[@Qty]*[@Price]").unwrap();
    model
        .set_user_input(0, 7, 1, "=SUM(Table1[Total])")
        .unwrap();
    model.convert_table_to_range("Table1").unwrap();
    assert!(model.get_tables(None).unwrap().is_empty());
    assert_eq!(content(&model, (3, 4)), "=$B3*$C3");
    assert_eq!(content(&model, (7, 1)), "=SUM($D$2:$D$4)");
    assert_eq!(text(&model, (7, 1)), "25");
    // No growth any more
    model.set_user_input(0, 5, 1, "x").unwrap();
    assert_eq!(content(&model, (5, 4)), "");
    model.undo().unwrap();
    model.undo().unwrap();
    assert_eq!(content(&model, (7, 1)), "=SUM(Table1[Total])");
    assert_eq!(content(&model, (3, 4)), "=[@Qty]*[@Price]");
    assert_eq!(text(&model, (7, 1)), "25");
}

#[test]
fn tables_survive_saving() {
    let mut model = sales();
    model.create_table(&area(1, 1, 3, 4), true, None).unwrap();
    model.set_user_input(0, 6, 1, "=SUM(Table1[Qty])").unwrap();
    let bytes = model.to_bytes();
    let model = UserModel::from_bytes(&bytes, "en").unwrap();
    assert_eq!(model.get_tables(None).unwrap().len(), 1);
    assert_eq!(text(&model, (6, 1)), "6");
    assert_eq!(content(&model, (6, 1)), "=SUM(Table1[Qty])");
}
