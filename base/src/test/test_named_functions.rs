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

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

fn double() -> NamedFunctionEntry {
    NamedFunctionEntry::new("DOUBLE", "x*2").parameters(vec![NamedFunctionParameterEntry::new("x")])
}

#[test]
fn update_fields() {
    let mut model = new_empty_model();
    model.new_named_function(double()).unwrap();
    model._set("A1", "=DOUBLE(4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8");

    // Same name, new everything else
    model
        .update_named_function(
            "double",
            None,
            NamedFunctionEntry::new("DOUBLE", "n * factor")
                .description("Scales")
                .parameters(vec![
                    NamedFunctionParameterEntry::new("n"),
                    NamedFunctionParameterEntry::new("factor")
                        .description("The factor")
                        .optional(true),
                ]),
        )
        .unwrap();
    model._set("A2", "=DOUBLE(4, 3)");
    model.evaluate();
    // `factor` is optional: omitted it is empty, and 4 * empty is 0
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"12");
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].description, "Scales");
    assert_eq!(list[0].formula, "n*factor");
    assert_eq!(list[0].parameters[1].description, "The factor");
    assert!(list[0].parameters[1].optional);

    assert_eq!(
        model.update_named_function("nope", None, double()),
        Err("Named function not found".to_string())
    );
}

#[test]
fn rename_propagates_to_cells() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_named_function(double()).unwrap();
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("A1", "=DOUBLE(4)");
    model._set("A2", "=SUM(MAP(B1:B2, DOUBLE))");
    model._set("A3", "=Sheet1!DOUBLE(5)");
    model._set("Sheet2!A1", "=double(1)+Sheet1!double(2)");
    // Bound variables with the same name are not touched
    model._set("A4", "=LET(DOUBLE, 3, DOUBLE*2)");
    model._set("A5", "=LAMBDA(DOUBLE, DOUBLE+1)(1)");
    model._set("A6", "=LET(f, LAMBDA(x, x), f(DOUBLE(3)))");
    model.evaluate();

    model
        .update_named_function("DOUBLE", None, double().name("TWICE"))
        .unwrap();

    assert_eq!(model._get_formula("A1"), "=TWICE(4)");
    assert_eq!(model._get_formula("A2"), "=SUM(MAP(B1:B2,TWICE))");
    assert_eq!(model._get_formula("A3"), "=Sheet1!TWICE(5)");
    assert_eq!(model._get_formula("Sheet2!A1"), "=TWICE(1)+Sheet1!TWICE(2)");
    assert_eq!(model._get_formula("A4"), "=LET(DOUBLE,3,DOUBLE*2)");
    assert_eq!(model._get_formula("A5"), "=LAMBDA(DOUBLE,DOUBLE+1)(1)");
    assert_eq!(model._get_formula("A6"), "=LET(f,LAMBDA(x,x),f(TWICE(3)))");
    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"6");
    assert_eq!(model._get_text("A3"), *"10");
    assert_eq!(model._get_text("Sheet2!A1"), *"6");
    assert_eq!(model._get_text("A4"), *"6");
    assert_eq!(model._get_text("A6"), *"6");
}

#[test]
fn rename_propagates_to_bodies() {
    let mut model = new_empty_model();
    model.new_named_function(double()).unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("QUADRUPLE", "DOUBLE(DOUBLE(x))")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("FACTORIAL", "IF(n>0, n*FACTORIAL(n-1), 1)")
                .parameters(vec![NamedFunctionParameterEntry::new("n")]),
        )
        .unwrap();
    model._set("A1", "=QUADRUPLE(3)");
    model._set("A2", "=FACTORIAL(4)");
    model.evaluate();

    model
        .update_named_function("DOUBLE", None, double().name("TWICE"))
        .unwrap();
    // The body of a recursive function is renamed too, even when it is
    // passed with the old name
    model
        .update_named_function(
            "FACTORIAL",
            None,
            NamedFunctionEntry::new("FACT2", "IF(n>0, n*FACTORIAL(n-1), 1)")
                .parameters(vec![NamedFunctionParameterEntry::new("n")]),
        )
        .unwrap();

    let list = model.get_named_function_list();
    assert_eq!(list[0].name, "TWICE");
    assert_eq!(list[1].formula, "TWICE(TWICE(x))");
    assert_eq!(list[2].name, "FACT2");
    assert_eq!(list[2].formula, "IF(n>0,n*FACT2(n-1),1)");
    model._set("A3", "=FACT2(3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"12");
    assert_eq!(model._get_text("A2"), *"24");
    assert_eq!(model._get_text("A3"), *"6");
}

