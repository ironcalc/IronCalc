#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Non-ASCII identifiers (e.g. =ä, =ы) must produce #NAME?, not #ERROR!.
// Excel treats any unrecognised name — including ones with non-ASCII letters — as a
// name error, never as a parse error.
#[test]
fn issue_751() {
    let mut model = new_empty_model();
    // Single non-ASCII letter (German umlaut)
    model._set("A1", "=ä");
    // Multi-character non-ASCII name (Cyrillic)
    model._set("A2", "=привет");
    // Mixed ASCII/non-ASCII
    model._set("A3", "=fórmula");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NAME?");
    assert_eq!(model._get_text("A2"), "#NAME?");
    assert_eq!(model._get_text("A3"), "#NAME?");
}
