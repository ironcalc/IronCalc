#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test] 
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DMIN()");
    model._set("A2", "=DMIN(2)");
    model._set("A3", "=DMIN(1, 2)");
    model._set("A4", "=DMIN(1, 2, 3, 4)");

    model._set("A5", "=DMAX()");
    model._set("A6", "=DMAX(2)");
    model._set("A7", "=DMAX(1, 2)");
    model._set("A8", "=DMAX(1, 2, 3, 4)");

    model._set("A9", "=DAVERAGE()");
    model._set("A10", "=DAVERAGE(2)");
    model._set("A11", "=DAVERAGE(1, 2)");
    model._set("A12", "=DAVERAGE(1, 2, 3, 4)");
    
    model._set("A13", "=DSUM()");
    model._set("A14", "=DSUM(2)");
    model._set("A15", "=DSUM(1, 2)");
    model._set("A16", "=DSUM(1, 2, 3, 4)");

    model._set("A17", "=DCOUNT()");
    model._set("A18", "=DCOUNT(2)");
    model._set("A19", "=DCOUNT(1, 2)");
    model._set("A20", "=DCOUNT(1, 2, 3, 4)");    

    model._set("A21", "=DGET()");
    model._set("A22", "=DGET(2)");
    model._set("A23", "=DGET(1, 2)");
    model._set("A24", "=DGET(1, 2, 3, 4)");

    

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"#ERROR!"); 
    assert_eq!(model._get_text("A11"), *"#ERROR!");
    assert_eq!(model._get_text("A12"), *"#ERROR!");

    assert_eq!(model._get_text("A13"), *"#ERROR!");
    assert_eq!(model._get_text("A14"), *"#ERROR!"); 
    assert_eq!(model._get_text("A15"), *"#ERROR!");
    assert_eq!(model._get_text("A16"), *"#ERROR!");

    assert_eq!(model._get_text("A17"), *"#ERROR!");
    assert_eq!(model._get_text("A18"), *"#ERROR!");
    assert_eq!(model._get_text("A19"), *"#ERROR!");
    assert_eq!(model._get_text("A20"), *"#ERROR!");

    assert_eq!(model._get_text("A21"), *"#ERROR!");
    assert_eq!(model._get_text("A22"), *"#ERROR!");
    assert_eq!(model._get_text("A23"), *"#ERROR!");
    assert_eq!(model._get_text("A24"), *"#ERROR!");
}
