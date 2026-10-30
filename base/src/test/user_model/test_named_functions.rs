#![allow(clippy::unwrap_used)]

use crate::model::{DefinedNameEntry, NamedFunctionEntry, NamedFunctionParameterEntry};
use crate::test::user_model::util::new_empty_user_model;

// The body as it is displayed: normalized
fn profit() -> NamedFunctionEntry {
    NamedFunctionEntry::new("PROFIT", "(revenue-cost)/revenue")
        .description("Profit margin")
        .parameters(vec![
            NamedFunctionParameterEntry::new("revenue").description("Total revenue"),
            NamedFunctionParameterEntry::new("cost"),
        ])
}

#[test]
fn create_undo_redo() {
    let mut model = new_empty_user_model();
    model.new_named_function(profit()).unwrap();
    model.set_user_input(0, 1, 1, "=PROFIT(200, 50)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");

    model.undo().unwrap(); // the input
    model.undo().unwrap(); // the function
    assert!(model.get_named_function_list().is_empty());

    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    model.redo().unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");
}

#[test]
fn delete_undo_redo() {
    let mut model = new_empty_user_model();
    model.new_named_function(profit()).unwrap();
    model.set_user_input(0, 1, 1, "=PROFIT(200, 50)").unwrap();

    assert_eq!(
        model.delete_named_function("PROFIT", Some(0)),
        Err("Named function not found".to_string())
    );
    model.delete_named_function("PROFIT", None).unwrap();
    assert!(model.get_named_function_list().is_empty());
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#NAME?");

    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");

    model.redo().unwrap();
    assert!(model.get_named_function_list().is_empty());
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#NAME?");
}

#[test]
fn lambda_defined_name_undo_redo() {
    let mut model = new_empty_user_model();
    model
        .new_defined_name(DefinedNameEntry::new("Twice", "=LAMBDA(x, 2*x)").comment("Doubles"))
        .unwrap();
    assert!(model.get_defined_name_list().is_empty());
    let expected = NamedFunctionEntry::new("Twice", "2*x")
        .description("Doubles")
        .parameters(vec![NamedFunctionParameterEntry::new("x")]);
    assert_eq!(model.get_named_function_list(), vec![expected.clone()]);

    model.undo().unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![expected]);
}

#[test]
fn validation() {
    let mut model = new_empty_user_model();
    assert_eq!(
        model.is_valid_named_function("SUM", None, &[], "1"),
        Err("Name: SUM is already a built-in function".to_string())
    );
    assert_eq!(
        model.is_valid_named_function("Fn", None, &[NamedFunctionParameterEntry::new("x")], "x"),
        Ok(None)
    );
    assert!(model.get_named_function_list().is_empty());
}

#[test]
fn update_undo_redo() {
    let mut model = new_empty_user_model();
    model.new_named_function(profit()).unwrap();
    model.set_user_input(0, 1, 1, "=PROFIT(200, 50)").unwrap();
    model.set_user_input(0, 2, 1, "=MAP(A1, PROFIT)").unwrap();

    let margin = NamedFunctionEntry::new("MARGIN", "(revenue-cost)/revenue")
        .description("Renamed")
        .parameters(vec![
            NamedFunctionParameterEntry::new("revenue"),
            NamedFunctionParameterEntry::new("cost"),
        ]);
    model
        .update_named_function("PROFIT", None, margin.clone())
        .unwrap();
    assert_eq!(model.get_named_function_list(), vec![margin.clone()]);
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=MARGIN(200,50)");
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "=MAP(A1,MARGIN)");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");

    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=PROFIT(200,50)");
    assert_eq!(model.get_cell_content(0, 2, 1).unwrap(), "=MAP(A1,PROFIT)");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");

    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![margin]);
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=MARGIN(200,50)");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");
}

