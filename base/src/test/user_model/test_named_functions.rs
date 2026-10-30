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
