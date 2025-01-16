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
//! ironcalc_base = { git = "https://github.com/ironcalc/IronCalc" }
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

#![warn(clippy::print_stdout)]

pub mod calc_result;
pub mod cell;
pub mod expressions;
pub mod formatter;
pub mod language;
pub mod locale;
pub mod new_empty;
pub mod number_format;
pub mod types;
pub mod worksheet;

mod actions;
mod arithmetic;
mod cast;
mod constants;
mod functions;
mod implicit_intersection;
mod model;
mod styles;
mod units;
mod user_model;
mod utils;
mod workbook;

#[cfg(test)]
mod test;

#[cfg(test)]
pub mod mock_time;

pub use model::get_milliseconds_since_epoch;
pub use model::Model;
pub use user_model::BorderArea;
pub use user_model::ClipboardData;
pub use user_model::UserModel;
