#![allow(clippy::unwrap_used)]

use crate::types::NumFmt;

// ---------------------------------------------------------------------------
// Deserialization — legacy string form (pre-NumFmt-struct era)
// ---------------------------------------------------------------------------

#[test]
fn deser_legacy_string_builtin() {
    // "mm-dd-yy" is ECMA-376 built-in ID 14 — must round-trip to the canonical ID.
    let json = r#""mm-dd-yy""#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, 14);
    assert_eq!(fmt.format_code, "mm-dd-yy");
}

#[test]
fn deser_legacy_string_custom() {
    // A format code not in the built-in table gets sentinel -1.
    let json = r#""dd mmmm yyyy""#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, -1);
    assert_eq!(fmt.format_code, "dd mmmm yyyy");
}

#[test]
fn deser_legacy_string_general() {
    let json = r#""general""#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, 0);
    assert_eq!(fmt.format_code, "general");
}

// ---------------------------------------------------------------------------
// Deserialization — current struct form
// ---------------------------------------------------------------------------

#[test]
fn deser_struct_form() {
    let json = r#"{"num_fmt_id":14,"format_code":"mm-dd-yy"}"#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, 14);
    assert_eq!(fmt.format_code, "mm-dd-yy");
}

#[test]
fn deser_struct_extra_fields_ignored() {
    // Forward-compat: unknown fields must not cause an error.
    let json = r#"{"num_fmt_id":0,"format_code":"general","future_field":true}"#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, 0);
    assert_eq!(fmt.format_code, "general");
}

// ---------------------------------------------------------------------------
// Serialization — target: emit format_code string only (fails until Task 2)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "fails until Task 2 implements custom Serialize"]
fn ser_emits_format_code_string() {
    let fmt = NumFmt { num_fmt_id: 14, format_code: "mm-dd-yy".to_string() };
    let json = serde_json::to_string(&fmt).unwrap();
    assert_eq!(json, r#""mm-dd-yy""#);
}

// ---------------------------------------------------------------------------
// Round-trip: serialize → deserialize → same value
// ---------------------------------------------------------------------------
// NOTE: This test passes with both the old struct-emit serializer and the new
// string-emit serializer (Task 2), since the deserializer accepts both forms.

#[test]
fn round_trip_builtin() {
    let original = NumFmt { num_fmt_id: 14, format_code: "mm-dd-yy".to_string() };
    let json = serde_json::to_string(&original).unwrap();
    let restored: NumFmt = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.num_fmt_id, original.num_fmt_id);
    assert_eq!(restored.format_code, original.format_code);
}

#[test]
fn round_trip_general() {
    let original = NumFmt::default();
    let json = serde_json::to_string(&original).unwrap();
    let restored: NumFmt = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.num_fmt_id, original.num_fmt_id);
}
