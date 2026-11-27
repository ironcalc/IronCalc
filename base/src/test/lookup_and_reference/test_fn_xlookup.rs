#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A1:A3 = ids, B1:D3 = rows of data, F1:H1 = headers, F2:H4 = columns of data
fn model() -> crate::model::Model<'static> {
    let mut model = new_empty_model();
    for (r, (id, b, c, d)) in [(1, "ann", "x", 10), (2, "bob", "y", 20), (3, "cy", "z", 30)]
        .iter()
        .enumerate()
    {
        let row = r + 1;
        model._set(&format!("A{row}"), &id.to_string());
        model._set(&format!("B{row}"), b);
        model._set(&format!("C{row}"), c);
        model._set(&format!("D{row}"), &d.to_string());
    }
    model._set("F1", "a");
    model._set("G1", "b");
    model._set("H1", "c");
    model._set("F2", "1");
    model._set("G2", "2");
    model._set("H2", "3");
    model._set("F3", "4");
    model._set("G3", "5");
    model._set("H3", "6");
    model
}

#[test]
fn single_column_return_is_unchanged() {
    let mut model = model();
    model._set("K1", "=XLOOKUP(2,A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("K1"), "bob");
    assert_eq!(model._get_text("L1"), "");
}

#[test]
fn multi_column_return_spills_the_row() {
    // Issue #1789
    let mut model = model();
    model._set("K1", "=XLOOKUP(2,A1:A3,B1:D3)");
    model._set("K3", "=XLOOKUP(3,A1:A3,B1:D3,,0,-1)");
    model.evaluate();
    assert_eq!(model._get_text("K1"), "bob");
    assert_eq!(model._get_text("L1"), "y");
    assert_eq!(model._get_text("M1"), "20");
    assert_eq!(model._get_text("K3"), "cy");
    assert_eq!(model._get_text("M3"), "30");
}

#[test]
fn multi_column_return_with_binary_search_and_approximate_match() {
    let mut model = model();
    model._set("K1", "=XLOOKUP(2,A1:A3,B1:D3,,0,2)");
    model._set("K2", "=XLOOKUP(2.5,A1:A3,B1:D3,,-1,2)");
    model.evaluate();
    assert_eq!(model._get_text("M1"), "20");
    assert_eq!(model._get_text("L2"), "y"); // next smaller item is 2
}

#[test]
fn horizontal_lookup_with_taller_return_spills_the_column() {
    let mut model = model();
    model._set("K1", r#"=XLOOKUP("b",F1:H1,F1:H3)"#);
    model.evaluate();
    assert_eq!(model._get_text("K1"), "b");
    assert_eq!(model._get_text("K2"), "2");
    assert_eq!(model._get_text("K3"), "5");
}

#[test]
fn return_array_must_span_the_lookup_array() {
    let mut model = model();
    model._set("K1", "=XLOOKUP(2,A1:A3,B1:D2)");
    model.evaluate();
    assert_eq!(model._get_text("K1"), "#VALUE!");
}

#[test]
fn not_found_still_returns_if_not_found() {
    let mut model = model();
    model._set("K1", r#"=XLOOKUP(9,A1:A3,B1:D3,"none")"#);
    model._set("K2", "=XLOOKUP(9,A1:A3,B1:D3)");
    model.evaluate();
    assert_eq!(model._get_text("K1"), "none");
    assert_eq!(model._get_text("K2"), "#N/A");
}
