//! Evaluating a workbook in one model and showing the results in another, so
//! that a host can evaluate off the UI thread (in a Web Worker, for instance).
//!
//! The model the UI shows pauses its evaluation and sends its changes
//! ([UserModel::flush_send_queue]) to the evaluating model. That one applies
//! them with its evaluation paused, creates an [EvaluationTracker], evaluates
//! and sends back [EvaluationTracker::take_results], which the model the UI
//! shows applies with [UserModel::apply_evaluation_results].
//!
//! The tracker is created after the changes are applied, not before: a formula
//! replaced and typed back in one batch has not changed for the evaluating
//! model, but the model the UI shows holds it unevaluated.
//!
//! Only one batch of changes waits for its results at a time. The results are
//! applied under the changes made since, which are undone and made again, in
//! the order the evaluating model sees them. Applied on top, they could put
//! back a spill cell where the user has just typed.

use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::BTreeMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem::discriminant;

use bitcode::{Decode, Encode};

use crate::types::{Cell, FormulaValue, SpillValue, Worksheet};

use super::history::DiffType;
use super::UserModel;

#[derive(Encode, Decode, Debug)]
struct ComputedCell {
    row: i32,
    column: i32,
    cell: Cell,
}

#[derive(Encode, Decode, Debug)]
struct RemovedSpill {
    row: i32,
    column: i32,
    replaced_by: Option<Cell>,
}

#[derive(Encode, Decode, Debug, Default)]
struct SheetResults {
    sheet_id: u32,
    changed: Vec<ComputedCell>,
    removed_spills: Vec<RemovedSpill>,
}

#[derive(Debug)]
struct TrackedCell {
    row: i32,
    column: i32,
    hash: u64,
}

/// Remembers the cells an evaluation can write, so that
/// [EvaluationTracker::take_results] returns only the ones it changed. It keeps
/// a 64-bit hash of each rather than a copy: a change goes unnoticed only if the
/// old and new cell hash the same, a chance of one in 2^64.
#[derive(Debug)]
pub struct EvaluationTracker {
    sheets: BTreeMap<u32, Vec<TrackedCell>>,
    hasher: RandomState,
}

fn is_formula(cell: &Cell) -> bool {
    matches!(cell, Cell::CellFormula { .. } | Cell::ArrayFormula { .. })
}

fn is_computed(cell: &Cell) -> bool {
    is_formula(cell) || matches!(cell, Cell::SpillCell { .. })
}

fn formula_index(cell: &Cell) -> Option<i32> {
    match cell {
        Cell::CellFormula { f, .. } | Cell::ArrayFormula { f, .. } => Some(*f),
        _ => None,
    }
}

/// In the natural order of the sheet, which `diff_sheet` relies on.
fn computed_cells(worksheet: &Worksheet) -> impl Iterator<Item = (i32, i32, Cow<'_, Cell>)> {
    let has_formulas = !worksheet.shared_formulas.is_empty();
    has_formulas
        .then(|| worksheet.sheet_data.cells())
        .into_iter()
        .flatten()
        .filter(|(_, _, cell)| is_computed(cell))
}

/// Every field is listed, with no `..`, so that a field added to [Cell] does
/// not compile until it is hashed too.
fn hash_cell(hasher: &RandomState, cell: &Cell) -> u64 {
    fn number(value: f64, state: &mut impl Hasher) {
        value.to_bits().hash(state);
    }
    fn formula_value(value: &FormulaValue, state: &mut impl Hasher) {
        discriminant(value).hash(state);
        match value {
            FormulaValue::Unevaluated => {}
            FormulaValue::Boolean(b) => b.hash(state),
            FormulaValue::Number(n) => number(*n, state),
            FormulaValue::Text(text) => text.hash(state),
            FormulaValue::Error { ei, d } => {
                discriminant(ei).hash(state);
                d.o.hash(state);
                d.m.hash(state);
            }
        }
    }
    let mut state = hasher.build_hasher();
    let state = &mut state;
    discriminant(cell).hash(state);
    match cell {
        Cell::EmptyCell { s } => s.hash(state),
        Cell::BooleanCell { v, s } => {
            v.hash(state);
            s.hash(state);
        }
        Cell::NumberCell { v, s } => {
            number(*v, state);
            s.hash(state);
        }
        Cell::ErrorCell { ei, s } => {
            discriminant(ei).hash(state);
            s.hash(state);
        }
        Cell::SharedString { si, s } => {
            si.hash(state);
            s.hash(state);
        }
        Cell::CellFormula { f, s, v } => {
            f.hash(state);
            s.hash(state);
            formula_value(v, state);
        }
        Cell::ArrayFormula { f, s, r, kind, v } => {
            f.hash(state);
            s.hash(state);
            r.hash(state);
            discriminant(kind).hash(state);
            formula_value(v, state);
        }
        Cell::SpillCell { s, a, v } => {
            s.hash(state);
            a.hash(state);
            discriminant(v).hash(state);
            match v {
                SpillValue::Boolean(b) => b.hash(state),
                SpillValue::Number(n) => number(*n, state),
                SpillValue::Text(text) => text.hash(state),
                SpillValue::Error(ei) => discriminant(ei).hash(state),
            }
        }
    }
    state.finish()
}

impl EvaluationTracker {
    /// Starts tracking the cells of `model` as they are now.
    pub fn new(model: &UserModel) -> EvaluationTracker {
        let hasher = RandomState::new();
        let sheets = model
            .model
            .workbook
            .worksheets
            .iter()
            .map(|worksheet| {
                let cells = computed_cells(worksheet)
                    .map(|(row, column, cell)| TrackedCell {
                        row,
                        column,
                        hash: hash_cell(&hasher, &cell),
                    })
                    .collect();
                (worksheet.sheet_id, cells)
            })
            .collect();
        EvaluationTracker { sheets, hasher }
    }

