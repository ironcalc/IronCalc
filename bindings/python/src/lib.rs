use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::{create_exception, wrap_pyfunction};
use serde::Serialize;

use xlsx::base::expressions::types::Area;
use xlsx::base::expressions::utils::{
    column_to_number, number_to_column, quote_name as quote_name_ic,
};
use xlsx::base::types::{Color, Workbook};
use xlsx::base::{Model, UserModel};
use xlsx::import;

mod raw_model;
mod types;
mod user_model;

pub use raw_model::PyModel;
pub use types::PyCellType;
pub use user_model::PyUserModel;

create_exception!(_ironcalc, WorkbookError, PyException);

pub(crate) fn to_py_err(e: impl std::fmt::Display) -> PyErr {
    WorkbookError::new_err(e.to_string())
}

/// Converts any serde-serializable value into a Python object (dicts, lists, ...)
pub(crate) fn to_python<'py, T: Serialize>(
    py: Python<'py>,
    value: &T,
) -> PyResult<Bound<'py, PyAny>> {
    pythonize::pythonize(py, value).map_err(to_py_err)
}

/// Converts a Python object (dicts, lists, ...) into a serde-deserializable value
pub(crate) fn from_python<T: serde::de::DeserializeOwned>(obj: &Bound<'_, PyAny>) -> PyResult<T> {
    pythonize::depythonize(obj).map_err(to_py_err)
}

/// Converts a Python value into a `Color`:
/// * `None` -> no color
/// * `"#RRGGBB"` -> an RGB color
/// * `[theme, tint]` -> a theme color
pub(crate) fn py_to_color(obj: &Bound<'_, PyAny>) -> PyResult<Color> {
    if obj.is_none() {
        return Ok(Color::None);
    }
    if let Ok(s) = obj.extract::<String>() {
        return Color::from_rgb(&s).map_err(to_py_err);
    }
    from_python::<Color>(obj)
}

pub(crate) fn area(
    sheet: u32,
    start_row: i32,
    start_column: i32,
    end_row: i32,
    end_column: i32,
) -> Area {
    Area {
        sheet,
        row: start_row,
        column: start_column,
        width: end_column - start_column + 1,
        height: end_row - start_row + 1,
    }
}

pub(crate) fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
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

