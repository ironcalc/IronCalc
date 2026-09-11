//! Evaluation of every formula in the workbook.
//!
//! The algorithm is the one described in `cold-evaluation.md`. In short:
//!
//! * A **pass** is the plain top-down recursion: to evaluate a cell, evaluate
//!   the cells it reads as it meets them. A cell met while it is already being
//!   evaluated closes a cycle. Dynamic array anchors are evaluated first, in a
//!   remembered order, then every cell of every sheet.
//! * A dynamic array writes cells other than its own, and which ones is only
//!   known once it has run. Anything that read one of those positions earlier
//!   in the pass read the wrong thing. So the pass records what formulas saw
//!   at the positions they read, and when an anchor is about to contradict a
//!   record, the pass is **abandoned and started again with that anchor
//!   first**. An anchor whose own inputs read its area is contradicted by
//!   itself wherever it sits: it is circular.
//! * Spill cells left by a previous evaluation are never read as values
//!   before their anchor has run in the current pass; they still block other
//!   arrays, which is how the array that spilled first keeps its cells. The
//!   one thing they cannot do is stand in for the anchor's output: when an
//!   anchor gives them up and only its own inputs were blocked by them, they
//!   were history, not a dependency. They are dropped for the rest of the
//!   evaluation and the pass starts again without them.
//!
//! The order of the anchors survives across evaluations, so a sheet that
//! needed restarts once evaluates in a single pass afterwards.
//!
//! Nothing here depends on the order in which cells are visited, only the
//! amount of work does.

use std::collections::{HashMap, HashSet};

use crate::calc_result::CalcResult;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::model::Model;
use crate::types::{ArrayKind, Cell};

/// `(sheet, row, column)`: a position in the workbook.
pub(crate) type CellKey = (u32, i32, i32);

fn key(cell: CellReferenceIndex) -> CellKey {
    (cell.sheet, cell.row, cell.column)
}

/// The state of a formula cell within the current pass. Absent means it has
/// not been evaluated yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CellState {
    Evaluating,
    Evaluated,
}

/// What a formula found at a position when it read it, as far as spills are
/// concerned. A spill written into a position seen `Empty`, or a spill cell
/// removed from a position seen `Occupied`, contradicts the reader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Seen {
    Empty,
    Occupied,
}

/// What was seen at one position in the current pass: the root of the
/// recursion that first read it as empty, and the root of the recursion on
/// whose behalf an array was first blocked by it. Both can be set for the
/// same position: a leftover spill cell of the anchor being evaluated is read
/// as empty on its behalf, and then blocks an array evaluated on demand
/// inside it. Keeping only the first record would let the anchor remove the
/// cell without contradicting the blocked array.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(crate) struct SeenRecord {
    pub(crate) empty: Option<CellReferenceIndex>,
    pub(crate) occupied: Option<CellReferenceIndex>,
}

/// Why the current pass is abandoned. In every case the anchor is moved to the
/// front of the evaluation order and the pass starts again.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Restart {
    /// A spill cell of this anchor was read before the anchor had run.
    StaleRead(CellReferenceIndex),
    /// What this anchor is about to write contradicts something read on behalf
    /// of another cell: the anchor should have run before it.
    Conflict(CellReferenceIndex),
    /// What this anchor is about to write contradicts only reads made on its
    /// own behalf: its inputs depend on its own output. It is circular.
    SelfContradiction(CellReferenceIndex),
    /// The spill cells this anchor is giving up, left by a previous
    /// evaluation, blocked an array evaluated on its own behalf and nothing
    /// else. The anchor's inputs depended on its history, not on its output:
    /// the cells are dropped for the rest of the evaluation.
    StaleCells(CellReferenceIndex, Vec<CellKey>),
}

impl Restart {
    fn anchor(&self) -> CellReferenceIndex {
        match self {
            Restart::StaleRead(anchor)
            | Restart::Conflict(anchor)
            | Restart::SelfContradiction(anchor)
            | Restart::StaleCells(anchor, _) => *anchor,
        }
    }
}

