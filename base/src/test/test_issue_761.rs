#![allow(clippy::unwrap_used)]

// Issue #761 — locale dates corrupt when double-click edited after a locale switch.
//
// Root cause: `get_localized_cell_content` (edit bar) and `get_formatted_cell_value`
// (grid) both used the stored format string rather than the locale's own short-date
// pattern. In a day-first locale (e.g. en-GB) the edit bar would show a month-first
// string; pressing Enter would then mis-parse the date and corrupt it.
//
// Fix: simple locale dates are stored with numFmtId 14 (LOCALE_SHORT_DATE_FMT_ID).
// Both render functions detect ID 14 and derive the format from
// `locale.dates.date_formats.short` at runtime.

use crate::{
    cell::CellValue,
    model::Model,
    number_format::{LOCALE_SHORT_DATE_FMT_ID, LOCALE_SHORT_DATE_TIME_FMT_ID},
    test::util::new_empty_model,
    types::{NumFmt, Styles},
};

fn en_gb_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en-GB", "UTC", "en").unwrap()
}

fn de_model<'a>() -> Model<'a> {
    Model::new_empty("model", "de", "UTC", "en").unwrap()
}

#[test]
fn en_us_round_trip_stable() {
    // April 3, 2025 in en-US (month/day). Edit bar shows locale "m/d/yy".
    let mut model = new_empty_model();
    model._set("A1", "4/3/2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), "4/3/25");
}

#[test]
fn en_gb_round_trip_stable() {
    // April 3, 2025 in en-GB (day/month). Edit bar shows locale "dd/mm/yyyy".
    let mut model = en_gb_model();
    model._set("A1", "03/04/2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

#[test]
fn en_gb_display_uses_locale_format() {
    let mut model = en_gb_model();
    model._set("A1", "03/04/2025");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "03/04/2025");
}

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

#[test]
fn iso_date_is_not_stored_as_id_14() {
    // ISO dates (year first) are locale-independent and keep a literal format string.
    let mut model = new_empty_model();
    model._set("A1", "2025/03/04");
    model.evaluate();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_ne!(
        num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "ISO date must NOT use numFmtId 14 — it has a specific format string"
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45720.0))
    );
    assert_eq!(model._get_text("A1"), "2025/03/04");
}

#[test]
fn two_digit_year_input_en_us() {
    let mut model = new_empty_model();
    model._set("A1", "4/3/25");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(model.get_localized_cell_content(0, 1, 1).unwrap(), "4/3/25");
}

#[test]
fn hyphen_separator_en_gb() {
    // Hyphens are accepted; the edit bar normalises to the locale's own separator.
    let mut model = en_gb_model();
    model._set("A1", "03-04-2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

#[test]
fn german_locale_round_trip() {
    let mut model = de_model();
    model._set("A1", "03.04.2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(model._get_text("A1"), "03.04.25");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03.04.25"
    );
}

#[test]
fn manual_date_format_preserved_on_entry() {
    // A cell with an explicit date format must not have it overwritten with numFmtId 14.
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = NumFmt::from_format_code("dd mmmm yyyy");
    model.set_cell_style(0, 1, 1, &style).unwrap();

    model._set("A1", "4/3/2025");
    model.evaluate();

    let style_after = model.get_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        style_after.num_fmt.format_code, "dd mmmm yyyy",
        "manual date format must be preserved when a date is re-entered"
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(45750.0))
    );
}

#[test]
fn invalid_month_stored_as_text() {
    let mut model = new_empty_model();
    model._set("A1", "13/01/2025");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::String("13/01/2025".to_string()))
    );
    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_ne!(num_fmt_id, LOCALE_SHORT_DATE_FMT_ID);
}

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
    assert_ne!(num_fmt_id, LOCALE_SHORT_DATE_FMT_ID);
}

#[test]
fn non_date_entry_into_date_cell_replaces_format() {
    // The should_apply_format guard only preserves date format when BOTH old and new are dates.
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = NumFmt::from_format_code("mm-dd-yy"); // → numFmtId=14
    model.set_cell_style(0, 1, 1, &style).unwrap();

    model._set("A1", "30%");
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(0.3))
    );
    let style_after = model.get_style_for_cell(0, 1, 1).unwrap();
    assert_eq!(
        style_after.num_fmt.format_code, "#,##0%",
        "percent entry must replace the locale-date format"
    );
}

