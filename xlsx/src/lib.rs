//! # IronCalc - Core API documentation
//!
//! This technical API documentation in aimed at developers who want to develop bindings for a different language,
//! build a UI based on the engine or just use the library in a Rust program
//!
//! ## Usage
//!
//! Add the dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ironcalc = { git = "https://github.com/ironcalc/IronCalc", version = "0.1.0"}
//! ```
//!
//! <small> until version 0.5.0 you should use the git dependencies as stated </small>
//!
//! A simple example:
//!
//!
//! ```rust
//! use ironcalc::{
//! base::{expressions::utils::number_to_column, model::Model},
//! export::save_to_xlsx,
//! };
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut model = Model::new_empty("hello-calc.xlsx", "en", "UTC")?;
//! // Adds a square of numbers in the first sheet
//! for row in 1..100 {
//!     for column in 1..100 {
//!         let value = row * column;
//!         model.set_user_input(0, row, column, format!("{}", value));
//!     }
//! }
//! // Adds a new sheet
//! model.add_sheet("Calculation")?;
//! // column 100 is CV
//! let last_column = number_to_column(100).unwrap();
//! let formula = format!("=SUM(Sheet1!A1:{}100)", last_column);
//! model.set_user_input(1, 1, 1, formula);
//!
//! // evaluates
//! model.evaluate();
//!
//! // saves to disk
//! save_to_xlsx(&model, "hello-calc.xlsx")?;
//! Ok(())
//! }
//! ```
//!
//! You can then just:
//!
//! ```bash
//! cargo run
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/favicon.ico"
)]

pub mod compare;
pub mod error;
pub mod export;
pub mod import;
pub use ironcalc_base as base;
