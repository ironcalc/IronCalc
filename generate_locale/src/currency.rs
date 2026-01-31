use std::collections::HashMap;
use std::fs;

use bitcode::Encode;
use serde::{Deserialize, Serialize};

use crate::constants::Currency;

// This comes from cldr-core/supplemental/currencyData.json
// {
//   "supplemental": {
//     "version": {
//       "_unicodeVersion": "…",
//       "_cldrVersion": "…"
//     },
//     "currencyData": {
//       "fractions": {
//         "ADP": { "_rounding": "0", "_digits": "0" },
//         "CHF": { "_rounding": "0.05", "_digits": "2", "_cashRounding": "0.05", "_cashDigits": "2" },
//         "DEFAULT": { "_rounding": "0", "_digits": "2" }
//       },
//       "region": {
//         "AD": [
//           { "ESP": { "_from": "1873-01-01", "_to": "2002-02-28", "_tender": "false" } },
//           { "ADP": { "_from": "1936-01-01", "_to": "2002-02-28" } },
//           { "EUR": { "_from": "2002-03-01" } }
//         ],
//         "DE": [
//           { "DEM": { "_from": "…", "_to": "…" } },
//           { "EUR": { "_from": "…" } }
//         ]
//       }
//     }
//   }
// }
#[derive(Serialize, Deserialize, Encode)]
struct CurrencyData {
    pub supplemental: Supplemental,
}

#[derive(Serialize, Deserialize, Encode)]
struct Supplemental {
    #[serde(rename = "currencyData")]
    pub currency_data: CurrencyRegion,
}
#[derive(Serialize, Deserialize, Encode)]
struct CurrencyRegion {
    pub region: HashMap<String, Vec<HashMap<String, CurrencyRange>>>,
}

#[derive(Serialize, Deserialize, Encode)]
struct CurrencyRange {
    pub _from: Option<String>,
    pub _to: Option<String>,
    pub _tender: Option<String>,
}

// This comes from cldr-numbers-full/main/{locale}/currencies.json
// {
//  "main": {
//    "en": {
//       "identity": { ... },
//       "numbers": {
//         "currencies": {
//           "EUR": { ... }
//         }
//       }
//     }
// }
#[derive(Serialize, Deserialize, Encode)]
struct Currencies {
    main: HashMap<String, CurrencyInfo>,
}

#[derive(Serialize, Deserialize, Encode)]
struct CurrencyInfo {
    numbers: CurrencyNumbers,
}

#[derive(Serialize, Deserialize, Encode)]
struct CurrencyNumbers {
    currencies: HashMap<String, CurrencyDetails>,
}

// "displayName": "euro",
// "displayName-count-one": "euro",
// "displayName-count-other": "euros",
// "symbol": "EUR",
// "symbol-alt-narrow": "€"
#[derive(Serialize, Deserialize, Encode)]
struct CurrencyDetails {
    #[serde(rename = "symbol-alt-narrow")]
    symbol_alt_narrow: Option<String>,
    symbol: Option<String>,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "displayName-count-one")]
    display_name_count_one: Option<String>,
    #[serde(rename = "displayName-count-other")]
    display_name_count_other: Option<String>,
}

pub fn get_locale_currency(cldr_dir: &str, locale_id: &str, id: &str) -> Result<Currency, String> {
    let numbers_file = format!(
        "{}cldr-json/cldr-core/supplemental/currencyData.json",
        cldr_dir
    );
    let contents =
        fs::read_to_string(&numbers_file).or(Err("Failed reading 'currencyData.json'"))?;
    let currency_data: CurrencyData = serde_json::from_str(&contents)
        .or(Err(format!("Failed parsing '{}' file", numbers_file)))?;
    // Get the territory ID from the locale ID
    let (_language_id, territory_id) = if locale_id.contains('-') {
        let parts: Vec<&str> = locale_id.split('-').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        panic!("Locale ID {} does not contain a territory ID", locale_id);
    };
    let region = &currency_data.supplemental.currency_data.region;
    let t = match region.get(&territory_id) {
        Some(t) => {
            // pick the last one:
            &t[0]
        }
        None => {
            return Err(format!(
                "No currency data found for territory ID {}",
                territory_id
            ))
        }
    };
    let keys: Vec<&String> = t.keys().collect();
    let currency_code = keys[keys.len() - 1];

    let symbol = get_currency_symbol(cldr_dir, id, currency_code)?;

    Ok(Currency {
        iso: currency_code.to_string(),
        symbol,
    })
}

// https://github.com/unicode-org/cldr-json/blob/main/cldr-json/cldr-numbers-full/main/en/currencies.json
fn get_currency_symbol(
    cldr_dir: &str,
    locale_id: &str,
    currency_code: &str,
) -> Result<String, String> {
    let currencies_file = format!(
        "{}cldr-json/cldr-numbers-full/main/{}/currencies.json",
        cldr_dir, locale_id
    );
    let contents = fs::read_to_string(&currencies_file)
        .or(Err(format!("Failed reading '{}'", currencies_file)))?;
    let currencies: Currencies =
        serde_json::from_str(&contents).or(Err("Failed parsing 'currencies.json' file"))?;
    let data = &currencies.main[locale_id].numbers.currencies[currency_code];
    let symbol = match &data.symbol_alt_narrow {
        Some(s) => s,
        None => currency_code,
    };
    Ok(symbol.to_string())
}