#[test]
fn update_in_spanish_undo_in_english() {
    let mut model = new_empty_user_model();
    model.set_language("es").unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("Signum", "SI(x>0, 1, -1)")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model
        .update_named_function(
            "Signum",
            None,
            NamedFunctionEntry::new("Signum", "SI(x>=0, 1, -1)")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model.set_user_input(0, 1, 1, "=Signum(0)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1");

    model.set_language("en").unwrap();
    model.undo().unwrap(); // the input
    model.undo().unwrap(); // the update
    assert_eq!(model.get_named_function_list()[0].formula, "IF(x>0,1,-1)");
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "IF(x>=0,1,-1)");
}

#[test]
fn defined_name_updated_to_lambda_undo_redo() {
    let mut model = new_empty_user_model();
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$A$1").comment("A rate"))
        .unwrap();
    model
        .update_defined_name(
            "Rate",
            None,
            DefinedNameEntry::new("Twice", "=LAMBDA(x, 2*x)"),
        )
        .unwrap();
    assert!(model.get_defined_name_list().is_empty());
    let expected = NamedFunctionEntry::new("Twice", "2*x")
        .parameters(vec![NamedFunctionParameterEntry::new("x")]);
    assert_eq!(model.get_named_function_list(), vec![expected.clone()]);

    model.undo().unwrap();
    assert!(model.get_named_function_list().is_empty());
    let names = model.get_defined_name_list();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "Rate");
    assert_eq!(names[0].formula, "Sheet1!$A$1");
    assert_eq!(names[0].comment, "A rate");

    model.redo().unwrap();
    assert!(model.get_defined_name_list().is_empty());
    assert_eq!(model.get_named_function_list(), vec![expected]);
}

#[test]
fn check_and_usage() {
    let mut model = new_empty_user_model();
    model.new_named_function(profit()).unwrap();
    model.set_user_input(0, 1, 1, "=PROFIT(200, 50)").unwrap();
    model.set_user_input(0, 2, 1, "=PROFIT(300, 50)").unwrap();
    assert_eq!(model.named_function_usage_count("PROFIT", None), Ok(2));

    let check = model.check_named_function(&profit(), None);
    assert_eq!(
        check.name,
        Some("A function or defined name called PROFIT already exists.".to_string())
    );
    let check = model.check_named_function(&profit(), Some(("PROFIT", None)));
    assert!(check.is_valid());
}

#[test]
fn delete_sheet_undo_redo() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    model.new_named_function(profit()).unwrap();
    model
        .new_named_function(profit().name("Local").scope(1))
        .unwrap();
    model.set_user_input(1, 1, 1, "=Local(200, 50)").unwrap();
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "0.75");

    model.delete_sheet(1).unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);

    model.undo().unwrap();
    assert_eq!(
        model.get_named_function_list(),
        vec![profit(), profit().name("Local").scope(1)]
    );
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "0.75");

    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    assert_eq!(model.get_worksheets_properties().len(), 1);
}

#[test]
fn duplicate_sheet_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "10").unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("local_fn", "x+Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameterEntry::new("x")])
                .scope(0),
        )
        .unwrap();
    model.new_named_function(profit()).unwrap();
    model.set_user_input(0, 1, 2, "=local_fn(1)").unwrap();

    model.duplicate_sheet(0).unwrap();
    assert_eq!(model.get_named_function_list().len(), 3);
    assert_eq!(model.get_formatted_cell_value(1, 1, 2).unwrap(), "11");

    model.undo().unwrap();
    assert_eq!(model.get_worksheets_properties().len(), 1);
    assert_eq!(model.get_named_function_list().len(), 2);

    model.redo().unwrap();
    assert_eq!(model.get_worksheets_properties().len(), 2);
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 3);
    assert_eq!(list[2].scope, Some(1));
    assert_eq!(list[2].formula, "x+'Sheet1 (1)'!$A$1");
    assert_eq!(model.get_formatted_cell_value(1, 1, 2).unwrap(), "11");
}

#[test]
fn rename_sheet_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "10").unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("Scaled", "x*Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model.rename_sheet(0, "Data").unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Data!$A$1");
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Sheet1!$A$1");
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Data!$A$1");
}
