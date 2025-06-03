use pyo3::exceptions::PyException;
use pyo3::{create_exception, prelude::*, wrap_pyfunction};

use types::{PySheetProperty, PyStyle};
use xlsx::base::types::{Style, Workbook};
use xlsx::base::{Model, UserModel};

use xlsx::export::{save_to_icalc, save_to_xlsx};
use xlsx::import;

mod types;

use crate::types::PyCellType;

create_exception!(_ironcalc, WorkbookError, PyException);

#[pyclass]
pub struct PyUserModel {
    /// The user model, which is a wrapper around the Model
    pub model: UserModel,
}

#[pymethods]
impl PyUserModel {
    /// Saves the user model to an xlsx file
    pub fn save_to_xlsx(&self, file: &str) -> PyResult<()> {
        let model = self.model.get_model();
        save_to_xlsx(model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Saves the user model to file in the internal binary ic format
    pub fn save_to_icalc(&self, file: &str) -> PyResult<()> {
        let model = self.model.get_model();
        save_to_icalc(model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn apply_external_diffs(&mut self, external_diffs: &[u8]) -> PyResult<()> {
        self.model
            .apply_external_diffs(external_diffs)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn flush_send_queue(&mut self) -> Vec<u8> {
        self.model.flush_send_queue()
    }

    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .set_user_input(sheet, row, column, value)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn to_bytes(&self) -> PyResult<Vec<u8>> {
        let bytes = self.model.to_bytes();
        Ok(bytes)
    }
}

/// This is a model implementing the 'raw' API
#[pyclass]
pub struct PyModel {
    model: Model,
}

#[pymethods]
impl PyModel {
    /// Saves the model to an xlsx file
    pub fn save_to_xlsx(&self, file: &str) -> PyResult<()> {
        save_to_xlsx(&self.model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Saves the model to file in the internal binary ic format
    pub fn save_to_icalc(&self, file: &str) -> PyResult<()> {
        save_to_icalc(&self.model, file).map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// To bytes
    pub fn to_bytes(&self) -> PyResult<Vec<u8>> {
        let bytes = self.model.to_bytes();
        Ok(bytes)
    }

    /// Evaluates the workbook
    pub fn evaluate(&mut self) {
        self.model.evaluate()
    }

    // Set values

    /// Set an input
    pub fn set_user_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> PyResult<()> {
        self.model
            .set_user_input(sheet, row, column, value.to_string())
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn clear_cell_contents(&mut self, sheet: u32, row: i32, column: i32) -> PyResult<()> {
        self.model
            .cell_clear_contents(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Get values

    /// Get raw value
    pub fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_cell_content(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Get cell type
    pub fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyCellType> {
        self.model
            .get_cell_type(sheet, row, column)
            .map(|cell_type| cell_type.into())
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    /// Get formatted value
    pub fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> PyResult<String> {
        self.model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Set styles
    pub fn set_cell_style(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        py_style: &PyStyle,
    ) -> PyResult<()> {
        let style: Style = py_style.into();
        self.model
            .set_cell_style(sheet, row, column, &style)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Get styles
    pub fn get_cell_style(&self, sheet: u32, row: i32, column: i32) -> PyResult<PyStyle> {
        let style = self
            .model
            .get_style_for_cell(sheet, row, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))?;
        Ok(style.into())
    }

    // column widths, row heights
    // insert/delete rows/columns

    pub fn insert_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .insert_rows(sheet, row, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn insert_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .insert_columns(sheet, column, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn delete_rows(&mut self, sheet: u32, row: i32, row_count: i32) -> PyResult<()> {
        self.model
            .delete_rows(sheet, row, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn delete_columns(&mut self, sheet: u32, column: i32, column_count: i32) -> PyResult<()> {
        self.model
            .delete_columns(sheet, column, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_column_width(&self, sheet: u32, column: i32) -> PyResult<f64> {
        self.model
            .get_column_width(sheet, column)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_row_height(&self, sheet: u32, row: i32) -> PyResult<f64> {
        self.model
            .get_row_height(sheet, row)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_column_width(&mut self, sheet: u32, column: i32, width: f64) -> PyResult<()> {
        self.model
            .set_column_width(sheet, column, width)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_row_height(&mut self, sheet: u32, row: i32, height: f64) -> PyResult<()> {
        self.model
            .set_row_height(sheet, row, height)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // frozen rows/columns

    pub fn get_frozen_columns_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_columns_count(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn get_frozen_rows_count(&self, sheet: u32) -> PyResult<i32> {
        self.model
            .get_frozen_rows_count(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_frozen_columns_count(&mut self, sheet: u32, column_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_columns(sheet, column_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn set_frozen_rows_count(&mut self, sheet: u32, row_count: i32) -> PyResult<()> {
        self.model
            .set_frozen_rows(sheet, row_count)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    // Manipulate sheets (add/remove/rename/change color)
    pub fn get_worksheets_properties(&self) -> PyResult<Vec<PySheetProperty>> {
        Ok(self
            .model
            .get_worksheets_properties()
            .into_iter()
            .map(|s| PySheetProperty {
                name: s.name,
                state: s.state,
                sheet_id: s.sheet_id,
                color: s.color,
            })
            .collect())
    }

    pub fn set_sheet_color(&mut self, sheet: u32, color: &str) -> PyResult<()> {
        self.model
            .set_sheet_color(sheet, color)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn add_sheet(&mut self, sheet_name: &str) -> PyResult<()> {
        self.model
            .add_sheet(sheet_name)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn new_sheet(&mut self) {
        self.model.new_sheet();
    }

    pub fn delete_sheet(&mut self, sheet: u32) -> PyResult<()> {
        self.model
            .delete_sheet(sheet)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    pub fn rename_sheet(&mut self, sheet: u32, new_name: &str) -> PyResult<()> {
        self.model
            .rename_sheet_by_index(sheet, new_name)
            .map_err(|e| WorkbookError::new_err(e.to_string()))
    }

    #[allow(clippy::panic)]
    pub fn test_panic(&self) -> PyResult<()> {
        panic!("This function panics for testing panic handling");
    }
}

// Create methods

/// Loads a function from an xlsx file
#[pyfunction]
pub fn load_from_xlsx(file_path: &str, locale: &str, tz: &str) -> PyResult<PyModel> {
    let model = import::load_from_xlsx(file_path, locale, tz)
        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

/// Loads a function from icalc binary representation
#[pyfunction]
pub fn load_from_icalc(file_name: &str) -> PyResult<PyModel> {
    let model =
        import::load_from_icalc(file_name).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

#[pyfunction]
pub fn load_from_bytes(bytes: &[u8]) -> PyResult<PyModel> {
    let workbook: Workbook =
        bitcode::decode(bytes).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    let model =
        Model::from_workbook(workbook).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

/// Creates an empty model
#[pyfunction]
pub fn create(name: &str, locale: &str, tz: &str) -> PyResult<PyModel> {
    let model =
        Model::new_empty(name, locale, tz).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyModel { model })
}

#[pyfunction]
pub fn create_user_model(name: &str, locale: &str, tz: &str) -> PyResult<PyUserModel> {
    let model = UserModel::new_empty(name, locale, tz)
        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
    Ok(PyUserModel { model })
}

#[pyfunction]
pub fn create_user_model_from_xlsx(
    file_path: &str,
    locale: &str,
    tz: &str,
) -> PyResult<PyUserModel> {
    let model = import::load_from_xlsx(file_path, locale, tz)
        .map_err(|e| WorkbookError::new_err(e.to_string()))?;
    let model = UserModel::from_model(model);
    Ok(PyUserModel { model })
}

#[pyfunction]
pub fn create_user_model_from_icalc(file_name: &str) -> PyResult<PyUserModel> {
    let model =
        import::load_from_icalc(file_name).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    let model = UserModel::from_model(model);
    Ok(PyUserModel { model })
}

#[pyfunction]
pub fn create_user_model_from_bytes(bytes: &[u8]) -> PyResult<PyUserModel> {
    let workbook: Workbook =
        bitcode::decode(bytes).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    let model =
        Model::from_workbook(workbook).map_err(|e| WorkbookError::new_err(e.to_string()))?;
    let user_model = UserModel::from_model(model);
    Ok(PyUserModel { model: user_model })
}

#[pyfunction]
#[allow(clippy::panic)]
pub fn test_panic() {
    panic!("This function panics for testing panic handling");
}

#[pymodule]
fn ironcalc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Add the package version to the module
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Add the functions to the module using the `?` operator
    m.add_function(wrap_pyfunction!(create, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_icalc, m)?)?;
    m.add_function(wrap_pyfunction!(load_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(test_panic, m)?)?;

    // User model functions
    m.add_function(wrap_pyfunction!(create_user_model, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(create_user_model_from_icalc, m)?)?;

    Ok(())
}