    /// What the evaluation changed since the tracker was created, for
    /// [UserModel::apply_evaluation_results].
    pub fn take_results(mut self, model: &UserModel) -> Vec<u8> {
        let mut results = Vec::new();
        for worksheet in &model.model.workbook.worksheets {
            let previous = self.sheets.remove(&worksheet.sheet_id).unwrap_or_default();
            let sheet_results = diff_sheet(&self.hasher, worksheet, previous);
            if !sheet_results.changed.is_empty() || !sheet_results.removed_spills.is_empty() {
                results.push(sheet_results);
            }
        }
        bitcode::encode(&results)
    }
}

fn diff_sheet(
    hasher: &RandomState,
    worksheet: &Worksheet,
    previous: Vec<TrackedCell>,
) -> SheetResults {
    let mut results = SheetResults {
        sheet_id: worksheet.sheet_id,
        ..Default::default()
    };
    let removed = |results: &mut SheetResults, old: TrackedCell| {
        let replaced_by = worksheet
            .cell(old.row, old.column)
            .map(|cell| cell.into_owned());
        results.removed_spills.push(RemovedSpill {
            row: old.row,
            column: old.column,
            replaced_by,
        });
    };
    let mut previous = previous.into_iter().peekable();
    for (row, column, cell) in computed_cells(worksheet) {
        while let Some(old) = previous.next_if(|old| (old.row, old.column) < (row, column)) {
            removed(&mut results, old);
        }
        let unchanged = previous
            .next_if(|old| (old.row, old.column) == (row, column))
            .is_some_and(|old| old.hash == hash_cell(hasher, &cell));
        if !unchanged {
            results.changed.push(ComputedCell {
                row,
                column,
                cell: cell.into_owned(),
            });
        }
    }
    for old in previous {
        removed(&mut results, old);
    }
    results
}

impl UserModel<'_> {
    /// Applies [EvaluationTracker::take_results] from the model that evaluates
    /// for this one, under the changes still in the send queue (see the module
    /// documentation). It does not evaluate, and leaves the history and the
    /// send queue as they were.
    pub fn apply_evaluation_results(&mut self, results: &[u8]) -> Result<(), String> {
        let results: Vec<SheetResults> = bitcode::decode(results)
            .map_err(|e| format!("Error parsing evaluation results: {e}"))?;
        let was_paused = self.pause_evaluation;
        self.pause_evaluation = true;
        let applied = self.apply_under_send_queue(results);
        self.pause_evaluation = was_paused;
        applied
    }

    fn apply_under_send_queue(&mut self, results: Vec<SheetResults>) -> Result<(), String> {
        let newer_changes = self.send_queue.clone();
        for change in newer_changes.iter().rev() {
            match change.r#type {
                DiffType::Redo => self.apply_undo_diff_list(&change.list)?,
                DiffType::Undo => self.apply_diff_list(&change.list)?,
            }
        }
        self.write_evaluation_results(results);
        for change in &newer_changes {
            match change.r#type {
                DiffType::Redo => self.apply_diff_list(&change.list)?,
                DiffType::Undo => self.apply_undo_diff_list(&change.list)?,
            }
        }
        Ok(())
    }

    fn write_evaluation_results(&mut self, results: Vec<SheetResults>) {
        for sheet_results in results {
            let Some(worksheet) = self
                .model
                .workbook
                .worksheets
                .iter_mut()
                .find(|worksheet| worksheet.sheet_id == sheet_results.sheet_id)
            else {
                continue;
            };
            let sheet_data = &mut worksheet.sheet_data;
            for ComputedCell { row, column, cell } in sheet_results.changed {
                let existing = sheet_data.cell(row, column).map(|c| c.into_owned());
                if !result_fits(existing.as_ref(), &cell) {
                    continue;
                }
                let style = existing
                    .map(|old| old.get_style())
                    .unwrap_or(cell.get_style());
                sheet_data.set_cell(row, column, with_style(cell, style));
            }
            for RemovedSpill {
                row,
                column,
                replaced_by,
            } in sheet_results.removed_spills
            {
                let is_spill = matches!(
                    sheet_data.cell(row, column).as_deref(),
                    Some(Cell::SpillCell { .. })
                );
                if !is_spill {
                    continue;
                }
                match replaced_by {
                    Some(cell) if !is_computed(&cell) => {
                        sheet_data.set_cell(row, column, cell);
                    }
                    _ => {
                        sheet_data.remove_cell(row, column);
                    }
                }
            }
        }
    }
}

/// Whether `result` can replace `existing`. It always can while both models are
/// in sync; if they are not, nothing the user typed is overwritten.
fn result_fits(existing: Option<&Cell>, result: &Cell) -> bool {
    match existing {
        Some(old) if is_formula(old) => {
            is_formula(result) && formula_index(old) == formula_index(result)
        }
        Some(Cell::SpillCell { .. }) | Some(Cell::EmptyCell { .. }) | None => {
            matches!(result, Cell::SpillCell { .. })
        }
        Some(_) => false,
    }
}

fn with_style(cell: Cell, style: i32) -> Cell {
    match cell {
        Cell::CellFormula { f, v, .. } => Cell::CellFormula { f, s: style, v },
        Cell::ArrayFormula { f, r, kind, v, .. } => Cell::ArrayFormula {
            f,
            s: style,
            r,
            kind,
            v,
        },
        Cell::SpillCell { a, v, .. } => Cell::SpillCell { s: style, a, v },
        other => other,
    }
}
