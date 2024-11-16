#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! Tests an Excel xlsx file.
//! Returns a list of differences in json format.
//! Saves an IronCalc version
//! This is primary for QA internal testing and will be superseded by an official
//! IronCalc CLI.
//!
//! Usage: test file.xlsx

use std::path;

use ironcalc::{export::save_to_icalc, import::load_from_xlsx};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <file.xlsx>", args[0]);
    }
    // first test the file
    let file_name = &args[1];

    let file_path = path::Path::new(file_name);
    let base_name = file_path.file_stem().unwrap().to_str().unwrap();
    let output_file_name = &format!("{base_name}.ic");
    let model = load_from_xlsx(file_name, "en", "UTC").unwrap();
    save_to_icalc(&model, output_file_name).unwrap();
}
