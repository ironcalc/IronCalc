//! # IronCalc - Core API documentation
//!
//! This technical API documentation in aimed at developers who want to develop bindings for a different language,
//! build a UI based on the engine or just use the library in a Rust program
//!
//! ## Basic usage
//!
//! Add the dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ironcalc = { git = "https://github.com/ironcalc/IronCalc", version = "0.1.2"}
//! ```
//!
//! <small> until version 0.5.0 you should use the git dependencies as stated </small>
//!
//! A simple example with some numbers, a new sheet and a formula:
//!
//!
//! ```rust
#![doc = include_str!("../examples/hello_calc.rs")]
//! ```
//!
//! ## Styling the workbook
//!
//! Adding colors, to cells, full columns or full rows is easy
//!
//! ```rust
#![doc = include_str!("../examples/hello_styles.rs")]
//! ```
//!
//! Changing column width and row heigh
//!
//! ```rust
#![doc = include_str!("../examples/widths_and_heights.rs")]
//! ```
//!  

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/ironcalc/ironcalc/main/assets/favicon.ico"
)]

pub mod compare;
pub mod error;
pub mod export;
pub mod import;
pub use ironcalc_base as base;
