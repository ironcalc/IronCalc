#![allow(clippy::unwrap_used)]

use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

pub fn new_empty_model<'a>() -> Model<'a> {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
}

/// Build and evaluate a model from `(cell, formula_or_input)` pairs.
///
/// The entry point for nearly every unit test in this crate.  Pass an array of
/// `(address, content)` pairs — addresses use the `"A1"` short form (sheet 0
/// is assumed) or the `"Sheet1!B3"` qualified form.  Content is either a
/// formula string (starts with `"="`) or a literal user input.  The model is
/// evaluated once before being returned.
///
/// # Basic formula test
///
/// ```rust
/// let model = setup_model(&[("A1", "=ABS(-42)")]);
/// assert_eq!(model._get_text("A1"), "42");
/// ```
///
/// # Multiple independent cells
///
/// Cells are written in order, so later pairs can reference earlier ones:
///
/// ```rust
/// let model = setup_model(&[
///     ("A1", "10"),
///     ("A2", "=A1 * 2"),
///     ("A3", "=SUM(A1:A2)"),
/// ]);
/// assert_eq!(model._get_text("A2"), "20");
/// assert_eq!(model._get_text("A3"), "30");
/// ```
///
/// # Cell-reference dependencies
///
/// Because the model evaluates once after all inputs are written, any
/// formula that references another cell works regardless of declaration order:
///
/// ```rust
/// let model = setup_model(&[
///     ("B1", "=UPPER(A1)"),   // references A1, written before A1
///     ("A1", "hello"),
/// ]);
/// assert_eq!(model._get_text("B1"), "HELLO");
/// ```
///
pub(crate) fn setup_model<'a>(pairs: &[(&str, &str)]) -> Model<'a> {
    let mut model = new_empty_model();
    for (cell, expr) in pairs {
        model._set(cell, expr);
    }
    model.evaluate();
    model
}

/// Assert that each formula produces the expected text output in a single model
/// evaluation pass.
///
/// Each `(formula, expected)` pair is placed in consecutive cells (`A1`, `A2`,
/// …); the model is evaluated once and every anchor cell is checked.
///
/// ```rust
/// // Static string literals:
/// assert_formulas(&[("=ABS(-5)", "5"), ("=ABS(-3.5)", "3.5")]);
///
/// // Date functions
/// assert_formulas(&[
///    ("=DATE(2023,6,15)", "6/15/2023"),
///    ("=DATE(2023,1,1)", "1/1/2023"),
///    ("=DATE(2023,12,31)", "12/31/2023"),
///    ("=DATE(1900,1,1)", "1/1/1900"),
///    ("=DATE(1900,3,1)", "3/1/1900"),
/// ]);
///
/// // Arithmetics
/// assert_formulas(&[
///     ("1", "1"),
///     ("=A1+1", "2"),
///     ("=A2+1", "3"),
///     ("=SUM(A1:A3)", "6"),
/// ]);
///
/// // Dependency
/// assert_formulas(&[
///     ("=YEAR(DATE(2023,6,15))", "2023"),
///     ("=MONTH(DATE(2023,6,15))", "6"),
///     ("=DAY(DATE(2023,6,15))", "15"),]);
///
/// // Array constant arithmetic
/// assert_formulas(&[
///     ("=SUM({1,2,3}+{3,4,5})", "18"),]);
/// ```
pub(crate) fn assert_formulas(cases: &[(&str, &str)]) {
    let cells: Vec<String> = (1..=cases.len()).map(|i| format!("A{i}")).collect();
    let pairs: Vec<(&str, &str)> = cells
        .iter()
        .zip(cases.iter())
        .map(|(cell, (formula, _))| (cell.as_str(), *formula))
        .collect();
    let model = setup_model(&pairs);
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(model._get_text(&cells[i]), *expected, "formula: {formula}");
    }
}

impl<'a> Model<'a> {
    pub fn _parse_reference(&self, cell: &str) -> CellReferenceIndex {
        if cell.contains('!') {
            self.parse_reference(cell).unwrap()
        } else {
            let sheet_name = self.get_worksheets_properties()[0].name.clone();
            self.parse_reference(&format!("{sheet_name}!{cell}"))
                .unwrap()
        }
    }
    pub fn _set(&mut self, cell: &str, value: &str) {
        let cell_reference = self._parse_reference(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.set_user_input(cell_reference.sheet, row, column, value.to_string())
            .unwrap();
    }
    pub fn _has_formula(&self, cell: &str) -> bool {
        self._get_formula_opt(cell).is_some()
    }
    pub fn _get_formula(&self, cell: &str) -> String {
        self._get_formula_opt(cell).unwrap_or_default()
    }
    fn _get_formula_opt(&self, cell: &str) -> Option<String> {
        let cell_reference = self._parse_reference(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.get_cell_formula(cell_reference.sheet, row, column)
            .unwrap()
    }
    pub fn _get_text_at(&self, sheet: u32, row: i32, column: i32) -> String {
        self.get_formatted_cell_value(sheet, row, column).unwrap()
    }
    pub fn _get_text(&self, cell: &str) -> String {
        let CellReferenceIndex { sheet, row, column } = self._parse_reference(cell);
        self._get_text_at(sheet, row, column)
    }
    pub fn _get_cell(&self, cell: &str) -> &Cell {
        let cell_reference = self._parse_reference(cell);
        let worksheet = self.workbook.worksheet(cell_reference.sheet).unwrap();
        worksheet
            .cell(cell_reference.row, cell_reference.column)
            .unwrap()
    }
}
