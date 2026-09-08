//! Builds a column where every cell reads the one below it:
//!
//!   A1 = A2+1
//!   A2 = A3+1
//!   ...
//!   A(n) = 1
//!
//! Natural order meets A1 first, and A1 needs the whole column before it can
//! be computed, so a plain recursive evaluation goes n formulas deep. This is
//! the workbook that used to overflow the stack. this would easily produce a
//! stack overflow with a naive recursive evaluation.
//! It is evaluated and saved as xlsx.
//!
//! Usage:
//!   cargo run --release --example deep_chain -- [rows] [file]
//!
//! Defaults: 1,000,000 rows, `deep-chain.xlsx`.

use std::time::Instant;

use ironcalc::{base::Model, export::save_to_xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rows: i32 = match args.next() {
        Some(value) => value.parse()?,
        None => 1_000_000,
    };
    let file_name = args.next().unwrap_or("deep-chain.xlsx".to_string());
    if !(1..=1_048_576).contains(&rows) {
        return Err("rows must be between 1 and 1,048,576".into());
    }

    let mut model = Model::new_empty(&file_name, "en", "UTC", "en")?;

    let start = Instant::now();
    for row in 1..rows {
        model.set_user_input(0, row, 1, format!("=A{}+1", row + 1))?;
    }
    model.set_user_input(0, rows, 1, "1".to_string())?;
    println!("built {rows} rows in {:?}", start.elapsed());

    let start = Instant::now();
    model.evaluate();
    println!("evaluated in {:?}", start.elapsed());

    // A1 counts the rows: every cell adds one to the cell below it.
    println!("A1 = {}", model.get_formatted_cell_value(0, 1, 1)?);
    println!("A{rows} = {}", model.get_formatted_cell_value(0, rows, 1)?);

    // save_to_xlsx refuses to overwrite a file
    if std::path::Path::new(&file_name).exists() {
        std::fs::remove_file(&file_name)?;
    }
    let start = Instant::now();
    save_to_xlsx(&model, &file_name)?;
    let size = std::fs::metadata(&file_name)?.len();
    println!(
        "saved {file_name} ({:.1} MB) in {:?}",
        size as f64 / (1024.0 * 1024.0),
        start.elapsed()
    );
    Ok(())
}
