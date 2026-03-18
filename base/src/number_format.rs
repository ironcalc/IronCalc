use crate::{
    constants::{SHORT_DATETIME_ID, SHORT_DATE_ID},
    formatter::{self, format::Formatted},
    locale::get_locale,
};

/// ECMA-376 built-in number format strings as `(numFmtId, format_code)` pairs.
/// https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.numberingformat?view=openxml-3.0.1
/// **Do not change IDs** — the numFmtId values are part of the ECMA-376 spec.
pub(crate) const DEFAULT_NUM_FMTS: &[(i32, &str)] = &[
    (0, "General"),
    (1, "0"),
    (2, "0.00"),
    (3, "#,##0"),
    (4, "#,##0.00"),
    // specification excludes IDs 5, 6, 7, and 8
    (9, "0%"),
    (10, "0.00%"),
    (11, "0.00E+00"),
    (12, "# ?/?"),
    (13, "# ??/??"),
    (14, "mm-dd-yy"),
    (15, "d-mmm-yy"),
    (16, "d-mmm"),
    (17, "mmm-yy"),
    (18, "h:mm AM/PM"),
    (19, "h:mm:ss AM/PM"),
    (20, "h:mm"),
    (21, "h:mm:ss"),
    (22, "m/d/yy h:mm"),
    // 23 - 31 Japanese (ja-JP) specialized date and time formats
    // 32 - 35 Chinese (zh-CN/zh-TW) and Korean (ko-KR) specialized number/date formats.
    // 36 specific localized currency formats
    (37, "#,##0 ;(#,##0)"),
    (38, "#,##0 ;[Red](#,##0)"),
    (39, "#,##0.00;(#,##0.00)"),
    (40, "#,##0.00;[Red](#,##0.00)"),
    // Accounting formats
    // 41 - 44
    (45, "mm:ss"),
    (46, "[h]:mm:ss"),
    (47, "mmss.0"),
    (48, "##0.0E+0"),
    (49, "@"),
];

pub(crate) struct DefaultFmts;

impl DefaultFmts {
    pub(crate) fn by_id(id: i32) -> Option<&'static str> {
        DEFAULT_NUM_FMTS
            .iter()
            .find(|&&(fid, _)| fid == id)
            .map(|&(_, s)| s)
    }

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

    /// 3: `#,##0`
    pub(crate) fn comma_int() -> String {
        DefaultFmts::id_fmt_or_general(3) //.unwrap_or("#,##0").to_string()
    }

    ///  4: `#,##0.00`
    pub(crate) fn comma_dec() -> String {
        DefaultFmts::id_fmt_or_general(4) //.unwrap_or("#,##0.00").to_string()
    }
    /// 9: `0%`
    pub(crate) fn percent_int() -> String {
        DefaultFmts::id_fmt_or_general(9) //.unwrap_or("0%").to_string()
    }

    ///  10: `0.00%`
    pub(crate) fn percent_dec() -> String {
        DefaultFmts::id_fmt_or_general(10) //.unwrap_or("0.00%").to_string()
    }

    /// 11: `0.00E+00`
    pub(crate) fn scientific_format() -> String {
        DefaultFmts::id_fmt_or_general(11)
    }

    // Formatter helper, otherwise we would need to do for each
    // function above to satisfy clippy.
    // DefaultFmts::by_id().unwrap_or("#,##0.00").to_string()
    fn id_fmt_or_general(id: i32) -> String {
        DEFAULT_NUM_FMTS
            .iter()
            .find(|&&(fid, _)| fid == id)
            .map(|&(_, s)| s)
            .unwrap_or("General")
            .to_string()
    }

    pub(crate) fn is_locale_date(id: i32) -> bool {
        id == SHORT_DATE_ID || id == SHORT_DATETIME_ID
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
            DefaultFmts::by_id(0),
            Some("General"),
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
            Some("m/d/yy h:mm"),
            "numFmtId 22 must be the locale short date+time"
        );
    }

    /// ECMA-376 §18.8.30 — canonical built-in format codes for the US (default) locale.
    ///
    /// Format codes data from:
    /// https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.numberingformat?view=openxml-3.0.1

    #[test]
    fn test_full_numbering_format_class() {
        let plain = "
        0

        General

        1

        0

        2

        0.00

        3

        #,##0

        4

        #,##0.00

        9

        0%

        10

        0.00%

        11

        0.00E+00

        12

        # ?/?

        13

        # ??/??

        14

        mm-dd-yy

        15

        d-mmm-yy

        16

        d-mmm

        17

        mmm-yy

        18

        h:mm AM/PM

        19

        h:mm:ss AM/PM

        20

        h:mm

        21

        h:mm:ss

        22

        m/d/yy h:mm

        37

        #,##0 ;(#,##0)

        38

        #,##0 ;[Red](#,##0)

        39

        #,##0.00;(#,##0.00)

        40

        #,##0.00;[Red](#,##0.00)

        45

        mm:ss

        46

        [h]:mm:ss

        47

        mmss.0

        48

        ##0.0E+0

        49

        @";

        let lines: Vec<&str> = plain
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        let pairs: Vec<(i32, &str)> = lines
            .chunks(2)
            .filter_map(|p| match p {
                [id, fmt] => id.parse::<i32>().ok().map(|n| (n, *fmt)),
                _ => None,
            })
            .collect();

        assert_eq!(DEFAULT_NUM_FMTS.len(), pairs.len());

        for &(id, expected) in pairs.iter() {
            assert_eq!(DefaultFmts::by_id(id), Some(expected), "numFmtId {id}");
        }
    }

    // Derive gap IDs from DEFAULT_NUM_FMTS — IDs in 0..ECMA_CUSTOM_FMT_MIN_ID
    // that are absent from the table — and verify the boundary.
    //     #[cfg(not(debug_assertions))]

    #[test]
    #[should_panic(expected = "is unknown")]
    fn test_no_builtin_in_gap_ranges() {
        use crate::constants::ECMA_CUSTOM_FMT_MIN_ID;
        use crate::types::NumFmt;
        use std::collections::HashSet;

        let known: HashSet<i32> = DEFAULT_NUM_FMTS.iter().map(|&(id, _)| id).collect();

        // Gap IDs are those absent from DEFAULT_NUM_FMTS within the built-in range.
        // The highest gap must be ECMA_CUSTOM_FMT_MIN_ID - 1 = 163.
        let gaps: Vec<i32> = (0..ECMA_CUSTOM_FMT_MIN_ID)
            .filter(|id| !known.contains(id))
            .collect();

        for id in gaps {
            dbg!(&id);
            let fmt = NumFmt::from_id(id, &[]);
            assert_eq!(
                fmt.num_fmt_id, 0,
                "gap numFmtId {id} must fall back to General (id 0)"
            );
            assert_eq!(
                fmt.format_code, "General",
                "gap numFmtId {id} must fall back to General format code"
            );
        }
    }
    #[test]
    #[should_panic(expected = "is unknown")]
    fn test_gap_id_panics_in_debug() {
        use crate::types::NumFmt;

        let _ = NumFmt::from_id(23, &[]); // one representative gap ID
    }
}
