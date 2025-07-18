use std::{collections::HashMap, sync::OnceLock};

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Clone)]
pub struct Booleans {
    pub r#true: String,
    pub r#false: String,
}

#[derive(Encode, Decode, Clone)]
pub struct Errors {
    pub r#ref: String,
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

#[derive(Encode, Decode, Clone)]
pub struct Language {
    pub booleans: Booleans,
    pub errors: Errors,
}

static LANGUAGES: OnceLock<HashMap<String, Language>> = OnceLock::new();

#[allow(clippy::expect_used)]
fn get_languages() -> &'static HashMap<String, Language> {
    LANGUAGES.get_or_init(|| {
        bitcode::decode(include_bytes!("language.bin")).expect("Failed parsing language file")
    })
}

pub fn get_language(id: &str) -> Result<&Language, String> {
    get_languages()
        .get(id)
        .ok_or_else(|| format!("Language is not supported: '{id}'"))
}
