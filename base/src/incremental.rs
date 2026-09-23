//! Recalculating only what an edit affects.
//!
//! A full [`Model::evaluate`] recalculates every formula. After it, the model
//! keeps a dependency index built from the formulas' text: for every cell,
//! which formulas refer to it. [`Model::evaluate_incremental`] then takes the
//! cells edited since and recalculates only the formulas that depend on them
//! (directly or through other formulas), plus the volatile ones (NOW, TODAY,
//! RAND, INDIRECT, OFFSET...).
//!
//! It falls back to a full evaluation whenever dynamic arrays are involved
//! (a spill that could grow, shrink, block or unblock), so results are always
//! the same as a full evaluation.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    cell::CellValue,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
    types::{ArrayKind, Cell},
};

type Key = (u32, i32, i32);
/// (first row, last row, formula) of a range over one column
type ColumnSpan = (i32, i32, Key);
/// (sheet, first row, first column, last row, last column, formula)
type WideRange = (u32, i32, i32, i32, i32, Key);

/// Ranges at most this many columns wide are indexed column by column.
const NARROW: i32 = 64;

#[derive(Clone, Debug)]
enum Ref {
    Cell(u32, i32, i32),
    Range(u32, i32, i32, i32, i32),
}

#[derive(Default)]
pub(crate) struct DependencyIndex {
    /// formula cell => the cells and ranges its text refers to
    forward: HashMap<Key, Vec<Ref>>,
    /// cell => formulas that refer to it
    by_cell: HashMap<Key, Vec<Key>>,
    /// (sheet, column) => narrow ranges over that column: (row1, row2, formula)
    by_column: HashMap<(u32, i32), Vec<ColumnSpan>>,
    /// wide ranges: (sheet, row1, column1, row2, column2, formula)
    wide: Vec<WideRange>,
    /// formulas that must always be recalculated
    volatile: HashSet<Key>,
    /// every spilling formula and the cells its spill covered
    spills: Vec<(Key, Vec<Key>)>,
}

fn is_volatile(kind: &Function) -> bool {
    matches!(
        kind,
        Function::Now
            | Function::Today
            | Function::Rand
            | Function::Randbetween
            | Function::Randarray
            | Function::Indirect
            | Function::Offset
            | Function::Cell
            | Function::Info
    )
}

/// Collects what a formula at `cell` refers to. Returns true when the
/// references can only be known by evaluating (the formula is volatile).
fn collect(node: &Node, cell: Key, refs: &mut Vec<Ref>) -> bool {
    let (row0, column0) = (cell.1, cell.2);
    let many = |nodes: &[Node], refs: &mut Vec<Ref>| {
        let mut volatile = false;
        for n in nodes {
            volatile |= collect(n, cell, refs);
        }
        volatile
    };
    match node {
        Node::ReferenceKind {
            sheet_index,
            absolute_row,
            absolute_column,
            row,
            column,
            ..
        } => {
            let r = if *absolute_row { *row } else { row + row0 };
            let c = if *absolute_column {
                *column
            } else {
                column + column0
            };
            refs.push(Ref::Cell(*sheet_index, r, c));
            false
        }
        Node::RangeKind {
            sheet_index,
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
            ..
        } => {
            let r1 = if *absolute_row1 { *row1 } else { row1 + row0 };
            let r2 = if *absolute_row2 { *row2 } else { row2 + row0 };
            let c1 = if *absolute_column1 {
                *column1
            } else {
                column1 + column0
            };
            let c2 = if *absolute_column2 {
                *column2
            } else {
                column2 + column0
            };
            refs.push(Ref::Range(
                *sheet_index,
                r1.min(r2),
                c1.min(c2),
                r1.max(r2),
                c1.max(c2),
            ));
            false
        }
        Node::OpRangeKind { left, right } => {
            // A1:INDEX(...) is only known when evaluated
            collect(left, cell, refs);
            collect(right, cell, refs);
            true
        }
        Node::OpConcatenateKind { left, right }
        | Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right }
        | Node::CompareKind { left, right, .. } => {
            let a = collect(left, cell, refs);
            collect(right, cell, refs) || a
        }
        Node::FunctionKind { kind, args } => many(args, refs) || is_volatile(kind),
        Node::LambdaDefKind { body, .. } => collect(body, cell, refs),
        Node::LambdaCallKind { lambda, args } => {
            let a = collect(lambda, cell, refs);
            many(args, refs) || a
        }
        Node::NamedFunctionKind { args, .. } => {
            many(args, refs);
            true
        }
        // A name or table may point anywhere, and can change
        Node::DefinedNameKind(_) | Node::TableNameKind(_) => true,
        Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::UnaryKind { right: child, .. } => collect(child, cell, refs),
        Node::BooleanKind(_)
        | Node::NumberKind(_)
        | Node::StringKind(_)
        | Node::WrongReferenceKind { .. }
        | Node::WrongRangeKind { .. }
        | Node::ArrayKind(_)
        | Node::NamedVariableKind { .. }
        | Node::ErrorKind(_)
        | Node::ParseErrorKind { .. }
        | Node::EmptyArgKind => false,
    }
}

