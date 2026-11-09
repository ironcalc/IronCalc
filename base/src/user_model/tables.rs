//! Tables in the user model: every action is one undo step.
//!
//! What Excel does and this follows:
//! * Format as Table (Ctrl+T) turns a range into a table named `Table1`,
//!   `Table2`... Its first row becomes the header row; without headers a row
//!   of `Column1`, `Column2`... is inserted above the data.
//! * Typing in the row just below a table, or the column just right of it,
//!   makes the table grow to take that cell in; a new row gets the formulas
//!   of the calculated columns, a new column gets a header.
//! * A formula typed in an empty table column (or one whose cells all hold
//!   the same formula) fills the whole column: a calculated column.
//! * Typing in a header renames the column, and every reference to it.
//! * The total row adds a row under the data with "Total" and a SUBTOTAL.
//! * Convert to Range turns the references to the table into plain ones.

use std::borrow::Cow;

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::TableReference;
use crate::expressions::types::Area;
use crate::table_ops::{
    area_reference, default_column_name, totals_function_code, unique_column_name,
    DEFAULT_TABLE_STYLE,
};
use crate::tables::{
    data_rows, find_table, structured_reference_to_string, table_area, table_at, TableArea,
};
use crate::types::{Cell, CellType, Table, TableColumn, TableStyleInfo};
use crate::{TableInfo, UserModel};

use super::history::Diff;

