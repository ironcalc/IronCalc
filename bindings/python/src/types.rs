use pyo3::prelude::*;
use xlsx::base::types::{
    Alignment, Border, BorderItem, BorderStyle, CellType, Fill, Font, FontScheme,
    HorizontalAlignment, Style, VerticalAlignment,
};

#[derive(Clone)]
#[pyclass]
pub struct PySheetProperty {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub state: String,
    #[pyo3(get)]
    pub sheet_id: u32,
    #[pyo3(get)]
    pub color: Option<String>,
}

#[pyclass]
#[derive(Clone)]
pub struct Cell {
    #[pyo3(get, set)]
    pub row: i32,
    #[pyo3(get, set)]
    pub column: i32,
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyHorizontalAlignment {
    Center,
    CenterContinuous,
    Distributed,
    Fill,
    General,
    Justify,
    Left,
    Right,
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyVerticalAlignment {
    Bottom,
    Center,
    Distributed,
    Justify,
    Top,
}

#[pyclass]
#[derive(Clone)]
pub struct PyAlignment {
    #[pyo3(get)]
    pub horizontal: PyHorizontalAlignment,
    #[pyo3(get)]
    pub vertical: PyVerticalAlignment,
    #[pyo3(get)]
    pub wrap_text: bool,
}

#[pyclass]
#[derive(Clone)]
pub struct PyStyle {
    #[pyo3(get)]
    pub alignment: Option<PyAlignment>,
    #[pyo3(get)]
    pub num_fmt: String,
    #[pyo3(get)]
    pub fill: PyFill,
    #[pyo3(get)]
    pub font: PyFont,
    #[pyo3(get)]
    pub border: PyBorder,
    #[pyo3(get)]
    pub quote_prefix: bool,
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyBorderStyle {
    Thin,
    Medium,
    Thick,
    Double,
    Dotted,
    SlantDashDot,
    MediumDashed,
    MediumDashDotDot,
    MediumDashDot,
}

#[pyclass]
#[derive(Clone)]
pub struct PyBorderItem {
    #[pyo3(get)]
    pub style: PyBorderStyle,
    #[pyo3(get)]
    pub color: Option<String>,
}

#[pyclass]
#[derive(Clone)]
pub struct PyBorder {
    #[pyo3(get)]
    pub diagonal_up: bool,
    #[pyo3(get)]
    pub diagonal_down: bool,
    #[pyo3(get)]
    pub left: Option<PyBorderItem>,
    #[pyo3(get)]
    pub right: Option<PyBorderItem>,
    #[pyo3(get)]
    pub top: Option<PyBorderItem>,
    #[pyo3(get)]
    pub bottom: Option<PyBorderItem>,
    #[pyo3(get)]
    pub diagonal: Option<PyBorderItem>,
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyFontScheme {
    Minor,
    Major,
    None,
}

#[pyclass]
#[derive(Clone)]
pub struct PyFont {
    #[pyo3(get)]
    pub strike: bool,
    #[pyo3(get)]
    pub u: bool,
    #[pyo3(get)]
    pub b: bool,
    #[pyo3(get)]
    pub i: bool,
    #[pyo3(get)]
    pub sz: i32,
    #[pyo3(get)]
    pub color: Option<String>,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub family: i32,
    #[pyo3(get)]
    pub scheme: PyFontScheme,
}

#[pyclass]
#[derive(Clone)]
pub struct PyFill {
    #[pyo3(get)]
    pub pattern_type: String,
    #[pyo3(get)]
    pub fg_color: Option<String>,
    #[pyo3(get)]
    pub bg_color: Option<String>,
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone)]
pub enum PyCellType {
    Number = 1,
    Text = 2,
    LogicalValue = 4,
    ErrorValue = 16,
    Array = 64,
    CompoundData = 128,
}

// Conversions from references to Py* types to non-Py types

// Enums

impl From<PyHorizontalAlignment> for HorizontalAlignment {
    fn from(py_align: PyHorizontalAlignment) -> Self {
        match py_align {
            PyHorizontalAlignment::Center => HorizontalAlignment::Center,
            PyHorizontalAlignment::CenterContinuous => HorizontalAlignment::CenterContinuous,
            PyHorizontalAlignment::Distributed => HorizontalAlignment::Distributed,
            PyHorizontalAlignment::Fill => HorizontalAlignment::Fill,
            PyHorizontalAlignment::General => HorizontalAlignment::General,
            PyHorizontalAlignment::Justify => HorizontalAlignment::Justify,
            PyHorizontalAlignment::Left => HorizontalAlignment::Left,
            PyHorizontalAlignment::Right => HorizontalAlignment::Right,
        }
    }
}

impl From<PyVerticalAlignment> for VerticalAlignment {
    fn from(py_align: PyVerticalAlignment) -> Self {
        match py_align {
            PyVerticalAlignment::Bottom => VerticalAlignment::Bottom,
            PyVerticalAlignment::Center => VerticalAlignment::Center,
            PyVerticalAlignment::Distributed => VerticalAlignment::Distributed,
            PyVerticalAlignment::Justify => VerticalAlignment::Justify,
            PyVerticalAlignment::Top => VerticalAlignment::Top,
        }
    }
}

impl From<PyFontScheme> for FontScheme {
    fn from(py_scheme: PyFontScheme) -> Self {
        match py_scheme {
            PyFontScheme::Minor => FontScheme::Minor,
            PyFontScheme::Major => FontScheme::Major,
            PyFontScheme::None => FontScheme::None,
        }
    }
}

impl From<&PyBorderStyle> for BorderStyle {
    fn from(py_style: &PyBorderStyle) -> Self {
        match *py_style {
            PyBorderStyle::Thin => BorderStyle::Thin,
            PyBorderStyle::Medium => BorderStyle::Medium,
            PyBorderStyle::Thick => BorderStyle::Thick,
            PyBorderStyle::Double => BorderStyle::Double,
            PyBorderStyle::Dotted => BorderStyle::Dotted,
            PyBorderStyle::SlantDashDot => BorderStyle::SlantDashDot,
            PyBorderStyle::MediumDashed => BorderStyle::MediumDashed,
            PyBorderStyle::MediumDashDotDot => BorderStyle::MediumDashDotDot,
            PyBorderStyle::MediumDashDot => BorderStyle::MediumDashDot,
        }
    }
}

// Structs

impl From<&PyFill> for Fill {
    fn from(py_fill: &PyFill) -> Self {
        Fill {
            pattern_type: py_fill.pattern_type.clone(),
            fg_color: py_fill.fg_color.clone(),
            bg_color: py_fill.bg_color.clone(),
        }
    }
}

impl From<&PyFont> for Font {
    fn from(py_font: &PyFont) -> Self {
        Font {
            strike: py_font.strike,
            u: py_font.u,
            b: py_font.b,
            i: py_font.i,
            sz: py_font.sz,
            color: py_font.color.clone(),
            name: py_font.name.clone(),
            family: py_font.family,
            scheme: py_font.scheme.clone().into(),
        }
    }
}

impl From<&PyBorderItem> for BorderItem {
    fn from(py_item: &PyBorderItem) -> Self {
        BorderItem {
            style: (&py_item.style).into(),
            color: py_item.color.clone(),
        }
    }
}

impl From<&PyBorder> for Border {
    fn from(py_border: &PyBorder) -> Self {
        Border {
            diagonal_up: py_border.diagonal_up,
            diagonal_down: py_border.diagonal_down,
            left: py_border.left.as_ref().map(|item| item.into()),
            right: py_border.right.as_ref().map(|item| item.into()),
            top: py_border.top.as_ref().map(|item| item.into()),
            bottom: py_border.bottom.as_ref().map(|item| item.into()),
            diagonal: py_border.diagonal.as_ref().map(|item| item.into()),
        }
    }
}

impl From<&PyAlignment> for Alignment {
    fn from(py_align: &PyAlignment) -> Self {
        Alignment {
            horizontal: py_align.horizontal.clone().into(),
            vertical: py_align.vertical.clone().into(),
            wrap_text: py_align.wrap_text,
        }
    }
}

impl From<&PyStyle> for Style {
    fn from(py_style: &PyStyle) -> Self {
        Style {
            alignment: py_style.alignment.as_ref().map(|a| a.into()),
            num_fmt: py_style.num_fmt.clone(),
            fill: (&py_style.fill).into(),
            font: (&py_style.font).into(),
            border: (&py_style.border).into(),
            quote_prefix: py_style.quote_prefix,
        }
    }
}

// From non-Py to Py
impl From<Fill> for PyFill {
    fn from(fill: Fill) -> Self {
        PyFill {
            pattern_type: fill.pattern_type,
            fg_color: fill.fg_color,
            bg_color: fill.bg_color,
        }
    }
}

// From non-Py to Py
impl From<HorizontalAlignment> for PyHorizontalAlignment {
    fn from(align: HorizontalAlignment) -> Self {
        match align {
            HorizontalAlignment::Center => PyHorizontalAlignment::Center,
            HorizontalAlignment::CenterContinuous => PyHorizontalAlignment::CenterContinuous,
            HorizontalAlignment::Distributed => PyHorizontalAlignment::Distributed,
            HorizontalAlignment::Fill => PyHorizontalAlignment::Fill,
            HorizontalAlignment::General => PyHorizontalAlignment::General,
            HorizontalAlignment::Justify => PyHorizontalAlignment::Justify,
            HorizontalAlignment::Left => PyHorizontalAlignment::Left,
            HorizontalAlignment::Right => PyHorizontalAlignment::Right,
        }
    }
}

// From non-Py to Py
impl From<VerticalAlignment> for PyVerticalAlignment {
    fn from(align: VerticalAlignment) -> Self {
        match align {
            VerticalAlignment::Bottom => PyVerticalAlignment::Bottom,
            VerticalAlignment::Center => PyVerticalAlignment::Center,
            VerticalAlignment::Distributed => PyVerticalAlignment::Distributed,
            VerticalAlignment::Justify => PyVerticalAlignment::Justify,
            VerticalAlignment::Top => PyVerticalAlignment::Top,
        }
    }
}

// From non-Py to Py
impl From<FontScheme> for PyFontScheme {
    fn from(scheme: FontScheme) -> Self {
        match scheme {
            FontScheme::Minor => PyFontScheme::Minor,
            FontScheme::Major => PyFontScheme::Major,
            FontScheme::None => PyFontScheme::None,
        }
    }
}

// From non-Py to Py
impl From<BorderStyle> for PyBorderStyle {
    fn from(style: BorderStyle) -> Self {
        match style {
            BorderStyle::Thin => PyBorderStyle::Thin,
            BorderStyle::Medium => PyBorderStyle::Medium,
            BorderStyle::Thick => PyBorderStyle::Thick,
            BorderStyle::Double => PyBorderStyle::Double,
            BorderStyle::Dotted => PyBorderStyle::Dotted,
            BorderStyle::SlantDashDot => PyBorderStyle::SlantDashDot,
            BorderStyle::MediumDashed => PyBorderStyle::MediumDashed,
            BorderStyle::MediumDashDotDot => PyBorderStyle::MediumDashDotDot,
            BorderStyle::MediumDashDot => PyBorderStyle::MediumDashDot,
        }
    }
}

// From non-Py to Py
impl From<Font> for PyFont {
    fn from(font: Font) -> Self {
        PyFont {
            strike: font.strike,
            u: font.u,
            b: font.b,
            i: font.i,
            sz: font.sz,
            color: font.color,
            name: font.name,
            family: font.family,
            scheme: font.scheme.into(),
        }
    }
}

// From non-Py to Py
impl From<BorderItem> for PyBorderItem {
    fn from(item: BorderItem) -> Self {
        PyBorderItem {
            style: item.style.into(),
            color: item.color,
        }
    }
}

// From non-Py to Py
impl From<Border> for PyBorder {
    fn from(border: Border) -> Self {
        PyBorder {
            diagonal_up: border.diagonal_up,
            diagonal_down: border.diagonal_down,
            left: border.left.map(|item| item.into()),
            right: border.right.map(|item| item.into()),
            top: border.top.map(|item| item.into()),
            bottom: border.bottom.map(|item| item.into()),
            diagonal: border.diagonal.map(|item| item.into()),
        }
    }
}

// From non-Py to Py
impl From<Alignment> for PyAlignment {
    fn from(align: Alignment) -> Self {
        PyAlignment {
            horizontal: align.horizontal.into(),
            vertical: align.vertical.into(),
            wrap_text: align.wrap_text,
        }
    }
}

// From non-Py to Py
impl From<Style> for PyStyle {
    fn from(style: Style) -> Self {
        PyStyle {
            alignment: style.alignment.map(|a| a.into()),
            num_fmt: style.num_fmt,
            fill: style.fill.into(),
            font: style.font.into(),
            border: style.border.into(),
            quote_prefix: style.quote_prefix,
        }
    }
}

// Conversion from PyCellType to CellType
impl From<PyCellType> for CellType {
    fn from(py_cell_type: PyCellType) -> Self {
        match py_cell_type {
            PyCellType::Number => CellType::Number,
            PyCellType::Text => CellType::Text,
            PyCellType::LogicalValue => CellType::LogicalValue,
            PyCellType::ErrorValue => CellType::ErrorValue,
            PyCellType::Array => CellType::Array,
            PyCellType::CompoundData => CellType::CompoundData,
        }
    }
}

// Conversion from CellType to PyCellType
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