impl DependencyIndex {
    fn add(&mut self, formula: Key, node: &Node) {
        let mut refs = Vec::new();
        if collect(node, formula, &mut refs) {
            self.volatile.insert(formula);
        }
        for r in &refs {
            match *r {
                Ref::Cell(s, row, column) => self
                    .by_cell
                    .entry((s, row, column))
                    .or_default()
                    .push(formula),
                Ref::Range(s, r1, c1, r2, c2) => {
                    if c2 - c1 < NARROW {
                        for c in c1..=c2 {
                            self.by_column
                                .entry((s, c))
                                .or_default()
                                .push((r1, r2, formula));
                        }
                    } else {
                        self.wide.push((s, r1, c1, r2, c2, formula));
                    }
                }
            }
        }
        self.forward.insert(formula, refs);
    }

    fn remove(&mut self, formula: Key) {
        self.volatile.remove(&formula);
        let Some(refs) = self.forward.remove(&formula) else {
            return;
        };
        for r in refs {
            match r {
                Ref::Cell(s, row, column) => {
                    if let Some(v) = self.by_cell.get_mut(&(s, row, column)) {
                        v.retain(|f| *f != formula);
                    }
                }
                Ref::Range(s, _, c1, _, c2) => {
                    if c2 - c1 < NARROW {
                        for c in c1..=c2 {
                            if let Some(v) = self.by_column.get_mut(&(s, c)) {
                                v.retain(|(_, _, f)| *f != formula);
                            }
                        }
                    } else {
                        self.wide.retain(|(.., f)| *f != formula);
                    }
                }
            }
        }
    }

    fn dependents(&self, cell: Key, out: &mut Vec<Key>) {
        let (s, row, column) = cell;
        if let Some(v) = self.by_cell.get(&cell) {
            out.extend_from_slice(v);
        }
        if let Some(v) = self.by_column.get(&(s, column)) {
            out.extend(
                v.iter()
                    .filter(|(r1, r2, _)| (*r1..=*r2).contains(&row))
                    .map(|(.., f)| *f),
            );
        }
        out.extend(
            self.wide
                .iter()
                .filter(|(ws, r1, c1, r2, c2, _)| {
                    *ws == s && (*r1..=*r2).contains(&row) && (*c1..=*c2).contains(&column)
                })
                .map(|(.., f)| *f),
        );
    }
}