impl UserModel<'_> {
    /// Writes `value` into a cell as part of a table action, recording its undo.
    fn write_for_table(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
        diff_list: &mut Vec<Diff>,
    ) -> Result<(), String> {
        let old_value = self
            .model
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .map(Cow::into_owned)
            .filter(|c| !matches!(c, Cell::SpillCell { .. }));
        self.model
            .set_user_input(sheet, row, column, value.to_string())?;
        diff_list.push(Diff::SetCellValue {
            sheet,
            row,
            column,
            new_value: value.to_string(),
            old_value: Box::new(old_value),
        });
        Ok(())
    }

    /// Writes `text` so that it stays text ("2024" as a header is text in Excel).
    fn write_text_for_table(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        text: &str,
        diff_list: &mut Vec<Diff>,
    ) -> Result<(), String> {
        let is_text_already = self.model.get_cell_type(sheet, row, column)? == CellType::Text
            && self.model.get_cell_formula(sheet, row, column)?.is_none()
            && self.model.get_formatted_cell_value(sheet, row, column)? == text;
        if is_text_already {
            return Ok(());
        }
        let mut scratch = Vec::new();
        self.write_for_table(sheet, row, column, text, &mut scratch)?;
        if self.model.get_cell_type(sheet, row, column)? != CellType::Text
            || self.model.get_formatted_cell_value(sheet, row, column)? != text
        {
            // A number, a date or a boolean as typed: the apostrophe keeps it text
            let quoted = format!("'{text}");
            self.model
                .set_user_input(sheet, row, column, quoted.clone())?;
            if let Some(Diff::SetCellValue { new_value, .. }) = scratch.last_mut() {
                *new_value = quoted;
            }
        }
        diff_list.extend(scratch);
        Ok(())
    }

    fn table_for_change(&self, name: &str) -> Result<(Table, TableArea, u32), String> {
        let table = find_table(&self.model.workbook.tables, name)
            .ok_or(format!("No table named '{name}'"))?
            .clone();
        let area = table_area(&table).ok_or("Invalid table range")?;
        let sheet = self
            .model
            .get_sheet_index_by_name(&table.sheet_name)
            .ok_or("The table's sheet is gone")?;
        Ok((table, area, sheet))
    }

    /// Puts `table` in place of the table called `old_name` (or adds it) and
    /// returns the diff that undoes it.
    fn replace_table(&mut self, old_name: Option<&str>, table: Option<Table>) -> Diff {
        let old_value = self.model.workbook.tables.clone();
        if let Some(name) = old_name {
            self.model.workbook.tables.remove(name);
        }
        if let Some(table) = table {
            self.model.workbook.tables.insert(table.name.clone(), table);
        }
        self.model.tables_changed();
        Diff::SetTables {
            old_value,
            new_value: self.model.workbook.tables.clone(),
        }
    }

    /// Format as Table: turns `area` into a table and returns its name. With
    /// `has_headers` the first row of the area is the header row; without, a
    /// header row is inserted above the data (the cells below shift down).
    /// `name` defaults to the next free `TableN`.
    pub fn create_table(
        &mut self,
        area: &Area,
        has_headers: bool,
        name: Option<&str>,
    ) -> Result<String, String> {
        let sheet = area.sheet;
        let worksheet = self.model.workbook.worksheet(sheet)?;
        let sheet_name = worksheet.get_name();
        let first_row = area.row;
        let first_column = area.column;
        let last_column = area.column + area.width - 1;
        let mut last_row = area.row + area.height - 1;
        if area.width < 1
            || area.height < 1
            || first_row < 1
            || first_column < 1
            || last_row >= LAST_ROW
            || last_column > LAST_COLUMN
        {
            return Err("Invalid range for a table".to_string());
        }
        let overlaps = |a: &TableArea| {
            a.first_row <= last_row + 1
                && a.last_row >= first_row
                && a.first_column <= last_column
                && a.last_column >= first_column
        };
        if self
            .model
            .workbook
            .tables
            .values()
            .any(|t| t.sheet_name == sheet_name && table_area(t).is_some_and(|a| overlaps(&a)))
        {
            return Err("A table can't overlap another table".to_string());
        }
        if worksheet.merged_cells.iter().any(|m| {
            m.row <= last_row
                && m.last_row() >= first_row
                && m.column <= last_column
                && m.last_column() >= first_column
        }) {
            return Err("A table can't contain merged cells".to_string());
        }
        let name = match name {
            Some(n) => {
                self.model.check_table_name(n, None)?;
                n.to_string()
            }
            None => self.model.next_table_name(),
        };

        let mut diff_list = Vec::new();
        if has_headers {
            if area.height == 1 {
                // Only headers: the table gets one empty data row
                last_row += 1;
            }
        } else {
            self.model
                .insert_cells_down(sheet, first_row, first_column, last_column)?;
            diff_list.push(Diff::InsertCellsDown {
                sheet,
                row: first_row,
                first_column,
                last_column,
            });
            last_row += 1;
        }

        let mut names: Vec<String> = Vec::new();
        for column in first_column..=last_column {
            let text = if has_headers {
                self.model
                    .get_formatted_cell_value(sheet, first_row, column)?
            } else {
                String::new()
            };
            let name = if text.is_empty() {
                default_column_name(&names)
            } else {
                unique_column_name(&names, &text)
            };
            names.push(name);
        }
        let table = Table {
            name: name.clone(),
            display_name: name.clone(),
            sheet_name,
            reference: area_reference(&TableArea {
                first_row,
                first_column,
                last_row,
                last_column,
            }),
            totals_row_count: 0,
            header_row_count: 1,
            header_row_dxf_id: None,
            data_dxf_id: None,
            totals_row_dxf_id: None,
            columns: names
                .iter()
                .enumerate()
                .map(|(i, n)| TableColumn {
                    id: i as u32 + 1,
                    name: n.clone(),
                    ..Default::default()
                })
                .collect(),
            style_info: TableStyleInfo {
                name: Some(DEFAULT_TABLE_STYLE.to_string()),
                show_row_stripes: true,
                ..Default::default()
            },
            has_filters: true,
        };
        diff_list.push(self.replace_table(None, Some(table)));
        for (i, column) in (first_column..=last_column).enumerate() {
            self.write_text_for_table(sheet, first_row, column, &names[i], &mut diff_list)?;
        }
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(name)
    }

    /// Renames a table; formulas that refer to it follow.
    pub fn rename_table(&mut self, name: &str, new_name: &str) -> Result<(), String> {
        let (table, _, _) = self.table_for_change(name)?;
        if table.name == new_name {
            // Like every other action, a no-op is still one (empty) undo step
            self.push_diff_list(vec![]);
            return Ok(());
        }
        self.model.rename_table(&table.name, new_name)?;
        self.push_diff_list(vec![Diff::RenameTable {
            old_name: table.name,
            new_name: new_name.to_string(),
        }]);
        Ok(())
    }

    /// Convert to Range: the table goes, its cells stay, and references to it
    /// become plain references to the same cells.
    pub fn convert_table_to_range(&mut self, name: &str) -> Result<(), String> {
        let (table, _, _) = self.table_for_change(name)?;
        let rewrites = self.model.formulas_without_table(&table.name)?;
        let mut diff_list = vec![self.replace_table(Some(&table.name), None)];
        for (sheet, row, column, text) in rewrites {
            self.write_for_table(sheet, row, column, &text, &mut diff_list)?;
        }
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Shows or hides the total row. Showing it writes "Total" under the
    /// first column and a SUBTOTAL under the last (a sum of numbers, else a
    /// count), in the row below the table, which must be empty.
    pub fn set_table_total_row(&mut self, name: &str, show: bool) -> Result<(), String> {
        let (mut table, area, sheet) = self.table_for_change(name)?;
        if show == (table.totals_row_count > 0) {
            self.push_diff_list(vec![]);
            return Ok(());
        }
        let mut writes: Vec<(i32, i32, String, bool)> = Vec::new();
        let mut area = area;
        if show {
            let row = area.last_row + 1;
            if row > LAST_ROW {
                return Err("No room for a total row".to_string());
            }
            for column in area.first_column..=area.last_column {
                if !self.model.is_empty_cell(sheet, row, column)? {
                    return Err("The row below the table isn't empty".to_string());
                }
            }
            let sheet_name = self.model.workbook.worksheet(sheet)?.get_name();
            if (area.first_column..=area.last_column)
                .any(|c| table_at(&self.model.workbook.tables, &sheet_name, row, c).is_some())
            {
                return Err("Another table is below this one".to_string());
            }
            let (data_first, data_last) = data_rows(&table, &area);
            let last = table.columns.len() - 1;
            let has_numbers = (data_first..=data_last).any(|r| {
                self.model.get_cell_type(sheet, r, area.last_column) == Ok(CellType::Number)
                    && !self
                        .model
                        .is_empty_cell(sheet, r, area.last_column)
                        .unwrap_or(true)
            });
            let function = if has_numbers { "sum" } else { "count" };
            if last > 0 {
                table.columns[0].totals_row_label = Some("Total".to_string());
                writes.push((row, area.first_column, "Total".to_string(), true));
            }
            table.columns[last].totals_row_function = Some(function.to_string());
            writes.push((
                row,
                area.last_column,
                self.total_formula(&table.name, &table.columns[last].name, function),
                false,
            ));
            area.last_row = row;
            table.totals_row_count = 1;
        } else {
            for column in area.first_column..=area.last_column {
                if !self.model.is_empty_cell(sheet, area.last_row, column)? {
                    writes.push((area.last_row, column, String::new(), false));
                }
            }
            for column in &mut table.columns {
                column.totals_row_function = None;
                column.totals_row_label = None;
            }
            area.last_row -= 1;
            table.totals_row_count = 0;
        }
        table.reference = area_reference(&area);
        let key = table.name.clone();
        let mut diff_list = vec![self.replace_table(Some(&key), Some(table))];
        for (row, column, value, is_text) in writes {
            if is_text {
                self.write_text_for_table(sheet, row, column, &value, &mut diff_list)?;
            } else {
                self.write_for_table(sheet, row, column, &value, &mut diff_list)?;
            }
        }
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(())
    }

    fn total_formula(&self, table: &str, column: &str, function: &str) -> String {
        let code = totals_function_code(function).unwrap_or(109);
        let reference = structured_reference_to_string(
            table,
            &None,
            &Some(TableReference::ColumnReference(column.to_string())),
        );
        format!("=SUBTOTAL({code},{reference})")
    }

    /// Picks the total row's function under a column: "none", "sum",
    /// "average", "count", "countNums", "max", "min", "stdDev" or "var".
    pub fn set_table_column_total(
        &mut self,
        name: &str,
        column: &str,
        function: &str,
    ) -> Result<(), String> {
        let (mut table, area, sheet) = self.table_for_change(name)?;
        if table.totals_row_count == 0 {
            return Err(format!("Table '{}' has no total row", table.name));
        }
        if function != "none" && totals_function_code(function).is_none() {
            return Err(format!("Unknown total function '{function}'"));
        }
        let index = table
            .columns
            .iter()
            .position(|c| c.name.to_lowercase() == column.to_lowercase())
            .ok_or(format!("No column '{column}' in table '{}'", table.name))?;
        let column_name = table.columns[index].name.clone();
        let value = if function == "none" {
            table.columns[index].totals_row_function = None;
            String::new()
        } else {
            table.columns[index].totals_row_function = Some(function.to_string());
            table.columns[index].totals_row_label = None;
            self.total_formula(&table.name, &column_name, function)
        };
        let key = table.name.clone();
        let mut diff_list = vec![self.replace_table(Some(&key), Some(table))];
        self.write_for_table(
            sheet,
            area.last_row,
            area.first_column + index as i32,
            &value,
            &mut diff_list,
        )?;
        self.push_diff_list(diff_list);
        self.evaluate_if_not_paused();
        Ok(())
    }

    /// Every table, or the tables on one sheet.
    pub fn get_tables(&self, sheet: Option<u32>) -> Result<Vec<TableInfo>, String> {
        self.model.get_tables(sheet)
    }

    /// The table that covers a cell, if any.
    pub fn get_table_at(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<Option<String>, String> {
        self.model.get_table_at(sheet, row, column)
    }

    /// What typing `value` in a cell does to the tables, done before the
    /// value is written: renaming a column (typing in a header; then the
    /// write is done here too and `true` comes back) or growing a table to
    /// take in a cell just below or right of it. Returns the diffs to put
    /// first in the action's list.
    pub(crate) fn table_effects_before_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<(Vec<Diff>, bool), String> {
        let sheet_name = self.model.workbook.worksheet(sheet)?.get_name();
        let tables = &self.model.workbook.tables;
        let mut diff_list = Vec::new();

        // A header: the column takes the typed name
        if let Some(table) = table_at(tables, &sheet_name, row, column).cloned() {
            let area = table_area(&table).ok_or("Invalid table range")?;
            if table.header_row_count == 0 || row != area.first_row {
                return Ok((diff_list, false));
            }
            let index = (column - area.first_column) as usize;
            let old = table.columns[index].name.clone();
            let others: Vec<String> = table
                .columns
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, c)| c.name.clone())
                .collect();
            let typed = value.strip_prefix('\'').unwrap_or(value);
            let new = if typed.is_empty() {
                default_column_name(&others)
            } else {
                unique_column_name(&others, typed)
            };
            if new != old {
                self.model.rename_table_column(&table.name, &old, &new)?;
                diff_list.push(Diff::RenameTableColumn {
                    table: table.name.clone(),
                    old_name: old,
                    new_name: new.clone(),
                });
            }
            self.write_text_for_table(sheet, row, column, &new, &mut diff_list)?;
            return Ok((diff_list, true));
        }
        if value.is_empty() {
            return Ok((diff_list, false));
        }

        // A cell just below or right of a table: the table grows
        for table in tables.values().filter(|t| t.sheet_name == sheet_name) {
            let Some(area) = table_area(table) else {
                continue;
            };
            let below = table.totals_row_count == 0
                && row == area.last_row + 1
                && (area.first_column..=area.last_column).contains(&column);
            let right =
                column == area.last_column + 1 && (area.first_row..=area.last_row).contains(&row);
            if !below && !right {
                continue;
            }
            let mut table = table.clone();
            let mut new_area = area;
            let (data_first, data_last) = data_rows(&table, &area);
            if below {
                new_area.last_row += 1;
            } else {
                new_area.last_column += 1;
            }
            // Never into another table
            let blocked = self.model.workbook.tables.values().any(|t| {
                t.name != table.name
                    && t.sheet_name == sheet_name
                    && table_area(t).is_some_and(|a| {
                        a.first_row <= new_area.last_row
                            && a.last_row >= new_area.first_row
                            && a.first_column <= new_area.last_column
                            && a.last_column >= new_area.first_column
                    })
            });
            if blocked {
                return Ok((diff_list, false));
            }
            table.reference = area_reference(&new_area);
            let mut header = None;
            let mut handled = false;
            if right {
                let names: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
                let typed_header = table.header_row_count > 0 && row == area.first_row;
                let name = if typed_header {
                    handled = true;
                    unique_column_name(&names, value.strip_prefix('\'').unwrap_or(value))
                } else {
                    default_column_name(&names)
                };
                let id = table.columns.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                table.columns.push(TableColumn {
                    id,
                    name: name.clone(),
                    ..Default::default()
                });
                if table.header_row_count > 0 {
                    header = Some(name);
                }
            }
            let key = table.name.clone();
            diff_list.push(self.replace_table(Some(&key), Some(table)));
            if let Some(name) = header {
                self.write_text_for_table(sheet, area.first_row, column, &name, &mut diff_list)?;
            }
            if below {
                // The new row gets the formulas of the calculated columns
                for c in area.first_column..=area.last_column {
                    if c == column || data_last < data_first {
                        continue;
                    }
                    if let Some(index) =
                        self.model
                            .common_formula(sheet, c, data_first..=data_last)?
                    {
                        let text = self.model.formula_text_at(sheet, index, row, c)?;
                        self.write_for_table(sheet, row, c, &text, &mut diff_list)?;
                    }
                }
            }
            return Ok((diff_list, handled));
        }
        Ok((diff_list, false))
    }

    /// A formula typed in a table column whose other data cells are empty, or
    /// all hold one formula, fills the column. Done after the formula is
    /// written; returns the diffs to add to the action's list.
    pub(crate) fn table_effects_after_input(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        value: &str,
    ) -> Result<Vec<Diff>, String> {
        let mut diff_list = Vec::new();
        if !value.starts_with('=') {
            return Ok(diff_list);
        }
        let sheet_name = self.model.workbook.worksheet(sheet)?.get_name();
        let Some(table) = table_at(&self.model.workbook.tables, &sheet_name, row, column) else {
            return Ok(diff_list);
        };
        let area = table_area(table).ok_or("Invalid table range")?;
        let (data_first, data_last) = data_rows(table, &area);
        if row < data_first || row > data_last || data_first == data_last {
            return Ok(diff_list);
        }
        let Some(index) = self
            .model
            .workbook
            .worksheet(sheet)?
            .cell(row, column)
            .and_then(|c| c.get_formula())
        else {
            return Ok(diff_list);
        };
        let others: Vec<i32> = (data_first..=data_last).filter(|r| *r != row).collect();
        let all_empty = others
            .iter()
            .all(|r| self.model.is_empty_cell(sheet, *r, column).unwrap_or(false));
        let worksheet = self.model.workbook.worksheet(sheet)?;
        let formulas: Vec<Option<i32>> = others
            .iter()
            .map(|r| worksheet.cell(*r, column).and_then(|c| c.get_formula()))
            .collect();
        let one_formula = formulas.iter().all(|f| f.is_some() && *f == formulas[0]);
        if !all_empty && !one_formula {
            return Ok(diff_list);
        }
        for r in others {
            let text = self.model.formula_text_at(sheet, index, r, column)?;
            if self.model.get_cell_formula(sheet, r, column)?.as_deref() != Some(text.as_str()) {
                self.write_for_table(sheet, r, column, &text, &mut diff_list)?;
            }
        }
        Ok(diff_list)
    }

    /// A no-op snapshot of the tables for a structural action (insert, delete
    /// or move of rows or columns), put first in its list: the action
    /// displaces the tables when it is replayed, but its undo can't always
    /// rebuild them, so undo restores this exact copy.
    pub(crate) fn tables_snapshot_diff(
        &self,
        before: std::collections::HashMap<String, Table>,
    ) -> Vec<Diff> {
        if self.model.workbook.tables == before {
            return vec![];
        }
        vec![Diff::SetTables {
            old_value: before.clone(),
            new_value: before,
        }]
    }
}
