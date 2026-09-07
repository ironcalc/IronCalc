#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod model;
mod user_model;

pub use model::Model;
pub use user_model::UserModel;

use serde::Serialize;

use napi::{bindgen_prelude::*, Env, Result, Unknown};

use ironcalc::base::expressions::types::Area;
use ironcalc::base::expressions::utils::{
  column_to_number, number_to_column, quote_name as quote_name_ic,
};
use ironcalc::base::types::Color;

pub(crate) fn to_js_error(error: impl std::fmt::Display) -> Error {
  Error::new(Status::Unknown, error.to_string())
}

pub(crate) fn leak_str(s: &str) -> &'static str {
  Box::leak(s.to_owned().into_boxed_str())
}

pub(crate) fn area(
  sheet: u32,
  start_row: i32,
  start_column: i32,
  end_row: i32,
  end_column: i32,
) -> Area {
  let (row1, row2) = if start_row <= end_row {
    (start_row, end_row)
  } else {
    (end_row, start_row)
  };
  let (col1, col2) = if start_column <= end_column {
    (start_column, end_column)
  } else {
    (end_column, start_column)
  };
  Area {
    sheet,
    row: row1,
    column: col1,
    width: col2 - col1 + 1,
    height: row2 - row1 + 1,
  }
}

/// Converts a JS value into a `Color`:
/// * `null` / `undefined` -> no color
/// * `"#RRGGBB"` -> an RGB color
/// * `[theme, tint]` -> a theme color
pub(crate) fn js_to_color(env: &Env, value: Option<Unknown>) -> Result<Color> {
  let color: Color = match value {
    None => Color::None,
    Some(value) => env.from_js_value(value).map_err(to_js_error)?,
  };
  if let Color::Rgb(rgb) = &color {
    Color::from_rgb(rgb).map_err(to_js_error)?;
  }
  Ok(color)
}

#[derive(Serialize)]
pub(crate) struct DefinedName {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scope: Option<u32>,
  pub formula: String,
}

/// The name and index of a newly created sheet.
#[napi(object)]
pub struct NewSheet {
  pub name: String,
  pub index: u32,
}

// Local mirror of `ironcalc_base::FmtSettings`, which does not implement Serialize
#[derive(Serialize)]
pub(crate) struct FmtSettings {
  pub currency: String,
  pub currency_format: String,
  pub short_date: String,
  pub short_date_example: String,
  pub long_date: String,
  pub long_date_example: String,
  pub number_fmt: String,
  pub number_example: String,
}

impl From<ironcalc::base::FmtSettings> for FmtSettings {
  fn from(settings: ironcalc::base::FmtSettings) -> Self {
    FmtSettings {
      currency: settings.currency,
      currency_format: settings.currency_format,
      short_date: settings.short_date,
      short_date_example: settings.short_date_example,
      long_date: settings.long_date,
      long_date_example: settings.long_date_example,
      number_fmt: settings.number_fmt,
      number_example: settings.number_example,
    }
  }
}

/// The type of the content of a cell, following Excel's TYPE() convention.
#[napi]
pub enum CellType {
  Number = 1,
  Text = 2,
  LogicalValue = 4,
  ErrorValue = 16,
  Array = 64,
  CompoundData = 128,
}

impl From<ironcalc::base::types::CellType> for CellType {
  fn from(cell_type: ironcalc::base::types::CellType) -> Self {
    match cell_type {
      ironcalc::base::types::CellType::Number => CellType::Number,
      ironcalc::base::types::CellType::Text => CellType::Text,
      ironcalc::base::types::CellType::LogicalValue => CellType::LogicalValue,
      ironcalc::base::types::CellType::ErrorValue => CellType::ErrorValue,
      ironcalc::base::types::CellType::Array => CellType::Array,
      ironcalc::base::types::CellType::CompoundData => CellType::CompoundData,
    }
  }
}

// Top level utility functions

/// Returns the column name ("A", "B", ..., "XFD") for a column number (1-indexed)
#[napi]
pub fn column_name_from_number(column: i32) -> Result<String> {
  number_to_column(column).ok_or_else(|| to_js_error("Invalid column number"))
}

/// Returns the column number (1-indexed) for a column name ("A", "B", ..., "XFD")
#[napi]
pub fn column_number_from_name(column: String) -> Result<i32> {
  column_to_number(&column).map_err(to_js_error)
}

/// Quotes a sheet name if needed so it can be used in a formula reference
#[napi]
pub fn quote_name(name: String) -> String {
  quote_name_ic(&name)
}

/// Returns the list of all supported timezones
#[napi]
pub fn get_all_timezones() -> Vec<String> {
  ironcalc::base::get_all_timezones()
}

/// Returns the list of all supported locales
#[napi]
pub fn get_supported_locales() -> Vec<String> {
  ironcalc::base::get_supported_locales()
}
