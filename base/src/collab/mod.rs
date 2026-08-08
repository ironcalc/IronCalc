pub mod codec;
pub mod fractional_index;
pub mod fractional_key;
pub mod hlc;
pub mod log;
pub mod model;
mod patch;
pub mod varint;

pub type DynError = Box<dyn std::error::Error>;