#[test]
fn rename_respects_scope() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_sheet();
    // A global MyFn and one local to Sheet2
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x*2")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x*3")
                .parameters(vec![NamedFunctionParameterEntry::new("x")])
                .scope(1),
        )
        .unwrap();
    model._set("A1", "=MyFn(1)");
    model._set("Sheet2!A1", "=MyFn(1)");
    model._set("Sheet3!A1", "=Sheet2!MyFn(1)+Sheet1!MyFn(1)");

    // Rename the global one: Sheet2 keeps calling its local function
    model
        .update_named_function(
            "MyFn",
            None,
            NamedFunctionEntry::new("Global", "x*2")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=Global(1)");
    assert_eq!(model._get_formula("Sheet2!A1"), "=MyFn(1)");
    assert_eq!(
        model._get_formula("Sheet3!A1"),
        "=Sheet2!MyFn(1)+Sheet1!Global(1)"
    );

    // Rename the local one
    model
        .update_named_function(
            "MyFn",
            Some(1),
            NamedFunctionEntry::new("Local", "x*3")
                .parameters(vec![NamedFunctionParameterEntry::new("x")])
                .scope(1),
        )
        .unwrap();
    assert_eq!(model._get_formula("Sheet2!A1"), "=Local(1)");
    assert_eq!(
        model._get_formula("Sheet3!A1"),
        "=Sheet2!Local(1)+Sheet1!Global(1)"
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("Sheet2!A1"), *"3");
    assert_eq!(model._get_text("Sheet3!A1"), *"5");
}

#[test]
fn change_scope() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_named_function(double()).unwrap();
    model._set("Sheet2!A1", "=DOUBLE(2)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!A1"), *"4");

    model
        .update_named_function("DOUBLE", None, double().scope(0))
        .unwrap();
    let list = model.get_named_function_list();
    assert_eq!(list[0].scope, Some(0));
    assert_eq!(model._get_text("Sheet2!A1"), *"#NAME?");
    assert_eq!(
        model.update_named_function("DOUBLE", None, double()),
        Err("Named function not found".to_string())
    );
}

#[test]
fn update_validation() {
    let mut model = new_empty_model();
    model.new_named_function(double()).unwrap();
    model
        .new_named_function(NamedFunctionEntry::new("Other", "1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Margin", "Sheet1!$A$1"))
        .unwrap();
    let taken = Err("Name: A function or defined name with this name already exists".to_string());
    assert_eq!(
        model.update_named_function("DOUBLE", None, double().name("other")),
        taken
    );
    assert_eq!(
        model.update_named_function("DOUBLE", None, double().name("margin")),
        taken
    );
    // Changing the case of its own name is fine
    model
        .update_named_function("DOUBLE", None, double().name("Double"))
        .unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "Double");
    assert_eq!(
        model.update_named_function("Double", None, double().name("SUM")),
        Err("Name: SUM is already a built-in function".to_string())
    );
    assert_eq!(
        model.update_named_function("Double", None, double().scope(5)),
        Err("Scope: Invalid sheet index".to_string())
    );
    assert_eq!(
        model.update_named_function(
            "Double",
            None,
            NamedFunctionEntry::new("Double", "x").parameters(vec![
                NamedFunctionParameterEntry::new("x"),
                NamedFunctionParameterEntry::new("x")
            ])
        ),
        Err("Parameters: Duplicate parameter name 'x'".to_string())
    );
    assert_eq!(
        model.update_named_function("Double", None, NamedFunctionEntry::new("Double", "A1")),
        Err("Formula: Invalid named function formula".to_string())
    );
    let long = "x".repeat(256);
    assert_eq!(
        model.update_named_function("Double", None, double().description(&long)),
        Err("Description: Description must be at most 255 characters".to_string())
    );
    // Nothing changed
    assert_eq!(model.get_named_function_list()[0].formula, "x*2");
}

