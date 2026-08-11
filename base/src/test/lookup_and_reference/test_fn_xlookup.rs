#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// Two key columns and a value column, the classic two-column-join shape.
fn join_model() -> crate::model::Model<'static> {
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("A2", "b");
    model._set("B1", "x");
    model._set("B2", "y");
    model._set("C1", "first");
    model._set("C2", "second");
    model
}

// ── computed lookup_array ────────────────────────────────────────────────────

#[test]
fn test_xlookup_computed_lookup_array_text_join() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b|y\",A1:A2&\"|\"&B1:B2,C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

#[test]
fn test_xlookup_computed_lookup_array_does_not_bypass_if_not_found() {
    let mut model = join_model();
    // The value is present, so if_not_found must not be reached.
    model._set(
        "E1",
        "=XLOOKUP(\"b|y\",A1:A2&\"|\"&B1:B2,C1:C2,\"fallback\")",
    );
    // The value is absent, so if_not_found must be reached.
    model._set(
        "E2",
        "=XLOOKUP(\"z|z\",A1:A2&\"|\"&B1:B2,C1:C2,\"fallback\")",
    );
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
    assert_eq!(model._get_text("E2"), "fallback");
}

#[test]
fn test_xlookup_computed_lookup_array_numeric() {
    let mut model = join_model();
    model._set("D1", "1");
    model._set("D2", "2");
    model._set("E1", "=XLOOKUP(4,D1:D2*2,C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

/// Documents an adjacent gap: `ROW(range)` does not spill in IronCalc, it
/// returns the first row as a scalar, so `XLOOKUP(2,ROW(A1:A2),C1:C2)` is a
/// scalar `lookup_array`, not a computed one. Out of scope here.
#[test]
fn test_row_over_a_range_does_not_spill() {
    let mut model = join_model();
    model._set("E1", "=ROW(A1:A2)");
    model._set("E2", "=ROWS(A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("E2"), "2");
}

#[test]
fn test_xlookup_computed_lookup_array_boolean_product() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(1,(A1:A2=\"b\")*(B1:B2=\"y\"),C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

#[test]
fn test_xlookup_array_literal_lookup_array() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b\",{\"a\";\"b\"},C1:C2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

// ── computed return_array ────────────────────────────────────────────────────

#[test]
fn test_xlookup_computed_return_array() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b\",A1:A2,C1:C2&\"\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

#[test]
fn test_xlookup_both_arrays_computed() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b|y\",A1:A2&\"|\"&B1:B2,C1:C2&\"!\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second!");
}

// ── shape and error handling over computed arrays ────────────────────────────

#[test]
fn test_xlookup_computed_arrays_of_different_length() {
    let mut model = join_model();
    model._set("A3", "c");
    model._set("E1", "=XLOOKUP(\"b\",A1:A3&\"\",C1:C2&\"\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#VALUE!");
}

#[test]
fn test_xlookup_row_oriented_computed_arrays() {
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("B1", "b");
    model._set("A2", "first");
    model._set("B2", "second");
    model._set("E1", "=XLOOKUP(\"b\",A1:B1&\"\",A2:B2&\"\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
}

#[test]
fn test_xlookup_computed_lookup_array_binary_search() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "one");
    model._set("B2", "two");
    model._set("B3", "three");
    // search_mode 2, ascending binary search over a computed array
    model._set("E1", "=XLOOKUP(2,A1:A3*1,B1:B3,\"nf\",0,2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "two");
}

// ── the scalar/stored-range path must be unchanged ───────────────────────────

#[test]
fn test_xlookup_stored_ranges_still_work() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b\",A1:A2,C1:C2)");
    model._set("E2", "=XLOOKUP(\"zzz\",A1:A2,C1:C2)");
    model._set("E3", "=XLOOKUP(\"zzz\",A1:A2,C1:C2,\"nf\")");
    model._set("E4", "=XLOOKUP(\"b\",A1:A2,3)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
    assert_eq!(model._get_text("E2"), "#N/A");
    assert_eq!(model._get_text("E3"), "nf");
    assert_eq!(model._get_text("E4"), "#VALUE!");
}

#[test]
fn test_xlookup_whole_column_ranges() {
    let mut model = join_model();
    model._set("E1", "=XLOOKUP(\"b\",A:A,C:C)");
    model._set("E2", "=XLOOKUP(\"zzz\",A:A,C:C,\"nf\")");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "second");
    assert_eq!(model._get_text("E2"), "nf");
}

// ── control group: the same computed array works in other lookups ────────────

#[test]
fn test_match_and_vlookup_over_the_same_computed_array() {
    let mut model = join_model();
    model._set("E1", "=MATCH(\"b|y\",A1:A2&\"|\"&B1:B2,0)");
    model._set("E2", "=XMATCH(\"b|y\",A1:A2&\"|\"&B1:B2)");
    model._set("E3", "=VLOOKUP(\"b|y\",A1:A2&\"|\"&B1:B2,1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("E2"), "2");
    assert_eq!(model._get_text("E3"), "b|y");
}