/// The driver's memory of the restarts of one evaluation, and its verdicts.
///
/// Every pass starts from the same sheet, so a pass is a function of the
/// anchor order and of the circular set. Two facts follow. An order that
/// comes back without the circular set having changed means the restarts
/// would repeat forever: every anchor moved since that order was first seen
/// is on a loop of anchors reading each other's areas, and all of them are
/// marked. And between two changes of the circular set every order is
/// distinct, so the restarts are finite; a budget of `n² + 2` guards against a
/// mistake in that reasoning by marking the restarting anchor once spent.
///
/// The sheet the passes start from changes only when stale spill cells are
/// dropped (`Restart::StaleCells`). That happens at most once per stale cell,
/// and the log starts afresh each time, as it does when the circular set
/// changes.
struct RestartLog {
    /// Anchors marked circular so far; passed to every pass.
    circular: Vec<CellKey>,
    /// The orders seen since the circular set or the starting sheet last
    /// changed.
    orders_seen: Vec<Vec<CellReferenceIndex>>,
    /// The anchor moved to the front to go from each seen order to the next.
    moves: Vec<CellReferenceIndex>,
    /// Every restart of this evaluation.
    restarts: u32,
    /// The restarts that count against the budget: all but the stale-cell ones.
    spent: u32,
    budget: u32,
}

impl RestartLog {
    fn new(initial_order: &[CellReferenceIndex]) -> Self {
        let anchors = initial_order.len() as u32;
        RestartLog {
            circular: Vec::new(),
            orders_seen: vec![initial_order.to_vec()],
            moves: Vec::new(),
            restarts: 0,
            spent: 0,
            budget: anchors * anchors + 2,
        }
    }

    /// Records a restart whose anchor has just been moved to the front, giving
    /// `order`, and marks whatever it proves circular.
    fn record(&mut self, restart: &Restart, order: &[CellReferenceIndex]) {
        self.restarts += 1;
        if let Restart::StaleCells(..) = restart {
            // The starting sheet changed: what a pass does changed with it.
            self.orders_seen = vec![order.to_vec()];
            self.moves.clear();
            return;
        }
        let anchor = restart.anchor();
        self.moves.push(anchor);
        let mut newly_circular = Vec::new();
        if matches!(restart, Restart::SelfContradiction(_)) || self.spent >= self.budget {
            newly_circular.push(anchor);
        } else if let Some(first_seen) = self.orders_seen.iter().position(|seen| seen == order) {
            newly_circular.extend(self.moves[first_seen..].iter().copied());
        }
        if newly_circular.is_empty() {
            self.orders_seen.push(order.to_vec());
        } else {
            for cell in newly_circular {
                if !self.circular.contains(&key(cell)) {
                    self.circular.push(key(cell));
                }
            }
            // The circular set changed: what a pass does changed with it.
            self.orders_seen = vec![order.to_vec()];
            self.moves.clear();
        }
        self.spent += 1;
    }
}

/// Everything the evaluation needs besides the workbook itself.
#[derive(Default)]
pub(crate) struct Evaluation {
    /// The dynamic anchors, in the order in which they are evaluated. Kept
    /// across evaluations: an anchor moved to the front by a restart stays
    /// there, so the next evaluation needs no restart. New anchors are appended
    /// in natural `(sheet, row, column)` order.
    pub(crate) anchor_order: Vec<CellReferenceIndex>,
    /// Formula cells touched in the current pass, and their state.
    pub(crate) cells: HashMap<CellKey, CellState>,
    /// Cells being evaluated, innermost last.
    pub(crate) stack: Vec<CellReferenceIndex>,
    /// Cells known to be circular. Marked on the stack when a read closes a
    /// loop, or by the driver when an anchor's spill contradicts its own inputs;
    /// the latter survive restarts within one evaluation.
    pub(crate) circular: HashSet<CellKey>,
    /// What formulas saw at the positions they read in the current pass, and
    /// the root of the recursion the read was made in (the cell the driver was
    /// evaluating at the time), one record per kind.
    pub(crate) seen: HashMap<CellKey, SeenRecord>,
    /// Set when the current pass must be abandoned. Every evaluation in
    /// progress then returns without storing anything.
    pub(crate) restart: Option<Restart>,
    /// True while `evaluate()` is running a pass. Outside a pass (conditional
    /// formatting at load time, formula helpers) there is no driver to restart
    /// anything, so cells are simply evaluated on demand.
    pub(crate) in_pass: bool,
    /// Number of restarts the last `evaluate()` needed. For tests and tuning.
    pub(crate) restarts_in_last_evaluation: u32,
}

