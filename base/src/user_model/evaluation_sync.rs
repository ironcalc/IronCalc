//! Moving the results of an evaluation from one model to another.
//!
//! A host can evaluate a workbook somewhere other than where it shows it, for
//! instance in a Web Worker, so that a long evaluation never blocks the UI.
//! Two models of the same workbook are kept in sync:
//!
//! * The model the UI reads pauses its evaluation
//!   ([UserModel::pause_evaluation]) and sends its changes
//!   ([UserModel::flush_send_queue]) to the other one.
//! * The other model applies them ([UserModel::apply_external_diffs]) with its
//!   evaluation paused too. It now holds exactly what the model the UI reads
//!   held when it sent them. An [EvaluationTracker] created at that point
//!   remembers it.
//! * The other model evaluates, and the tracker tells what the evaluation
//!   changed ([EvaluationTracker::take_results]).
//! * The model the UI reads applies those results with
//!   [UserModel::apply_evaluation_results], without evaluating anything, and
//!   only then sends the changes the user made in the meantime.
//!
//! There is never more than one set of changes waiting for its results. The
//! changes made in the meantime wait in the send queue of the model the UI
//! reads, and the results are applied *under* them: those changes are undone,
//! the results applied and the changes made again. That is the order in which
//! the other model sees them, so both models end up holding the same. Applying
//! the results on top instead could put back a value one of those changes had
//! removed (a spill cell where the user has just written, for instance).
//!
//! The tracker has to be created after the changes are applied and before the
//! evaluation. A change can undo another (a formula replaced and typed back)
//! without the evaluating model ever seeing the difference, but the model the
//! UI reads holds that formula unevaluated: compared with what the evaluating
//! model held before the changes nothing would have changed, compared with
//! what the model the UI reads holds the formula has a new value.
//!
//! An evaluation only writes three kinds of cells: formula cells (their value),
//! the anchors of array formulas (their value and spill range) and spill cells.
//! Those are the cells the tracker follows and the only ones the results touch.
//!
//! As a last line of defence, a result that does not fit the cell it is meant
//! for (a formula result for a cell that no longer holds that formula, a spill
//! value for a cell that holds a value) is skipped: nothing the user typed is
//! ever overwritten.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use bitcode::{Decode, Encode};

use crate::types::Cell;

use super::history::DiffType;
use super::UserModel;

/// A cell written by the evaluation, at a row and column of a sheet.
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
struct ComputedCell {
    row: i32,
    column: i32,
    cell: Cell,
}

/// What an evaluation changed in one sheet. Sheets are named by their
/// `sheet_id`, which, unlike their index, does not change when sheets are
/// added, deleted or moved.
#[derive(Encode, Decode, Debug, Default)]
struct SheetResults {
    sheet_id: u32,
    /// Cells written by the evaluation whose value is new.
    changed: Vec<ComputedCell>,
    /// Positions that held a spill cell and no longer do, with the cell the
    /// evaluating model has there now (`None` if it has none).
    removed_spills: Vec<(i32, i32, Option<Cell>)>,
}

/// Keeps the cells an evaluation writes, as they were when the tracker was
/// created or after the last call to [EvaluationTracker::take_results], so
/// that the next call returns only what changed. See the module documentation
/// for when to create one.
///
/// The cells are kept per sheet, in the natural order of the sheet (by row, and
/// within a row by column), the order in which the sheet hands them out. Finding
/// what changed is then a single merge of two ordered lists.
#[derive(Debug, Default)]
pub struct EvaluationTracker {
    sheets: BTreeMap<u32, Vec<ComputedCell>>,
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

fn computed_cells(model: &UserModel) -> BTreeMap<u32, Vec<ComputedCell>> {
    model
        .model
        .workbook
        .worksheets
        .iter()
        .map(|worksheet| {
            let cells = worksheet
                .sheet_data
                .cells()
                .filter(|(_, _, cell)| is_computed(cell))
                .map(|(row, column, cell)| ComputedCell {
                    row,
                    column,
                    cell: cell.into_owned(),
                })
                .collect();
            (worksheet.sheet_id, cells)
        })
        .collect()
}

impl EvaluationTracker {
    /// Starts following the cells of `model` as they are now. Results are only
    /// returned for what changes from here on.
    pub fn new(model: &UserModel) -> EvaluationTracker {
        EvaluationTracker {
            sheets: computed_cells(model),
        }
    }

