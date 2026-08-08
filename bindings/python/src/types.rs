use pyo3::prelude::*;
use xlsx::base::types::CellType;

/// The type of the content of a cell, following Excel's TYPE() convention.
#[pyclass(name = "CellType", eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyCellType {
    Number = 1,
    Text = 2,
    LogicalValue = 4,
    ErrorValue = 16,
    Array = 64,
    CompoundData = 128,
}

impl From<CellType> for PyCellType {
    fn from(cell_type: CellType) -> Self {
        match cell_type {
            CellType::Number => PyCellType::Number,
            CellType::Text => PyCellType::Text,
            CellType::LogicalValue => PyCellType::LogicalValue,
            CellType::ErrorValue => PyCellType::ErrorValue,
            CellType::Array => PyCellType::Array,
            CellType::CompoundData => PyCellType::CompoundData,
        }
    }
}
