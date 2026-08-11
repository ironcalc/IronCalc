#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

fn setup() -> crate::model::Model<'static> {
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("A2", "b");
    model._set("B1", "x");
    model._set("B2", "y");
    model._set("C1", "first");
    model._set("C2", "second");
    model
}

#[test]
fn test_xlookup_range_basic() {
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("A3", "cherry");
    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("D1", "=XLOOKUP(\"banana\",A1:A3,B1:B3)");
    model._set("D2", "=XLOOKUP(\"missing\",A1:A3,B1:B3)");
    model._set("D3", "=XLOOKUP(\"missing\",A1:A3,B1:B3,\"nope\")");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    assert_eq!(model._get_text("D2"), "#N/A");
    assert_eq!(model._get_text("D3"), "nope");
}

#[test]
fn test_xlookup_computed_lookup_array() {
    // A computed lookup_array must be searchable, not just a range reference (#1338).
    let mut model = setup();
    model._set("E1", "=XLOOKUP(\"b|y\",A1:A2&\"|\"&B1:B2,C1:C2)");
    model._set(
        "E2",
        "=XLOOKUP(\"z|z\",A1:A2&\"|\"&B1:B2,C1:C2,\"fallback\")",
    );
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
    assert_eq!(model._get_text("E2"), "fallback");
}

#[test]
fn test_xlookup_computed_return_array() {
    // A computed return_array (array constant) must also be indexable (#1338).
    let mut model = new_empty_model();
    model._set("A1", "apple");
    model._set("A2", "banana");
    model._set("A3", "cherry");
    model._set("D1", "=XLOOKUP(\"banana\",A1:A3,{10;20;30})");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "20");
}

#[test]
fn test_xlookup_approximate_and_binary() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    model._set("B1", "ten");
    model._set("B2", "twenty");
    model._set("B3", "thirty");
    // exact-or-next-smaller (match_mode -1): 25 -> 20 -> "twenty"
    model._set("D1", "=XLOOKUP(25,A1:A3,B1:B3,,-1)");
    // binary search ascending (search_mode 2), exact
    model._set("D2", "=XLOOKUP(30,A1:A3,B1:B3,,0,2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "twenty");
    assert_eq!(model._get_text("D2"), "thirty");
}
