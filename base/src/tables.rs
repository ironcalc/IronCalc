//! Excel tables ("Format as Table", ECMA-376 section 18.5).
//!
//! A table is a named range with a header row, data rows and an optional
//! total row. Formulas reach it through structured references
//! (`Table1[Price]`, `[@Qty]*[@Price]`), which stay structured references in
//! the parsed formula ([`Node::TableReferenceKind`]) and are resolved to cells
//! only when a formula is evaluated. So a reference follows its table when
//! the table grows, shrinks or moves, and renaming a table or a column only
//! rewrites names.
//!
//! This file holds the lookups shared by the parser, the evaluator and the
//! model; the operations on tables (create, rename, convert to a range, total
//! row, growing at the edges) are in `table_ops.rs` and `user_model/tables.rs`.

use std::collections::HashMap;

use crate::expressions::parser::{parse_range, Node};
use crate::expressions::token::{Error, TableReference, TableSpecifier};
use crate::expressions::types::CellReferenceIndex;
use crate::types::{Table, TableColumn};

/// The cells a table covers, header and total rows included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TableArea {
    pub(crate) first_row: i32,
    pub(crate) first_column: i32,
    pub(crate) last_row: i32,
    pub(crate) last_column: i32,
}

impl TableArea {
    pub(crate) fn contains(&self, row: i32, column: i32) -> bool {
        row >= self.first_row
            && row <= self.last_row
            && column >= self.first_column
            && column <= self.last_column
    }
}

pub(crate) fn table_area(table: &Table) -> Option<TableArea> {
    let (first_column, first_row, last_column, last_row) = parse_range(&table.reference).ok()?;
    Some(TableArea {
        first_row,
        first_column,
        last_row,
        last_column,
    })
}

/// The first and last data rows (no header, no total row).
pub(crate) fn data_rows(table: &Table, area: &TableArea) -> (i32, i32) {
    (
        area.first_row + table.header_row_count as i32,
        area.last_row - table.totals_row_count as i32,
    )
}

/// Table names are case-insensitive, as in Excel.
pub(crate) fn find_table<'t>(tables: &'t HashMap<String, Table>, name: &str) -> Option<&'t Table> {
    let lower = name.to_lowercase();
    tables
        .values()
        .find(|t| t.name.to_lowercase() == lower || t.display_name.to_lowercase() == lower)
}

/// The table that covers a cell, if any.
pub(crate) fn table_at<'t>(
    tables: &'t HashMap<String, Table>,
    sheet_name: &str,
    row: i32,
    column: i32,
) -> Option<&'t Table> {
    tables.values().find(|t| {
        t.sheet_name == sheet_name && table_area(t).is_some_and(|a| a.contains(row, column))
    })
}

/// Column names are case-insensitive too.
pub(crate) fn find_column<'t>(table: &'t Table, name: &str) -> Option<&'t TableColumn> {
    let lower = name.to_lowercase();
    table
        .columns
        .iter()
        .find(|c| c.name.to_lowercase() == lower)
}

fn column_offset(table: &Table, name: &str) -> Option<i32> {
    let lower = name.to_lowercase();
    table
        .columns
        .iter()
        .position(|c| c.name.to_lowercase() == lower)
        .map(|i| i as i32)
}

/// What a structured reference stands for now: a cell, a range or an error.
/// `row` is the row of the formula's cell (for `[@Column]`).
pub(crate) fn resolve_structured_reference(
    table: &Table,
    sheet_index: u32,
    specifier: &Option<TableSpecifier>,
    table_reference: &Option<TableReference>,
    row: i32,
) -> Node {
    let area = match table_area(table) {
        Some(a) => a,
        None => return Node::ErrorKind(Error::REF),
    };
    let (data_first, data_last) = data_rows(table, &area);
    let (row1, row2) = match specifier {
        None | Some(TableSpecifier::Data) => (data_first, data_last),
        Some(TableSpecifier::All) => (area.first_row, area.last_row),
        Some(TableSpecifier::Headers) => {
            if table.header_row_count == 0 {
                return Node::ErrorKind(Error::REF);
            }
            (area.first_row, area.first_row)
        }
        Some(TableSpecifier::Totals) => {
            // Table1[#Totals] is #REF! while the table has no total row
            if table.totals_row_count == 0 {
                return Node::ErrorKind(Error::REF);
            }
            (area.last_row, area.last_row)
        }
        Some(TableSpecifier::ThisRow) => {
            // [@Price] outside the data rows is #VALUE!, as in Excel
            if row < data_first || row > data_last {
                return Node::ErrorKind(Error::VALUE);
            }
            (row, row)
        }
    };
    let (column1, column2) = match table_reference {
        None => (area.first_column, area.last_column),
        Some(TableReference::ColumnReference(name)) => match column_offset(table, name) {
            Some(i) => (area.first_column + i, area.first_column + i),
            None => return Node::ErrorKind(Error::REF),
        },
        Some(TableReference::RangeReference((left, right))) => {
            match (column_offset(table, left), column_offset(table, right)) {
                (Some(l), Some(r)) => (area.first_column + l.min(r), area.first_column + l.max(r)),
                _ => return Node::ErrorKind(Error::REF),
            }
        }
    };
    if row1 == row2 && column1 == column2 {
        Node::ReferenceKind {
            sheet_name: None,
            sheet_index,
            absolute_row: true,
            absolute_column: true,
            row: row1,
            column: column1,
        }
    } else {
        Node::RangeKind {
            sheet_name: None,
            sheet_index,
            absolute_row1: true,
            absolute_column1: true,
            row1,
            column1,
            absolute_row2: true,
            absolute_column2: true,
            row2,
            column2,
        }
    }
}

