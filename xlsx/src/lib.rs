//! # IronCalc - Core API documentation
//!
//! This technical API documentation is aimed at developers.
//! It is used to build language bindings (like python, javascript or nodejs) or to build full fledged applications like TironCalc in the terminal or IronCalc, the Web application.
//!
//! ## Basic usage
//!
//! Add the dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ironcalc = { git = "https://github.com/ironcalc/IronCalc", tag = "v0.5.0" }
//! ```
//!
//! A simple example with some numbers, a new sheet and a formula:
//!
//!
//! ```rust
#![doc = include_str!("../examples/hello_calc.rs")]
//! ```
//!
//! ## Examples
//!
//! This is a collection of full fledged examples you can use as a starting point or for learning purposes.
//! You might find the code in the examples folder
//!
//! ### Styling the workbook
//!
//! Adding colors, to cells, full columns or full rows is easy
//!
//! ```rust
#![doc = include_str!("../examples/hello_styles.rs")]
//! ```
//!
//! ### Changing column width and row heigh
//!
//! ```rust
#![doc = include_str!("../examples/widths_and_heights.rs")]
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