#[test]
fn defined_name_updated_to_lambda_is_a_named_function() {
    let mut model = new_empty_model();
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$A$1"))
        .unwrap();
    model
        .update_defined_name(
            "Rate",
            None,
            DefinedNameEntry::new("Twice", "=LAMBDA(x, 2*x)").comment("Doubles"),
        )
        .unwrap();
    assert!(model.get_defined_name_list().is_empty());
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Twice");
    assert_eq!(list[0].description, "Doubles");
    assert_eq!(list[0].formula, "2*x");
    model._set("A1", "=Twice(21)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"42");

    // A defined name cannot take the name of a function
    model
        .new_defined_name(DefinedNameEntry::new("Other", "Sheet1!$A$1"))
        .unwrap();
    assert_eq!(
        model.update_defined_name("Other", None, DefinedNameEntry::new("twice", "Sheet1!$A$1")),
        Err("Name: Defined name already exists".to_string())
    );
}

#[test]
fn renamed_defined_name_is_renamed_in_bodies() {
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

    model
        .update_defined_name("VAT", None, DefinedNameEntry::new("TAX", "Sheet1!$B$1"))
        .unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "net*(1+TAX)");
    assert_eq!(model._get_text("A1"), *"120");
}

// ---------------------------------------------------------------------------
// Check
// ---------------------------------------------------------------------------

use crate::model::NamedFunctionCheck;

fn ok_check(parameter_count: usize) -> NamedFunctionCheck {
    NamedFunctionCheck {
        parameters: vec![None; parameter_count],
        ..Default::default()
    }
}

#[test]
fn check_valid() {
    let mut model = new_empty_model();
    let check = model.check_named_function(&profit(), None);
    assert_eq!(check, ok_check(2));
    assert!(check.is_valid());

    // A recursive function, with no parameters used as a lambda, in Spanish
    model.set_language("es").unwrap();
    let check = model.check_named_function(
        &NamedFunctionEntry::new("FACTORIAL", "SI(n>0, n*FACTORIAL(n-1), 1)")
            .parameters(vec![NamedFunctionParameterEntry::new("n")]),
        None,
    );
    assert_eq!(check, ok_check(1));
}

#[test]
fn check_name() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Margin", "Sheet1!$A$1"))
        .unwrap();
    let cases = [
        ("", "Name is required."),
        ("  ", "Name is required."),
        ("AB12", "This name looks like a cell reference (like AB12)."),
        ("R1C1", "This name looks like a cell reference (like AB12)."),
        (
            "My Fn",
            "Use letters, numbers, and underscores, starting with a letter.",
        ),
        (
            "2fn",
            "Use letters, numbers, and underscores, starting with a letter.",
        ),
        ("sum", "SUM is already a built-in function."),
        (
            "profit",
            "A function or defined name called PROFIT already exists.",
        ),
        (
            "MARGIN",
            "A function or defined name called MARGIN already exists.",
        ),
    ];
    for (name, message) in cases {
        let check = model.check_named_function(&NamedFunctionEntry::new(name, "1"), None);
        assert_eq!(check.name, Some(message.to_string()), "{name}");
        assert!(!check.is_valid());
    }
    // Its own name while editing is fine, in any case; another scope is a
    // different function
    let check = model.check_named_function(
        &NamedFunctionEntry::new("profit", "1"),
        Some(("PROFIT", None)),
    );
    assert_eq!(check.name, None);
    let check = model.check_named_function(&NamedFunctionEntry::new("profit", "1").scope(0), None);
    assert_eq!(check.name, None);
    let check = model.check_named_function(&NamedFunctionEntry::new("Fn", "1").scope(7), None);
    assert_eq!(
        check.name,
        Some("The sheet of the function does not exist.".to_string())
    );
}

