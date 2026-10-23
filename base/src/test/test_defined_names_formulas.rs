#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A defined name whose formula is not a reference: a constant or an expression
#[test]
fn constants_and_expressions() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model._set("K2", "20");
    model._set("K3", "30");
    model.new_defined_name("Pi", None, "3.14159").unwrap();
    model
        .new_defined_name("Greeting", None, "\"Hello\"")
        .unwrap();
    model.new_defined_name("Yes", None, "TRUE").unwrap();
    model
        .new_defined_name("Total", None, "SUM(Sheet1!$K$1:$K$3)")
        .unwrap();
    model
        .new_defined_name("Double", None, "Sheet1!$K$1*2")
        .unwrap();
    model
        .new_defined_name("TotalPlus", None, "Total+Double")
        .unwrap();
    model
        .new_defined_name("Dyn", None, "OFFSET(Sheet1!$K$1,0,0,2,1)")
        .unwrap();
    assert!(model.new_defined_name("Broken", None, "SUM(").is_err());

    model._set("A1", "=Pi");
    model._set("A2", "=Greeting&\" World\"");
    model._set("A3", "=IF(Yes, 1, 2)");
    model._set("A4", "=Total");
    model._set("A5", "=Double");
    model._set("A6", "=TotalPlus");
    model._set("A7", "=SUM(Dyn)");
    model._set("A8", "=Pi*2");
    model.evaluate();

    for (cell, expected) in [
        ("A1", "3.14159"),
        ("A2", "Hello World"),
        ("A3", "1"),
        ("A4", "60"),
        ("A5", "20"),
        ("A6", "80"),
        ("A7", "30"),
        ("A8", "6.28318"),
    ] {
        assert_eq!(model._get_text(cell), expected, "cell {cell}");
    }

    // Changing an input re-evaluates names that depend on it
    model._set("K1", "100");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "150");
    assert_eq!(model._get_text("A5"), "200");
    assert_eq!(model._get_text("A6"), "350");
    assert_eq!(model._get_text("A7"), "120");
}

// A name local to a sheet can be used from another sheet by qualifying it:
// `Data!Scope_Test`. `Sheet!GlobalName` is the workbook name.
#[test]
fn sheet_qualified_names() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model._set("Sheet1!A1", "3");
    model._set("Data!A1", "12");
    // Workbook name and a name local to Data with the same name
    model
        .new_defined_name("Scope_Test", None, "Sheet1!$A$1")
        .unwrap();
    model
        .new_defined_name("Scope_Test", Some(1), "Data!$A$1")
        .unwrap();

    model._set("Sheet1!B1", "=Scope_Test");
    model._set("Sheet1!B2", "=Data!Scope_Test");
    model._set("Sheet1!B3", "=Sheet1!Scope_Test");
    model._set("Sheet1!B4", "=Data!Scope_Test*2");
    model._set("Sheet1!B5", "=Data!Nope");
    model._set("Data!B1", "=Scope_Test");
    model._set("Data!B2", "=Sheet1!Scope_Test");
    model.evaluate();

    assert_eq!(model._get_text("Sheet1!B1"), "3");
    assert_eq!(model._get_text("Sheet1!B2"), "12");
    assert_eq!(model._get_text("Sheet1!B3"), "3");
    assert_eq!(model._get_text("Sheet1!B4"), "24");
    assert_eq!(model._get_text("Sheet1!B5"), "#NAME?");
    assert_eq!(model._get_text("Data!B1"), "12");
    assert_eq!(model._get_text("Data!B2"), "3");
    assert_eq!(model._get_formula("Sheet1!B2"), "=Data!Scope_Test");
    assert_eq!(model._get_formula("Sheet1!B5"), "=Data!Nope");

    // Renaming the sheet renames the qualifier
    model.rename_sheet_by_index(1, "Data Set").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Sheet1!B2"), "='Data Set'!Scope_Test");
    assert_eq!(model._get_formula("Sheet1!B4"), "='Data Set'!Scope_Test*2");
    assert_eq!(model._get_text("Sheet1!B2"), "12");

    // Moving the sheet keeps the name resolving to the right sheet
    model.move_sheet(1, 0).unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Sheet1!B2"), "='Data Set'!Scope_Test");
    assert_eq!(model._get_text("Sheet1!B2"), "12");
    assert_eq!(model._get_text("Sheet1!B1"), "3");
    // 'Data Set' is now the first sheet
    assert_eq!(model._get_text_at(0, 1, 2), "12");
    assert_eq!(model._get_text_at(0, 2, 2), "3");
}

// A name whose text starts like a cell reference (`Q1_Total`) is a name
#[test]
fn qualified_name_starting_like_a_reference() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model._set("Data!Q1", "77");
    model.new_defined_name("Q1_Total", None, "100").unwrap();
    model._set("Sheet1!A1", "=Data!Q1_Total");
    model._set("Sheet1!A2", "=Data!Q1");
    model._set("Sheet1!A3", "=SUM(Data!Q1_Total,Data!Q1)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!A1"), "100");
    assert_eq!(model._get_formula("Sheet1!A1"), "=Data!Q1_Total");
    assert_eq!(model._get_text("Sheet1!A2"), "77");
    assert_eq!(model._get_text("Sheet1!A3"), "177");
}

// The names used in the formula of a sheet-local name resolve in that sheet
#[test]
fn local_name_formulas_resolve_in_their_sheet() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model.new_defined_name("Rate", None, "1").unwrap();
    model.new_defined_name("Rate", Some(1), "10").unwrap();
    model
        .new_defined_name("DoubleRate", Some(1), "Rate*2")
        .unwrap();
    model.new_defined_name("OnlyLocal", Some(1), "7").unwrap();
    // A local name may depend on a name that exists only in its sheet
    model
        .new_defined_name("UsesLocal", Some(1), "OnlyLocal+1")
        .unwrap();
    // A workbook name uses the workbook Rate
    model
        .new_defined_name("GlobalDouble", None, "Rate*2")
        .unwrap();

    model._set("Sheet1!A1", "=Data!DoubleRate");
    model._set("Sheet1!A2", "=Data!UsesLocal");
    model._set("Sheet1!A3", "=GlobalDouble");
    model._set("Data!A1", "=DoubleRate");
    model._set("Data!A2", "=GlobalDouble");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!A1"), "20");
    assert_eq!(model._get_text("Sheet1!A2"), "8");
    assert_eq!(model._get_text("Sheet1!A3"), "2");
    assert_eq!(model._get_text("Data!A1"), "20");
    assert_eq!(model._get_text("Data!A2"), "2");

    // Still true after the sheets are reordered
    model.move_sheet(1, 0).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!A1"), "20");
    assert_eq!(model._get_text("Sheet1!A2"), "8");
    assert_eq!(model._get_text("Data!A1"), "20");
}
