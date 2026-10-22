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
