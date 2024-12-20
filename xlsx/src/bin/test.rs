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

use ironcalc::{compare::test_file, export::save_to_xlsx, import::load_from_xlsx};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <file.xlsx>", args[0]);
    }
    // first test the file
    let file_name = &args[1];
    println!("Testing file: {file_name}");
    if let Err(message) = test_file(file_name) {
        println!("{}", message);
        panic!("Model was evaluated inconsistently with XLSX data.")
    }

    // save a copy my_xlsx_file.xlsx => my_xlsx_file.output.xlsx
    let file_path = path::Path::new(file_name);
    let base_name = file_path.file_stem().unwrap().to_str().unwrap();
    let output_file_name = &format!("{base_name}.output.xlsx");
    let mut model = load_from_xlsx(file_name, "en", "UTC").unwrap();
    model.evaluate();
    println!("Saving result as: {output_file_name}. Please open with Excel and test.");
    save_to_xlsx(&model, output_file_name).unwrap();
}
