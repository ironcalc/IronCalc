#![allow(clippy::unwrap_used)]

use crate::model::{DefinedNameEntry, NamedFunctionEntry};
use crate::test::util::new_empty_model;
use crate::types::NamedFunctionParameter;
use crate::Model;

fn profit() -> NamedFunctionEntry {
    NamedFunctionEntry::new("PROFIT", "=(revenue - cost) / revenue")
        .description("Profit margin")
        .parameters(vec![
            NamedFunctionParameter::new("revenue").description("Total revenue"),
            NamedFunctionParameter::new("cost"),
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
                    NamedFunctionParameter::new("net"),
                    NamedFunctionParameter::new("rate").optional(true),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("QUADRUPLE", "DOUBLE(DOUBLE(x))")
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x * 3")
                .parameters(vec![NamedFunctionParameter::new("x")])
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
                .parameters(vec![NamedFunctionParameter::new("net")]),
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
        Err("A function or defined name called MARGIN already exists.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Profit", "1")),
        Err("A function or defined name called PROFIT already exists.".to_string())
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
        Err("SUM is already a built-in function.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("sum", "1")),
        Err("SUM is already a built-in function.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("A1", "1")),
        Err("This name looks like a cell reference (like AB12).".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("My Fn", "1")),
        Err("Use letters, numbers, and underscores, starting with a letter.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("", "1")),
        Err("Name is required.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "1").scope(3)),
        Err("The sheet of the function does not exist.".to_string())
    );
    assert_eq!(
        model.new_named_function(
            NamedFunctionEntry::new("Fn", "x").parameters(vec![NamedFunctionParameter::new("B2")])
        ),
        Err("This name looks like a cell reference. Try another.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "x").parameters(vec![
            NamedFunctionParameter::new("x"),
            NamedFunctionParameter::new("X")
        ])),
        Err("Each argument needs a different name.".to_string())
    );
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "1 +")),
        Err("There's an error in this formula.".to_string())
    );
    // References must be qualified with a sheet, like in defined names
    assert_eq!(
        model.new_named_function(NamedFunctionEntry::new("Fn", "A1 + 1")),
        Err("References in the formula need a sheet (like Sheet1!A1).".to_string())
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
        model.new_named_function(
            NamedFunctionEntry::new("Fn2", "x")
                .parameters(vec![NamedFunctionParameter::new("x").description(&long)])
        ),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
    NamedFunctionEntry::new("DOUBLE", "x*2").parameters(vec![NamedFunctionParameter::new("x")])
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
                    NamedFunctionParameter::new("n"),
                    NamedFunctionParameter::new("factor")
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("FACTORIAL", "IF(n>0, n*FACTORIAL(n-1), 1)")
                .parameters(vec![NamedFunctionParameter::new("n")]),
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
                .parameters(vec![NamedFunctionParameter::new("n")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("MyFn", "x*3")
                .parameters(vec![NamedFunctionParameter::new("x")])
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")])
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
    assert_eq!(
        model.update_named_function("DOUBLE", None, double().name("other")),
        Err("A function or defined name called OTHER already exists.".to_string())
    );
    assert_eq!(
        model.update_named_function("DOUBLE", None, double().name("margin")),
        Err("A function or defined name called MARGIN already exists.".to_string())
    );
    // Changing the case of its own name is fine
    model
        .update_named_function("DOUBLE", None, double().name("Double"))
        .unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "Double");
    assert_eq!(
        model.update_named_function("Double", None, double().name("SUM")),
        Err("SUM is already a built-in function.".to_string())
    );
    assert_eq!(
        model.update_named_function("Double", None, double().scope(5)),
        Err("The sheet of the function does not exist.".to_string())
    );
    assert_eq!(
        model.update_named_function(
            "Double",
            None,
            NamedFunctionEntry::new("Double", "x").parameters(vec![
                NamedFunctionParameter::new("x"),
                NamedFunctionParameter::new("x")
            ])
        ),
        Err("Each argument needs a different name.".to_string())
    );
    assert_eq!(
        model.update_named_function("Double", None, NamedFunctionEntry::new("Double", "A1")),
        Err("References in the formula need a sheet (like Sheet1!A1).".to_string())
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
                .parameters(vec![NamedFunctionParameter::new("net")]),
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
            .parameters(vec![NamedFunctionParameter::new("n")]),
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
        NamedFunctionParameter::new("a"),
        NamedFunctionParameter::new(" "),
        NamedFunctionParameter::new("B2"),
        NamedFunctionParameter::new("my arg"),
        NamedFunctionParameter::new("A"),
        NamedFunctionParameter::new("vat"),
        NamedFunctionParameter::new("b"),
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
            // A parameter may be called like a defined name: the parameter wins
            None,
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
fn check_optional_parameters_go_last() {
    let mut model = new_empty_model();
    let error = Some("Arguments after an optional one must be optional too.".to_string());
    let with = |optional: [bool; 4]| {
        NamedFunctionEntry::new("Fn", "w + x + y + z").parameters(
            ["w", "x", "y", "z"]
                .iter()
                .zip(optional)
                .map(|(name, optional)| NamedFunctionParameter::new(name).optional(optional))
                .collect(),
        )
    };

    // Every required parameter after the first optional one is reported
    let check = model.check_named_function(&with([false, true, false, false]), None);
    assert_eq!(
        check.parameters,
        vec![None, None, error.clone(), error.clone()]
    );
    assert!(check.formula.is_empty());
    assert!(!check.is_valid());
    assert_eq!(check.first_error(), error);

    // An optional one in between does not make the next required one valid
    let check = model.check_named_function(&with([true, false, true, false]), None);
    assert_eq!(
        check.parameters,
        vec![None, error.clone(), None, error.clone()]
    );

    // The optional ones last, all of them optional or none of them
    for optional in [
        [false, false, true, true],
        [false, false, false, true],
        [true, true, true, true],
        [false, false, false, false],
    ] {
        let check = model.check_named_function(&with(optional), None);
        assert_eq!(check, ok_check(4), "{optional:?}");
    }

    // Another problem with the parameter is reported first
    let entry = NamedFunctionEntry::new("Fn", "a").parameters(vec![
        NamedFunctionParameter::new("a").optional(true),
        NamedFunctionParameter::new("B2"),
    ]);
    let check = model.check_named_function(&entry, None);
    assert_eq!(
        check.parameters,
        vec![
            None,
            Some("This name looks like a cell reference. Try another.".to_string()),
        ]
    );
}

#[test]
fn required_parameter_after_optional_is_rejected() {
    let mut model = new_empty_model();
    let error = Err("Arguments after an optional one must be optional too.".to_string());
    let wrong = NamedFunctionEntry::new("ADD", "a + b").parameters(vec![
        NamedFunctionParameter::new("a").optional(true),
        NamedFunctionParameter::new("b"),
    ]);
    let right = NamedFunctionEntry::new("ADD", "a+b").parameters(vec![
        NamedFunctionParameter::new("a"),
        NamedFunctionParameter::new("b").optional(true),
    ]);

    // Creating it
    assert_eq!(model.new_named_function(wrong.clone()), error);
    assert!(model.get_named_function_list().is_empty());

    // Updating a valid one
    model.new_named_function(right.clone()).unwrap();
    assert_eq!(model.update_named_function("ADD", None, wrong), error);
    assert_eq!(model.get_named_function_list(), vec![right]);

    // A defined name that is a LAMBDA follows the same rule
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("PLUS", "=LAMBDA([a], b, a + b)")),
        Err("Formula: Arguments after an optional one must be optional too.".to_string())
    );
    model
        .new_defined_name(DefinedNameEntry::new("PLUS", "=LAMBDA(a, [b], a + b)"))
        .unwrap();
    model._set("A1", "=PLUS(2)");
    model._set("A2", "=PLUS(2, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"5");
}

#[test]
fn check_formula() {
    let mut model = new_empty_model();
    model.new_named_function(double()).unwrap();
    let with = |formula: &str| {
        NamedFunctionEntry::new("PROFIT", formula).parameters(vec![
            NamedFunctionParameter::new("revenue"),
            NamedFunctionParameter::new("cost"),
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

    // Unknown names are errors; unused parameters are warnings
    let check = model.check_named_function(&with("(revenue - coste) / price"), None);
    assert_eq!(
        check.formula,
        vec![
            "\"coste\" isn't an argument. Add it above or check the spelling.".to_string(),
            "\"price\" isn't an argument. Add it above or check the spelling.".to_string(),
        ]
    );
    assert_eq!(
        check.warnings,
        vec!["\"cost\" isn't used in the formula.".to_string()]
    );
    assert!(!check.is_valid());

    // Parameters are case-sensitive, like the evaluation
    let check = model.check_named_function(&with("Revenue - cost"), None);
    assert_eq!(
        check.formula,
        vec!["\"Revenue\" isn't an argument. Add it above or check the spelling.".to_string()]
    );

    // Functions: known, itself, a parameter used as a function, unknown
    let check = model.check_named_function(
        &with("DOUBLE(revenue) + PROFIT(revenue, cost) + cost(1) + NOPE(2)"),
        None,
    );
    assert_eq!(
        check.formula,
        vec!["\"NOPE\" isn't a known function.".to_string()]
    );
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
            .parameters(vec![NamedFunctionParameter::new("n")]),
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
                .parameters(vec![NamedFunctionParameter::new("x")])
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
                .parameters(vec![NamedFunctionParameter::new("x")]),
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

#[test]
fn hidden_function() {
    let mut model = new_empty_model();
    model.new_named_function(profit().hidden(true)).unwrap();
    // A hidden function is called like any other
    model._set("A1", "=PROFIT(200, 50)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "0.75");
    assert!(model.get_named_function_list()[0].hidden);
    assert!(model.workbook.named_functions[0].hidden);

    // The flag is one more field to update
    model
        .update_named_function("PROFIT", None, profit())
        .unwrap();
    assert!(!model.get_named_function_list()[0].hidden);
    model
        .update_named_function("PROFIT", None, profit().hidden(true))
        .unwrap();

    // And it survives a round trip through bytes
    let bytes = model.to_bytes();
    let model = Model::from_bytes(&bytes, "en").unwrap();
    assert!(model.get_named_function_list()[0].hidden);
}

#[test]
fn hidden_lambda_defined_name_is_a_hidden_function() {
    let mut model = new_empty_model();
    model
        .new_defined_name(DefinedNameEntry::new("Twice", "LAMBDA(x, 2*x)").hidden(true))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$A$1"))
        .unwrap();
    // A defined name updated to a hidden LAMBDA is a hidden function too
    model
        .update_defined_name(
            "Rate",
            None,
            DefinedNameEntry::new("Thrice", "LAMBDA(x, 3*x)").hidden(true),
        )
        .unwrap();
    let functions = model.get_named_function_list();
    assert_eq!(functions.len(), 2);
    assert!(functions.iter().all(|f| f.hidden));
}

#[test]
fn duplicate_sheet_keeps_hidden() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("LOCAL", "x+Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameter::new("x")])
                .scope(0)
                .hidden(true),
        )
        .unwrap();
    model.duplicate_sheet(0).unwrap();
    let functions = model.get_named_function_list();
    assert_eq!(functions.len(), 2);
    assert!(functions.iter().all(|f| f.hidden));
}

// A LAMBDA parameter, a LET variable or a parameter of a named function may
// be called like an existing named function: the bound name wins and is kept
// as the user typed it.
#[test]
fn bound_name_called_like_a_named_function() {
    let mut model = new_empty_model();
    model.new_named_function(profit()).unwrap();

    // A parameter of a named function
    model
        .new_named_function(
            NamedFunctionEntry::new("F", "profit*2")
                .parameters(vec![NamedFunctionParameter::new("profit")]),
        )
        .unwrap();
    model._set("A1", "=F(5)");
    // A LAMBDA parameter and a LET variable
    model._set("A2", "=LAMBDA(profit, profit*2)(5)");
    model._set("A3", "=LET(profit, 3, profit+1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "10");
    assert_eq!(model.get_named_function_list()[1].formula, "profit*2");
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_formula("A2"), "=LAMBDA(profit,profit*2)(5)");
    assert_eq!(model._get_text("A3"), "4");
    assert_eq!(model._get_formula("A3"), "=LET(profit,3,profit+1)");

    // Renaming the function leaves the bound names alone
    model
        .update_named_function("PROFIT", None, profit().name("MARGIN"))
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_named_function_list()[1].formula, "profit*2");
    assert_eq!(model._get_text("A1"), "10");
    assert_eq!(model._get_formula("A2"), "=LAMBDA(profit,profit*2)(5)");
    assert_eq!(model._get_formula("A3"), "=LET(profit,3,profit+1)");
}

// ---------------------------------------------------------------------------
// Regressions
// ---------------------------------------------------------------------------

fn twice() -> NamedFunctionEntry {
    NamedFunctionEntry::new("TWICE", "2*x").parameters(vec![NamedFunctionParameter::new("x")])
}

// A renamed function is renamed in the defined names that call it
#[test]
fn rename_propagates_to_defined_names() {
    let mut model = new_empty_model();
    model.new_named_function(twice()).unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Dbl", "TWICE(Sheet1!$A$1)"))
        .unwrap();
    model._set("A1", "21");
    model._set("A2", "=Dbl");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "42");

    model
        .update_named_function("TWICE", None, twice().name("DOUBLE"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A2"), "42");
    assert_eq!(
        model.get_defined_name_list()[0].formula,
        "DOUBLE(Sheet1!$A$1)"
    );
}

// A parameter wins over a defined name called the same, whichever came
// first: the function keeps working.
#[test]
fn defined_name_does_not_capture_a_parameter() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("TWICE", "2*rate")
                .parameters(vec![NamedFunctionParameter::new("rate")]),
        )
        .unwrap();
    model._set("A1", "=TWICE(21)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "42");

    let _ = model.new_defined_name(DefinedNameEntry::new("rate", "Sheet1!$C$1"));
    model.evaluate();
    assert_eq!(model._get_text("A1"), "42");
}

// A defined name qualified with a sheet is not an unknown function
#[test]
fn check_accepts_a_qualified_defined_name() {
    let mut model = new_empty_model();
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$C$1"))
        .unwrap();
    let entry = NamedFunctionEntry::new("WITH_RATE", "Sheet1!Rate*x")
        .parameters(vec![NamedFunctionParameter::new("x")]);
    let check = model.check_named_function(&entry, None);
    assert_eq!(check, ok_check(1));
    model.new_named_function(entry).unwrap();
}

// A function created as a LAMBDA defined name follows the rules of the
// named functions
#[test]
fn lambda_defined_name_cannot_be_called_like_a_builtin_function() {
    let mut model = new_empty_model();
    let result = model.new_defined_name(DefinedNameEntry::new("SUM", "LAMBDA(x, x+1)"));
    assert!(result.is_err(), "{:?}", model.get_named_function_list());
    assert!(model.get_named_function_list().is_empty());
}

#[test]
fn lambda_defined_name_cannot_repeat_a_parameter() {
    let mut model = new_empty_model();
    let result = model.new_defined_name(DefinedNameEntry::new("Twin", "LAMBDA(x, x, x+1)"));
    assert!(result.is_err(), "{:?}", model.get_named_function_list());
    assert!(model.get_named_function_list().is_empty());
}

// A workbook function the check accepts works from every sheet: it cannot
// rely on a function local to a sheet, unless it is qualified
#[test]
fn checked_workbook_function_works_from_every_sheet() {
    let mut model = new_empty_model();
    model.new_sheet();
    model
        .new_named_function(
            NamedFunctionEntry::new("LOCALFN", "x+1")
                .parameters(vec![NamedFunctionParameter::new("x")])
                .scope(0),
        )
        .unwrap();
    let entry = NamedFunctionEntry::new("GLOBALFN", "LOCALFN(x)")
        .parameters(vec![NamedFunctionParameter::new("x")]);
    let check = model.check_named_function(&entry, None);
    assert_eq!(
        check.formula,
        vec!["\"LOCALFN\" isn't a known function.".to_string()]
    );

    let entry = NamedFunctionEntry::new("GLOBALFN", "Sheet1!LOCALFN(x)")
        .parameters(vec![NamedFunctionParameter::new("x")]);
    assert!(model.check_named_function(&entry, None).is_valid());
    model.new_named_function(entry).unwrap();
    model._set("Sheet1!A1", "=GLOBALFN(1)");
    model._set("Sheet2!A1", "=GLOBALFN(1)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!A1"), "2");
    assert_eq!(model._get_text("Sheet2!A1"), "2");
}

// Names are compared the same way everywhere, also when they are not ASCII:
// a copied sheet does not get two functions called the same
#[test]
fn duplicate_sheet_with_non_ascii_names() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("Año", "x")
                .parameters(vec![NamedFunctionParameter::new("x")])
                .scope(0),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("AÑO", "x+Sheet1!$A$1")
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model.duplicate_sheet(0).unwrap();
    let in_copy: Vec<String> = model
        .get_named_function_list()
        .iter()
        .filter(|f| f.scope == Some(1))
        .map(|f| f.name.clone())
        .collect();
    // The function local to the source hides the workbook one
    assert_eq!(in_copy, vec!["Año".to_string()]);
}

// A function used through another function is used
#[test]
fn usage_count_through_another_function() {
    let mut model = new_empty_model();
    model
        .new_named_function(
            NamedFunctionEntry::new("INNER", "x+1")
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model
        .new_named_function(
            NamedFunctionEntry::new("OUTER", "INNER(x)")
                .parameters(vec![NamedFunctionParameter::new("x")]),
        )
        .unwrap();
    model._set("A1", "=OUTER(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model.named_function_usage_count("OUTER", None), Ok(1));
    assert_eq!(model.named_function_usage_count("INNER", None), Ok(1));
}

// A function that is not built-in in the language it is typed in is unknown
// (`TRUE` is `VERDADERO` in Spanish), and stays unknown: it is not taken for
// the English function when the formulas are parsed again or in a body.
#[test]
fn function_of_another_language_is_unknown() {
    let mut model = new_empty_model();
    model.set_language("es").unwrap();
    model._set("A1", "=TRUE()+1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#¿NOMBRE?");

    // In a body it is an unknown function, like any other
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("H", "LAMBDA(x, TRUE()+x)")),
        Err("Formula: \"TRUE\" isn't a known function.".to_string())
    );
    let entry =
        NamedFunctionEntry::new("H", "TRUE()+x").parameters(vec![NamedFunctionParameter::new("x")]);
    assert_eq!(
        model.check_named_function(&entry, None).formula,
        vec!["\"TRUE\" isn't a known function.".to_string()]
    );

    // A new name parses every formula again
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "Sheet1!$C$1"))
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#¿NOMBRE?");
    // (the name of an unknown function is stored in lowercase)
    assert_eq!(model._get_formula("A1"), "=true()+1");

    // In English it is still not the built-in function, and it shows
    model.set_language("en").unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NAME?");
    assert_eq!(model._get_formula("A1"), "=_xludf.true()+1");
}

// The call of a function qualified with a sheet reports the argument being
// typed, like any other call
#[test]
fn qualified_call_reports_the_argument() {
    use crate::expressions::parser::ExpectedTokens;

    let mut model = new_empty_model();
    model.new_named_function(twice()).unwrap();
    let formula = "=TWICE(2";
    let plain = model
        .formula_completion(0, 1, 1, formula, formula.chars().count())
        .unwrap();
    assert_eq!(
        plain.expecting,
        vec![ExpectedTokens::Argument("TWICE".to_string(), 1)]
    );
    let formula = "=Sheet1!TWICE(2";
    let qualified = model
        .formula_completion(0, 1, 1, formula, formula.chars().count())
        .unwrap();
    assert_eq!(
        qualified.expecting,
        vec![ExpectedTokens::Argument("TWICE".to_string(), 1)]
    );
}

// ---------------------------------------------------------------------------
// A name that is a built-in function in another language
// ---------------------------------------------------------------------------

// `VERDADERO` is not a function in English, in Spanish it is `TRUE`
fn verdadero() -> NamedFunctionEntry {
    NamedFunctionEntry::new("VERDADERO", "FALSE").description("Not what it looks like")
}

#[test]
fn name_that_is_a_builtin_in_another_language_takes_the_prefix() {
    let mut model = new_empty_model();
    model.new_named_function(verdadero()).unwrap();
    model._set("A1", "=VERDADERO()");
    model._set("A2", "=TRUE()");
    model.evaluate();
    assert_eq!(model.get_named_function_list()[0].name, "VERDADERO");
    assert_eq!(model._get_formula("A1"), "=VERDADERO()");
    assert_eq!(model._get_text("A1"), *"FALSE");

    // In Spanish the name is shown as it is called in a formula
    model.set_language("es").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=_xludf.VERDADERO()");
    assert_eq!(model._get_formula("A2"), "=VERDADERO()");
    let list = model.get_named_function_list();
    assert_eq!(list[0].name, "_xludf.VERDADERO");
    assert_eq!(model._get_text("A1"), *"FALSO");
    assert_eq!(model._get_text("A2"), *"VERDADERO");

    // And back in English
    model.set_language("en").unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "VERDADERO");
    assert_eq!(model._get_formula("A1"), "=VERDADERO()");
}

