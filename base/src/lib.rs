//! # IronCalcBase engine API
//!
//! This is the documentation for the base engine API.
//!
//! # Basic usage
//!
//! Add the dependency in Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! ironcalc_base = { git = "https://github.com/ironcalc/IronCalc", version = "0.1"}
//! ```
//!
//! <small> until version 0.5.0 you should use the git dependencies as stated </small>
//!
//! In this example we use the excel function `CONCAT` to concatenate strings in cells `A1` and `B1`:
//!
//! ```rust
#![doc = include_str!("../examples/hello_world.rs")]
//! ```
//!
//! In this example, we demonstrate our ability to handle formulas and errors:
//!
//! ```rust
#![doc = include_str!("../examples/formulas_and_errors.rs")]
//! ```

pub mod calc_result;
pub mod cell;
pub mod expressions;
pub mod formatter;
pub mod language;
pub mod locale;
pub mod model;
pub mod new_empty;
pub mod number_format;
pub mod types;
pub mod worksheet;

mod functions;

mod actions;
mod cast;
mod constants;
mod styles;

mod diffs;
mod implicit_intersection;

mod units;
mod utils;
mod workbook;
mod garbage_collector;

#[cfg(test)]
mod test;

#[cfg(test)]
pub mod mock_time;
