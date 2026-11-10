//! Operations on tables at the model level: names, renames, the cells a
//! table operation shifts, and the formulas "Convert to Range" rewrites.
//! None of them is undoable here; the user model (`user_model/tables.rs`)
//! wraps them with their undo history.

use std::collections::HashMap;

use serde::Serialize;

use crate::actions::CellsMoved;
use crate::expressions::lexer::LexerMode;
use crate::expressions::parser::stringify::{to_localized_string, to_rc_format, DisplaceData};
use crate::expressions::parser::Node;
use crate::expressions::token::TableReference;
use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
use crate::expressions::utils::{is_valid_identifier, number_to_column};
use crate::tables::{
    find_table, for_each_node_mut, has_table_reference, table_area, table_at, TableArea,
};
use crate::types::Table;
use crate::Model;

/// The table style Excel's "Format as Table" button (and Ctrl+T) applies.
pub(crate) const DEFAULT_TABLE_STYLE: &str = "TableStyleMedium2";

/// The functions of a total row cell and their SUBTOTAL codes (the 100s skip
/// hidden rows, which is what Excel writes).
pub(crate) fn totals_function_code(function: &str) -> Option<i32> {
    Some(match function {
        "average" => 101,
        "countNums" => 102,
        "count" => 103,
        "max" => 104,
        "min" => 105,
        "stdDev" => 107,
        "sum" => 109,
        "var" => 110,
        _ => return None,
    })
}

