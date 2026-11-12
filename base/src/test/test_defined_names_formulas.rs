#![allow(clippy::unwrap_used)]

use crate::model::DefinedNameEntry;
use crate::test::util::new_empty_model;

// A defined name whose formula is not a reference: a constant or an expression
#[test]
fn constants_and_expressions() {
    let mut model = new_empty_model();
    model._set("K1", "10");
    model._set("K2", "20");
    model._set("K3", "30");
    model
        .new_defined_name(DefinedNameEntry::new("Pi", "3.14159"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Greeting", "\"Hello\""))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Yes", "TRUE"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Total", "SUM(Sheet1!$K$1:$K$3)"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Double", "Sheet1!$K$1*2"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("TotalPlus", "Total+Double"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Dyn", "OFFSET(Sheet1!$K$1,0,0,2,1)"))
        .unwrap();
    assert!(model
        .new_defined_name(DefinedNameEntry::new("Broken", "SUM("))
        .is_err());

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
        .new_defined_name(DefinedNameEntry::new("Scope_Test", "Sheet1!$A$1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Scope_Test", "Data!$A$1").scope(1))
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
    model
        .new_defined_name(DefinedNameEntry::new("Q1_Total", "100"))
        .unwrap();
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
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "1"))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("Rate", "10").scope(1))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("DoubleRate", "Rate*2").scope(1))
        .unwrap();
    model
        .new_defined_name(DefinedNameEntry::new("OnlyLocal", "7").scope(1))
        .unwrap();
    // A local name may depend on a name that exists only in its sheet
    model
        .new_defined_name(DefinedNameEntry::new("UsesLocal", "OnlyLocal+1").scope(1))
        .unwrap();
    // A workbook name uses the workbook Rate
    model
        .new_defined_name(DefinedNameEntry::new("GlobalDouble", "Rate*2"))
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

// A name whose formula reaches the name again would never end: it is #NUM!
#[test]
fn names_that_reach_themselves() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (name, formula) in [
        // Itself
        ("Loop", "Loop+1"),
        // Each other
        ("CycA", "CycB+0"),
        ("CycB", "CycA+0"),
        // Each other, as references
        ("RefA", "RefB"),
        ("RefB", "RefA"),
        // Each other, unless K1 is 1
        ("Maybe", "IF(Sheet1!$K$1=1,1,Other+1)"),
        ("Other", "Maybe+1"),
        // Not a loop: a name used twice in a formula
        ("Base", "Sheet1!$K$2*2"),
        ("Twice", "Base+Base"),
        // Not a loop: the cell to the right of the cell that uses the name
        ("Next", "Sheet1!B1+1"),
        // Not a loop: the workbook name uses the name local to Data
        ("Price", "Data!Price*2"),
        // A name for a name for a cell: read for its value it is the number
        // in the cell, not a reference to it
        ("Cell", "Sheet1!$K$2"),
        ("Alias", "Cell"),
        // A loop through a cell is a circular reference, as before
        ("Mine", "Sheet1!$A$8+1"),
    ] {
        model
            .new_defined_name(DefinedNameEntry::new(name, formula))
            .unwrap();
    }
    model
        .new_defined_name(DefinedNameEntry::new("Price", "10").scope(1))
        .unwrap();

    model._set("K1", "1");
    model._set("K2", "5");
    model._set("A1", "=Loop");
    model._set("A2", "=CycA");
    model._set("A3", "=COUNTIF(RefA,\"x\")");
    model._set("A4", "=Maybe");
    model._set("A5", "=Twice");
    model._set("A6", "=Price");
    model._set("A7", "=ISNUMBER(Alias)");
    model._set("A8", "=Mine");
    // M9 runs Next, which reads N9, which runs Next for itself
    model._set("M9", "=Next");
    model._set("N9", "=Next");
    model._set("O9", "5");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "1");
    assert_eq!(model._get_text("A5"), "20");
    assert_eq!(model._get_text("A6"), "20");
    assert_eq!(model._get_text("A7"), "TRUE");
    assert_eq!(model._get_text("A8"), "#CIRC!");
    assert_eq!(model._get_text("M9"), "7");
    assert_eq!(model._get_text("N9"), "6");

    model._set("K1", "2");
    model.evaluate();
    assert_eq!(model._get_text("A4"), "#NUM!");
}

// While a deep chain of cells unwinds, a read comes back empty, so a name
// that ends can take the branch that loops. That run is thrown away
#[test]
fn name_that_ends_while_cells_unwind() {
    let mut model = new_empty_model();
    model.set_max_evaluation_depth(2);
    model
        .new_defined_name(DefinedNameEntry::new("Cond", "IF(Sheet1!$Z$1=1,1,Cond+1)"))
        .unwrap();
    model._set("A1", "=Cond");
    model._set("Z1", "=Z2");
    model._set("Z2", "=Z3");
    model._set("Z3", "1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
}
