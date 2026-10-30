#![allow(clippy::unwrap_used)]

use crate::model::{DefinedNameEntry, NamedFunctionEntry};
use crate::test::user_model::util::new_empty_user_model;
use crate::types::NamedFunctionParameter;

// The body as it is displayed: normalized
fn profit() -> NamedFunctionEntry {
    NamedFunctionEntry::new("PROFIT", "(revenue-cost)/revenue")
        .description("Profit margin")
        .parameters(vec![
            NamedFunctionParameter::new("revenue").description("Total revenue"),
            NamedFunctionParameter::new("cost"),
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
        .parameters(vec![NamedFunctionParameter::new("x")]);
    assert_eq!(model.get_named_function_list(), vec![expected.clone()]);

    model.undo().unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![expected]);
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
            NamedFunctionParameter::new("revenue"),
            NamedFunctionParameter::new("cost"),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .update_named_function(
            "Signum",
            None,
            NamedFunctionEntry::new("Signum", "SI(x>=0, 1, -1)")
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
    let expected =
        NamedFunctionEntry::new("Twice", "2*x").parameters(vec![NamedFunctionParameter::new("x")]);
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
                .parameters(vec![NamedFunctionParameter::new("x")])
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model.rename_sheet(0, "Data").unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Data!$A$1");
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Sheet1!$A$1");
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Data!$A$1");
}

/// Undo restores a function as it was, even if it would not be valid now:
/// here its body calls a function that no longer exists.
#[test]
fn undo_restores_function_with_unknown_names() {
    let mut model = new_empty_user_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("DOUBLE", "x*2")
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    let quadruple = NamedFunctionEntry::new("QUADRUPLE", "DOUBLE(DOUBLE(x))")
        .parameters(vec![NamedFunctionParameter::new("x")]);
    model.new_named_function(quadruple.clone()).unwrap();
    model.delete_named_function("DOUBLE", None).unwrap();
    model.delete_named_function("QUADRUPLE", None).unwrap();

    // It can't be created anymore
    assert_eq!(
        model.new_named_function(quadruple.clone()),
        Err("\"DOUBLE\" isn't a known function.".to_string())
    );
    // But undo brings it back
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![quadruple]);
}

#[test]
fn hidden_undo_redo() {
    let mut model = new_empty_user_model();
    model.new_named_function(profit().hidden(true)).unwrap();
    model.set_user_input(0, 1, 1, "=PROFIT(200, 50)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "0.75");
    assert_eq!(model.get_named_function_list(), vec![profit().hidden(true)]);

    // Unhide
    model
        .update_named_function("PROFIT", None, profit())
        .unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit().hidden(true)]);
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit()]);
    model.undo().unwrap();

    // Undoing a delete brings back a hidden function
    model.delete_named_function("PROFIT", None).unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![profit().hidden(true)]);

    // A hidden defined name that is a LAMBDA is a hidden function
    model
        .new_defined_name(DefinedNameEntry::new("Twice", "LAMBDA(x, 2*x)").hidden(true))
        .unwrap();
    model.undo().unwrap();
    model.redo().unwrap();
    let functions = model.get_named_function_list();
    assert_eq!(functions.len(), 2);
    assert!(functions[1].hidden);
}

// Undo and redo keep the body of a function in a locale whose decimal
// separator is the comma: the English `SUM(1,2)` is not read as `SUM(1.2)`.
#[test]
fn undo_redo_in_a_decimal_comma_locale() {
    let mut model = new_empty_user_model();
    model.set_locale("de").unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("S", "x+SUM(1;2)")
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model.set_user_input(0, 1, 1, "=S(0)").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "3");
    assert_eq!(model.get_named_function_list()[0].formula, "x+SUM(1;2)");

    model.undo().unwrap(); // the input
    model.undo().unwrap(); // the function
    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x+SUM(1;2)");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "3");
}

// A function whose name is a built-in function in Spanish is listed there
// with the `_xludf.` prefix, and edited by that name
#[test]
fn update_by_prefixed_name_undo_redo() {
    let mut model = new_empty_user_model();
    model
        .new_named_function(NamedFunctionEntry::new("VERDADERO", "FALSE"))
        .unwrap();
    model.set_user_input(0, 1, 1, "=VERDADERO()").unwrap();
    model.set_language("es").unwrap();

    let listed = model.get_named_function_list()[0].clone();
    assert_eq!(listed.name, "_xludf.VERDADERO");
    let edited = listed.clone().description("Edited");
    model
        .update_named_function("_xludf.VERDADERO", None, edited.clone())
        .unwrap();
    assert_eq!(model.get_named_function_list(), vec![edited.clone()]);

    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![listed.clone()]);
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![edited]);

    model
        .delete_named_function("_xludf.VERDADERO", None)
        .unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "_xludf.VERDADERO");
    assert_eq!(
        model.get_cell_content(0, 1, 1).unwrap(),
        "=_xludf.VERDADERO()"
    );
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "FALSO");
}

// A function whose name is a built-in function in English, created in
// Spanish, is listed in English with the `_xludf.` prefix
#[test]
fn builtin_in_english_undo_redo() {
    let mut model = new_empty_user_model();
    model.set_language("es").unwrap();
    model
        .new_named_function(NamedFunctionEntry::new("NOW", "7"))
        .unwrap();
    model.set_user_input(0, 1, 1, "=NOW()").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "7");

    model.undo().unwrap();
    model.undo().unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "NOW");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "7");

    model.set_language("en").unwrap();
    let listed = model.get_named_function_list()[0].clone();
    assert_eq!(listed.name, "_xludf.NOW");
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=_xludf.NOW()");

    let mut edited = listed.clone();
    edited.formula = "8".to_string();
    model
        .update_named_function("_xludf.NOW", None, edited.clone())
        .unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "8");
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![listed]);
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "7");
    model.redo().unwrap();
    assert_eq!(model.get_named_function_list(), vec![edited]);

    model.delete_named_function("_xludf.NOW", None).unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.undo().unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "_xludf.NOW");
    assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "8");
}
