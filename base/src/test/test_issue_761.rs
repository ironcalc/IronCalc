#![allow(clippy::unwrap_used)]

// Issue #761 — locale dates corrupt when double-click edited after a locale switch.
//
// Root cause: `get_localized_cell_content` (edit bar) and
// `get_formatted_cell_value` (grid) both used the stored format *string*
// (e.g. "m/d/yy" for en-US) rather than the locale's own short-date pattern.
// In a day-first locale (e.g. en-GB) the edit bar would show a month-first
// string; pressing Enter would then mis-parse the date and corrupt it.
//
// The fix: simple locale dates are stored with numFmtId 14
// (LOCALE_SHORT_DATE_FMT_ID).  Both render functions detect ID 14 and derive
// the format from `locale.dates.date_formats.short` at runtime.

use crate::{
    cell::CellValue, model::Model, number_format::LOCALE_SHORT_DATE_FMT_ID,
    test::util::new_empty_model,
};

fn en_gb_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en-GB", "UTC", "en").unwrap()
}

fn de_model<'a>() -> Model<'a> {
    Model::new_empty("model", "de", "UTC", "en").unwrap()
}

// ── en-US ──────────────────────────────────────────────────────────────────

/// Entering a locale date in en-US stores the correct serial and the edit bar
/// shows the locale's short-date pattern ("m/d/yy").  Because the cell carries
/// numFmtId 14, the edit bar is locale-derived; re-entering the shown string
/// in en-US unambiguously reproduces the same date.
#[test]
fn en_us_round_trip_stable() {
    let mut model = new_empty_model(); // en-US
    model._set("A1", "4/3/2025"); // April 3, 2025 (month/day in en-US)
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    // en-US short-date format is "m/d/yy" — edit bar shows 2-digit year.
    assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), "4/3/25");
}

// ── en-GB ──────────────────────────────────────────────────────────────────

/// Same date (April 3, 2025) entered in en-GB (dd/mm/yyyy) must produce the
/// same serial as in en-US and the edit bar must show the locale format.
#[test]
fn en_gb_round_trip_stable() {
    let mut model = en_gb_model();
    model._set("A1", "03/04/2025"); // April 3, 2025 (day/month in en-GB)
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    // en-GB short-date format is "dd/mm/yyyy" — same as the input string.
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

/// Grid display in en-GB must use the locale's "dd/mm/yyyy" pattern, not the
/// en-US "m/d/yy" that is the stored format-string default for numFmtId 14.
#[test]
fn en_gb_display_uses_locale_format() {
    let mut model = en_gb_model();
    model._set("A1", "03/04/2025"); // April 3
    model.evaluate();

    assert_eq!(model._get_text("A1"), "03/04/2025");
}

// ── numFmtId invariants ────────────────────────────────────────────────────

/// After entering a locale date, the cell style must store numFmtId=14
/// (LOCALE_SHORT_DATE_FMT_ID) — not a custom format string.  This is the
/// structural guarantee that makes locale-derived rendering possible.
#[test]
fn locale_date_stored_as_num_fmt_id_14() {
    let mut model = new_empty_model();
    model._set("A1", "4/3/2025");
    model.evaluate();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_eq!(
        num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "locale date must be stored as numFmtId {LOCALE_SHORT_DATE_FMT_ID}, got {num_fmt_id}"
    );
}

/// ISO dates (yyyy/mm/dd) must use a literal format string, NOT numFmtId=14,
/// because ISO format is locale-independent and must be preserved as-is.
#[test]
fn iso_date_is_not_stored_as_id_14() {
    let mut model = new_empty_model();
    model._set("A1", "2025/03/04"); // ISO: year first
    model.evaluate();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_ne!(
        num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "ISO date must NOT use numFmtId 14 — it has a specific format string"
    );
    // The stored serial is still March 4, 2025 regardless of format.
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45720.0))
    );
    // Grid display uses the literal format, not the locale pattern.
    assert_eq!(model._get_text("A1"), "2025/03/04");
}

// ── Separator variants ─────────────────────────────────────────────────────