impl<'a> Model<'a> {
    /// Evaluates every formula in the workbook.
    ///
    /// Runs passes until one completes without a restart. A restart moves the
    /// anchor that caused it to the front of the anchor order; `RestartLog`
    /// decides whether it also marks anchors circular. A marked anchor stores
    /// `#CIRC!` and keeps no spill cells, so there is nothing of it to read or
    /// to contradict once its stale cells are gone. Every other restart moves
    /// an anchor forward without repeating an order, or drops stale cells,
    /// which happens at most once per cell, so the loop ends.
    pub fn evaluate(&mut self) {
        self.sync_anchor_order();
        // Every pass starts from the same sheet: what an abandoned pass wrote
        // is undone. This is what makes a pass a function of the anchor order.
        // The only change to that sheet within an evaluation is the dropping
        // of stale spill cells an anchor gave up.
        let mut spills_before = self.dynamic_spills();
        let mut log = RestartLog::new(&self.evaluation.anchor_order);
        while let Some(restart) = self.run_pass(&log.circular) {
            if let Restart::StaleCells(_, cells) = &restart {
                spills_before.retain(|(position, _)| !cells.contains(&key(*position)));
            }
            self.restore_dynamic_spills(&spills_before);
            let anchor = restart.anchor();
            self.evaluation.anchor_order.retain(|a| *a != anchor);
            self.evaluation.anchor_order.insert(0, anchor);
            log.record(&restart, &self.evaluation.anchor_order);
        }
        self.evaluation.restarts_in_last_evaluation = log.restarts;
        self.evaluate_conditional_formatting();
    }

    /// Every position that holds a cell, in natural `(sheet, row, column)` order.
    fn all_positions(&self) -> Vec<CellReferenceIndex> {
        self.get_all_cells()
            .into_iter()
            .map(|index| CellReferenceIndex {
                sheet: index.index,
                row: index.row,
                column: index.column,
            })
            .collect()
    }

    /// Every dynamic anchor and every spill cell of a dynamic anchor, as they
    /// are now.
    fn dynamic_spills(&self) -> Vec<(CellReferenceIndex, Cell)> {
        self.all_positions()
            .into_iter()
            .filter_map(|position| {
                let cell = self.fetch_cell(position)?;
                self.belongs_to_a_dynamic_array(position, cell)
                    .then(|| (position, cell.clone()))
            })
            .collect()
    }

    /// Puts the dynamic anchors and their spill cells back to `snapshot`,
    /// removing the spill cells written since.
    fn restore_dynamic_spills(&mut self, snapshot: &[(CellReferenceIndex, Cell)]) {
        let written_since: Vec<CellReferenceIndex> = self
            .dynamic_spills()
            .into_iter()
            .filter(|(_, cell)| matches!(cell, Cell::SpillCell { .. }))
            .map(|(position, _)| position)
            .collect();
        for position in written_since {
            if let Ok(worksheet) = self.workbook.worksheet_mut(position.sheet) {
                let _ = worksheet.cell_clear_contents(position.row, position.column);
            }
        }
        for (position, cell) in snapshot {
            if let Ok(worksheet) = self.workbook.worksheet_mut(position.sheet) {
                let _ = worksheet.update_cell(position.row, position.column, cell.clone());
            }
        }
    }

    fn belongs_to_a_dynamic_array(&self, position: CellReferenceIndex, cell: &Cell) -> bool {
        match cell {
            Cell::ArrayFormula {
                kind: ArrayKind::Dynamic,
                ..
            } => true,
            Cell::SpillCell { a, .. } => matches!(
                self.fetch_cell(CellReferenceIndex {
                    sheet: position.sheet,
                    row: a.0,
                    column: a.1,
                }),
                Some(Cell::ArrayFormula {
                    kind: ArrayKind::Dynamic,
                    ..
                })
            ),
            _ => false,
        }
    }