#[test]
fn check_parameters() {
    let mut model = new_empty_model();
    model
        .new_defined_name(DefinedNameEntry::new("VAT", "Sheet1!$A$1"))
        .unwrap();
    let entry = NamedFunctionEntry::new("Fn", "a + b").parameters(vec![
        NamedFunctionParameterEntry::new("a"),
        NamedFunctionParameterEntry::new(" "),
        NamedFunctionParameterEntry::new("B2"),
        NamedFunctionParameterEntry::new("my arg"),
        NamedFunctionParameterEntry::new("A"),
        NamedFunctionParameterEntry::new("vat"),
        NamedFunctionParameterEntry::new("b"),
    ]);
    let check = model.check_named_function(&entry, None);
    assert_eq!(
        check.parameters,
        vec![
            None,
            Some("Each argument needs a name.".to_string()),
            Some("This name looks like a cell reference. Try another.".to_string()),
            Some(
                "Argument names must start with a letter and contain only letters, numbers, and underscores."
                    .to_string()
            ),
            Some("Each argument needs a different name.".to_string()),
            Some("A defined name called vat already exists. Try another.".to_string()),
            None,
        ]
    );
    assert!(check.formula.is_empty());
    // The invalid ones are not used, the valid ones are
    assert_eq!(
        check.warnings,
        vec![
            "\"B2\" isn't used in the formula.".to_string(),
            "\"my arg\" isn't used in the formula.".to_string(),
            "\"A\" isn't used in the formula.".to_string(),
            "\"vat\" isn't used in the formula.".to_string(),
        ]
    );
    assert!(!check.is_valid());
}

#[test]
fn check_formula() {
    let mut model = new_empty_model();
    model.new_named_function(double()).unwrap();
    let with = |formula: &str| {
        NamedFunctionEntry::new("PROFIT", formula).parameters(vec![
            NamedFunctionParameterEntry::new("revenue"),
            NamedFunctionParameterEntry::new("cost"),
        ])
    };

    let check = model.check_named_function(&with(""), None);
    assert_eq!(check.formula, vec!["Formula is required.".to_string()]);
    let check = model.check_named_function(&with(" = "), None);
    assert_eq!(check.formula, vec!["Formula is required.".to_string()]);

    let check = model.check_named_function(&with("revenue -"), None);
    assert_eq!(
        check.formula,
        vec!["There's an error in this formula.".to_string()]
    );
    assert!(check.warnings.is_empty());

    let check = model.check_named_function(&with("revenue - cost + A1"), None);
    assert_eq!(
        check.formula,
        vec!["References in the formula need a sheet (like Sheet1!A1).".to_string()]
    );

    // Unknown names are errors and chips in red; unused parameters are warnings
    let check = model.check_named_function(&with("(revenue - coste) / price"), None);
    assert_eq!(
        check.formula,
        vec![
            "\"coste\" isn't an argument. Add it above or check the spelling.".to_string(),
            "\"price\" isn't an argument. Add it above or check the spelling.".to_string(),
        ]
    );
    assert_eq!(check.unknown_names, vec!["coste", "price"]);
    assert_eq!(
        check.warnings,
        vec!["\"cost\" isn't used in the formula.".to_string()]
    );
    assert!(!check.is_valid());

    // Parameters are case-sensitive, like the evaluation
    let check = model.check_named_function(&with("Revenue - cost"), None);
    assert_eq!(check.unknown_names, vec!["Revenue"]);

    // Functions: known, itself, a parameter used as a function, unknown
    let check = model.check_named_function(
        &with("DOUBLE(revenue) + PROFIT(revenue, cost) + cost(1) + NOPE(2)"),
        None,
    );
    assert_eq!(
        check.formula,
        vec!["\"NOPE\" isn't a known function.".to_string()]
    );
    assert_eq!(check.unknown_names, vec!["NOPE"]);
    assert!(check.warnings.is_empty());

    // Functions used as values, LET and LAMBDA variables are fine
    let check = model.check_named_function(
        &with("LET(x, revenue, SUM(MAP(x, DOUBLE)) + LAMBDA(y, y + cost)(1))"),
        None,
    );
    assert_eq!(check, ok_check(2));

    // A function being renamed may call itself by the new name
    let check = model.check_named_function(
        &NamedFunctionEntry::new("FACT2", "IF(n>0, n*FACT2(n-1), 1)")
            .parameters(vec![NamedFunctionParameterEntry::new("n")]),
        Some(("FACTORIAL", None)),
    );
    assert_eq!(check, ok_check(1));
}

