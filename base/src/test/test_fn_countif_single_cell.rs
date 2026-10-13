#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Scratch probe (NOT for upstreaming as-is): does upstream reproduce the
// single-cell COUNTIF defect? Excel treats every *reference* handed to the
// criteria-range parameter as a range, even a 1x1 one, so COUNTIF(A1,"x")
// answers 0 or 1 and never #VALUE!.
#[test]
fn probe_countif_single_cell_reference() {
    let mut model = new_empty_model();

    model._set("A1", "COVERED");
    model._set("A2", "OTHER");

    // 1x1 reference, criterion matches -> Excel: 1
    model._set("C1", "=COUNTIF(A1,\"COVERED\")");
    // 1x1 reference, criterion does not match -> Excel: 0
    model._set("C2", "=COUNTIF(A2,\"COVERED\")");
    // control: honest 2-cell range -> Excel: 1
    model._set("C3", "=COUNTIF(A1:A2,\"COVERED\")");

    model.evaluate();

    println!("COUNTIF(A1,\"COVERED\")   = {}", model._get_text("C1"));
    println!("COUNTIF(A2,\"COVERED\")   = {}", model._get_text("C2"));
    println!("COUNTIF(A1:A2,\"COVERED\")= {}", model._get_text("C3"));

    assert_eq!(model._get_text("C3"), *"1", "control range case");
    assert_eq!(model._get_text("C1"), *"1", "single-cell reference, match");
    assert_eq!(model._get_text("C2"), *"0", "single-cell reference, no match");
}

// The originally-reported shape: a defined name that resolves to ONE cell.
#[test]
fn probe_countif_single_cell_defined_name() {
    let mut model = new_empty_model();

    model._set("P293", "COVERED");
    model
        .new_defined_name("OARprop_OAB_TPB_owner", None, "Sheet1!$P$293")
        .unwrap();

    model._set("C1", "=COUNTIF(OARprop_OAB_TPB_owner,\"COVERED\")");
    model._set("C2", "=IF(COUNTIF(OARprop_OAB_TPB_owner,\"COVERED\")>0,\"yes\",\"no\")");

    model.evaluate();

    println!("COUNTIF(name->$P$293) = {}", model._get_text("C1"));
    println!("IF(...>0)             = {}", model._get_text("C2"));

    assert_eq!(model._get_text("C1"), *"1", "defined name to a single cell");
    assert_eq!(model._get_text("C2"), *"yes");
}

// The same collapse feeds apply_ifs, so the whole family should be probed.
#[test]
fn probe_ifs_family_single_cell_reference() {
    let mut model = new_empty_model();

    model._set("A1", "COVERED");
    model._set("B1", "10");

    model._set("C1", "=SUMIF(A1,\"COVERED\",B1)");
    model._set("C2", "=SUMIFS(B1,A1,\"COVERED\")");
    model._set("C3", "=AVERAGEIF(A1,\"COVERED\",B1)");
    model._set("C4", "=AVERAGEIFS(B1,A1,\"COVERED\")");
    model._set("C5", "=MAXIFS(B1,A1,\"COVERED\")");
    model._set("C6", "=MINIFS(B1,A1,\"COVERED\")");
    model._set("C7", "=COUNTIFS(A1,\"COVERED\")");

    model.evaluate();

    for (cell, label) in [
        ("C1", "SUMIF"),
        ("C2", "SUMIFS"),
        ("C3", "AVERAGEIF"),
        ("C4", "AVERAGEIFS"),
        ("C5", "MAXIFS"),
        ("C6", "MINIFS"),
        ("C7", "COUNTIFS"),
    ] {
        println!("{label:<11} single-cell = {}", model._get_text(cell));
    }

    assert_eq!(model._get_text("C1"), *"10", "SUMIF");
    assert_eq!(model._get_text("C2"), *"10", "SUMIFS");
    assert_eq!(model._get_text("C3"), *"10", "AVERAGEIF");
    assert_eq!(model._get_text("C4"), *"10", "AVERAGEIFS");
    assert_eq!(model._get_text("C5"), *"10", "MAXIFS");
    assert_eq!(model._get_text("C6"), *"10", "MINIFS");
    assert_eq!(model._get_text("C7"), *"1", "COUNTIFS");
}
