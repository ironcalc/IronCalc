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

// ---------------------------------------------------------------------------
// Validation — struct form with inconsistent id/code
// ---------------------------------------------------------------------------

#[test]
fn deser_struct_inconsistent_id_falls_back_to_code() {
    // num_fmt_id 14 maps to "mm-dd-yy", not "0.00%".
    // When the two disagree, format_code wins (it is the source of truth).
    let json = r#"{"num_fmt_id":14,"format_code":"0.00%"}"#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    // format_code is preserved as-is; num_fmt_id is re-derived from it.
    assert_eq!(fmt.format_code, "0.00%");
    // "0.00%" is ECMA-376 built-in ID 10.
    assert_eq!(fmt.num_fmt_id, 10);
}

#[test]
fn deser_struct_missing_format_code_is_error() {
    // format_code is required; a struct with only num_fmt_id must fail.
    let json = r#"{"num_fmt_id":14}"#;
    let result: Result<NumFmt, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Validation — custom format codes with stored_id in/out of ECMA custom range
// ---------------------------------------------------------------------------

#[test]
fn deser_rejects_builtin_range_id_for_custom_format_code() {
    // stored_id=5 is a built-in range ID; format_code "0.000##" is not a built-in.
    // Accepting a built-in-range ID for a custom code would create an inconsistent
    // NumFmt where the id and code disagree — corrupt on XLSX export.
    let json = r#"{"num_fmt_id": 5, "format_code": "0.000##"}"#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.format_code, "0.000##");
    assert_eq!(fmt.num_fmt_id, -1, "built-in-range stored_id must be discarded for a custom code");
}

#[test]
fn deser_accepts_custom_range_id_for_custom_format_code() {
    // stored_id=180 is in the ECMA-376 custom range (≥ 164) — round-trip must preserve it.
    let json = r#"{"num_fmt_id": 180, "format_code": "0.000##"}"#;
    let fmt: NumFmt = serde_json::from_str(json).unwrap();
    assert_eq!(fmt.num_fmt_id, 180);
    assert_eq!(fmt.format_code, "0.000##");
}