/// `wanted`, or `wanted2`, `wanted3`... the first that no other column has
/// (Excel's rule for duplicate headers). Case-insensitive.
pub(crate) fn unique_column_name(others: &[String], wanted: &str) -> String {
    let taken = |n: &str| others.iter().any(|o| o.to_lowercase() == n.to_lowercase());
    if !taken(wanted) {
        return wanted.to_string();
    }
    let mut i = 2;
    loop {
        let candidate = format!("{wanted}{i}");
        if !taken(&candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// `Column1`, `Column2`...: the first that no other column has.
pub(crate) fn default_column_name(others: &[String]) -> String {
    let mut i = 1;
    loop {
        let candidate = format!("Column{i}");
        if !others
            .iter()
            .any(|o| o.to_lowercase() == candidate.to_lowercase())
        {
            return candidate;
        }
        i += 1;
    }
}

/// "A1:C5"
pub(crate) fn area_reference(area: &TableArea) -> String {
    format!(
        "{}{}:{}{}",
        number_to_column(area.first_column).unwrap_or_default(),
        area.first_row,
        number_to_column(area.last_column).unwrap_or_default(),
        area.last_row
    )
}

fn rename_table_in_node(node: &mut Node, old_name: &str, new_name: &str) {
    let old = old_name.to_lowercase();
    for_each_node_mut(node, &mut |n| match n {
        Node::TableReferenceKind { table_name, .. } | Node::TableNameKind(table_name)
            if table_name.to_lowercase() == old =>
        {
            *table_name = new_name.to_string();
        }
        _ => {}
    });
}

fn rename_column_in_node(node: &mut Node, table: &str, old_name: &str, new_name: &str) {
    let table = table.to_lowercase();
    let old = old_name.to_lowercase();
    let rename = |name: &mut String| {
        if name.to_lowercase() == old {
            *name = new_name.to_string();
        }
    };
    for_each_node_mut(node, &mut |n| {
        if let Node::TableReferenceKind {
            table_name,
            table_reference: Some(reference),
            ..
        } = n
        {
            if table_name.to_lowercase() != table {
                return;
            }
            match reference {
                TableReference::ColumnReference(c) => rename(c),
                TableReference::RangeReference((l, r)) => {
                    rename(l);
                    rename(r);
                }
            }
        }
    });
}

/// A table, as the bindings describe it.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TableInfo {
    /// The table's name
    pub name: String,
    /// Index of its sheet
    pub sheet: u32,
    /// First row (the header row, if it has one)
    pub row: i32,
    /// First column
    pub column: i32,
    /// Number of columns
    pub width: i32,
    /// Number of rows, header and total rows included
    pub height: i32,
    /// Whether the first row is a header row
    pub header_row: bool,
    /// Whether the last row is a total row
    pub total_row: bool,
    /// The columns, left to right
    pub columns: Vec<TableColumnInfo>,
    /// The table style ("TableStyleMedium2")
    pub style: Option<String>,
    /// Banded rows
    pub banded_rows: bool,
    /// Banded columns
    pub banded_columns: bool,
    /// The first column stands out
    pub first_column: bool,
    /// The last column stands out
    pub last_column: bool,
    /// The header row has filter buttons
    pub filter_button: bool,
}

/// A table column, as the bindings describe it.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TableColumnInfo {
    /// Its header
    pub name: String,
    /// The total row's function under it ("sum", "average"...), if any
    pub total_function: Option<String>,
    /// The total row's label under it ("Total"), if any
    pub total_label: Option<String>,
}

impl Model<'_> {
    /// The tables changed: let the parser know (formulas are not reparsed).
    pub(crate) fn tables_changed(&mut self) {
        self.parser.set_tables(self.workbook.tables.clone());
    }

    /// Installs a whole set of tables (undo and redo) and reparses every formula.
    pub(crate) fn set_tables(&mut self, tables: HashMap<String, Table>) {
        self.workbook.tables = tables;
        self.reset_parsed_structures();
    }

    /// Every table, or the tables on one sheet, top to bottom.
    pub fn get_tables(&self, sheet: Option<u32>) -> Result<Vec<TableInfo>, String> {
        let mut out = Vec::new();
        for table in self.workbook.tables.values() {
            let sheet_index = match self.get_sheet_index_by_name(&table.sheet_name) {
                Some(s) => s,
                None => continue,
            };
            if sheet.is_some_and(|s| s != sheet_index) {
                continue;
            }
            let area = table_area(table).ok_or("Invalid table range")?;
            out.push(TableInfo {
                name: table.name.clone(),
                sheet: sheet_index,
                row: area.first_row,
                column: area.first_column,
                width: area.last_column - area.first_column + 1,
                height: area.last_row - area.first_row + 1,
                header_row: table.header_row_count > 0,
                total_row: table.totals_row_count > 0,
                columns: table
                    .columns
                    .iter()
                    .map(|c| TableColumnInfo {
                        name: c.name.clone(),
                        total_function: c.totals_row_function.clone(),
                        total_label: c.totals_row_label.clone(),
                    })
                    .collect(),
                style: table.style_info.name.clone(),
                banded_rows: table.style_info.show_row_stripes,
                banded_columns: table.style_info.show_column_stripes,
                first_column: table.style_info.show_first_column,
                last_column: table.style_info.show_last_column,
                filter_button: table.has_filters,
            });
        }
        out.sort_by_key(|t| (t.sheet, t.row, t.column));
        Ok(out)
    }

    /// The name of the table that covers a cell, if any.
    pub fn get_table_at(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<String>, String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        Ok(table_at(&self.workbook.tables, &sheet_name, row, column).map(|t| t.name.clone()))
    }

    /// Checks a table name against Excel's rules: a valid name (letters,
    /// digits, '_' and '.', no spaces, not a cell reference, not "R" or "C")
    /// that no other table or defined name has. `current` is the table being
    /// renamed, if any.
    pub fn check_table_name(&self, name: &str, current: Option<&str>) -> Result<(), String> {
        if name.chars().count() > 255 || !is_valid_identifier(name) {
            return Err(format!("'{name}' isn't a valid table name"));
        }
        let lower = name.to_lowercase();
        let is_current = current.is_some_and(|c| c.to_lowercase() == lower);
        if !is_current && find_table(&self.workbook.tables, name).is_some() {
            return Err(format!("A table named '{name}' already exists"));
        }
        if self
            .workbook
            .defined_names
            .iter()
            .any(|d| d.name.to_lowercase() == lower)
        {
            return Err(format!("'{name}' is already a defined name"));
        }
        Ok(())
    }

    /// `Table1`, `Table2`...: the first name nothing else has.
    pub(crate) fn next_table_name(&self) -> String {
        let mut i = 1;
        loop {
            let name = format!("Table{i}");
            if self.check_table_name(&name, None).is_ok() {
                return name;
            }
            i += 1;
        }
    }

    /// Rewrites every formula of the workbook with `change`. They are stored
    /// in R1C1 and in English, and shared by the cells that hold them, so one
    /// rewrite covers every cell.
    fn rewrite_all_formulas(&mut self, change: &dyn Fn(&mut Node)) {
        self.parser.set_lexer_mode(LexerMode::R1C1);
        for worksheet in &mut self.workbook.worksheets {
            let cell_reference = CellReferenceRC {
                sheet: worksheet.get_name(),
                row: 1,
                column: 1,
            };
            let mut formulas = Vec::new();
            for formula in &worksheet.shared_formulas {
                let mut node = self.parser.parse(formula, &cell_reference);
                if matches!(node, Node::ParseErrorKind { .. }) {
                    formulas.push(formula.clone());
                } else {
                    change(&mut node);
                    formulas.push(to_rc_format(&node));
                }
            }
            worksheet.shared_formulas = formulas;
        }
        self.parser.set_lexer_mode(LexerMode::A1);
    }

    /// Renames a table and every reference to it.
    pub fn rename_table(&mut self, name: &str, new_name: &str) -> Result<(), String> {
        let mut table = find_table(&self.workbook.tables, name)
            .ok_or(format!("No table named '{name}'"))?
            .clone();
        self.check_table_name(new_name, Some(&table.name))?;
        let old_name = table.name.clone();
        self.rewrite_all_formulas(&|node| rename_table_in_node(node, &old_name, new_name));
        self.workbook.tables.remove(&old_name);
        table.name = new_name.to_string();
        table.display_name = new_name.to_string();
        self.workbook.tables.insert(new_name.to_string(), table);
        self.reset_parsed_structures();
        Ok(())
    }

    /// Renames a column of a table and every reference to it.
    pub fn rename_table_column(
        &mut self,
        table_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), String> {
        let table = find_table(&self.workbook.tables, table_name)
            .ok_or(format!("No table named '{table_name}'"))?;
        let key = table.name.clone();
        let index = table
            .columns
            .iter()
            .position(|c| c.name.to_lowercase() == old_name.to_lowercase())
            .ok_or(format!("No column '{old_name}' in table '{key}'"))?;
        if new_name.is_empty()
            || table
                .columns
                .iter()
                .enumerate()
                .any(|(i, c)| i != index && c.name.to_lowercase() == new_name.to_lowercase())
        {
            return Err(format!("Table '{key}' can't have a column '{new_name}'"));
        }
        let old = table.columns[index].name.clone();
        self.rewrite_all_formulas(&|node| rename_column_in_node(node, &key, &old, new_name));
        if let Some(t) = self.workbook.tables.get_mut(&key) {
            t.columns[index].name = new_name.to_string();
        }
        self.reset_parsed_structures();
        Ok(())
    }

    /// Shifts the cells of columns `first_column..=last_column` from `row`
    /// down by one, like Excel's Insert > Shift cells down, and moves the
    /// references to them.
    pub(crate) fn insert_cells_down(
        &mut self,
        sheet: u32,
        row: i32,
        first_column: i32,
        last_column: i32,
    ) -> Result<(), String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        if worksheet.dimension().max_row >= crate::constants::LAST_ROW {
            return Err("No room to shift the cells down".to_string());
        }
        let mut rows = worksheet.sheet_data.rows();
        rows.reverse();
        for column in first_column..=last_column {
            for r in rows.iter().copied().filter(|r| *r >= row) {
                if self.workbook.worksheet(sheet)?.cell(r, column).is_some() {
                    self.move_cell(sheet, r, column, r + 1, column)?;
                }
            }
            self.displace_formulas(
                &DisplaceData::CellVertical {
                    sheet,
                    row,
                    column,
                    delta: 1,
                },
                CellsMoved::None,
            )?;
        }
        Ok(())
    }

    /// Undoes `insert_cells_down`: removes row `row` of those columns and
    /// shifts the cells below up by one.
    pub(crate) fn delete_cells_up(
        &mut self,
        sheet: u32,
        row: i32,
        first_column: i32,
        last_column: i32,
    ) -> Result<(), String> {
        let rows = self.workbook.worksheet(sheet)?.sheet_data.rows();
        for column in first_column..=last_column {
            self.workbook
                .worksheet_mut(sheet)?
                .remove_cell(row, column)?;
            for r in rows.iter().copied().filter(|r| *r > row) {
                if self.workbook.worksheet(sheet)?.cell(r, column).is_some() {
                    self.move_cell(sheet, r, column, r - 1, column)?;
                }
            }
            self.displace_formulas(
                &DisplaceData::CellVertical {
                    sheet,
                    row,
                    column,
                    delta: -1,
                },
                CellsMoved::None,
            )?;
        }
        Ok(())
    }

    /// The shared formula every one of `rows` in `column` holds (a calculated
    /// column), if they all hold the same one.
    pub(crate) fn common_formula(
        &self,
        sheet: u32,
        column: i32,
        rows: std::ops::RangeInclusive<i32>,
    ) -> Result<Option<i32>, String> {
        let worksheet = self.workbook.worksheet(sheet)?;
        let mut common = None;
        for r in rows {
            let index = match worksheet.cell(r, column).and_then(|c| c.get_formula()) {
                Some(i) => i,
                None => return Ok(None),
            };
            if common.is_some_and(|c| c != index) {
                return Ok(None);
            }
            common = Some(index);
        }
        Ok(common)
    }

    /// What to type in (`sheet`, `row`, `column`) to get shared formula
    /// `index` there: what filling the formula into that cell gives.
    pub(crate) fn formula_text_at(
        &self,
        sheet: u32,
        index: i32,
        row: i32,
        column: i32,
    ) -> Result<String, String> {
        let node = &self
            .parsed_formulas
            .get(sheet as usize)
            .and_then(|f| f.get(index as usize))
            .ok_or("Formula not found")?
            .0;
        let context = CellReferenceRC {
            sheet: self.workbook.worksheet(sheet)?.get_name(),
            row,
            column,
        };
        Ok(format!(
            "={}",
            to_localized_string(node, &context, self.locale, self.language)
        ))
    }

    /// For the formula bar: references to the table the cell is in lose the
    /// table name (`[@Price]`), as Excel shows them. None if nothing changes.
    pub(crate) fn with_own_table_names_omitted(
        &self,
        node: &Node,
        sheet_name: &str,
        row: i32,
        column: i32,
    ) -> Option<Node> {
        if !has_table_reference(node) {
            return None;
        }
        let own = table_at(&self.workbook.tables, sheet_name, row, column)?
            .name
            .to_lowercase();
        let mut shown = node.clone();
        for_each_node_mut(&mut shown, &mut |n| {
            if let Node::TableReferenceKind { table_name, .. } = n {
                if table_name.to_lowercase() == own {
                    table_name.clear();
                }
            }
        });
        Some(shown)
    }

    fn table_names_on_sheet(&self, sheet: u32) -> Result<Vec<String>, String> {
        let sheet_name = self.workbook.worksheet(sheet)?.get_name();
        let mut names: Vec<String> = self
            .workbook
            .tables
            .values()
            .filter(|t| t.sheet_name == sheet_name)
            .map(|t| t.name.clone())
            .collect();
        names.sort();
        Ok(names)
    }

    /// Moves, grows or shrinks the tables of `sheet` after `delta` rows were
    /// inserted (`delta > 0`) or deleted at `row`, as Excel does: rows
    /// inserted inside a table are data rows and get the formulas of its
    /// calculated columns; a deleted header or total row goes from the table;
    /// a table whose rows are all deleted goes.
    pub(crate) fn displace_tables_for_rows(
        &mut self,
        sheet: u32,
        row: i32,
        delta: i32,
    ) -> Result<(), String> {
        let names = self.table_names_on_sheet(sheet)?;
        if names.is_empty() {
            return Ok(());
        }
        let mut fills = Vec::new();
        for name in names {
            let Some(mut table) = self.workbook.tables.get(&name).cloned() else {
                continue;
            };
            let Some(a) = table_area(&table) else {
                continue;
            };
            let mut b = a;
            if delta > 0 {
                if row <= a.first_row {
                    b.first_row += delta;
                    b.last_row += delta;
                } else if row <= a.last_row {
                    b.last_row += delta;
                    fills.push((name.clone(), row, delta));
                } else {
                    continue;
                }
            } else {
                let count = -delta;
                let (d_first, d_last) = (row, row + count - 1);
                if d_last < a.first_row {
                    b.first_row -= count;
                    b.last_row -= count;
                } else if d_first > a.last_row {
                    continue;
                } else if d_first <= a.first_row && d_last >= a.last_row {
                    self.workbook.tables.remove(&name);
                    continue;
                } else {
                    if table.header_row_count > 0 && d_first <= a.first_row {
                        table.header_row_count = 0;
                    }
                    if table.totals_row_count > 0 && d_last >= a.last_row {
                        table.totals_row_count = 0;
                    }
                    b.first_row = a.first_row.min(d_first);
                    b.last_row = if a.last_row > d_last {
                        a.last_row - count
                    } else {
                        d_first - 1
                    };
                    // A table keeps at least one data row
                    let least =
                        b.first_row + (table.header_row_count + table.totals_row_count) as i32;
                    b.last_row = b.last_row.max(least);
                }
            }
            table.reference = area_reference(&b);
            self.workbook.tables.insert(name, table);
        }
        self.tables_changed();
        // New data rows take the calculated columns' formulas
        for (name, row, count) in fills {
            let Some(table) = self.workbook.tables.get(&name).cloned() else {
                continue;
            };
            let Some(a) = table_area(&table) else {
                continue;
            };
            let (d_first, d_last) = crate::tables::data_rows(&table, &a);
            let inserted = row..row + count;
            let old_rows: Vec<i32> = (d_first..=d_last)
                .filter(|r| !inserted.contains(r))
                .collect();
            for column in a.first_column..=a.last_column {
                let worksheet = self.workbook.worksheet(sheet)?;
                let formulas: Vec<Option<i32>> = old_rows
                    .iter()
                    .map(|r| worksheet.cell(*r, column).and_then(|c| c.get_formula()))
                    .collect();
                let Some(Some(index)) = formulas.first().copied() else {
                    continue;
                };
                if formulas.iter().any(|f| *f != Some(index)) {
                    continue;
                }
                for r in inserted.clone().filter(|r| (d_first..=d_last).contains(r)) {
                    let text = self.formula_text_at(sheet, index, r, column)?;
                    self.set_user_input(sheet, r, column, text)?;
                }
            }
        }
        Ok(())
    }

    /// Moves, grows or shrinks the tables of `sheet` after `delta` columns
    /// were inserted (`delta > 0`) or deleted at `column`. Columns inserted
    /// inside a table are new table columns, `Column1`...; deleted ones go
    /// from the table, and references to them become #REF!.
    pub(crate) fn displace_tables_for_columns(
        &mut self,
        sheet: u32,
        column: i32,
        delta: i32,
    ) -> Result<(), String> {
        let names = self.table_names_on_sheet(sheet)?;
        if names.is_empty() {
            return Ok(());
        }
        let mut headers = Vec::new();
        for name in names {
            let Some(mut table) = self.workbook.tables.get(&name).cloned() else {
                continue;
            };
            let Some(a) = table_area(&table) else {
                continue;
            };
            let mut b = a;
            if delta > 0 {
                if column <= a.first_column {
                    b.first_column += delta;
                    b.last_column += delta;
                } else if column <= a.last_column {
                    b.last_column += delta;
                    let at = (column - a.first_column) as usize;
                    let last_id = table.columns.iter().map(|c| c.id).max().unwrap_or(0);
                    for i in 0..delta {
                        let names: Vec<String> =
                            table.columns.iter().map(|c| c.name.clone()).collect();
                        let new_name = default_column_name(&names);
                        let id = last_id + 1 + i as u32;
                        table.columns.insert(
                            at + i as usize,
                            crate::types::TableColumn {
                                id,
                                name: new_name.clone(),
                                ..Default::default()
                            },
                        );
                        if table.header_row_count > 0 {
                            headers.push((a.first_row, column + i, new_name));
                        }
                    }
                } else {
                    continue;
                }
            } else {
                let count = -delta;
                let (d_first, d_last) = (column, column + count - 1);
                if d_last < a.first_column {
                    b.first_column -= count;
                    b.last_column -= count;
                } else if d_first > a.last_column {
                    continue;
                } else if d_first <= a.first_column && d_last >= a.last_column {
                    self.workbook.tables.remove(&name);
                    continue;
                } else {
                    let mut kept = Vec::new();
                    for (i, c) in table.columns.iter().enumerate() {
                        let absolute = a.first_column + i as i32;
                        if absolute < d_first || absolute > d_last {
                            kept.push(c.clone());
                        }
                    }
                    table.columns = kept;
                    b.first_column = a.first_column.min(d_first);
                    b.last_column = b.first_column + table.columns.len() as i32 - 1;
                }
            }
            table.reference = area_reference(&b);
            self.workbook.tables.insert(name, table);
        }
        self.tables_changed();
        for (row, column, name) in headers {
            self.set_user_input(sheet, row, column, name)?;
        }
        Ok(())
    }

    /// Every formula that refers to table `name`, with the text that refers
    /// to the same cells without the table: what "Convert to Range" writes.
    /// `[@Price]` becomes `$C2`, its row following the cell; everything else
    /// becomes an absolute reference.
    pub(crate) fn formulas_without_table(
        &self,
        name: &str,
    ) -> Result<Vec<(u32, i32, i32, String)>, String> {
        let table =
            find_table(&self.workbook.tables, name).ok_or(format!("No table named '{name}'"))?;
        let lower = table.name.to_lowercase();
        self.get_sheet_index_by_name(&table.sheet_name)
            .ok_or("The table's sheet is gone")?;
        let mut out = Vec::new();
        for (sheet_index, worksheet) in self.workbook.worksheets.iter().enumerate() {
            let sheet = sheet_index as u32;
            for (row, column, cell) in worksheet.sheet_data.cells() {
                let index = match cell.get_formula() {
                    Some(i) => i,
                    None => continue,
                };
                let node = &self.parsed_formulas[sheet_index][index as usize].0;
                let cell = CellReferenceIndex { sheet, row, column };
                let rewritten = self
                    .table_references_as_cells_where(node, cell, &|t| t.to_lowercase() == lower);
                if let Some(node) = rewritten {
                    let context = CellReferenceRC {
                        sheet: worksheet.get_name(),
                        row,
                        column,
                    };
                    let text = to_localized_string(&node, &context, self.locale, self.language);
                    out.push((sheet, row, column, format!("={text}")));
                }
            }
        }
        Ok(out)
    }
}