#[test]
fn usage_count() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_named_function(double()).unwrap();
    model
        .new_named_function(double().name("Local").scope(1))
        .unwrap();
    // The one local to Sheet2 shadows the global one there
    model.new_named_function(double().scope(1)).unwrap();
    assert_eq!(model.named_function_usage_count("DOUBLE", None), Ok(0));

    model._set("A1", "=DOUBLE(1)");
    model._set("A2", "=DOUBLE(1) + DOUBLE(2)"); // one cell
    model._set("A3", "=SUM(MAP(B1:B2, DOUBLE))");
    model._set("A4", "=Sheet2!DOUBLE(1)"); // the local one
    model._set("A5", "=LET(DOUBLE, 1, DOUBLE)"); // bound
    model._set("A6", "=Sheet1!DOUBLE(1)");
    model._set("A7", "5");
    model._set("Sheet2!A1", "=DOUBLE(1)"); // the local one
    model._set("Sheet2!A2", "=Sheet1!DOUBLE(1)");
    model._set("Sheet2!A3", "=Local(1)");
    model.evaluate();

    assert_eq!(model.named_function_usage_count("double", None), Ok(5));
    assert_eq!(model.named_function_usage_count("DOUBLE", Some(1)), Ok(2));
    assert_eq!(model.named_function_usage_count("Local", Some(1)), Ok(1));
    assert_eq!(
        model.named_function_usage_count("Local", None),
        Err("Named function not found".to_string())
    );

    // Renaming keeps the count
    model
        .update_named_function("DOUBLE", None, double().name("TWICE"))
        .unwrap();
    assert_eq!(model.named_function_usage_count("TWICE", None), Ok(5));
    assert_eq!(model.named_function_usage_count("DOUBLE", Some(1)), Ok(2));
}

// ---------------------------------------------------------------------------
// Sheet lifecycle
// ---------------------------------------------------------------------------

#[test]
fn delete_sheet_drops_its_functions() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_sheet();
    model.new_named_function(double()).unwrap();
    model
        .new_named_function(double().name("Local2").scope(1))
        .unwrap();
    model
        .new_named_function(double().name("Local3").scope(2))
        .unwrap();
    model._set("A1", "=DOUBLE(1)+Local2(1)");
    model._set("Sheet3!A1", "=Local3(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NAME?"); // Local2 is local to Sheet2
    assert_eq!(model._get_text("Sheet3!A1"), *"2");

    model.delete_sheet(1).unwrap();
    let list = model.get_named_function_list();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "DOUBLE");
    assert_eq!(list[0].scope, None);
    // Sheet3 is now at index 1 and keeps its function
    assert_eq!(list[1].name, "Local3");
    assert_eq!(list[1].scope, Some(1));
    assert_eq!(model._get_text("Sheet3!A1"), *"2");
    assert_eq!(
        model.named_function_usage_count("Local2", Some(1)),
        Err("Named function not found".to_string())
    );
}

