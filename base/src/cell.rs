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
            Cell::CellFormula { f, .. }
            | Cell::CellFormulaBoolean { f, .. }
            | Cell::CellFormulaNumber { f, .. }
            | Cell::CellFormulaString { f, .. }
            | Cell::CellFormulaError { f, .. }
            | Cell::DynamicCellFormula { f, .. }
            | Cell::DynamicCellFormulaBoolean { f, .. }
            | Cell::DynamicCellFormulaNumber { f, .. }
            | Cell::DynamicCellFormulaString { f, .. }
            | Cell::DynamicCellFormulaError { f, .. } => Some(*f),
            Cell::EmptyCell { .. }
            | Cell::BooleanCell { .. }
            | Cell::NumberCell { .. }
            | Cell::ErrorCell { .. }
            | Cell::SharedString { .. }
            | Cell::SpillNumberCell { .. }
            | Cell::SpillBooleanCell { .. }
            | Cell::SpillErrorCell { .. }
            | Cell::SpillStringCell { .. } => None,
        }
    }

    /// Returns the dynamic range of a cell if any.
    pub fn get_dynamic_range(&self) -> Option<(i32, i32)> {
        match self {
            Cell::DynamicCellFormula { r, .. } => Some(*r),
            Cell::DynamicCellFormulaBoolean { r, .. } => Some(*r),
            Cell::DynamicCellFormulaNumber { r, .. } => Some(*r),
            Cell::DynamicCellFormulaString { r, .. } => Some(*r),
            Cell::DynamicCellFormulaError { r, .. } => Some(*r),
            Cell::EmptyCell { .. }
            | Cell::BooleanCell { .. }
            | Cell::NumberCell { .. }
            | Cell::ErrorCell { .. }
            | Cell::SharedString { .. }
            | Cell::CellFormula { .. }
            | Cell::CellFormulaBoolean { .. }
            | Cell::CellFormulaNumber { .. }
            | Cell::CellFormulaString { .. }
            | Cell::CellFormulaError { .. }
            | Cell::SpillNumberCell { .. }
            | Cell::SpillBooleanCell { .. }
            | Cell::SpillErrorCell { .. }
            | Cell::SpillStringCell { .. } => None,
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
            Cell::SpillBooleanCell { s, .. } => *s = style,
            Cell::SpillNumberCell { s, .. } => *s = style,
            Cell::SpillStringCell { s, .. } => *s = style,
            Cell::SpillErrorCell { s, .. } => *s = style,
            Cell::DynamicCellFormula { s, .. } => *s = style,
            Cell::DynamicCellFormulaBoolean { s, .. } => *s = style,
            Cell::DynamicCellFormulaNumber { s, .. } => *s = style,
            Cell::DynamicCellFormulaString { s, .. } => *s = style,
            Cell::DynamicCellFormulaError { s, .. } => *s = style,
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
            Cell::SpillBooleanCell { s, .. } => *s,
            Cell::SpillNumberCell { s, .. } => *s,
            Cell::SpillStringCell { s, .. } => *s,
            Cell::SpillErrorCell { s, .. } => *s,
            Cell::DynamicCellFormula { s, .. } => *s,
            Cell::DynamicCellFormulaBoolean { s, .. } => *s,
            Cell::DynamicCellFormulaNumber { s, .. } => *s,
            Cell::DynamicCellFormulaString { s, .. } => *s,
            Cell::DynamicCellFormulaError { s, .. } => *s,
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
            Cell::SpillBooleanCell { .. } => CellType::LogicalValue,
            Cell::SpillNumberCell { .. } => CellType::Number,
            Cell::SpillStringCell { .. } => CellType::Text,
            Cell::SpillErrorCell { .. } => CellType::ErrorValue,
            Cell::DynamicCellFormula { .. } => CellType::Number,
            Cell::DynamicCellFormulaBoolean { .. } => CellType::LogicalValue,
            Cell::DynamicCellFormulaNumber { .. } => CellType::Number,
            Cell::DynamicCellFormulaString { .. } => CellType::Text,
            Cell::DynamicCellFormulaError { .. } => CellType::ErrorValue,
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
            Cell::ErrorCell { ei, .. } | Cell::SpillErrorCell { ei, .. } => {
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
            Cell::DynamicCellFormula { .. } | Cell::CellFormula { .. } => {
                CellValue::String("#ERROR!".to_string())
            }
            Cell::DynamicCellFormulaBoolean { v, .. } | Cell::CellFormulaBoolean { v, .. } => {
                CellValue::Boolean(*v)
            }
            Cell::DynamicCellFormulaNumber { v, .. } | Cell::CellFormulaNumber { v, .. } => {
                CellValue::Number(*v)
            }
            Cell::DynamicCellFormulaString { v, .. } | Cell::CellFormulaString { v, .. } => {
                CellValue::String(v.clone())
            }
            Cell::DynamicCellFormulaError { ei, .. } | Cell::CellFormulaError { ei, .. } => {
                let v = ei.to_localized_error_string(language);
                CellValue::String(v)
            }
            Cell::SpillBooleanCell { v, .. } => CellValue::Boolean(*v),
            Cell::SpillNumberCell { v, .. } => CellValue::Number(*v),
            Cell::SpillStringCell { v, .. } => CellValue::String(v.clone()),
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