    /// Brings `anchor_order` in line with the dynamic anchors that exist now:
    /// anchors that are gone are dropped, new ones are appended in natural
    /// order, the rest keep their relative order.
    fn sync_anchor_order(&mut self) {
        let current = self.dynamic_anchors_in_natural_order();
        let current_set: HashSet<CellReferenceIndex> = current.iter().copied().collect();
        let order = &mut self.evaluation.anchor_order;
        order.retain(|anchor| current_set.contains(anchor));
        let known: HashSet<CellReferenceIndex> = order.iter().copied().collect();
        order.extend(current.into_iter().filter(|a| !known.contains(a)));
    }

    fn dynamic_anchors_in_natural_order(&self) -> Vec<CellReferenceIndex> {
        self.all_positions()
            .into_iter()
            .filter(|cell| {
                matches!(
                    self.fetch_cell(*cell),
                    Some(Cell::ArrayFormula {
                        kind: ArrayKind::Dynamic,
                        ..
                    })
                )
            })
            .collect()
    }

    /// One pass over the workbook: anchors first, then every cell. Returns the
    /// reason the pass had to be abandoned, if any.
    fn run_pass(&mut self, circular_anchors: &[CellKey]) -> Option<Restart> {
        let state = &mut self.evaluation;
        state.cells.clear();
        state.stack.clear();
        state.seen.clear();
        state.restart = None;
        state.circular = circular_anchors.iter().copied().collect();
        state.in_pass = true;
        // dynamic links (HYPERLINK) are rebuilt on every evaluation
        self.links.clear();
        self.clear_variable_stack();
        self.clear_lambdas();

        let anchors = self.evaluation.anchor_order.clone();
        let everything = self.all_positions();
        let mut restart = None;
        for cell in anchors.into_iter().chain(everything) {
            self.evaluate_cell(cell);
            if self.evaluation.restart.is_some() {
                restart = self.evaluation.restart.take();
                break;
            }
        }
        self.evaluation.in_pass = false;
        restart
    }

    /// The value of a cell, evaluating it first if it is a formula that has
    /// not been evaluated in this pass. Reads are performed on behalf of the
    /// cell on top of the stack.
    pub(crate) fn evaluate_cell(&mut self, cell_reference: CellReferenceIndex) -> CalcResult {
        if self.evaluation.restart.is_some() {
            // The pass is being abandoned; nothing computed now is kept.
            return CalcResult::EmptyCell;
        }
        let Some(cell) = self.fetch_cell(cell_reference).cloned() else {
            self.record_seen(cell_reference, Seen::Empty);
            return CalcResult::EmptyCell;
        };
        match &cell {
            Cell::SpillCell { a, .. } => self.evaluate_spill_cell(cell_reference, *a),
            Cell::CellFormula { f, .. } | Cell::ArrayFormula { f, .. } => {
                self.evaluate_formula_cell(cell_reference, &cell, *f)
            }
            Cell::EmptyCell { .. } => {
                self.record_seen(cell_reference, Seen::Empty);
                CalcResult::EmptyCell
            }
            _ => self.get_cell_value(&cell, cell_reference),
        }
    }

