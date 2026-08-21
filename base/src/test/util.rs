#![allow(clippy::unwrap_used)]

use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::Cell;

/// The model the corpus runs against. Under `collab-test` it is the collaborative one, so every
/// test below exercises the patch path instead of the ordinal writers.
#[cfg(not(feature = "collab-test"))]
pub type TestModel<'a> = Model<'a>;
#[cfg(feature = "collab-test")]
pub type TestModel<'a> = crate::collab::model::CollabModel<'a>;

#[cfg(not(feature = "collab-test"))]
pub fn new_empty_model() -> TestModel<'static> {
    Model::new_empty("model", "en", "UTC", "en").unwrap()
}

/// A collaborative replica starts with no sheets at all — every sheet arrives as a patch — while
/// the corpus assumes `Sheet1` is already there.
#[cfg(feature = "collab-test")]
pub fn new_empty_model() -> TestModel<'static> {
    let mut model = crate::collab::model::CollabModel::new(1);
    model.new_sheet();
    model
}

/// The corpus' shorthands. Written once and given to both models: `collab-test` only changes which
/// one `new_empty_model` hands out, and plenty of tests build an ordinal `Model` themselves.
macro_rules! test_helpers {
    ($model:ty) => {
        impl $model {
            pub fn _parse_reference(&self, cell: &str) -> CellReferenceIndex {
                if cell.contains('!') {
                    self.parse_reference(cell).unwrap()
                } else {
                    let sheet_name = self.workbook.worksheets[0].get_name();
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
    };
}

test_helpers!(Model<'_>);
#[cfg(feature = "collab-test")]
test_helpers!(crate::collab::model::CollabModel<'_>);

#[cfg(not(feature = "collab-test"))]
impl TestModel<'_> {
    pub fn _cell_clear_contents(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<(), String> {
        let area = crate::expressions::types::Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_contents(&area)
    }

    pub fn _cell_clear_all(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        let area = crate::expressions::types::Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.range_clear_all(&area)
    }
}

#[cfg(feature = "collab-test")]
impl TestModel<'_> {
    pub fn _cell_clear_contents(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<(), String> {
        self.cell_clear_contents(sheet, row, column)
    }

    pub fn _cell_clear_all(&mut self, sheet: u32, row: i32, column: i32) -> Result<(), String> {
        self.cell_clear_all(sheet, row, column)
    }
}
