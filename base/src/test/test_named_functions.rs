#![allow(clippy::unwrap_used)]

use crate::model::{DefinedNameEntry, NamedFunctionEntry, NamedFunctionParameterEntry};
use crate::test::util::new_empty_model;
use crate::Model;

fn profit() -> NamedFunctionEntry {
    NamedFunctionEntry::new("PROFIT", "=(revenue - cost) / revenue")
        .description("Profit margin")
        .parameters(vec![
            NamedFunctionParameterEntry::new("revenue").description("Total revenue"),
            NamedFunctionParameterEntry::new("cost"),
        ])
}

#[test]
fn create_and_call() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();
    model._set("A1", "=PROFIT(200, 50)");
    model._set("A2", "=profit(200, 50)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.75");
    assert_eq!(model._get_text("A2"), *"0.75");

    // The list shows what was entered, the body without the leading '='
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "PROFIT");
    assert_eq!(list[0].scope, None);
    assert_eq!(list[0].description, "Profit margin");
    assert_eq!(list[0].formula, "(revenue-cost)/revenue");
    assert_eq!(list[0].parameters.len(), 2);
    assert_eq!(list[0].parameters[0].name, "revenue");
    assert_eq!(list[0].parameters[0].description, "Total revenue");
    assert!(!list[0].parameters[0].optional);
    assert_eq!(list[0].parameters[1].name, "cost");
    // Stored without the '='
    assert_eq!(
        model.workbook.named_functions[0].formula,
        "(revenue-cost)/revenue"
    );
    // Not a defined name
    assert!(model.get_defined_name_list().is_empty());
}

#[test]
fn wrong_argument_count_is_value_error() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();
    model._set("A1", "=PROFIT(200)");
    model._set("A2", "=PROFIT(200, 50, 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}

#[test]
fn optional_parameters() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("ADD_VAT", "net * (1 + IF(ISOMITTED(rate), 0.2, rate))")
                .parameters(vec![
                    NamedFunctionParameterEntry::new("net"),
                    NamedFunctionParameterEntry::new("rate").optional(true),
                ]),
        )
        .unwrap();
    model._set("A1", "=ADD_VAT(100)");
    model._set("A2", "=ADD_VAT(100, 0.1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"120");
    assert_eq!(model._get_text("A2"), *"110");
}

#[test]
fn no_parameters() {
    let mut model = new_empty_model();
    model
        .new_named_function(NamedFunctionEntry::new("FORTY_TWO", "6*7"))
        .unwrap();
    model._set("A1", "=FORTY_TWO()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"42");
}

#[test]
fn recursion() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("FACTORIAL", "IF(x > 0, x * FACTORIAL(x - 1), 1)")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model._set("A1", "=FACTORIAL(5)");
    model._set("A2", "=FACTORIAL(0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"120");
    assert_eq!(model._get_text("A2"), *"1");
}

#[test]
fn functions_calling_functions() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("DOUBLE", "x * 2")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("QUADRUPLE", "DOUBLE(DOUBLE(x))")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model._set("A1", "=QUADRUPLE(3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"12");
}

#[test]
fn bare_name_is_a_lambda_value() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("DOUBLE", "x * 2")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("A1", "=SUM(MAP(B1:B3, DOUBLE))");
    model._set("A2", "=DOUBLE");
    model._set("A3", "=Sheet1!DOUBLE(4)");
    model._set("A4", "=ISREF(DOUBLE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"12");
    // A lambda value in a cell, like `=LAMBDA(x, x)`
    assert_eq!(model._get_text("A2"), *"#CALC!");
    assert_eq!(model._get_text("A3"), *"8");
    assert_eq!(model._get_text("A4"), *"FALSE");
}

#[test]
fn sheet_local_function_takes_precedence_over_global() {
    let mut model = new_empty_model();
    model.new_sheet(); // Sheet2 at index 1
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x * 2")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x * 3")
                .parameters(vec![NamedFunctionParameterEntry::new("x")])
                .scope(1),
        )
        .unwrap();
    model._set("A1", "=MyFn(5)");
    model._set("Sheet2!A1", "=MyFn(5)");
    model._set("A2", "=Sheet2!MyFn(5)");
    model._set("Sheet2!A2", "=Sheet1!MyFn(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"10");
    assert_eq!(model._get_text("Sheet2!A1"), *"15");
    assert_eq!(model._get_text("A2"), *"15");
    // Sheet1 has no local MyFn: the workbook one
    assert_eq!(model._get_text("Sheet2!A2"), *"10");

    let list = model.get_named_function_list();
    assert_eq!(list[0].scope, None);
    assert_eq!(list[1].scope, Some(1));
}

#[test]
fn body_can_use_defined_names() {
    let mut model = new_empty_model();
    model._set("B1", "0.2");
    model
        .new_defined_name(DefinedNameEntry::new("VAT", "Sheet1!$B$1"))
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("WITH_VAT", "net * (1 + VAT)")
                .parameters(vec![NamedFunctionParameterEntry::new("net")]),
        )
        .unwrap();
    model._set("A1", "=WITH_VAT(100)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"120");

    model._set("B1", "0.5");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"150");
}