    /// Returns what the evaluation changed since the tracker was created or last
    /// called, ready for [UserModel::apply_evaluation_results].
    pub fn take_results(&mut self, model: &UserModel) -> Vec<u8> {
        let current = computed_cells(model);
        let mut results = Vec::new();
        for (sheet_id, cells) in &current {
            let previous = self
                .sheets
                .get(sheet_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let sheet_results = diff_sheet(model, *sheet_id, previous, cells);
            if !sheet_results.changed.is_empty() || !sheet_results.removed_spills.is_empty() {
                results.push(sheet_results);
            }
        }
        // Sheets that are gone are gone in both models: there is nothing to send.
        self.sheets = current;
        bitcode::encode(&results)
    }
}

/// Merges the cells a sheet had (`previous`) with the ones it has now
/// (`current`), both in natural order.
fn diff_sheet(
    model: &UserModel,
    sheet_id: u32,
    previous: &[ComputedCell],
    current: &[ComputedCell],
) -> SheetResults {
    let mut results = SheetResults {
        sheet_id,
        ..Default::default()
    };
    let removed = |results: &mut SheetResults, old: &ComputedCell| {
        // Only spill cells can disappear without the user removing them. A
        // formula the user removed is already gone from the other model.
        if matches!(old.cell, Cell::SpillCell { .. }) {
            let now = model
                .model
                .workbook
                .worksheets
                .iter()
                .find(|worksheet| worksheet.sheet_id == sheet_id)
                .and_then(|worksheet| worksheet.cell(old.row, old.column))
                .map(|cell| cell.into_owned());
            results.removed_spills.push((old.row, old.column, now));
        }
    };
    let (mut i, mut j) = (0, 0);
    while i < previous.len() || j < current.len() {
        let order = match (previous.get(i), current.get(j)) {
            (Some(old), Some(new)) => (old.row, old.column).cmp(&(new.row, new.column)),
            (Some(_), None) => Ordering::Less,
            (None, _) => Ordering::Greater,
        };
        match order {
            Ordering::Less => {
                removed(&mut results, &previous[i]);
                i += 1;
            }
            Ordering::Greater => {
                results.changed.push(current[j].clone());
                j += 1;
            }
            Ordering::Equal => {
                if previous[i] != current[j] {
                    results.changed.push(current[j].clone());
                }
                i += 1;
                j += 1;
            }
        }
    }
    results
}

impl UserModel<'_> {
    /// Applies the results of an evaluation made by another model of the same
    /// workbook (see [EvaluationTracker]), for the changes this model last sent
    /// ([UserModel::flush_send_queue]). The changes made since, still in the
    /// send queue, are undone while the results are applied and then made
    /// again (see the module documentation).
    ///
    /// It does not evaluate and leaves the history and the send queue as they
    /// were: the results are not a change made by the user.
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
        // Applying a diff list does not touch the send queue
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

    /// Writes the results in the cells they are for. Results that do not fit
    /// the cell they are meant for are skipped (see the module documentation).
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
                let fits = match &existing {
                    // The value of a formula only fits the same formula
                    Some(old) if is_formula(old) => {
                        is_formula(&cell) && formula_index(old) == formula_index(&cell)
                    }
                    // A spill only lands where there is nothing or another spill
                    Some(Cell::SpillCell { .. }) | Some(Cell::EmptyCell { .. }) | None => {
                        matches!(cell, Cell::SpillCell { .. })
                    }
                    Some(_) => false,
                };
                if !fits {
                    continue;
                }
                // The style of a cell is not the evaluation's business
                let style = existing
                    .map(|old| old.get_style())
                    .unwrap_or(cell.get_style());
                sheet_data.set_cell(row, column, with_style(cell, style));
            }
            for (row, column, now) in sheet_results.removed_spills {
                let is_spill = matches!(
                    sheet_data.cell(row, column).as_deref(),
                    Some(Cell::SpillCell { .. })
                );
                if !is_spill {
                    continue;
                }
                match now {
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
