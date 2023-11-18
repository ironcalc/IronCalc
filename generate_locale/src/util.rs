use std::fs;

use crate::constants::LOCAL_TYPE;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AlI {
    modern: Vec<String>,
    full: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvailableLocales {
    available_locales: AlI,
}

pub fn get_all_locales_id(cldr_dir: &str) -> Vec<String> {
    let al_file = format!("{}cldr-json/cldr-core/availableLocales.json", cldr_dir);
    let contents = fs::read_to_string(al_file).unwrap();
    let locales: AvailableLocales = serde_json::from_str(&contents).unwrap();
    if LOCAL_TYPE == "modern" {
        locales.available_locales.modern
    } else {
        locales.available_locales.full
    }
}
