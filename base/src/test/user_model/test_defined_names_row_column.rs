#![allow(clippy::unwrap_used)]

// Defined names follow the cells they point at when rows or columns are
// inserted, deleted or moved, and go back with them on undo.

use crate::model::DefinedNameEntry;
use crate::test::user_model::util::new_empty_user_model;
use crate::test::util::new_empty_model;
use crate::UserModel;

fn name_formula(model: &UserModel, name: &str) -> String {
    model
        .get_defined_name_list()
        .into_iter()
        .find(|entry| entry.name == name)
        .unwrap()
        .formula
}

// H14 = 5000, Budget = Sheet1!$H$14, Items = Sheet1!$H$14:$H$16 and
// A1 = Budget + SUM(Items)
fn model_with_names<'a>() -> UserModel<'a> {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 14, 8, "5000").unwrap();
    model.set_user_input(0, 15, 8, "1").unwrap();
    model.set_user_input(0, 16, 8, "2").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Items", "Sheet1!$H$14:$H$16"))
        .unwrap();
    model.set_user_input(0, 1, 1, "=Budget+SUM(Items)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");
    model
}

#[test]
fn insert_rows_undo_redo() {
    let mut model = model_with_names();

    model.insert_rows(0, 10, 2).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$16");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$16:$H$18");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$16");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$16:$H$18");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");
}

#[test]
fn insert_columns_undo_redo() {
    let mut model = model_with_names();

    model.insert_columns(0, 8, 3).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$K$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$K$14:$K$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$K$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$K$14:$K$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");
}

#[test]
fn delete_rows_before_the_target_undo_redo() {
    let mut model = model_with_names();

    model.delete_rows(0, 3, 4).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$10");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$10:$H$12");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$10");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$10:$H$12");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");
}

// Deleting the cell a name points at leaves the name with a #REF!, and
// inserting the row again cannot tell where it was: undo has to put it back
#[test]
fn delete_the_row_of_the_target_undo_redo() {
    let mut model = model_with_names();

    model.delete_rows(0, 14, 1).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "#REF!");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#REF!");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 14, 8).unwrap(), "5000");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "#REF!");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#REF!");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");
}

#[test]
fn delete_the_column_of_the_target_undo_redo() {
    let mut model = model_with_names();

    model.delete_columns(0, 7, 3).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "#REF!");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#REF!");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 14, 8).unwrap(), "5000");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "#REF!");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#REF!");
}

#[test]
fn move_rows_and_columns_undo_redo() {
    let mut model = model_with_names();

    model.move_rows_action(0, 14, 1, 4).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$18");
    model.move_columns_action(0, 8, 1, -2).unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$F$18");
    assert_eq!(model.get_formatted_cell_value(0, 18, 6).unwrap(), "5000");

    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$18");
    model.undo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$H$14");
    assert_eq!(name_formula(&model, "Items"), "Sheet1!$H$14:$H$16");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10003");

    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(name_formula(&model, "Budget"), "Sheet1!$F$18");
}

// The changes reach another model through the queue of diffs
#[test]
fn diffs_sent_to_another_model() {
    let mut model1 = UserModel::from_model(new_empty_model());
    model1.set_user_input(0, 14, 8, "5000").unwrap();
    model1
        .new_defined_name(DefinedNameEntry::new("Budget", "Sheet1!$H$14"))
        .unwrap();
    model1
        .new_defined_name(DefinedNameEntry::new("Gone", "Sheet1!$B$2"))
        .unwrap();
    model1.set_user_input(0, 1, 1, "=Budget").unwrap();
    model1.insert_rows(0, 10, 2).unwrap();
    model1.insert_columns(0, 3, 1).unwrap();
    model1.delete_rows(0, 2, 1).unwrap();
    model1.undo().unwrap();
    model1.redo().unwrap();
    let send_queue = model1.flush_send_queue();

    let mut model2 = UserModel::from_model(new_empty_model());
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(name_formula(&model1, "Budget"), "Sheet1!$I$15");
    assert_eq!(name_formula(&model1, "Gone"), "#REF!");
    assert_eq!(
        model2.get_defined_name_list(),
        model1.get_defined_name_list()
    );
    assert_eq!(model2.get_formatted_cell_value(0, 1, 1).unwrap(), "5000");
}

// The formula of a name is stored in English whatever the language is
#[test]
fn names_in_another_language() {
    let mut model = new_empty_user_model();
    model.set_language("es").unwrap();
    model.set_locale("es").unwrap();
    model.set_user_input(0, 14, 8, "5000").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new(
            "Suma",
            "=SUMA(Sheet1!$H$14:$H$16;1,5)",
        ))
        .unwrap();
    model.set_user_input(0, 1, 1, "=Suma").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "5001,5");

    model.insert_rows(0, 2, 2).unwrap();
    assert_eq!(
        model.model.workbook.defined_names[0].formula,
        "=SUM(Sheet1!$H$16:$H$18,1.5)"
    );
    assert_eq!(
        name_formula(&model, "Suma"),
        "=SUMA(Sheet1!$H$16:$H$18;1,5)"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "5001,5");

    // The rows of the whole range are deleted, and come back
    model.delete_rows(0, 16, 3).unwrap();
    assert!(model.model.workbook.defined_names[0]
        .formula
        .contains("#REF!"));
    model.undo().unwrap();
    assert_eq!(
        name_formula(&model, "Suma"),
        "=SUMA(Sheet1!$H$16:$H$18;1,5)"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "5001,5");

    model.undo().unwrap();
    assert_eq!(
        name_formula(&model, "Suma"),
        "=SUMA(Sheet1!$H$14:$H$16;1,5)"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "5001,5");
}