    /// A spill cell holds what its anchor wrote. Only a value written by the
    /// anchor in the current pass is a value; anything else is either the
    /// anchor's business (a CSE area is fixed and forwards to its anchor) or a
    /// leftover of a previous evaluation, which is read as empty or triggers a
    /// restart so that the anchor runs first.
    fn evaluate_spill_cell(
        &mut self,
        cell_reference: CellReferenceIndex,
        anchor: (i32, i32),
    ) -> CalcResult {
        let anchor_reference = CellReferenceIndex {
            sheet: cell_reference.sheet,
            row: anchor.0,
            column: anchor.1,
        };
        let Some(Cell::ArrayFormula { kind, .. }) = self.fetch_cell(anchor_reference).cloned()
        else {
            // An orphan: its anchor is gone. Nothing will ever write it again.
            self.record_seen(cell_reference, Seen::Empty);
            return CalcResult::EmptyCell;
        };
        let anchor_state = self.evaluation.cells.get(&key(anchor_reference)).copied();
        match (kind, anchor_state) {
            // Written by the anchor in this pass.
            (_, Some(CellState::Evaluated)) => self.stored_value(cell_reference),
            // A CSE area is fixed: its anchor is evaluated first and the cell
            // holds what it wrote. If the anchor is running, this read closes a
            // cycle, which `evaluate_cell` reports.
            (ArrayKind::Cse, Some(CellState::Evaluating)) => self.evaluate_cell(anchor_reference),
            (ArrayKind::Cse, None) => {
                self.evaluate_cell(anchor_reference);
                self.stored_value(cell_reference)
            }
            // The anchor is running and this read is on its behalf: the area
            // counts as empty until the anchor commits. If the anchor then
            // writes this position, its inputs depended on its own output.
            (ArrayKind::Dynamic, Some(CellState::Evaluating)) => {
                self.record_seen(cell_reference, Seen::Empty);
                CalcResult::EmptyCell
            }
            // Left over from a previous evaluation: the anchor should have run
            // before anyone read its cells. Restart with it first. Outside a
            // pass there is no driver, so the anchor is simply evaluated now.
            (ArrayKind::Dynamic, None) => {
                if self.evaluation.in_pass {
                    self.evaluation.restart = Some(Restart::StaleRead(anchor_reference));
                    CalcResult::EmptyCell
                } else {
                    self.evaluate_cell(anchor_reference);
                    self.stored_value(cell_reference)
                }
            }
        }
    }

    fn evaluate_formula_cell(
        &mut self,
        cell_reference: CellReferenceIndex,
        cell: &Cell,
        formula: i32,
    ) -> CalcResult {
        let key = key(cell_reference);
        match self.evaluation.cells.get(&key) {
            Some(CellState::Evaluating) => {
                self.mark_cycle(cell_reference);
                return circular_reference(cell_reference);
            }
            Some(CellState::Evaluated) => return self.get_cell_value(cell, cell_reference),
            None => {}
        }
        self.evaluation.cells.insert(key, CellState::Evaluating);
        self.evaluation.stack.push(cell_reference);
        let result = if self.evaluation.circular.contains(&key) {
            circular_reference(cell_reference)
        } else {
            self.compute_formula(cell_reference, formula)
        };
        // The cell may have been found on a cycle while it ran.
        let result = if self.evaluation.circular.contains(&key) {
            circular_reference(cell_reference)
        } else {
            result
        };
        let stored = if self.evaluation.restart.is_some() {
            // Abandoned pass: store nothing.
            Ok(())
        } else {
            self.set_cells_with_result(cell_reference, cell, &result)
        };
        self.evaluation.stack.pop();
        if let Err(message) = stored {
            // Not expected to happen; keep the cell from being evaluated again.
            self.evaluation.cells.insert(key, CellState::Evaluated);
            return CalcResult::new_error(Error::ERROR, cell_reference, message);
        }
        if self.evaluation.restart.is_some() {
            return CalcResult::EmptyCell;
        }
        self.evaluation.cells.insert(key, CellState::Evaluated);
        // Return what was stored, so that a dependent evaluated in this pass
        // sees the same value it would read from the sheet later: the first
        // element of a spilled array, or the error the result was turned into.
        self.stored_value(cell_reference)
    }