#[test]
fn cross_locale_serial_displays_as_locale_format() {
    // Core issue 761: a serial stored with numFmtId=14 in en-US must render
    // with the en-GB pattern when the model locale is en-GB.
    let serial = 45750.0; // April 3, 2025
    let mut model = en_gb_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = NumFmt::from_format_code("mm-dd-yy"); // → numFmtId=14
    model.set_cell_style(0, 1, 1, &style).unwrap();
    model.update_cell_with_number(0, 1, 1, serial).unwrap();
    model.evaluate();

    assert_eq!(model._get_text("A1"), "03/04/2025");
    assert_eq!(
        model.get_localized_cell_content(0, 1, 1).unwrap(),
        "03/04/2025"
    );
}

#[test]
fn formula_date_result_respects_locale_format() {
    let mut model = en_gb_model();
    let mut style = model.get_style_for_cell(0, 1, 2).unwrap();
    style.num_fmt = NumFmt::from_format_code("mm-dd-yy"); // → numFmtId=14
    model.set_cell_style(0, 1, 2, &style).unwrap();

    model
        .update_cell_with_formula(0, 1, 2, "=DATE(2025,4,3)".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(45750.0))
    );
    assert_eq!(model._get_text("B1"), "03/04/2025");
}

#[test]
fn date_functions_extract_correct_components() {
    // YEAR/MONTH/DAY on a locale-date cell prove the serial is semantically correct.
    let mut model = new_empty_model();
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

#[test]
fn date_functions_date_fn() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2025,10,11)");
    model.evaluate();

    // en-US "m/d/yy" — numFmtId 14 renders with 2-digit year.
    assert_eq!(model._get_text("A1"), "10/11/25");
}

#[test]
fn date_fn_stores_locale_fmt_id() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2025,10,11)");
    model.evaluate();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_eq!(
        num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "=DATE() result must store numFmtId={LOCALE_SHORT_DATE_FMT_ID}, got {num_fmt_id}"
    );
}

#[test]
fn date_fn_locale_switch_updates_display() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2025,10,11)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "10/11/25"); // en-US month-first

    model.set_locale("en-GB").unwrap();

    assert_eq!(model._get_text("A1"), "11/10/2025"); // en-GB day-first
}

#[test]
fn num_fmt_builtin_format_code_resolves_canonical_id() {
    let general = NumFmt::from_format_code("general");
    assert_eq!(general.num_fmt_id, 0, "\"general\" must map to numFmtId 0");

    let locale_date = NumFmt::from_format_code("mm-dd-yy");
    assert_eq!(
        locale_date.num_fmt_id, LOCALE_SHORT_DATE_FMT_ID,
        "\"mm-dd-yy\" must map to LOCALE_SHORT_DATE_FMT_ID ({})",
        LOCALE_SHORT_DATE_FMT_ID,
    );
}

#[test]
fn num_fmt_custom_format_code_has_placeholder_id() {
    // Custom codes not in the built-in table use -1 as a sentinel until registered.
    let custom = NumFmt::from_format_code("dd/mm/yyyy hh:mm:ss");
    assert_eq!(custom.format_code, "dd/mm/yyyy hh:mm:ss");
    assert_eq!(custom.num_fmt_id, -1);
}

#[test]
fn set_cell_style_reuses_cell_xfs_for_same_custom_format() {
    let mut model = new_empty_model();
    let code = "dd/mm/yyyy hh:mm:ss";

    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = NumFmt::from_format_code(code);
    model.set_cell_style(0, 1, 1, &style).unwrap();
    let xfs_len_after_first = model.workbook.styles.cell_xfs.len();

    let mut style2 = model.get_style_for_cell(0, 2, 1).unwrap();
    style2.num_fmt = NumFmt::from_format_code(code);
    model.set_cell_style(0, 2, 1, &style2).unwrap();

    assert_eq!(
        model.workbook.styles.cell_xfs.len(),
        xfs_len_after_first,
        "second apply of the same custom format must reuse the existing CellXfs entry"
    );
}

#[test]
fn get_style_with_format_no_duplicate_cell_xfs() {
    // Applying the same custom format twice via get_style_with_format must not duplicate CellXfs.
    let mut model = new_empty_model();
    let code = "dd/mm/yyyy hh:mm:ss";
    let styles = &mut model.workbook.styles;

    let idx1 = styles.get_style_with_format(0, code).unwrap();
    let xfs_len_after_first = styles.cell_xfs.len();

    let idx2 = styles.get_style_with_format(0, code).unwrap();

    assert_eq!(styles.cell_xfs.len(), xfs_len_after_first);
    assert_eq!(idx1, idx2);
}

#[test]
fn format_code_for_id_returns_correct_code() {
    let styles = Styles {
        num_fmts: vec![NumFmt {
            num_fmt_id: 164,
            format_code: "dd/mm/yyyy hh:mm:ss".to_string(),
        }],
        ..Styles::default()
    };

    assert_eq!(styles.format_code_for_id(0), "general");
    assert_eq!(styles.format_code_for_id(9), "0%");
    assert_eq!(
        styles.format_code_for_id(LOCALE_SHORT_DATE_FMT_ID),
        "mm-dd-yy"
    );
    assert_eq!(styles.format_code_for_id(164), "dd/mm/yyyy hh:mm:ss");
    assert_eq!(styles.format_code_for_id(999), "general"); // unknown → fallback
}

