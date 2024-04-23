#![allow(clippy::unwrap_used)]

use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

pub fn new_empty_model() -> Model {
    Model::new_empty("model", "en", "UTC").unwrap()
}

impl Model {
    fn _parse_reference(&self, cell: &str) -> CellReferenceIndex {
        if cell.contains('!') {
            self.parse_reference(cell).unwrap()
        } else {
            self.parse_reference(&format!("Sheet1!{}", cell)).unwrap()
        }
    }
    pub fn _set(&mut self, cell: &str, value: &str) {
        let cell_reference = self._parse_reference(cell);
        let column = cell_reference.column;
        let row = cell_reference.row;
        self.set_user_input(cell_reference.sheet, row, column, value.to_string());
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
        self.cell_formula(cell_reference.sheet, row, column)
            .unwrap()
    }
    pub fn _get_text_at(&self, sheet: u32, row: i32, column: i32) -> String {
        self.formatted_cell_value(sheet, row, column).unwrap()
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

pub fn assert_complex_eq(a: &str, b: &str, epsilon: Option<f64>) {
    let epsilon = epsilon.unwrap_or(1e-14);

    let re_a = a.split('+').next().unwrap().parse::<f64>().unwrap();
    let im_a = a
        .split('+')
        .last()
        .unwrap()
        .trim_end_matches('i')
        .parse::<f64>()
        .unwrap();

    let re_b = b.split('+').next().unwrap().parse::<f64>().unwrap();
    let im_b = b
        .split('+')
        .last()
        .unwrap()
        .trim_end_matches('i')
        .parse::<f64>()
        .unwrap();

    let diff_re = (re_a - re_b).abs();
    let diff_im = (im_a - im_b).abs();

    assert!(diff_re < epsilon && diff_im < epsilon, "{} != {}", a, b);
}
