#![allow(clippy::unwrap_used)]

use crate::Model;

pub fn new_german_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "de").unwrap()
}

// Non-ASCII identifiers (e.g. =ä, =ы) must produce #NAME?, not #ERROR!.
// Excel treats any unrecognised name — including ones with non-ASCII letters — as a
// name error, never as a parse error.
#[test]
fn issue_751() {
    let mut model = new_german_empty_model();
    // Single non-ASCII letter (German umlaut)
    model._set("A1", "=ä");
    // Multi-character non-ASCII name (Cyrillic)
    model._set("A2", "=привет");
    // Mixed ASCII/non-ASCII
    model._set("A3", "=fórmula");

    model._set("B1", "1");
    model._set("B2", "2");
    model._set("B3", "3");
    model._set("B4", "4");
    model._set("B5", "5");
    model._set("B6", "=ZÄHLENWENN(B1:B5,\">2\")");

    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NAME?");
    assert_eq!(model._get_text("A2"), "#NAME?");
    assert_eq!(model._get_text("A3"), "#NAME?");
    assert_eq!(model._get_text("B6"), *"3");
}
