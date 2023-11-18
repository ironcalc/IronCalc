use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::constants::{NumbersProperties, LOCAL_TYPE};

#[derive(Serialize, Deserialize)]
struct CaGCalendarsFormat {
    format: HashMap<String, HashMap<String, String>>,
}
#[derive(Serialize, Deserialize)]
struct CaGCalendarsII {
    months: CaGCalendarsFormat,
    days: CaGCalendarsFormat,
}

#[derive(Serialize, Deserialize)]
struct NumbersJSONId {
    identity: Value,
    numbers: NumbersProperties,
}

#[derive(Serialize, Deserialize)]
struct NumbersJSON {
    main: HashMap<String, NumbersJSONId>,
}

pub fn get_numbers_formatting(
    cldr_dir: &str,
    locale_id: &str,
) -> Result<NumbersProperties, String> {
    let numbers_file = format!(
        "{}cldr-json/cldr-numbers-{}/main/{}/numbers.json",
        cldr_dir, LOCAL_TYPE, locale_id
    );

    let contents = fs::read_to_string(numbers_file).or(Err("Failed reading 'numbers.json'"))?;
    let numbers_json: &NumbersJSON =
        &serde_json::from_str(&contents).or(Err("Failed parsing 'numbers.json' file"))?;
    // Grouping is either
    // * #,##,##0.### (indian way)
    // * #,##0.### (standard)
    // * 0.###### (posix)
    // anything else is an error
    let grouping_str = &numbers_json.main[locale_id]
        .numbers
        .decimal_formats
        .standard;
    let _grouping = if grouping_str == "#,##0.###" {
        "standard"
    } else if grouping_str == "#,##,##0.###" {
        "indian"
    } else if grouping_str == "0.######" {
        "posix"
    } else {
        let message = format!(
            "Unexpected grouping {} in locale {}",
            grouping_str, locale_id
        );
        return Err(message);
    };
    Ok(numbers_json.main[locale_id].numbers.clone())
}
