use crate::{
    formatter::{self, format::Formatted},
    locale::get_locale,
};

/// ECMA-376 built-in number format strings as `(numFmtId, format_code)` pairs.
///
/// **Do not change IDs** — the numFmtId values are part of the ECMA-376 spec.
/// Entries can be reordered freely; IDs must remain stable.
/// See `builtin_num_fmts_spec_ids_are_correct` for guards on critical entries.
pub(crate) const DEFAULT_NUM_FMTS: &[(i32, &str)] = &[
    (0, "general"),
    (1, "0"),
    (2, "0.00"),
    (3, "#,##0"),
    (4, "#,##0.00"),
    (5, r#"$#,##0; \ - $#,##0"#),
    (6, r#"$#,##0; [Red] \ - $#,##0"#),
    (7, r#"$#,##0.00; \ - $#,##0.00"#),
    (8, r#"$#,##0.00; [Red] \ - $#,##0.00"#),
    (9, "0%"),
    (10, "0.00%"),
    (11, "0.00E + 00"),
    (12, "#?/?"),
    (13, "#?? / ??"),
    (14, "mm-dd-yy"),
    (15, "d-mmm-yy"),
    (16, "d-mmm"),
    (17, "mmm-yy"),
    (18, "h:mm AM / PM"),
    (19, "h:mm:ss AM / PM"),
    (20, "h:mm"),
    (21, "h:mm:ss"),
    (22, "m / d / yy h:mm"),
    (23, "#,##0;()#,##0)"),
    (24, "#,##0; [Red]()#,##0)"),
    (25, "#,##0.00;()#,##0.00)"),
    (26, "#,##0.00; [Red]()#,##0.00)"),
    (
        27,
        r#"_()$”*#,##0.00 _); _()$”* \()#,##0.00\); _()$”* - ?? _); _()@_)"#,
    ),
    (28, "mm:ss"),
    (29, "[h]:mm:ss"),
    (30, "mmss .0"),
    (31, "##0.0E + 0"),
    (32, "@"),
    (33, "[$ -404] e / m / d "),
    (34, "m / d / yy"),
    (35, "[$ -404] e / m / d"),
    (36, "[$ -404] e / / d"),
    (37, "[$ -404] e / m / d"),
    (38, "t0"),
    (39, "t0.00"),
    (40, "t#,##0"),
    (41, "t#,##0.00"),
    (42, "t0%"),
    (43, "t0.00 %"),
    (44, "t#?/?"),
];

/// Zero-sized accessor for the built-in number format table.
///
/// Centralises all pattern matching over `NUM_FMTS` so call sites read clearly.
pub(crate) struct DefaultFmts;

impl DefaultFmts {
    /// Format code for a built-in `numFmtId`, or `None` if not a built-in.
    pub(crate) fn by_id(id: i32) -> Option<&'static str> {
        DEFAULT_NUM_FMTS
            .iter()
            .find(|&&(fid, _)| fid == id)
            .map(|&(_, s)| s)
    }

    /// Built-in `numFmtId` for a format code, or `None` if not a built-in.
    pub(crate) fn by_code(code: &str) -> Option<i32> {
        DEFAULT_NUM_FMTS
            .iter()
            .find(|&&(_, s)| s == code)
            .map(|&(id, _)| id)
    }

    /// True if `id` belongs to the built-in table.
    pub(crate) fn contains_id(id: i32) -> bool {
        Self::by_id(id).is_some()
    }

    /// ECMA-376 numFmtId 14: locale short date.
    pub(crate) const SHORT_DATE_ID: i32 = 14;
    /// ECMA-376 numFmtId 22: locale short date+time.
    pub(crate) const SHORT_DATETIME_ID: i32 = 22;

    /// True if `id` is a locale date sentinel (14 or 22).
    pub(crate) fn is_locale_date(id: i32) -> bool {
        id == Self::SHORT_DATE_ID || id == Self::SHORT_DATETIME_ID
    }
}

pub fn to_precision(value: f64, precision: usize) -> f64 {
    if value.is_infinite() || value.is_nan() {
        return value;
    }
    to_precision_str(value, precision)
        .parse::<f64>()
        .unwrap_or({
            // TODO: do this in a way that does not require a possible error
            0.0
        })
}

/// It rounds a `f64` with `p` significant figures:
/// ```
///     use ironcalc_base::number_format;
///     assert_eq!(number_format::to_precision(0.1+0.2, 15), 0.3);
///     assert_eq!(number_format::to_excel_precision_str(0.1+0.2), "0.3");
/// ```
/// This intends to be equivalent to the js: `${parseFloat(value.toPrecision(precision)})`
/// See ([ecma](https://tc39.es/ecma262/#sec-number.prototype.toprecision)).
pub fn to_excel_precision_str(value: f64) -> String {
    to_precision_str(value, 15)
}

pub fn to_excel_precision(value: f64, precision: usize) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let s = format!("{:.*e}", precision.saturating_sub(1), value);
    s.parse::<f64>().unwrap_or(value)
}

pub fn to_precision_str(value: f64, precision: usize) -> String {
    if !value.is_finite() {
        if value.is_infinite() {
            return "inf".to_string();
        } else {
            return "NaN".to_string();
        }
    }

    let s = format!("{:.*e}", precision.saturating_sub(1), value);
    let parsed = s.parse::<f64>().unwrap_or(value);

    // I would love to use the std library. There is not a speed concern here
    // problem is it doesn't do the right thing
    // Also ryu is my favorite _modern_ algorithm
    let mut buffer = ryu::Buffer::new();
    let text = buffer.format(parsed);
    // The above algorithm converts 2 to 2.0 regrettably
    if let Some(stripped) = text.strip_suffix(".0") {
        return stripped.to_string();
    }
    text.to_string()
}

pub fn format_number(value: f64, format_code: &str, locale: &str) -> Formatted {
    let locale = match get_locale(locale) {
        Ok(l) => l,
        Err(_) => {
            return Formatted {
                text: "#ERROR!".to_owned(),
                color: None,
                error: Some("Invalid locale".to_string()),
            }
        }
    };
    formatter::format::format_number(value, format_code, locale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_num_fmts_spec_ids_are_correct() {
        assert_eq!(
            DEFAULT_NUM_FMTS.len(),
            45,
            "NUM_FMTS length changed — update and verify all numFmtIds"
        );
        assert_eq!(
            DefaultFmts::by_id(0),
            Some("general"),
            "numFmtId 0 must be General"
        );
        assert_eq!(DefaultFmts::by_id(9), Some("0%"), "numFmtId 9 must be 0%");
        assert_eq!(
            DefaultFmts::by_id(14),
            Some("mm-dd-yy"),
            "numFmtId 14 must be the ECMA-376 locale short date"
        );
        assert_eq!(
            DefaultFmts::by_id(22),
            Some("m / d / yy h:mm"),
            "numFmtId 22 must be the locale short date+time"
        );
    }

    /// ECMA-376 §18.8.30 — canonical built-in format codes for the US (default) locale.
    ///
    /// Reference: Apache POI `BuiltinFormats.java`, cross-checked with the spec.
    /// IDs 23–36 are locale-specific (East-Asian date/time) and absent for US locale.
    /// IDs 45–49 are the ECMA-376 time/special formats currently missing from this table.
    ///
    /// This test is intentionally RED until DEFAULT_NUM_FMTS is corrected.
    #[test]
    #[ignore = "not yet implemented"]
    fn builtin_num_fmts_ecma376_us_locale() {
        let cases: &[(i32, &str)] = &[
            (0, "general"),
            (1, "0"),
            (2, "0.00"),
            (3, "#,##0"),
            (4, "#,##0.00"),
            // IDs 5–8: US-dollar currency, negative in parentheses with alignment gap.
            (5, "$#,##0_);($#,##0)"),
            (6, "$#,##0_);[Red]($#,##0)"),
            (7, "$#,##0.00_);($#,##0.00)"),
            (8, "$#,##0.00_);[Red]($#,##0.00)"),
            (9, "0%"),
            (10, "0.00%"),
            (11, "0.00E+00"), // no spaces around +
            (12, "# ?/?"),    // space before ?
            (13, "# ??/??"),  // space before ?, no spaces around /
            (14, "m/d/yy"),   // ECMA-376 locale short-date sentinel
            (15, "d-mmm-yy"),
            (16, "d-mmm"),
            (17, "mmm-yy"),
            (18, "h:mm AM/PM"), // no spaces around /
            (19, "h:mm:ss AM/PM"),
            (20, "h:mm"),
            (21, "h:mm:ss"),
            (22, "m/d/yy h:mm"), // no spaces around /
            // 23–36: absent — locale-specific (East-Asian), not defined for US locale.
            // 37–40: parenthesised accounting variants (no currency symbol).
            (37, "#,##0_);(#,##0)"),
            (38, "#,##0_);[Red](#,##0)"),
            (39, "#,##0.00_);(#,##0.00)"),
            (40, "#,##0.00_);[Red](#,##0.00)"),
            // 41–44: fill-character accounting variants.
            (41, r#"_(* #,##0_);_(* (#,##0);_(* "-"_);_(@_)"#),
            (42, r#"_("$"* #,##0_);_("$"* (#,##0);_("$"* "-"_);_(@_)"#),
            (43, r#"_(* #,##0.00_);_(* (#,##0.00);_(* "-"??_);_(@_)"#),
            (
                44,
                r#"_("$"* #,##0.00_);_("$"* (#,##0.00);_("$"* "-"??_);_(@_)"#,
            ),
            // 45–49: time / special — currently missing from IronCalc's table.
            (45, "mm:ss"),
            (46, "[h]:mm:ss"),
            (47, "mmss.0"),
            (48, "##0.0E+0"),
            (49, "@"),
        ];

        for &(id, expected) in cases {
            assert_eq!(
                DefaultFmts::by_id(id),
                Some(expected),
                "numFmtId {id}: expected {expected:?}, got {:?}",
                DefaultFmts::by_id(id)
            );
        }

        // IDs 23–36 must be absent for US locale.
        for id in 23_i32..=36 {
            assert!(
                DefaultFmts::by_id(id).is_none(),
                "numFmtId {id} must be absent for US locale, got {:?}",
                DefaultFmts::by_id(id)
            );
        }
    }
}