/// Two-digit year input ("4/3/25") must parse as April 3, 2025 in en-US.
/// The edit bar renders with the locale format "m/d/yy" — identical to the
/// input — confirming the round-trip is stable.
#[test]
fn two_digit_year_input_en_us() {
    let mut model = new_empty_model();
    model._set("A1", "4/3/25"); // en-US: month/day/2-digit-year
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    // Edit bar uses "m/d/yy" — same as input, confirming round-trip stability.
    assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), "4/3/25");
}

/// Hyphen separator ("03-04-2025") must be accepted in en-GB (day-first).
/// The edit bar normalises separators to the locale's own pattern (slashes).
#[test]
fn hyphen_separator_en_gb() {
    let mut model = en_gb_model();
    model._set("A1", "03-04-2025"); // April 3, 2025 with hyphens
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    // Edit bar uses locale format "dd/mm/yyyy" regardless of input separator.
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

// ── German locale ──────────────────────────────────────────────────────────

/// German locale uses dots as separators and day-first order ("dd.mm.yy").
/// Both grid display and edit-bar must use this pattern.
#[test]
fn german_locale_round_trip() {
    let mut model = de_model();
    model._set("A1", "03.04.2025"); // April 3, 2025 in German dd.mm.yyyy
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    // German short-date format is "dd.mm.yy" (2-digit year).
    assert_eq!(model._get_text("A1"), "03.04.25");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03.04.25"
    );
}

// ── Manual format preservation ─────────────────────────────────────────────

/// When a cell is pre-formatted with an explicit date format (e.g. "dd mmmm
/// yyyy"), entering a new date must NOT overwrite that format with numFmtId 14.
/// The `should_apply_format` guard protects date→date reassignments.
#[test]
fn manual_date_format_preserved_on_entry() {
    let mut model = new_empty_model();

    // Pre-format A1 with an explicit long date format.
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = "dd mmmm yyyy".to_string();
    model.set_cell_style(0, 1, 1, &style).unwrap();

    // Enter a locale date — must keep the explicit format, not switch to ID 14.
    model._set("A1", "4/3/2025");
    model.evaluate();

    let style_after = model.get_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        style_after.num_fmt, "dd mmmm yyyy",
        "manual date format must be preserved when a date is re-entered"
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
}

// ── Invalid date inputs ─────────────────────────────────────────

/// Month 13 is not a valid month.  `parse_date` must reject it and the input
/// must be stored as a text string — not as a date cell with numFmtId=14.
#[test]
fn invalid_month_stored_as_text() {
    let mut model = new_empty_model();
    model._set("A1", "13/01/2025");
    model.evaluate();

    // Stored as a raw string, not as a number.
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::String("13/01/2025".to_string()))
    );
    // Must NOT have acquired ID-14 style.
    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_ne!(num_fmt_id, LOCALE_SHORT_DATE_FMT_ID);
}

/// February 29 does not exist in 2025 (not a leap year).  The input must be
/// rejected by `date_to_serial_number` and stored as text.
#[test]
fn feb_29_non_leap_year_stored_as_text() {
    let mut model = new_empty_model();
    model._set("A1", "02/29/2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::String("02/29/2025".to_string()))
    );
}

/// April has 30 days; day 31 is out of range.  Must be stored as text.
#[test]
fn day_overflow_stored_as_text() {
    let mut model = new_empty_model();
    model._set("A1", "04/31/2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::String("04/31/2025".to_string()))
    );
}

// ── Format-side boundaries ─────────────────────────────────────

/// Entering a plain number must not create a locale-date cell.
/// `parse_formatted_number` returns `None` for the format spec, so no style
/// change is applied and the cell retains its default (non-date) format.
#[test]
fn plain_number_does_not_create_date_cell() {
    let mut model = new_empty_model();
    model._set("A1", "42");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(42.0))
    );
    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_ne!(
        num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "a plain number must not acquire numFmtId 14"
    );
}