fn specifier_text(specifier: &TableSpecifier) -> &'static str {
    match specifier {
        TableSpecifier::All => "#All",
        TableSpecifier::Data => "#Data",
        TableSpecifier::Headers => "#Headers",
        TableSpecifier::ThisRow => "#This Row",
        TableSpecifier::Totals => "#Totals",
    }
}

/// Characters that make Excel put a column name in its own brackets
/// (`Table1[[Total $]]`), as listed in "Using structured references with
/// Excel tables". After '@', a space does too: `[@[Unit Price]]`.
fn needs_brackets(name: &str, after_at: bool) -> bool {
    name.starts_with(' ')
        || name.ends_with(' ')
        || name
            .chars()
            .any(|c| "\t\n\r,:.[]#'\"{}$^&*+=-><@/".contains(c) || (after_at && c == ' '))
}

/// `'` escapes `[`, `]`, `#`, `@` and itself inside a column name.
fn escape_column_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if matches!(c, '\'' | '[' | ']' | '#' | '@') {
            out.push('\'');
        }
        out.push(c);
    }
    out
}

/// The text of a structured reference, the way Excel writes it:
/// `Table1[Price]`, `Table1[@Price]`, `Table1[@[Unit Price]]`,
/// `Table1[[#Totals],[Price]]`, `Table1[[Jan]:[Dec]]`. An empty `table_name`
/// leaves the name out (`[@Price]`), as Excel shows references to the table a
/// cell is in.
pub(crate) fn structured_reference_to_string(
    table_name: &str,
    specifier: &Option<TableSpecifier>,
    table_reference: &Option<TableReference>,
) -> String {
    let t = table_name;
    let bracketed = |name: &str| format!("[{}]", escape_column_name(name));
    match (specifier, table_reference) {
        (Some(TableSpecifier::ThisRow), None) => format!("{t}[@]"),
        (Some(TableSpecifier::ThisRow), Some(TableReference::ColumnReference(c))) => {
            if needs_brackets(c, true) {
                format!("{t}[@{}]", bracketed(c))
            } else {
                format!("{t}[@{}]", escape_column_name(c))
            }
        }
        (Some(TableSpecifier::ThisRow), Some(TableReference::RangeReference((l, r)))) => {
            format!("{t}[@{}:{}]", bracketed(l), bracketed(r))
        }
        (Some(s), None) => format!("{t}[{}]", specifier_text(s)),
        (Some(s), Some(TableReference::ColumnReference(c))) => {
            format!("{t}[[{}],{}]", specifier_text(s), bracketed(c))
        }
        (Some(s), Some(TableReference::RangeReference((l, r)))) => {
            format!(
                "{t}[[{}],{}:{}]",
                specifier_text(s),
                bracketed(l),
                bracketed(r)
            )
        }
        (None, None) => format!("{t}[#Data]"),
        (None, Some(TableReference::ColumnReference(c))) => {
            if needs_brackets(c, false) {
                format!("{t}[{}]", bracketed(c))
            } else {
                format!("{t}[{}]", escape_column_name(c))
            }
        }
        (None, Some(TableReference::RangeReference((l, r)))) => {
            format!("{t}[{}:{}]", bracketed(l), bracketed(r))
        }
    }
}

/// Calls `f` on `node` and then on every node below it.
pub(crate) fn for_each_node_mut(node: &mut Node, f: &mut dyn FnMut(&mut Node)) {
    f(node);
    match node {
        Node::OpRangeKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::CompareKind { left, right, .. } => {
            for_each_node_mut(left, f);
            for_each_node_mut(right, f);
        }
        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
            for arg in args {
                for_each_node_mut(arg, f);
            }
        }
        Node::LambdaDefKind { body, .. } => for_each_node_mut(body, f),
        Node::LambdaCallKind { lambda, args } => {
            for_each_node_mut(lambda, f);
            for arg in args {
                for_each_node_mut(arg, f);
            }
        }
        Node::UnaryKind { right, .. } => for_each_node_mut(right, f),
        Node::ImplicitIntersection { child, .. } | Node::SpillRangeOperator { child } => {
            for_each_node_mut(child, f)
        }
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::DefinedNameKind { .. }
        | Node::TableNameKind(_)
        | Node::TableReferenceKind { .. }
        | Node::NamedVariableKind { .. }
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => {}
    }
}

