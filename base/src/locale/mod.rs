use std::{collections::HashMap, sync::OnceLock};

use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Clone)]
pub struct Locale {
    pub dates: Dates,
    pub numbers: NumbersProperties,
    pub currency: Currency,
}

#[derive(Encode, Decode, Clone)]
pub struct Currency {
    pub iso: String,
    pub symbol: String,
}

#[derive(Encode, Decode, Clone)]
pub struct NumbersProperties {
    pub symbols: NumbersSymbols,
    pub decimal_formats: DecimalFormats,
    pub currency_formats: CurrencyFormats,
}

#[derive(Encode, Decode, Clone)]
pub struct Dates {
    pub day_names: Vec<String>,
    pub day_names_short: Vec<String>,
    pub months: Vec<String>,
    pub months_short: Vec<String>,
    pub months_letter: Vec<String>,
}

#[derive(Encode, Decode, Clone)]
pub struct NumbersSymbols {
    pub decimal: String,
    pub group: String,
    pub list: String,
    pub percent_sign: String,
    pub plus_sign: String,
    pub minus_sign: String,
    pub approximately_sign: String,
    pub exponential: String,
    pub superscripting_exponent: String,
    pub per_mille: String,
    pub infinity: String,
    pub nan: String,
    pub time_separator: String,
}

// See: https://cldr.unicode.org/translation/number-currency-formats/number-and-currency-patterns
#[derive(Encode, Decode, Clone)]
pub struct CurrencyFormats {
    pub standard: String,
    pub standard_alpha_next_to_number: Option<String>,
    pub standard_no_currency: String,
    pub accounting: String,
    pub accounting_alpha_next_to_number: Option<String>,
    pub accounting_no_currency: String,
}

#[derive(Encode, Decode, Clone)]
pub struct DecimalFormats {
    pub standard: String,
}

static LOCALES: OnceLock<HashMap<String, Locale>> = OnceLock::new();

#[allow(clippy::expect_used)]
fn get_locales() -> &'static HashMap<String, Locale> {
    LOCALES.get_or_init(|| {
        bitcode::decode(include_bytes!("locales.bin")).expect("Failed parsing locale")
    })
}

pub fn get_locale(id: &str) -> Result<&Locale, String> {
    get_locales()
        .get(id)
        .ok_or_else(|| format!("Invalid locale: '{id}'"))
}
