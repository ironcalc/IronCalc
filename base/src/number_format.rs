use crate::{
    formatter::{self, format::Formatted},
    locale::get_locale,
    types::NumFmt,
};

const DEFAULT_NUM_FMTS: &[&str] = &[
    "general",
    "0",
    "0.00",
    "#,## 0",
    "#,## 0.00",
    "$#,## 0; \\ - $#,## 0",
    "$#,## 0; [Red] \\ - $#,## 0",
    "$#,## 0.00; \\ - $#,## 0.00",
    "$#,## 0.00; [Red] \\ - $#,## 0.00",
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
    "#,## 0;()#,## 0)",
    "#,## 0; [Red]()#,## 0)",
    "#,## 0.00;()#,## 0.00)",
    "#,## 0.00; [Red]()#,## 0.00)",
    "_()$”*#,## 0.00 _); _()$”* \\()#,## 0.00\\); _()$”* - ?? _); _()@_)",
    "mm:ss",
    "[h]:mm:ss",
    "mmss .0",
    "## 0.0E + 0",
    "@",
    "[$ -404] e / m / d ",
    "m / d / yy",
    "[$ -404] e / m / d",
    "[$ -404] e / / d",
    "[$ -404] e / m / d",
    "t0",
    "t0.00",
    "t#,## 0",
    "t#,## 0.00",
    "t0%",
    "t0.00 %",
    "t#?/?",
];

pub fn get_default_num_fmt_id(num_fmt: &str) -> Option<i32> {
    for (index, default_num_fmt) in DEFAULT_NUM_FMTS.iter().enumerate() {
        if default_num_fmt == &num_fmt {
            return Some(index as i32);
        };
    }
    None
}

pub fn get_num_fmt(num_fmt_id: i32, num_fmts: &[NumFmt]) -> String {
    // Check if it defined
    for num_fmt in num_fmts {
        if num_fmt.num_fmt_id == num_fmt_id {
            return num_fmt.format_code.clone();
        }
    }
    // Return one of the default ones
    if num_fmt_id < DEFAULT_NUM_FMTS.len() as i32 {
        return DEFAULT_NUM_FMTS[num_fmt_id as usize].to_string();
    }
    // Return general
    DEFAULT_NUM_FMTS[0].to_string()
}

pub fn get_new_num_fmt_index(num_fmts: &[NumFmt]) -> i32 {
    let mut index = DEFAULT_NUM_FMTS.len() as i32;
    let mut found = true;
    while found {
        found = false;
        for num_fmt in num_fmts {
            if num_fmt.num_fmt_id == index {
                found = true;
                index += 1;
                break;
            }
        }
    }
    index
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
/// FIXME: There has to be a better algorithm :/
pub fn to_excel_precision_str(value: f64) -> String {
    to_precision_str(value, 15)
}
pub fn to_precision_str(value: f64, precision: usize) -> String {
    if value.is_infinite() {
        return "inf".to_string();
    }
    if value.is_nan() {
        return "NaN".to_string();
    }
    let exponent = value.abs().log10().floor();
    let base = value / 10.0_f64.powf(exponent);
    let base = format!("{0:.1$}", base, precision - 1);
    let value = format!("{}e{}", base, exponent).parse::<f64>().unwrap_or({
        // TODO: do this in a way that does not require a possible error
        0.0
    });
    // I would love to use the std library. There is not a speed concern here
    // problem is it doesn't do the right thing
    // Also ryu is my favorite _modern_ algorithm
    let mut buffer = ryu::Buffer::new();
    let text = buffer.format(value);
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