#[test]
fn duplicate_sheet_copies_functions() {
    let mut model = new_empty_model();
    model.add_sheet("Other").unwrap();
    model._set("A1", "10");
    model._set("Other!A1", "1");
    // Local to the source, retargeted
    model
        .new_named_function(
            NamedFunctionEntry::new("local_fn", "x + Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameterEntry::new("x")])
                .scope(0),
        )
        .unwrap();
    // Workbook functions: referencing the source becomes local to the copy,
    // the others are left alone
    model
        .new_named_function(NamedFunctionEntry::new("from_source", "Sheet1!$A$1 * 2"))
        .unwrap();
    model
        .new_named_function(NamedFunctionEntry::new("from_other", "Other!$A$1 * 2"))
        .unwrap();
    model.new_named_function(double()).unwrap();
    // Both workbook and local to the source: the local one wins
    model
        .new_named_function(NamedFunctionEntry::new("shared", "Sheet1!$A$1"))
        .unwrap();
    model
        .new_named_function(NamedFunctionEntry::new("shared", "Sheet1!$A$1 + 1").scope(0))
        .unwrap();
    model._set("B1", "=local_fn(1) + from_source() + shared()");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"42");

    let (new_name, new_index) = model.duplicate_sheet(0).unwrap();
    assert_eq!(new_name, "Sheet1 (1)");
    let list = model.get_named_function_list();
    let on_copy: Vec<&NamedFunctionEntry> = list
        .iter()
        .filter(|nf| nf.scope == Some(new_index))
        .collect();
    assert_eq!(on_copy.len(), 3);
    assert_eq!(on_copy[0].name, "local_fn");
    assert_eq!(on_copy[0].formula, "x+'Sheet1 (1)'!$A$1");
    assert_eq!(on_copy[0].parameters.len(), 1);
    assert_eq!(on_copy[1].name, "shared");
    assert_eq!(on_copy[1].formula, "'Sheet1 (1)'!$A$1+1");
    assert_eq!(on_copy[2].name, "from_source");
    assert_eq!(on_copy[2].formula, "'Sheet1 (1)'!$A$1*2");
    // The originals are untouched
    assert_eq!(list.len(), 6 + 3);
    assert_eq!(list[0].formula, "x+Sheet1!$A$1");
    assert_eq!(list[0].scope, Some(0));

    // The copy computes with its own cells
    model
        .set_user_input(new_index, 1, 1, "100".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"42");
    assert_eq!(model._get_text_at(new_index, 1, 2), *"402");
}

#[test]
fn rename_sheet_updates_bodies_and_calls() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "10");
    model
        .new_named_function(
            NamedFunctionEntry::new("Scaled", "x * Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameterEntry::new("x")]),
        )
        .unwrap();
    model.new_named_function(double().scope(1)).unwrap();
    model._set("B1", "=Scaled(2) + Sheet2!DOUBLE(1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), *"22");

    model.rename_sheet_by_index(0, "Data").unwrap();
    model.rename_sheet_by_index(1, "Calc").unwrap();
    assert_eq!(model.get_named_function_list()[0].formula, "x*Data!$A$1");
    assert_eq!(model._get_formula("Data!B1"), "=Scaled(2)+Calc!DOUBLE(1)");
    assert_eq!(model._get_text("Data!B1"), *"22");

    // Still stored in English after a rename in Spanish
    model.set_language("es").unwrap();
    model.rename_sheet_by_index(0, "Datos").unwrap();
    assert_eq!(model.workbook.named_functions[0].formula, "x*Datos!$A$1");
}