#[test]
fn get_style_with_num_fmt_id_rejects_orphan_id() {
    let mut model = new_empty_model();
    let styles = &mut model.workbook.styles;

    assert!(styles.get_style_with_num_fmt_id(0, 999).is_err());
}

#[test]
fn get_style_with_num_fmt_id_accepts_builtin_id() {
    let mut model = new_empty_model();
    let styles = &mut model.workbook.styles;

    assert!(styles
        .get_style_with_num_fmt_id(0, LOCALE_SHORT_DATE_FMT_ID)
        .is_ok());
}

#[test]
fn get_style_with_num_fmt_id_accepts_registered_custom_id() {
    let mut model = new_empty_model();
    let styles = &mut model.workbook.styles;

    let registered = NumFmt::get_or_register("dd/mm/yyyy hh:mm:ss", &mut styles.num_fmts);
    assert!(registered.num_fmt_id >= 0);

    assert!(styles
        .get_style_with_num_fmt_id(0, registered.num_fmt_id)
        .is_ok());
}

#[test]
fn custom_format_sentinel_never_stored_in_cell_xfs() {
    // from_format_code uses -1 as a sentinel; set_cell_style must resolve it before writing.
    let mut model = new_empty_model();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.num_fmt = NumFmt::from_format_code("dd/mm/yyyy hh:mm:ss");
    assert_eq!(
        style.num_fmt.num_fmt_id, -1,
        "sentinel must be -1 before registration"
    );

    model.set_cell_style(0, 1, 1, &style).unwrap();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let stored_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert!(
        stored_id >= 0,
        "CellXfs must not store the -1 sentinel; got {stored_id}"
    );

    let entry = model
        .workbook
        .styles
        .num_fmts
        .iter()
        .find(|f| f.num_fmt_id == stored_id);
    assert!(
        entry.is_some(),
        "custom format must be registered in num_fmts"
    );
    assert_eq!(entry.unwrap().format_code, "dd/mm/yyyy hh:mm:ss");
}

#[test]
fn now_fn_stores_locale_datetime_fmt_id() {
    // =NOW() must store numFmtId=22 (LOCALE_SHORT_DATE_TIME_FMT_ID) so that
    // locale switches can re-derive the datetime pattern at render time.
    let mut model = new_empty_model();
    model._set("A1", "=NOW()");
    model.evaluate();

    let style_index = model.get_cell_style_index(0, 1, 1).unwrap();
    let num_fmt_id = model.workbook.styles.cell_xfs[style_index as usize].num_fmt_id;
    assert_eq!(
        num_fmt_id, LOCALE_SHORT_DATE_TIME_FMT_ID,
        "=NOW() result must store numFmtId={LOCALE_SHORT_DATE_TIME_FMT_ID}, got {num_fmt_id}"
    );
}

#[test]
fn now_fn_locale_switch_updates_display() {
    // numFmtId 22 (LOCALE_SHORT_DATE_TIME_FMT_ID) must derive its format from
    // the active locale, not the literal built-in format string — the same
    // guarantee that date (numFmtId 14) provides.
    //
    // The meridiem token (AM/PM vs 24-hour) distinguishes the two locales
    // reliably regardless of the current date/time value:
    //   en-US time_formats.short = "h:mm a"  → rendered as "h:mm AM/PM"
    //   en-GB time_formats.short = "HH:mm"   → 24-hour, no meridiem token
    let mut model_us = new_empty_model(); // en-US
    model_us._set("A1", "=NOW()");
    model_us.evaluate();
    let us_display = model_us._get_text("A1");

    let mut model_gb = en_gb_model();
    model_gb._set("A1", "=NOW()");
    model_gb.evaluate();
    let gb_display = model_gb._get_text("A1");

    assert!(
        us_display.contains("AM") || us_display.contains("PM"),
        "en-US datetime must contain AM/PM meridiem token; got: {us_display}"
    );
    assert!(
        !gb_display.contains("AM") && !gb_display.contains("PM"),
        "en-GB datetime must use 24-hour format (no AM/PM); got: {gb_display}"
    );
}

#[test]
fn date_arithmetic_composes_correctly() {
    let mut model = new_empty_model();
    model._set("A1", "4/3/2025"); // April 3, 2025 = serial 45750
    model._set("B1", "=A1+30"); // May 3, 2025 = serial 45780
    model._set("C1", "=B1-A1"); // difference = 30
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
