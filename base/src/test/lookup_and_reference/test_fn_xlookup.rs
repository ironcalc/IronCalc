#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn new_deals_model<'a>() -> crate::model::Model<'a> {
    let mut model = new_empty_model();
    model._set("A1", "D1");
    model._set("A2", "D2");
    model._set("A3", "D3");
    model._set("A4", "D4");
    model._set("B1", "Ana");
    model._set("B2", "Ben");
    model._set("B3", "Ana");
    model._set("B4", "Chen");
    model._set("C1", "Lost");
    model._set("C2", "Won");
    model._set("C3", "Won");
    model._set("C4", "Open");
    model
}

// ── computed lookup_array and return_array (#1338) ───────────────────────────

#[test]
fn test_xlookup_computed_lookup_array() {
    let mut model = new_deals_model();
    // The first deal that Ana won
    model._set("E1", "=XLOOKUP(1,(B1:B4=\"Ana\")*(C1:C4=\"Won\"),A1:A4)");
    // A join key built from two columns
    model._set("E2", "=XLOOKUP(\"Ana|Won\",B1:B4&\"|\"&C1:C4,A1:A4)");
    // if_not_found is honoured
    model._set("E3", "=XLOOKUP(1,(B1:B4=\"Zoe\")*1,A1:A4,\"none\")");
    model._set("E4", "=XLOOKUP(1,(B1:B4=\"Zoe\")*1,A1:A4)");
    // Array literals, also with a binary search
    model._set("E5", "=XLOOKUP(\"b\",{\"a\";\"b\";\"c\"},{10;20;30})");
    model._set("E6", "=XLOOKUP(25,{10,20,30},{\"x\",\"y\",\"z\"},,-1,2)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "D3");
    assert_eq!(model._get_text("E2"), "D3");
    assert_eq!(model._get_text("E3"), "none");
    assert_eq!(model._get_text("E4"), "#N/A");
    assert_eq!(model._get_text("E5"), "20");
    assert_eq!(model._get_text("E6"), "y");
}

#[test]
fn test_xlookup_computed_return_array() {
    let mut model = new_deals_model();
    model._set("E1", "=XLOOKUP(\"D2\",A1:A4,B1:B4&\"-\"&C1:C4)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "Ben-Won");
}

#[test]
fn test_xlookup_not_a_vector() {
    let mut model = new_deals_model();
    model._set("E1", "=XLOOKUP(1,(B1:C4=\"Ana\")*1,A1:A4)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "#ERROR!");
}

// ── several columns or rows returned ─────────────────────────────────────────

#[test]
fn test_xlookup_returns_a_row() {
    let mut model = new_deals_model();
    model._set("E1", "=XLOOKUP(\"D3\",A1:A4,B1:C4)");
    model._set("E2", "=XLOOKUP(\"D4\",A1:A4,{1,2;3,4;5,6;7,8})");
    // The return array must be as tall as the lookup array
    model._set("E3", "=XLOOKUP(\"D3\",A1:A4,B1:C5)");
    // A single-cell lookup array needs a single-cell return array
    model._set("E4", "=XLOOKUP(\"D1\",A1:A1,B1:B3)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "Ana");
    assert_eq!(model._get_text("F1"), "Won");
    assert_eq!(model._get_text("E2"), "7");
    assert_eq!(model._get_text("F2"), "8");
    assert_eq!(model._get_text("E3"), "#VALUE!");
    assert_eq!(model._get_text("E4"), "#VALUE!");
}

#[test]
fn test_xlookup_returns_a_column() {
    let mut model = new_empty_model();
    model._set("A1", "Q1");
    model._set("B1", "Q2");
    model._set("C1", "Q3");
    model._set("A2", "10");
    model._set("B2", "20");
    model._set("C2", "30");
    model._set("A3", "1");
    model._set("B3", "2");
    model._set("C3", "3");
    model._set("E1", "=XLOOKUP(\"Q2\",A1:C1,A2:C3)");
    model.evaluate();

    assert_eq!(model._get_text("E1"), "20");
    assert_eq!(model._get_text("E2"), "2");
}
