use std::fs;
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::PathBuf,
};

use constants::{Currency, Locale};

use clap::Parser;
use numbers::get_numbers_formatting;

mod cldr_utils;
mod constants;
mod currency;
mod dates;
mod numbers;
mod util;

use currency::get_locale_currency;
use dates::get_dates_formatting;
use util::get_all_locales_id;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    /// File with the list of required locales
    #[clap(long, value_parser)]
    locales: Option<PathBuf>,

    /// Folder with the cldr data
    #[clap(long, value_parser)]
    cldr_dir: String,

    /// output json file with all locale info
    #[clap(long, value_parser)]
    output: PathBuf,
}

/// Version of the cldr-json checkout, as published in its npm packages.
fn get_cldr_version(cldr_dir: &str) -> Result<String, String> {
    let package_file = format!("{}cldr-json/cldr-core/package.json", cldr_dir);
    let contents = fs::read_to_string(&package_file)
        .or(Err(format!("Failed reading file: {}", package_file)))?;
    let package: serde_json::Value = serde_json::from_str(&contents)
        .or(Err(format!("Failed parsing file: {}", package_file)))?;
    package["version"]
        .as_str()
        .map(str::to_string)
        .ok_or(format!("No version in {}", package_file))
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    let cldr_dir = opt.cldr_dir;
    let locales_list: Vec<String> = if let Some(locales_path) = opt.locales {
        let locales_path_str = locales_path.display().to_string();
        let contents = fs::read_to_string(locales_path)
            .or(Err(format!("Failed reading file: {}", locales_path_str)))?;
        serde_json::from_str(&contents).or(Err(format!(
            "Failed parsing locales file: {}",
            locales_path_str
        )))?
    } else {
        get_all_locales_id(&cldr_dir)
    };

    let mut locales = HashMap::new();

    for locale_id in &locales_list {
        let full_locale_id = match locale_id.as_str() {
            "en" => "en-US",
            "es" => "es-ES",
            "fr" => "fr-FR",
            "de" => "de-DE",
            "it" => "it-IT",
            _ => locale_id.as_str(),
        };
        let dates = get_dates_formatting(&cldr_dir, locale_id)?;
        let numbers = get_numbers_formatting(&cldr_dir, locale_id)?;
        let currency = get_locale_currency(&cldr_dir, full_locale_id, locale_id)?;
        let currency = Currency {
            iso: currency.iso.clone(),
            symbol: currency.symbol.clone(),
        };
        locales.insert(
            locale_id.clone(),
            Locale {
                dates,
                numbers,
                currency,
            },
        );
    }

    // The json file is the human-readable form of locales.bin. It starts with
    // a `_meta` entry recording which CLDR release the data was taken from, so
    // the version can be quoted in license notices (the data is redistributed
    // under the Unicode License v3, see base/THIRD-PARTY-NOTICES.md).
    let mut json = serde_json::Map::new();
    json.insert(
        "_meta".to_string(),
        serde_json::json!({
            "cldr_version": get_cldr_version(&cldr_dir)?,
            "source": "https://github.com/unicode-org/cldr-json",
            "license": "Unicode-3.0",
        }),
    );
    // Locales sorted by id; fields keep their declaration order (preserve_order).
    let sorted: BTreeMap<&String, &Locale> = locales.iter().collect();
    match serde_json::to_value(&sorted).or(Err("Failed to stringify data"))? {
        serde_json::Value::Object(map) => json.extend(map),
        _ => return Err("Expected a json object".to_string()),
    }
    let s = serde_json::to_string_pretty(&json).or(Err("Failed to stringify data"))?;
    let mut f = fs::File::create(opt.output).or(Err("Failed to create file"))?;
    f.write_all(s.as_bytes()).or(Err("Failed writing"))?;

    // save to locales.bin using bitcode
    let bytes = bitcode::encode(&locales);
    let mut f_bin = fs::File::create("locales.bin").or(Err("Failed to create locales.bin"))?;
    f_bin
        .write_all(&bytes)
        .or(Err("Failed writing locales.bin"))?;
    Ok(())
}
