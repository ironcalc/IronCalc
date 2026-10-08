#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

//! Profiles an Excel xlsx file.
//! Prints the time it takes to load the file, to evaluate it and to
//! re-evaluate it after changing an input cell.
//! This is primary for QA internal testing and will be superseded by an official
//! IronCalc CLI.
//!
//! Usage: test file.xlsx

use std::time::Instant;

use ironcalc::import::load_from_xlsx;

/// Sheet index of the cells we read and write
const SHEET: u32 = 1;
/// Cell we change to trigger a re-evaluation (B4)
const INPUT_CELL: (i32, i32) = (4, 2);
/// Cell we read after each evaluation (D2)
const OUTPUT_CELL: (i32, i32) = (2, 4);

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <file.xlsx>", args[0]);
    }
    let file_name = &args[1];
    println!("Testing file: {file_name}");
    let total = Instant::now();

    let start = Instant::now();
    let mut model = load_from_xlsx(file_name, "en", "UTC", "en").unwrap();
    println!("Load: {:.2?}", start.elapsed());

    let start = Instant::now();
    model.evaluate();
    println!("Evaluation: {:.2?}", start.elapsed());
    let value = model
        .get_cell_value_by_index(SHEET, OUTPUT_CELL.0, OUTPUT_CELL.1)
        .unwrap();
    println!("Evaluated: {value:?}");

    model
        .set_user_input(SHEET, INPUT_CELL.0, INPUT_CELL.1, " slow".to_string())
        .unwrap();
    let start = Instant::now();
    model.evaluate();
    println!("Re-evaluation: {:.2?}", start.elapsed());
    let value = model
        .get_cell_value_by_index(SHEET, OUTPUT_CELL.0, OUTPUT_CELL.1)
        .unwrap();
    println!("Evaluated: {value:?}");

    println!("Total: {:.2?}", total.elapsed());
}
