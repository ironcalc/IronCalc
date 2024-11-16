use std::collections::HashMap;

use bitcode::{Decode, Encode};
use once_cell::sync::Lazy;

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

#[allow(clippy::expect_used)]
static LANGUAGES: Lazy<HashMap<String, Language>> = Lazy::new(|| {
    bitcode::decode(include_bytes!("language.bin")).expect("Failed parsing language file")
});

pub fn get_language(id: &str) -> Result<&Language, String> {
    let language = LANGUAGES
        .get(id)
        .ok_or(format!("Language is not supported: '{}'", id))?;
    Ok(language)
}