#[test]
fn name_with_the_prefix_is_edited_and_deleted() {
    let mut model = new_empty_model();
    model.new_named_function(verdadero()).unwrap();
    model._set("A1", "=VERDADERO()");
    model.evaluate();
    model.set_language("es").unwrap();

    // The function as it is listed is valid, and can be saved as it is
    let listed = model.get_named_function_list()[0].clone();
    let current = Some(("_xludf.VERDADERO", None));
    assert_eq!(model.check_named_function(&listed, current), ok_check(0));
    model
        .update_named_function("_xludf.VERDADERO", None, listed.clone())
        .unwrap();
    assert_eq!(model.get_named_function_list(), vec![listed.clone()]);
    assert_eq!(model._get_formula("A1"), "=_xludf.VERDADERO()");

    // The prefix is not part of the name that is stored
    assert_eq!(model.workbook.named_functions[0].name, "VERDADERO");

    // Its description changes, with the name as it is listed or without the
    // prefix, in upper case as the editor writes it
    for name in ["_xludf.VERDADERO", "_XLUDF.VERDADERO", "VERDADERO"] {
        let edited = listed.clone().description(name);
        model
            .update_named_function(name, None, edited.clone())
            .unwrap();
        assert_eq!(model.get_named_function_list(), vec![edited]);
    }

    assert_eq!(
        model.named_function_usage_count("_xludf.VERDADERO", None),
        Ok(1)
    );

    // Renamed, the cells follow
    model
        .update_named_function("_xludf.VERDADERO", None, listed.clone().name("MIO"))
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=MIO()");
    model
        .update_named_function("MIO", None, listed.clone())
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=_xludf.VERDADERO()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSO");

    model
        .delete_named_function("_xludf.VERDADERO", None)
        .unwrap();
    assert!(model.get_named_function_list().is_empty());
}