/// Entering a non-date value (e.g. a percentage) into a cell already carrying
/// numFmtId=14 must REPLACE the date format.  The `should_apply_format` guard
/// only skips the assignment when BOTH the old AND new values are dates.
#[test]
fn non_date_entry_into_date_cell_replaces_format() {
    let mut model = new_empty_model();

    // Pre-format A1 as a locale-date cell.
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = "mm-dd-yy".to_string(); // → numFmtId=14
    model.set_cell_style(0, 1, 1, &style).unwrap();

    // "30%" is not a date — should_apply_format is true so the format changes.
    model._set("A1", "30%");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(0.3))
    );
    let style_after = model.get_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        style_after.num_fmt, "#,##0%",
        "percent entry must replace the locale-date format"
    );
}

// ── Cross-locale rendering ────────────────────────────────────────────────

/// The core issue 761 scenario: a date serial stored with numFmtId=14
/// (as produced by en-US entry) must render with the *current* model's locale
/// format — not with the en-US literal "m/d/yy" embedded in DEFAULT_NUM_FMTS.
///
/// Simulates: date entered in en-US → serialised → loaded in an en-GB model.
/// "mm-dd-yy" is DEFAULT_NUM_FMTS[14], so `set_cell_style` stores numFmtId=14
/// exactly as en-US entry would have done.
#[test]
fn cross_locale_serial_displays_as_locale_format() {
    let serial = 45750.0; // April 3, 2025

    let mut model = en_gb_model();

    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = "mm-dd-yy".to_string(); // maps to numFmtId=14 in DEFAULT_NUM_FMTS
    model.set_cell_style(0, 1, 1, &style).unwrap();
    model.update_cell_with_number(0, 1, 1, serial).unwrap();
    model.evaluate();

    // Must show en-GB day-first format, NOT the en-US "4/3/25".
    assert_eq!(model._get_text("A1"), "03/04/2025");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

// ── Formula cell rendering ────────────────────────────────────────────────

/// =DATE(2025,4,3) in a cell pre-formatted as numFmtId=14 must display with
/// the locale's short-date pattern.  This exercises the formula-cell branch of
/// `get_formatted_cell_value`, which applies the same ID-14 check as the
/// raw-value path.
#[test]
fn formula_date_result_respects_locale_format() {
    let mut model = en_gb_model();

    // Pre-format B1 with ID 14 ("mm-dd-yy" → numFmtId=14).
    let mut style = model.get_style_for_cell(0, 1, 2).unwrap();
    style.num_fmt = "mm-dd-yy".to_string();
    model.set_cell_style(0, 1, 2, &style).unwrap();

    // `update_cell_with_formula` reads the current style index and preserves it.
    model
        .update_cell_with_formula(0, 1, 2, "=DATE(2025,4,3)".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(45750.0))
    );
    // Grid display uses en-GB locale format for the ID-14 formula cell.
    assert_eq!(model._get_text("B1"), "03/04/2025");
}

// ── Excel date functions ──────────────────────────────────────────────────

/// YEAR(), MONTH(), DAY() on a locale-date cell must return the correct
/// calendar components — proving the stored serial is semantically correct,
/// not just that the display looks right.
///
/// This catches silent mis-parses (e.g. month and day swapped) that would
/// produce the same display text but a wrong serial.
#[test]
fn date_functions_extract_correct_components() {
    let mut model = new_empty_model(); // en-US: month/day order
    model._set("A1", "4/3/2025"); // April 3, 2025
    model._set("B1", "=YEAR(A1)");
    model._set("C1", "=MONTH(A1)");
    model._set("D1", "=DAY(A1)");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(2025.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!C1"),
        Ok(CellValue::Number(4.0)), // April — not 3 (would indicate month/day swap)
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!D1"),
        Ok(CellValue::Number(3.0))
    );
}

// ── Date arithmetic ───────────────────────────────────────────────────────

/// Locale date cells must work as formula operands.  =A1+30 must add 30 days
/// to the stored serial, proving ID-14 cells are plain numbers internally.
/// =B1-A1 must recover the exact day count (no rounding or format interference).
#[test]
fn date_arithmetic_composes_correctly() {
    let mut model = new_empty_model(); // en-US
    model._set("A1", "4/3/2025"); // April 3, 2025 = serial 45750
    model._set("B1", "=A1+30"); // May 3, 2025 = serial 45780
    model._set("C1", "=B1-A1"); // difference must be exactly 30
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(45780.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!C1"),
        Ok(CellValue::Number(30.0))
    );
}
