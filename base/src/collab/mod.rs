pub mod codec;
pub mod fractional_index;
pub mod fractional_key;
pub mod log;
mod model;
mod patch;
pub mod varint;
mod workbook;

pub type DynError = Box<dyn std::error::Error>;