impl Model<'_> {
    fn formula_node(&self, cell: Key) -> Option<&Node> {
        let f = self
            .workbook
            .worksheets
            .get(cell.0 as usize)?
            .cell(cell.1, cell.2)?
            .get_formula()?;
        self.parsed_formulas
            .get(cell.0 as usize)?
            .get(f as usize)
            .map(|(node, _)| node)
    }

    /// A formula whose result covers more than its own cell (a spill or a
    /// legacy array formula), or a cell of such a result. A dynamic formula
    /// that fits in its own cell is an ordinary formula here.
    fn is_array_formula(&self, cell: Key) -> bool {
        match self
            .workbook
            .worksheets
            .get(cell.0 as usize)
            .and_then(|ws| ws.cell(cell.1, cell.2))
        {
            Some(Cell::ArrayFormula {
                kind: ArrayKind::Dynamic,
                r,
                ..
            }) => *r != (1, 1),
            Some(Cell::ArrayFormula { .. }) | Some(Cell::SpillCell { .. }) => true,
            _ => false,
        }
    }

    /// Builds the dependency index from every formula in the workbook.
    pub(crate) fn build_dependency_index(&mut self) {
        let mut index = DependencyIndex::default();
        for (sheet, ws) in self.workbook.worksheets.iter().enumerate() {
            for (row, cells) in &ws.sheet_data {
                for (column, cell) in cells {
                    if let Some(f) = cell.get_formula() {
                        if let Some((node, _)) = self.parsed_formulas[sheet].get(f as usize) {
                            index.add((sheet as u32, *row, *column), node);
                        }
                    }
                }
            }
        }
        // Formulas that started spilling in this evaluation count too
        self.collect_spill_cells();
        index.spills = self
            .spill_cells
            .iter()
            .map(|anchor| {
                (
                    (anchor.sheet, anchor.row, anchor.column),
                    self.spill_area(*anchor),
                )
            })
            .collect();
        self.dependency_index = Some(Box::new(index));
    }

    fn spill_area(&self, anchor: CellReferenceIndex) -> Vec<Key> {
        self.get_spill_area(anchor)
            .into_iter()
            .map(|c| (c.sheet, c.row, c.column))
            .collect()
    }

    // Whether an edit may change a spill (grow, shrink, block or unblock it).
    fn edit_touches_spills(&self, index: &DependencyIndex, edited: &[Key]) -> bool {
        for (anchor, area) in &index.spills {
            let now = CellReferenceIndex {
                sheet: anchor.0,
                row: anchor.1,
                column: anchor.2,
            };
            if matches!(
                self.get_cell_value_by_index(anchor.0, anchor.1, anchor.2),
                Ok(CellValue::String(s)) if s == "#SPILL!"
            ) {
                return true;
            }
            // typing into a spill resets it
            if self.spill_area(now) != *area || edited.iter().any(|e| area.contains(e)) {
                return true;
            }
        }
        edited.iter().any(|e| self.is_array_formula(*e))
    }

    /// Recalculates after the cells in `edited` (sheet, row, column; 1-based)
    /// got new content, and nothing else changed since the last evaluation:
    /// only the formulas that depend on those cells, and the volatile ones,
    /// are recalculated. Returns the cells recalculated, or `None` when a
    /// full evaluation was needed and ran instead (no evaluation yet, or a
    /// dynamic array involved). After a structural change (rows, columns,
    /// sheets, defined names) call [`Model::evaluate`] instead.
    pub fn evaluate_incremental(
        &mut self,
        edited: &[(u32, i32, i32)],
    ) -> Option<Vec<(u32, i32, i32)>> {
        let Some(mut index) = self.dependency_index.take() else {
            self.evaluate();
            return None;
        };
        if self.edit_touches_spills(&index, edited) {
            self.evaluate();
            return None;
        }
        for &cell in edited {
            index.remove(cell);
            if let Some(node) = self.formula_node(cell) {
                let node = node.clone();
                index.add(cell, &node);
            }
        }

        // Everything that depends on the edits, directly or not
        let mut seen: HashSet<Key> = HashSet::new();
        let mut queue: VecDeque<Key> = edited.iter().copied().collect();
        queue.extend(index.volatile.iter().copied());
        let mut next = Vec::new();
        while let Some(cell) = queue.pop_front() {
            if !seen.insert(cell) {
                continue;
            }
            next.clear();
            index.dependents(cell, &mut next);
            queue.extend(next.iter().copied().filter(|d| !seen.contains(d)));
        }
        let mut dirty: Vec<Key> = seen
            .into_iter()
            .filter(|c| self.formula_node(*c).is_some())
            .collect();
        // The same order as a full evaluation, so that formulas caught in a
        // circular reference come out the same too.
        dirty.sort_unstable();
        if dirty.iter().any(|c| self.is_array_formula(*c)) {
            self.evaluate();
            return None;
        }

        for cell in &dirty {
            self.cells.remove(cell);
            self.support.remove(&CellReferenceIndex {
                sheet: cell.0,
                row: cell.1,
                column: cell.2,
            });
            self.links.remove(cell);
        }
        let circular_before = self.circular_hits;
        for cell in &dirty {
            self.evaluate_cell(CellReferenceIndex {
                sheet: cell.0,
                row: cell.1,
                column: cell.2,
            });
        }
        // A formula that now spills changes cells nothing was watching.
        // A circular reference among the recalculated formulas comes out
        // according to the order they are visited in, so it gets the order
        // of a full evaluation.
        if self.circular_hits != circular_before || dirty.iter().any(|c| self.is_array_formula(*c))
        {
            self.evaluate();
            return None;
        }
        self.evaluate_conditional_formatting();
        self.dependency_index = Some(index);
        Some(dirty)
    }
}
