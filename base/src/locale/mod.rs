use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Locale {
    pub dates: Dates,
    pub numbers: NumbersProperties,
    pub currency: Currency,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Currency {
    pub iso: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NumbersProperties {
    #[serde(rename = "symbols-numberSystem-latn")]
    pub symbols: NumbersSymbols,
    #[serde(rename = "decimalFormats-numberSystem-latn")]
    pub decimal_formats: DecimalFormats,
    #[serde(rename = "currencyFormats-numberSystem-latn")]
    pub currency_formats: CurrencyFormats,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dates {
    pub day_names: Vec<String>,
    pub day_names_short: Vec<String>,
    pub months: Vec<String>,
    pub months_short: Vec<String>,
    pub months_letter: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
#[derive(Serialize, Deserialize, Clone)]
pub struct CurrencyFormats {
    pub standard: String,
    #[serde(rename = "standard-alphaNextToNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_alpha_next_to_number: Option<String>,
    #[serde(rename = "standard-noCurrency")]
    pub standard_no_currency: String,
    pub accounting: String,
    #[serde(rename = "accounting-alphaNextToNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounting_alpha_next_to_number: Option<String>,
    #[serde(rename = "accounting-noCurrency")]
    pub accounting_no_currency: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecimalFormats {
    pub standard: String,
}

static LOCALES: Lazy<HashMap<String, Locale>> = Lazy::new(|| {
    serde_json::from_str(include_str!("locales.json")).expect("Failed parsing locale")
});

pub fn get_locale(_id: &str) -> Result<&Locale, String> {
    // TODO: pass the locale once we implement locales in Rust
    let locale = LOCALES.get("en").ok_or("Invalid locale")?;
    Ok(locale)
}

// TODO: Remove this function one we implement locales properly
pub fn get_locale_fix(id: &str) -> Result<&Locale, String> {
    let locale = LOCALES.get(id).ok_or("Invalid locale")?;
    Ok(locale)
}