#[test]
fn delete() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();
    model._set("A1", "=PROFIT(200, 50)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.75");

    assert_eq!(
        model.delete_named_function("profit", Some(0)),
        Err("Named function not found".to_string())
    );
    model.delete_named_function("profit", None).unwrap();
    assert!(model.get_named_function_list().is_empty());
    assert_eq!(model._get_text("A1"), *"#NAME?");
}

#[test]
fn lambda_defined_name_is_a_named_function() {
    let mut model = new_empty_model();
    model
        .new_defined_name(
            DefinedNameEntry::new(
                "Profit",
                "=LAMBDA(revenue, [cost], (revenue - cost) / revenue)",
            )
            .comment("Profit margin"),
        )
        .unwrap();
    assert!(model.get_defined_name_list().is_empty());
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Profit");
    assert_eq!(list[0].description, "Profit margin");
    assert_eq!(list[0].formula, "(revenue-cost)/revenue");
    assert_eq!(list[0].parameters.len(), 2);
    assert_eq!(list[0].parameters[0].name, "revenue");
    assert!(!list[0].parameters[0].optional);
    assert_eq!(list[0].parameters[1].name, "cost");
    assert!(list[0].parameters[1].optional);

    model._set("A1", "=Profit(200, 50)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.75");

    // Scoped
    model.new_sheet();
    model
        .new_defined_name(DefinedNameEntry::new("Twice", "=LAMBDA(x, 2*x)").scope(1))
        .unwrap();
    let list = model.get_named_function_list();
    assert_eq!(list[1].name, "Twice");
    assert_eq!(list[1].scope, Some(1));
    assert_eq!(list[1].formula, "2*x");
}

#[test]
fn names_and_functions_share_a_namespace() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Margin", "Sheet1!$A$1"))
        .unwrap();

    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("profit", "Sheet1!$A$1")),
        Err("Name: Defined name already exists".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("margin", "1")),
        Err("Name: A function or defined name with this name already exists".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Profit", "1")),
        Err("Name: A function or defined name with this name already exists".to_string())
    );
    // A different scope is fine
    model
        .new_named_function(NamedFunctionEntry::new("Profit", "1").scope(0))
        .unwrap();
    assert_eq!(model.get_named_function_list().len(), 2);
}

#[test]
fn validation() {
    let mut model = new_empty_model();
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("SUM", "1")),
        Err("Name: SUM is already a built-in function".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("sum", "1")),
        Err("Name: SUM is already a built-in function".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("A1", "1")),
        Err("Name: Invalid function name".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("My Fn", "1")),
        Err("Name: Invalid function name".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("", "1")),
        Err("Name: Invalid function name".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "1").scope(3)),
        Err("Scope: Invalid sheet index".to_string())
    );
    assert_eq!(
        model.new_named_function(
            NamedFunctionEntry::new("Fn", "x")
                .parameters(vec![NamedFunctionParameterEntry::new("B2")])
        ),
        Err("Parameters: Invalid parameter name 'B2'".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "x").parameters(vec![
            NamedFunctionParameterEntry::new("x"),
            NamedFunctionParameterEntry::new("X")
        ])),
        Err("Parameters: Duplicate parameter name 'X'".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "1 +")),
        Err("Formula: Invalid named function formula".to_string())
    );
    // References must be qualified with a sheet, like in defined names
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "A1 + 1")),
        Err("Formula: Invalid named function formula".to_string())
    );
    model
        .new_named_function(NamedFunctionEntry::new("Fn", "Sheet1!A1 + 1"))
        .unwrap();

    let long = "x".repeat(256);
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn2", "1").description(&long)),
        Err("Description: Description must be at most 255 characters".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn2", "x").parameters(vec![
            NamedFunctionParameterEntry::new("x").description(&long)
        ])),
        Err("Description: Description must be at most 255 characters".to_string())
    );
    assert_eq!(model.get_named_function_list().len(), 1);
}

#[test]
fn body_in_the_active_language() {
    let mut model = new_empty_model();
    model.set_language("es").unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("Signum", "SI(x>0, 1, -1)")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    // Stored in English
    assert_eq!(model.workbook.named_functions[0].formula, "IF(x>0,1,-1)");
    // Displayed in Spanish
    assert_eq!(model.get_named_function_list()[0].formula, "SI(x>0,1,-1)");
    model._set("A1", "=Signum(-3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"-1");
}

#[test]
fn round_trip_through_bytes() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_named_function(profit().scope(1)).unwrap();
    model._set("Sheet2!A1", "=PROFIT(200, 50)");
    model.evaluate();

    let bytes = model.to_bytes();
    let mut model2 = Model::from_bytes(&bytes, "en").unwrap();
    model2.evaluate();
    assert_eq!(model2._get_text("Sheet2!A1"), *"0.75");
    assert_eq!(
        model2.get_named_function_list(),
        model.get_named_function_list()
    );
}
