pub mod fractional_index;
pub mod fractional_key;
pub mod log;
mod model;
mod patch;
mod workbook;

pub type DynError = Box<dyn std::error::Error>;