#[test]
fn check_name_that_is_a_builtin_in_the_active_language() {
    let mut model = new_empty_model();
    model.set_language("es").unwrap();
    let with = |name: &str| NamedFunctionEntry::new(name, "FALSO");

    // Without the prefix it is the built-in function
    let check = model.check_named_function(&with("VERDADERO"), None);
    assert_eq!(
        check.name,
        Some("VERDADERO is already a built-in function.".to_string())
    );
    assert_eq!(
        model.new_named_function(with("verdadero")),
        Err("VERDADERO is already a built-in function.".to_string())
    );

    // With it, it is a function of the user
    assert_eq!(
        model.check_named_function(&with("_xludf.VERDADERO"), None),
        ok_check(0)
    );
    model.new_named_function(with("_xludf.VERDADERO")).unwrap();
    assert_eq!(model.workbook.named_functions[0].name, "VERDADERO");
    model._set("A1", "=_xludf.VERDADERO()");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSO");

    // There is only one function called like that
    let check = model.check_named_function(&with("_xludf.Verdadero"), None);
    assert_eq!(
        check.name,
        Some("A function or defined name called VERDADERO already exists.".to_string())
    );

    // The prefix alone is not a name, and it does no harm on any other name
    let check = model.check_named_function(&with("_xludf."), None);
    assert_eq!(check.name, Some("Name is required.".to_string()));
    model.new_named_function(with("_xludf.Plain")).unwrap();
    assert_eq!(model.get_named_function_list()[1].name, "Plain");
}

