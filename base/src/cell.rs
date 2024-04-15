use crate::{
    expressions::token::Error, language::Language, number_format::to_excel_precision_str, types::*,
};

/// A CellValue is the representation of the cell content.
#[derive(Debug, PartialEq)]
pub enum CellValue {
    None,
    String(String),
    Number(f64),
    Boolean(bool),
}

impl From<f64> for CellValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for CellValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for CellValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<bool> for CellValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl Cell {
    /// Creates a new Cell with a shared string (`si` is the string index)
    pub fn new_string(si: i32, s: i32) -> Cell {
        Cell::SharedString { si, s }
    }

    /// Creates a new Cell with a number
    pub fn new_number(v: f64, s: i32) -> Cell {
        Cell::NumberCell { v, s }
    }

    /// Creates a new Cell with a boolean
    pub fn new_boolean(v: bool, s: i32) -> Cell {
        Cell::BooleanCell { v, s }
    }

    /// Creates a new Cell with an error value
    pub fn new_error(ei: Error, s: i32) -> Cell {
        Cell::ErrorCell { ei, s }
    }

    /// Creates a new Cell with an unevaluated formula
    pub fn new_formula(f: i32, s: i32) -> Cell {
        Cell::CellFormula { f, s }
    }

    /// Returns the formula of a cell if any.
    pub fn get_formula(&self) -> Option<i32> {
        match self {
            Cell::CellFormula { f, .. } => Some(*f),
            Cell::CellFormulaBoolean { f, .. } => Some(*f),
            Cell::CellFormulaNumber { f, .. } => Some(*f),
            Cell::CellFormulaString { f, .. } => Some(*f),
            Cell::CellFormulaError { f, .. } => Some(*f),
            _ => None,
        }
    }

    pub fn has_formula(&self) -> bool {
        self.get_formula().is_some()
    }

    pub fn set_style(&mut self, style: i32) {
        match self {
            Cell::EmptyCell { s, .. } => *s = style,
            Cell::BooleanCell { s, .. } => *s = style,
            Cell::NumberCell { s, .. } => *s = style,
            Cell::ErrorCell { s, .. } => *s = style,
            Cell::SharedString { s, .. } => *s = style,
            Cell::CellFormula { s, .. } => *s = style,
            Cell::CellFormulaBoolean { s, .. } => *s = style,
            Cell::CellFormulaNumber { s, .. } => *s = style,
            Cell::CellFormulaString { s, .. } => *s = style,
            Cell::CellFormulaError { s, .. } => *s = style,
        };
    }

    pub fn get_style(&self) -> i32 {
        match self {
            Cell::EmptyCell { s, .. } => *s,
            Cell::BooleanCell { s, .. } => *s,
            Cell::NumberCell { s, .. } => *s,
            Cell::ErrorCell { s, .. } => *s,
            Cell::SharedString { s, .. } => *s,
            Cell::CellFormula { s, .. } => *s,
            Cell::CellFormulaBoolean { s, .. } => *s,
            Cell::CellFormulaNumber { s, .. } => *s,
            Cell::CellFormulaString { s, .. } => *s,
            Cell::CellFormulaError { s, .. } => *s,
        }
    }

    pub fn get_type(&self) -> CellType {
        match self {
            Cell::EmptyCell { .. } => CellType::Number,
            Cell::BooleanCell { .. } => CellType::LogicalValue,
            Cell::NumberCell { .. } => CellType::Number,
            Cell::ErrorCell { .. } => CellType::ErrorValue,
            Cell::SharedString { .. } => CellType::Text,
            Cell::CellFormula { .. } => CellType::Number,
            Cell::CellFormulaBoolean { .. } => CellType::LogicalValue,
            Cell::CellFormulaNumber { .. } => CellType::Number,
            Cell::CellFormulaString { .. } => CellType::Text,
            Cell::CellFormulaError { .. } => CellType::ErrorValue,
        }
    }

    pub fn get_text(&self, shared_strings: &[String], language: &Language) -> String {
        match self.value(shared_strings, language) {
            CellValue::None => "".to_string(),
            CellValue::String(v) => v,
            CellValue::Boolean(v) => v.to_string().to_uppercase(),
            CellValue::Number(v) => to_excel_precision_str(v),
        }
    }

    pub fn value(&self, shared_strings: &[String], language: &Language) -> CellValue {
        match self {
            Cell::EmptyCell { .. } => CellValue::None,
            Cell::BooleanCell { v, s: _ } => CellValue::Boolean(*v),
            Cell::NumberCell { v, s: _ } => CellValue::Number(*v),
            Cell::ErrorCell { ei, .. } => {
                let v = ei.to_localized_error_string(language);
                CellValue::String(v)
            }
            Cell::SharedString { si, .. } => {
                let s = shared_strings.get(*si as usize);
                let v = match s {
                    Some(str) => str.clone(),
                    None => "".to_string(),
                };
                CellValue::String(v)
            }
            Cell::CellFormula { .. } => CellValue::String("#ERROR!".to_string()),
            Cell::CellFormulaBoolean { v, .. } => CellValue::Boolean(*v),
            Cell::CellFormulaNumber { v, .. } => CellValue::Number(*v),
            Cell::CellFormulaString { v, .. } => CellValue::String(v.clone()),
            Cell::CellFormulaError { ei, .. } => {
                let v = ei.to_localized_error_string(language);
                CellValue::String(v)
            }
        }
    }

    pub fn formatted_value<F>(
        &self,
        shared_strings: &[String],
        language: &Language,
        format_number: F,
    ) -> String
    where
        F: Fn(f64) -> String,
    {
        match self.value(shared_strings, language) {
            CellValue::None => "".to_string(),
            CellValue::String(value) => value,
            CellValue::Boolean(value) => value.to_string().to_uppercase(),
            CellValue::Number(value) => format_number(value),
        }
    }
}
