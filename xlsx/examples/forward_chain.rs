//! Builds a column where every cell reads the one above it:
//!
//!   A1 = 1
//!   A2 = A1+1
//!   A3 = A2+1
//!   ...
//!
//! This is `deep_chain` the right way up. Natural order meets A1 first, a
//! constant, and every formula after it finds the cell it reads already
//! evaluated, so the recursion is never deeper than one formula, however long
//! the column is. It is evaluated and saved as xlsx.
//!
//! Usage:
//!   cargo run --release --example forward_chain -- [rows] [file]
//!
//! Defaults: 1,000,000 rows, `forward-chain.xlsx`.

use std::time::Instant;

use ironcalc::{base::Model, export::save_to_xlsx};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let rows: i32 = match args.next() {
        Some(value) => value.parse()?,
        None => 1_000_000,
    };
    let file_name = args.next().unwrap_or("forward-chain.xlsx".to_string());
    if !(1..=1_048_576).contains(&rows) {
        return Err("rows must be between 1 and 1,048,576".into());
    }

    let mut model = Model::new_empty(&file_name, "en", "UTC", "en")?;

    let start = Instant::now();
    model.set_user_input(0, 1, 1, "1".to_string())?;
    for row in 2..=rows {
        model.set_user_input(0, row, 1, format!("=A{}+1", row - 1))?;
    }
    println!("built {rows} rows in {:?}", start.elapsed());

    let start = Instant::now();
    model.evaluate();
    println!("evaluated in {:?}", start.elapsed());

    // The last cell counts the rows: every cell adds one to the cell above it.
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