// ---------------------------------------------------------------------------
// A name that is a built-in function in English
// ---------------------------------------------------------------------------

// `NOW` is not a function in Spanish, there it is `AHORA`
fn now() -> NamedFunctionEntry {
    NamedFunctionEntry::new("NOW", "x+7").parameters(vec![NamedFunctionParameter::new("x")])
}

#[test]
fn name_that_is_a_builtin_in_english_is_created_in_another_language() {
    let mut model = new_empty_model();
    model.set_language("es").unwrap();
    assert_eq!(model.check_named_function(&now(), None), ok_check(1));
    model.new_named_function(now()).unwrap();
    model
        .new_named_function(NamedFunctionEntry::new("TWICE", "2*NOW(1)"))
        .unwrap();
    model._set("A1", "=NOW(1)");
    model._set("A2", "=TWICE()");
    model._set("A3", "=TIPO(AHORA())");
    model.evaluate();

    // In Spanish there is nothing to tell apart
    let list = model.get_named_function_list();
    assert_eq!(list[0].name, "NOW");
    assert_eq!(list[1].formula, "2*NOW(1)");
    assert_eq!(model._get_formula("A1"), "=NOW(1)");
    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"16");
    assert_eq!(model.named_function_usage_count("NOW", None), Ok(2));

    // The prefix is not part of the name that is stored, the calls have it
    assert_eq!(model.workbook.named_functions[0].name, "NOW");
    assert_eq!(model.workbook.named_functions[1].formula, "2*_xludf.NOW(1)");

    // In English the name is shown as it is called in a formula
    model.set_language("en").unwrap();
    model.evaluate();
    let list = model.get_named_function_list();
    assert_eq!(list[0].name, "_xludf.NOW");
    assert_eq!(list[1].formula, "2*_xludf.NOW(1)");
    assert_eq!(model._get_formula("A1"), "=_xludf.NOW(1)");
    assert_eq!(model._get_formula("A3"), "=TYPE(NOW())");
    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"16");
    assert_eq!(model._get_text("A3"), *"1");
    assert_eq!(model.named_function_usage_count("_xludf.NOW", None), Ok(2));

    // Typed in English
    model._set("A4", "=_xludf.NOW(10)");
    model._set("A5", "=TYPE(NOW())");
    model.evaluate();
    assert_eq!(model._get_text("A4"), *"17");
    assert_eq!(model._get_text("A5"), *"1");

    // The workbook is saved and loaded
    let bytes = model.to_bytes();
    let mut model = Model::from_bytes(&bytes, "en").unwrap();
    model.evaluate();
    assert_eq!(model.get_named_function_list()[0].name, "_xludf.NOW");
    assert_eq!(model._get_formula("A4"), "=_xludf.NOW(10)");
    assert_eq!(model._get_text("A4"), *"17");

    // And back in Spanish
    model.set_language("es").unwrap();
    assert_eq!(model.get_named_function_list()[0].name, "NOW");
    assert_eq!(model._get_formula("A4"), "=NOW(10)");
}

