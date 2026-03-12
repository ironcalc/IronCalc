use crate::{
    formatter::{self, format::Formatted},
    locale::get_locale,
};

/// Built-in number format strings indexed by their ECMA-376 numFmtId.
///
/// **ORDERING IS SPEC-MANDATED — do not insert, remove, or reorder entries.**
/// The index in this array IS the numFmtId as defined in ECMA-376.
/// Code throughout the codebase (and every `.xlsx` file on disk) uses these
/// IDs by their numeric value, not by position in this array. Any accidental
/// reordering will cause silent mis-formatting for any ID after the change.
///
/// The test `builtin_num_fmts_index_matches_spec_ids` guards the
/// critical positions.
pub(crate) const DEFAULT_NUM_FMTS: &[&str] = &[
    "general",
    "0",
    "0.00",
    "#,##0",
    "#,##0.00",
    "$#,##0; \\ - $#,##0",
    "$#,##0; [Red] \\ - $#,##0",
    "$#,##0.00; \\ - $#,##0.00",
    "$#,##0.00; [Red] \\ - $#,##0.00",
    "0%",
    "0.00%",
    "0.00E + 00",
    "#?/?",
    "#?? / ??",
    "mm-dd-yy",
    "d-mmm-yy",
    "d-mmm",
    "mmm-yy",
    "h:mm AM / PM",
    "h:mm:ss AM / PM",
    "h:mm",
    "h:mm:ss",
    "m / d / yy h:mm",
    "#,##0;()#,##0)",
    "#,##0; [Red]()#,##0)",
    "#,##0.00;()#,##0.00)",
    "#,##0.00; [Red]()#,##0.00)",
    "_()$”*#,##0.00 _); _()$”* \\()#,##0.00\\); _()$”* - ?? _); _()@_)",
    "mm:ss",
    "[h]:mm:ss",
    "mmss .0",
    "##0.0E + 0",
    "@",
    "[$ -404] e / m / d ",
    "m / d / yy",
    "[$ -404] e / m / d",
    "[$ -404] e / / d",
    "[$ -404] e / m / d",
    "t0",
    "t0.00",
    "t#,##0",
    "t#,##0.00",
    "t0%",
    "t0.00 %",
    "t#?/?",
];

/// ECMA-376 numFmtId for the locale-derived short date.
///
/// The value `14` is mandated by the Office Open XML specification.
/// https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/0e59abdb-7f4e-48fc-9b89-67832fa11789
pub const SHORT_DATE_FMT_ID: i32 = 14;

/// ECMA-376 numFmtId for the locale-derived short date+time ("m / d / yy h:mm").
pub const SHORT_DATE_TIME_FMT_ID: i32 = 22;

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

    /// Verify that critical ECMA-376 built-in numFmtIds are at the correct
    /// positions in `DEFAULT_NUM_FMTS`.  The array index IS the numFmtId, so
    /// inserting or removing any entry before these positions would silently
    /// break format detection throughout the codebase.
    #[test]
    fn builtin_num_fmts_index_matches_spec_ids() {
        assert_eq!(DEFAULT_NUM_FMTS.len(), 45, "DEFAULT_NUM_FMTS length changed — update this assertion and verify all numFmtIds");
        assert_eq!(DEFAULT_NUM_FMTS[0], "general", "numFmtId 0 must be General");
        assert_eq!(DEFAULT_NUM_FMTS[9], "0%", "numFmtId 9 must be 0%");
        assert_eq!(
            DEFAULT_NUM_FMTS[SHORT_DATE_FMT_ID as usize], "mm-dd-yy",
            "numFmtId 14 must be the ECMA-376 locale short date 'mm-dd-yy'"
        );
        assert_eq!(
            DEFAULT_NUM_FMTS[22], "m / d / yy h:mm",
            "numFmtId 22 must be the locale short date+time"
        );
    }
}
