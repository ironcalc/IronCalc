use std::collections::HashMap;

use crate::cf_types::CfRule;
use crate::collab::fractional_index::{FractionalIndex, KeyAlias};
use crate::collab::model::{StableCellAddress, StableRange};
use crate::expressions::parser::DefinedNameS;
use crate::expressions::token::Error;
use crate::types::{
    Cell, Color, DefinedName, Metadata, SheetState, Styles, Theme, Workbook, WorkbookSettings,
    WorkbookView, Worksheet, WorksheetView,
};
use crate::worksheet::{NavigationDirection, WorksheetDimension};

pub struct CollaborativeWorkbook {
    pub shared_strings: Vec<String>,
    pub defined_names: Vec<DefinedName>,
    pub worksheets: Vec<CollaborativeWorksheet>,
    pub styles: Styles,
    pub name: String,
    pub settings: WorkbookSettings,
    pub metadata: Metadata,
    pub tables: HashMap<String, CollaborativeTable>,
    pub views: HashMap<u32, WorkbookView>,
    pub theme: Theme,
    pub sheet_order: FractionalIndex,
}

impl CollaborativeWorkbook {
    pub fn get_worksheet_names(&self) -> Vec<String> {
        todo!()
    }
    pub fn get_worksheet_ids(&self) -> Vec<u32> {
        todo!()
    }
    pub fn worksheet(&self, worksheet_index: u32) -> Result<&CollaborativeWorksheet, String> {
        todo!()
    }
    pub fn worksheet_mut(
        &mut self,
        worksheet_index: u32,
    ) -> Result<&mut CollaborativeWorksheet, String> {
        todo!()
    }
    pub fn get_defined_names_with_scope(&self) -> Vec<DefinedNameS> {
        todo!()
    }

    pub fn to_workbook(&self) -> Workbook {
        todo!()
    }

    pub fn from_workbook(workbook: &Workbook, session: [u8; 4]) -> Self {
        todo!()
    }
}

/// Collaborative counterpart of [`Worksheet`].
pub struct CollaborativeWorksheet {
    pub name: String,
    /// A `KeyAlias` obtained from `CollaborativeWorkbook::sheet_order`. We use [FractionalKey]
    /// for worksheets for concurrent inserts.
    pub sheet_id: KeyAlias,
    pub state: SheetState,
    pub color: Color,
    pub rows_index: FractionalIndex,
    pub cols_index: FractionalIndex,
    pub sheet_data: HashMap<KeyAlias, HashMap<KeyAlias, Cell>>,

    pub rows: HashMap<KeyAlias, RowProperties>,
    pub cols: HashMap<KeyAlias, ColProperties>,

    pub shared_formulas: Vec<String>,
    pub merge_cells: Vec<StableRange>,
    pub comments: Vec<CollaborativeComment>,
    pub frozen_rows: i32,
    pub frozen_columns: i32,
    pub views: HashMap<u32, WorksheetView>,
    pub show_grid_lines: bool,
    pub conditional_formatting: Vec<CollaborativeConditionalFormatting>,
}

impl CollaborativeWorksheet {
    fn row_alias(&self, row: i32) -> Option<KeyAlias> {
        todo!()
    }
    fn col_alias(&self, column: i32) -> Option<KeyAlias> {
        todo!()
    }
    /// Alias -> sorted position. Requires a reverse lookup on `FractionalIndex`
    /// (see the note at the bottom — it currently only offers key->alias).
    fn row_position(&self, alias: KeyAlias) -> Option<i32> {
        todo!()
    }
    fn col_position(&self, alias: KeyAlias) -> Option<i32> {
        todo!()
    }

    pub fn stable_address(&self, row: i32, column: i32) -> Option<StableCellAddress> {
        todo!()
    }
    /// Ensure a stable key exists for `row`/`column`.
    fn resolve_or_create(&mut self, row: i32, column: i32) -> (KeyAlias, KeyAlias) {
        todo!()
    }

    pub fn insert_rows(&mut self, at: i32, count: usize) -> Result<(), String> {
        todo!()
    }
    pub fn delete_rows(&mut self, at: i32, count: usize) -> Result<(), String> {
        todo!()
    }
    pub fn insert_columns(&mut self, at: i32, count: usize) -> Result<(), String> {
        todo!()
    }
    pub fn delete_columns(&mut self, at: i32, count: usize) -> Result<(), String> {
        todo!()
    }

    pub fn get_name(&self) -> String {
        todo!()
    }
    pub fn get_sheet_id(&self) -> u32 {
        todo!()
    }
    pub fn set_name(&mut self, name: &str) {
        todo!()
    }