#[test]
fn name_that_is_a_builtin_in_english_is_created_in_english() {
    let mut model = new_empty_model();

    // Without the prefix it is the built-in function
    let message = "NOW is already a built-in function.";
    let check = model.check_named_function(&now(), None);
    assert_eq!(check.name, Some(message.to_string()));
    assert_eq!(model.new_named_function(now()), Err(message.to_string()));

    // With it, it is a function of the user
    for name in ["_xludf.NOW", "_XLUDF.now"] {
        assert_eq!(
            model.check_named_function(&now().name(name), None),
            ok_check(1),
            "{name}"
        );
    }
    model.new_named_function(now().name("_xludf.NOW")).unwrap();
    assert_eq!(model.workbook.named_functions[0].name, "NOW");
    model._set("A1", "=_xludf.NOW(1)");
    model._set("A2", "=TYPE(NOW())");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8");
    assert_eq!(model._get_text("A2"), *"1");

    // There is only one function called like that
    let check = model.check_named_function(&now().name("_xludf.Now"), None);
    assert_eq!(
        check.name,
        Some("A function or defined name called NOW already exists.".to_string())
    );

    // The function as it is listed is valid, and can be saved as it is
    let listed = model.get_named_function_list()[0].clone();
    assert_eq!(listed.name, "_xludf.NOW");
    let current = Some(("_xludf.NOW", None));
    assert_eq!(model.check_named_function(&listed, current), ok_check(1));
    model
        .update_named_function("_xludf.NOW", None, listed.clone())
        .unwrap();
    assert_eq!(model.get_named_function_list(), vec![listed.clone()]);

    // It cannot lose the prefix
    assert_eq!(
        model.update_named_function("_xludf.NOW", None, listed.clone().name("NOW")),
        Err(message.to_string())
    );

    // Renamed, the cells follow
    model
        .update_named_function("_xludf.NOW", None, listed.clone().name("MINE"))
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=MINE(1)");
    assert_eq!(model._get_formula("A2"), "=TYPE(NOW())");
    model
        .update_named_function("MINE", None, listed.clone())
        .unwrap();
    assert_eq!(model._get_formula("A1"), "=_xludf.NOW(1)");
    assert_eq!(model._get_formula("A2"), "=TYPE(NOW())");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8");

    model.delete_named_function("_xludf.NOW", None).unwrap();
    assert!(model.get_named_function_list().is_empty());
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NAME?");
}

// A function created as a LAMBDA defined name follows the same rules
#[test]
fn lambda_defined_name_that_is_a_builtin_in_english() {
    let mut model = new_empty_model();
    assert_eq!(
        model.new_defined_name(DefinedNameEntry::new("NOW", "LAMBDA(x, x+7)")),
        Err("Name: NOW is already a built-in function.".to_string())
    );
    model
        .new_defined_name(DefinedNameEntry::new("_xludf.NOW", "LAMBDA(x, x+7)"))
        .unwrap();
    assert_eq!(model.workbook.named_functions[0].name, "NOW");
    assert_eq!(model.get_named_function_list()[0].name, "_xludf.NOW");
    model._set("A1", "=_xludf.NOW(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8");

    // In Spanish it needs no prefix
    let mut model = new_empty_model();
    model.set_language("es").unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("NOW", "LAMBDA(x, x+7)"))
        .unwrap();
    assert_eq!(model.workbook.named_functions[0].name, "NOW");
    model._set("A1", "=NOW(1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8");
}