impl From<xlsx::base::FmtSettings> for FmtSettings {
    fn from(settings: xlsx::base::FmtSettings) -> Self {
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

// Top level utility functions

/// Returns the column name ("A", "B", ..., "XFD") for a column number (1-indexed)
#[pyfunction]
pub fn column_name_from_number(column: i32) -> PyResult<String> {
    number_to_column(column).ok_or_else(|| to_py_err("Invalid column number"))
}

/// Returns the column number (1-indexed) for a column name ("A", "B", ..., "XFD")
#[pyfunction]
pub fn column_number_from_name(column: &str) -> PyResult<i32> {
    column_to_number(column).map_err(to_py_err)
}

/// Quotes a sheet name if needed so it can be used in a formula reference
#[pyfunction]
pub fn quote_name(name: &str) -> String {
    quote_name_ic(name)
}

/// Returns the list of all supported timezones
#[pyfunction]
pub fn get_all_timezones() -> Vec<String> {
    xlsx::base::get_all_timezones()
}

/// Returns the list of all supported locales
#[pyfunction]
pub fn get_supported_locales() -> Vec<String> {
    xlsx::base::get_supported_locales()
}

// Create methods for the raw API

/// Loads a model from an xlsx file (raw API)
#[pyfunction]
#[pyo3(signature = (file_path, locale="en", tz="UTC", language_id="en"))]
pub fn load_from_xlsx(
    file_path: &str,
    locale: &str,
    tz: &str,
    language_id: &str,
) -> PyResult<PyModel> {
    let language_id = leak_str(language_id);
    let model = import::load_from_xlsx(file_path, locale, tz, language_id).map_err(to_py_err)?;
    Ok(PyModel { model })
}

/// Loads a model from a file in the internal binary ic format (raw API)
#[pyfunction]
#[pyo3(signature = (file_name, language_id="en"))]
pub fn load_from_icalc(file_name: &str, language_id: &str) -> PyResult<PyModel> {
    let language_id = leak_str(language_id);
    let model = import::load_from_icalc(file_name, language_id).map_err(to_py_err)?;
    Ok(PyModel { model })
}

/// Loads a model from bytes in the internal binary ic format (raw API).
/// This is the same format produced by `save_to_icalc` and `to_bytes`.
#[pyfunction]
#[pyo3(signature = (bytes, language_id="en"))]
pub fn load_from_bytes(bytes: &[u8], language_id: &str) -> PyResult<PyModel> {
    let workbook: Workbook = bitcode::decode(bytes).map_err(to_py_err)?;
    let language_id = leak_str(language_id);
    let model = Model::from_workbook(workbook, language_id).map_err(to_py_err)?;
    Ok(PyModel { model })
}

/// Creates an empty model (raw API)
#[pyfunction]
#[pyo3(signature = (name, locale="en", tz="UTC", language_id="en"))]
pub fn create(name: &str, locale: &str, tz: &str, language_id: &str) -> PyResult<PyModel> {
    let name = leak_str(name);
    let locale = leak_str(locale);
    let tz = leak_str(tz);
    let language_id = leak_str(language_id);
    let model = Model::new_empty(name, locale, tz, language_id).map_err(to_py_err)?;
    Ok(PyModel { model })
}

// Create methods for the user API

/// Creates an empty model (user API)
#[pyfunction]
#[pyo3(signature = (name, locale="en", tz="UTC", language_id="en"))]
pub fn create_user_model(
    name: &str,
    locale: &str,
    tz: &str,
    language_id: &str,
) -> PyResult<PyUserModel> {
    let name = leak_str(name);
    let locale = leak_str(locale);
    let tz = leak_str(tz);
    let language_id = leak_str(language_id);
    let model = UserModel::new_empty(name, locale, tz, language_id).map_err(to_py_err)?;
    Ok(PyUserModel { model })
}

/// Creates a user model from an xlsx file
#[pyfunction]
#[pyo3(signature = (file_path, locale="en", tz="UTC", language_id="en"))]
pub fn create_user_model_from_xlsx(
    file_path: &str,
    locale: &str,
    tz: &str,
    language_id: &str,
) -> PyResult<PyUserModel> {
    let language_id = leak_str(language_id);
    let model = import::load_from_xlsx(file_path, locale, tz, language_id).map_err(to_py_err)?;
    let model = UserModel::from_model(model);
    Ok(PyUserModel { model })
}

/// Creates a user model from an icalc file
#[pyfunction]
#[pyo3(signature = (file_name, language_id="en"))]
pub fn create_user_model_from_icalc(file_name: &str, language_id: &str) -> PyResult<PyUserModel> {
    let language_id = leak_str(language_id);
    let model = import::load_from_icalc(file_name, language_id).map_err(to_py_err)?;
    let model = UserModel::from_model(model);
    Ok(PyUserModel { model })
}

/// Creates a user model from bytes in the internal binary ic format.
/// This is the same format produced by `save_to_icalc` and `to_bytes`.
#[pyfunction]
#[pyo3(signature = (bytes, language_id="en"))]
pub fn create_user_model_from_bytes(bytes: &[u8], language_id: &str) -> PyResult<PyUserModel> {
    let workbook: Workbook = bitcode::decode(bytes).map_err(to_py_err)?;
    let language_id = leak_str(language_id);
    let model = Model::from_workbook(workbook, language_id).map_err(to_py_err)?;
    let user_model = UserModel::from_model(model);
    Ok(PyUserModel { model: user_model })
}

#[pyfunction]
#[allow(clippy::panic)]
pub fn test_panic() {
    panic!("This function panics for testing panic handling");
}

#[pymodule]
fn _ironcalc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("WorkbookError", m.py().get_type::<WorkbookError>())?;

    m.add_class::<PyModel>()?;
    m.add_class::<PyUserModel>()?;
    m.add_class::<PyCellType>()?;

    // Raw API functions
    m.add_function(wrap_pyfunction!(create, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_icalc, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(test_panic, m)?)?;

    // User API functions
    m.add_function(wrap_pyfunction!(create_user_model, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_icalc, m)?)?;

    // Utilities
    m.add_function(wrap_pyfunction!(column_name_from_number, m)?)?;
    m.add_function(wrap_pyfunction!(column_number_from_name, m)?)?;
    m.add_function(wrap_pyfunction!(quote_name, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_timezones, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_locales, m)?)?;

    Ok(())
}