    /// Runs the parsed formula and turns a reference into a value or an array.
    fn compute_formula(&mut self, cell_reference: CellReferenceIndex, formula: i32) -> CalcResult {
        let node = self.parsed_formulas[cell_reference.sheet as usize][formula as usize]
            .0
            .clone();
        match self.evaluate_node_in_context(&node, cell_reference) {
            CalcResult::Range { left, right } => {
                if left == right {
                    self.evaluate_cell(left)
                } else {
                    let height = right.row - left.row + 1;
                    let width = right.column - left.column + 1;
                    if cell_reference.row + height - 1 > LAST_ROW
                        || cell_reference.column + width - 1 > LAST_COLUMN
                    {
                        CalcResult::new_error(
                            Error::SPILL,
                            cell_reference,
                            "Spill would exceed worksheet bounds".to_string(),
                        )
                    } else {
                        CalcResult::Array(self.evaluate_range(left, right))
                    }
                }
            }
            CalcResult::Lambda(_) => CalcResult::new_error(
                Error::CALC,
                cell_reference,
                "A LAMBDA was returned but not called".to_string(),
            ),
            result => result,
        }
    }

    /// The value currently stored at a position, empty if there is no cell.
    fn stored_value(&self, cell_reference: CellReferenceIndex) -> CalcResult {
        match self.fetch_cell(cell_reference) {
            Some(cell) => self.get_cell_value(cell, cell_reference),
            None => CalcResult::EmptyCell,
        }
    }

    /// A read closed a loop at `from`: every cell on the stack from `from` to the
    /// top is on the loop. They all store `#CIRC!`, whatever their formula would
    /// have made of the error, so that the verdict does not depend on where the
    /// recursion entered the loop.
    fn mark_cycle(&mut self, from: CellReferenceIndex) {
        let Some(start) = self.evaluation.stack.iter().rposition(|c| *c == from) else {
            return;
        };
        for cell in &self.evaluation.stack[start..] {
            self.evaluation.circular.insert(key(*cell));
        }
    }

    /// Records what the formula being evaluated found at a position, together
    /// with the root of the current recursion. The first record of each kind
    /// is kept. Reads made by the driver itself, or outside a pass, are
    /// nobody's dependency.
    pub(crate) fn record_seen(&mut self, position: CellReferenceIndex, seen: Seen) {
        if !self.evaluation.in_pass {
            return;
        }
        if let Some(&root) = self.evaluation.stack.first() {
            let record = self.evaluation.seen.entry(key(position)).or_default();
            let slot = match seen {
                Seen::Empty => &mut record.empty,
                Seen::Occupied => &mut record.occupied,
            };
            if slot.is_none() {
                *slot = Some(root);
            }
        }
    }

    /// Checks, on behalf of the dynamic anchor about to commit, whether writing
    /// spill cells at `writes` and removing its own spill cells at `clears` would
    /// contradict what a formula read earlier in this pass. If so, the pass is
    /// abandoned so that it can start again with the anchor first, and `true`
    /// is returned: the anchor must not write anything. When every contradicted
    /// read was made on the anchor's own behalf, the anchor is circular, unless
    /// only the removals contradict: then the cells were stale, and they go.
    pub(crate) fn spill_contradicts_a_read(
        &mut self,
        anchor: CellReferenceIndex,
        writes: &[CellKey],
        clears: &[CellKey],
    ) -> bool {
        if !self.evaluation.in_pass {
            return false;
        }
        let seen = &self.evaluation.seen;
        let contradicted_by_writes: Vec<CellReferenceIndex> = writes
            .iter()
            .filter_map(|p| seen.get(p).and_then(|record| record.empty))
            .collect();
        let contradicted_by_clears: Vec<CellReferenceIndex> = clears
            .iter()
            .filter_map(|p| seen.get(p).and_then(|record| record.occupied))
            .collect();
        if contradicted_by_writes.is_empty() && contradicted_by_clears.is_empty() {
            return false;
        }
        let only_own_reads = contradicted_by_writes
            .iter()
            .chain(&contradicted_by_clears)
            .all(|root| *root == anchor);
        self.evaluation.restart = Some(if !only_own_reads {
            Restart::Conflict(anchor)
        } else if contradicted_by_writes.is_empty() {
            Restart::StaleCells(anchor, clears.to_vec())
        } else {
            Restart::SelfContradiction(anchor)
        });
        true
    }
}

fn circular_reference(cell_reference: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::CIRC,
        cell_reference,
        "Circular reference detected".to_string(),
    )
}
