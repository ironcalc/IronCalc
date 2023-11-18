use std::fs;
use std::{collections::HashMap, io::Write, path::PathBuf};

use constants::{Locale, Currency};

use clap::Parser;
use numbers::get_numbers_formatting;

mod constants;
mod dates;
mod numbers;
mod util;

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

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    let cldr_dir = opt.cldr_dir;
    let locales_list: Vec<String> = if let Some(locales_path) = opt.locales {
        let contents = fs::read_to_string(locales_path).or(Err("Failed reading file"))?;
        serde_json::from_str(&contents).or(Err("Failed parsing file"))?
    } else {
        get_all_locales_id(&cldr_dir)
    };

    let mut locales = HashMap::new();

    for locale_id in locales_list {
        let dates = get_dates_formatting(&cldr_dir, &locale_id)?;
        let numbers = get_numbers_formatting(&cldr_dir, &locale_id)?;
        // HACK: the currency is not a part of the cldr locale
        // We just stick here one and make this adaptable in the calc module for now
        let currency = Currency {
            iso: "USD".to_string(),
            symbol: "$".to_string()
        };
        locales.insert(locale_id, Locale { dates, numbers, currency });
    }

    let s = serde_json::to_string(&locales).or(Err("Failed to stringify data"))?;
    let mut f = fs::File::create(opt.output).or(Err("Failed to create file"))?;
    f.write_all(s.as_bytes()).or(Err("Failed writing"))?;
    Ok(())
}
