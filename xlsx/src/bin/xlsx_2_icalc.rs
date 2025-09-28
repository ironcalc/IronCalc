#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! Tests an Excel xlsx file.
//! Returns a list of differences in json format.
//! Saves an IronCalc version
//! This is primary for QA internal testing and will be superseded by an official
//! IronCalc CLI.
//!
//! Usage: test file.xlsx [output.icalc]

use std::path;

use ironcalc::{export::save_to_icalc, import::load_from_xlsx};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        panic!("Usage: {} <file.xlsx> [output.icalc]", args[0]);
    }
    // first test the file
    let file_name = &args[1];

    let output_file_name = if args.len() == 3 {
        &args[2]
    } else {
        let file_path = path::Path::new(file_name);
        let base_name = file_path.file_stem().unwrap().to_str().unwrap();
        &format!("{base_name}.ic")
    };

    let model = load_from_xlsx(file_name, "en", "UTC").unwrap();
    save_to_icalc(&model, output_file_name).unwrap();
}
