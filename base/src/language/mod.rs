use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Booleans {
    #[serde(rename = "true")]
    pub true_value: String,
    #[serde(rename = "false")]
    pub false_value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Errors {
    #[serde(rename = "ref")]
    pub ref_value: String,
    pub name: String,
    pub value: String,
    pub div: String,
    pub na: String,
    pub num: String,
    pub nimpl: String,
    pub spill: String,
    pub calc: String,
    pub circ: String,
    pub error: String,
    pub null: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Language {
    pub booleans: Booleans,
    pub errors: Errors,
}

static LANGUAGES: Lazy<HashMap<String, Language>> = Lazy::new(|| {
    serde_json::from_str(include_str!("language.json")).expect("Failed parsing language file")
});

pub fn get_language(id: &str) -> Result<&Language, String> {
    let language = LANGUAGES
        .get(id)
        .ok_or(format!("Language is not supported: '{}'", id))?;
    Ok(language)
}