    pub fn cell(&self, row: i32, column: i32) -> Option<&Cell> {
        todo!()
    }
    pub fn is_empty_cell(&self, row: i32, column: i32) -> Result<bool, String> {
        todo!()
    }
    pub fn dimension(&self) -> WorksheetDimension {
        todo!()
    }
    pub fn navigate_to_edge_in_direction(
        &self,
        row: i32,
        column: i32,
        direction: NavigationDirection,
    ) -> Result<(i32, i32), String> {
        todo!()
    }

    pub fn set_cell_with_formula(
        &mut self,
        row: i32,
        column: i32,
        index: i32,
        style: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_dynamic_formula(
        &mut self,
        row: i32,
        column: i32,
        index: i32,
        style: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_array_formula(
        &mut self,
        row: i32,
        column: i32,
        index: i32,
        style: i32,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_number(
        &mut self,
        row: i32,
        column: i32,
        value: f64,
        style: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_string(
        &mut self,
        row: i32,
        column: i32,
        index: i32,
        style: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_boolean(
        &mut self,
        row: i32,
        column: i32,
        value: bool,
        style: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_with_error(
        &mut self,
        row: i32,
        column: i32,
        error: Error,
        style: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn cell_clear_contents(&mut self, row: i32, column: i32) -> Result<(), String> {
        todo!()
    }
    pub fn cell_clear_contents_with_style(
        &mut self,
        row: i32,
        column: i32,
        style_index: i32,
    ) -> Result<(), String> {
        todo!()
    }

    pub fn get_style(&self, row: i32, column: i32) -> i32 {
        todo!()
    }
    pub fn set_style(&mut self, style_index: i32) -> Result<(), String> {
        todo!()
    }
    pub fn set_cell_style(
        &mut self,
        row: i32,
        column: i32,
        style_index: i32,
    ) -> Result<(), String> {
        todo!()
    }
    pub fn set_column_style(&mut self, column: i32, style_index: i32) -> Result<(), String> {
        todo!()
    }
    pub fn set_row_style(&mut self, row: i32, style_index: i32) -> Result<(), String> {
        todo!()
    }
    pub fn delete_row_style(&mut self, row: i32) -> Result<(), String> {
        todo!()
    }
    pub fn delete_column_style(&mut self, column: i32) -> Result<(), String> {
        todo!()
    }
    pub fn get_column_style(&self, column: i32) -> Result<Option<i32>, String> {
        todo!()
    }

    pub fn set_frozen_rows(&mut self, frozen_rows: i32) -> Result<(), String> {
        todo!()
    }
    pub fn set_frozen_columns(&mut self, frozen_columns: i32) -> Result<(), String> {
        todo!()
    }
    pub fn set_row_hidden(&mut self, row: i32, hidden: bool) -> Result<(), String> {
        todo!()
    }
    pub fn set_row_height(&mut self, row: i32, height: f64) -> Result<(), String> {
        todo!()
    }
    pub fn set_column_width(&mut self, column: i32, width: f64) -> Result<(), String> {
        todo!()
    }
    pub fn set_column_hidden(&mut self, column: i32, hidden: bool) -> Result<(), String> {
        todo!()
    }
    pub fn get_column_width(&self, column: i32) -> Result<f64, String> {
        todo!()
    }
    pub fn get_actual_column_width(&self, column: i32) -> Result<f64, String> {
        todo!()
    }
    pub fn is_column_hidden(&self, column: i32) -> Result<bool, String> {
        todo!()
    }
    pub fn is_row_hidden(&self, row: i32) -> Result<bool, String> {
        todo!()
    }
    pub fn row_height(&self, row: i32) -> Result<f64, String> {
        todo!()
    }

    pub fn to_worksheet(&self) -> Worksheet {
        todo!()
    }
    pub fn from_worksheet(worksheet: &Worksheet, session: [u8; 4]) -> Self {
        todo!()
    }
}

pub struct RowProperties {
    pub height: f64,
    pub custom_format: bool,
    pub custom_height: bool,
    pub s: i32,
    pub hidden: bool,
}

pub struct ColProperties {
    pub width: f64,
    pub custom_width: bool,
    pub hidden: bool,
    pub style: Option<i32>,
}

pub struct CollaborativeComment {
    pub text: String,
    pub author_name: String,
    pub author_id: Option<String>,
    pub cell_ref: StableCellAddress,
}

pub struct CollaborativeConditionalFormatting {
    pub range: Vec<StableRange>,
    pub cf_rule: CfRule,
    pub priority: u32,
}

pub struct CollaborativeTable {
    pub name: String,
    pub display_name: String,
    pub sheet_id: KeyAlias,
    pub reference: StableRange,
    // --- carried over from `Table` verbatim (non-positional) ---
    // pub totals_row_count: u32,
    // pub header_row_count: u32,
    // pub header_row_dxf_id: Option<u32>,
    // pub data_dxf_id: Option<u32>,
    // pub totals_row_dxf_id: Option<u32>,
    // pub columns: Vec<TableColumn>,
    // pub style_info: TableStyleInfo,
    // pub has_filters: bool,
}