/// Whether a formula has a structured reference or a bare table name in it.
pub(crate) fn has_table_reference(node: &Node) -> bool {
    match node {
        Node::TableReferenceKind { .. } | Node::TableNameKind(_) => true,
        Node::OpRangeKind { left, right }
        | Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::CompareKind { left, right, .. } => {
            has_table_reference(left) || has_table_reference(right)
        }
        Node::FunctionKind { args, .. } | Node::NamedFunctionKind { args, .. } => {
            args.iter().any(has_table_reference)
        }
        Node::LambdaDefKind { body, .. } => has_table_reference(body),
        Node::LambdaCallKind { lambda, args } => {
            has_table_reference(lambda) || args.iter().any(has_table_reference)
        }
        Node::UnaryKind { right, .. } => has_table_reference(right),
        Node::ImplicitIntersection { child, .. } | Node::SpillRangeOperator { child } => {
            has_table_reference(child)
        }
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. }
        | Node::RangeKind { .. }
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::DefinedNameKind { .. }
        | Node::NamedVariableKind { .. }
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => false,
    }
}

impl crate::Model<'_> {
    /// A copy of `node` with every structured reference and bare table name
    /// replaced by the cells it stands for when evaluated in `cell`.
    pub(crate) fn resolve_table_references(&self, node: &Node, cell: CellReferenceIndex) -> Node {
        let mut resolved = node.clone();
        for_each_node_mut(&mut resolved, &mut |n| {
            let (table_name, specifier, table_reference) = match n {
                Node::TableReferenceKind {
                    table_name,
                    specifier,
                    table_reference,
                } => (
                    table_name.clone(),
                    specifier.clone(),
                    table_reference.clone(),
                ),
                // `=SUM(Table1)` is the table's data
                Node::TableNameKind(name) => (name.clone(), None, None),
                _ => return,
            };
            *n = match find_table(&self.workbook.tables, &table_name) {
                Some(table) => match self.get_sheet_index_by_name(&table.sheet_name) {
                    Some(sheet_index) => resolve_structured_reference(
                        table,
                        sheet_index,
                        &specifier,
                        &table_reference,
                        cell.row,
                    ),
                    None => Node::ErrorKind(Error::REF),
                },
                None => Node::ErrorKind(Error::REF),
            };
        });
        resolved
    }

    /// `node`, the formula of `cell`, with every structured reference written
    /// as the cells it stands for now, the way "Convert to Range" writes it;
    /// `None` if it has none. The xlsx export writes formulas this way, as it
    /// writes no table definitions yet: a workbook it saves calculates the
    /// same when it is read again.
    pub fn table_references_as_cells(&self, node: &Node, cell: CellReferenceIndex) -> Option<Node> {
        self.table_references_as_cells_where(node, cell, &|_| true)
    }

    /// Like [`Self::table_references_as_cells`], for the references to the
    /// tables whose name `include` accepts. `[@Price]` becomes `$C2`, its row
    /// following the cell; everything else becomes an absolute reference, with
    /// the table's sheet when it is not the cell's.
    pub(crate) fn table_references_as_cells_where(
        &self,
        node: &Node,
        cell: CellReferenceIndex,
        include: &dyn Fn(&str) -> bool,
    ) -> Option<Node> {
        if !has_table_reference(node) {
            return None;
        }
        let mut changed = false;
        let mut node = node.clone();
        for_each_node_mut(&mut node, &mut |n| {
            let (table_name, specifier, table_reference) = match n {
                Node::TableReferenceKind {
                    table_name,
                    specifier,
                    table_reference,
                } => (
                    table_name.clone(),
                    specifier.clone(),
                    table_reference.clone(),
                ),
                Node::TableNameKind(name) => (name.clone(), None, None),
                _ => return,
            };
            if !include(&table_name) {
                return;
            }
            changed = true;
            let Some((table, table_sheet)) = find_table(&self.workbook.tables, &table_name)
                .and_then(|t| Some((t, self.get_sheet_index_by_name(&t.sheet_name)?)))
            else {
                *n = Node::ErrorKind(Error::REF);
                return;
            };
            let mut resolved = resolve_structured_reference(
                table,
                table_sheet,
                &specifier,
                &table_reference,
                cell.row,
            );
            let other_sheet = (table_sheet != cell.sheet).then(|| table.sheet_name.clone());
            let this_row = specifier == Some(TableSpecifier::ThisRow);
            match &mut resolved {
                Node::ReferenceKind {
                    sheet_name,
                    absolute_row,
                    row,
                    ..
                } => {
                    *sheet_name = other_sheet;
                    if this_row {
                        *absolute_row = false;
                        *row -= cell.row;
                    }
                }
                Node::RangeKind {
                    sheet_name,
                    absolute_row1,
                    absolute_row2,
                    row1,
                    row2,
                    ..
                } => {
                    *sheet_name = other_sheet;
                    if this_row {
                        *absolute_row1 = false;
                        *absolute_row2 = false;
                        *row1 -= cell.row;
                        *row2 -= cell.row;
                    }
                }
                _ => {}
            }
            *n = resolved;
        });
        changed.then_some(node)
    }
}
