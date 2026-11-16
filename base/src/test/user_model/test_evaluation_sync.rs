#![allow(clippy::unwrap_used)]

use crate::test::user_model::util::new_empty_user_model;
use crate::{EvaluationTracker, UserModel};

/// A model the UI reads, that does not evaluate, and a model that evaluates for
/// it, kept in sync the way a host running the evaluation in a worker would.
struct Pair<'a> {
    ui: UserModel<'a>,
    worker: UserModel<'a>,
}

impl<'a> Pair<'a> {
    fn new() -> Pair<'a> {
        let mut ui = new_empty_user_model();
        let worker = new_empty_user_model();
        ui.pause_evaluation();
        Pair { ui, worker }
    }

    /// Sends the changes made in the UI model so far to the worker
    fn send(&mut self) -> Vec<u8> {
        self.ui.flush_send_queue()
    }

    /// What the worker does with a batch of changes: apply them without
    /// evaluating, remember what the UI model holds now, evaluate, and return
    /// what changed.
    fn evaluate(&mut self, batch: &[Vec<u8>]) -> Vec<u8> {
        self.worker.pause_evaluation();
        for diffs in batch {
            self.worker.apply_external_diffs(diffs).unwrap();
        }
        self.worker.resume_evaluation();
        let mut tracker = EvaluationTracker::new(&self.worker);
        self.worker.evaluate();
        tracker.take_results(&self.worker)
    }

    /// A full round trip
    fn sync(&mut self) {
        let diffs = self.send();
        let results = self.evaluate(&[diffs]);
        self.ui.apply_evaluation_results(&results).unwrap();
    }

    /// Asserts both models show the same in the top left corner of every sheet.
    fn assert_same(&self) {
        let sheets = self.worker.get_worksheets_properties().len() as u32;
        assert_eq!(self.ui.get_worksheets_properties().len() as u32, sheets);
        for sheet in 0..sheets {
            for row in 1..=12 {
                for column in 1..=8 {
                    assert_eq!(
                        self.ui.get_formatted_cell_value(sheet, row, column),
                        self.worker.get_formatted_cell_value(sheet, row, column),
                        "sheet {sheet}, row {row}, column {column}"
                    );
                }
            }
        }
    }

    fn value(&self, sheet: u32, row: i32, column: i32) -> String {
        self.ui
            .get_formatted_cell_value(sheet, row, column)
            .unwrap()
    }
}

#[test]
fn formula_results_reach_the_paused_model() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "2").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1*10").unwrap();
    pair.ui.set_user_input(0, 1, 3, "=B1+A1").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "20");
    assert_eq!(pair.value(0, 1, 3), "22");

    // Changing a value updates every formula that depends on it
    pair.ui.set_user_input(0, 1, 1, "5").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "50");
    assert_eq!(pair.value(0, 1, 3), "55");
    pair.assert_same();
}

#[test]
fn paused_model_does_not_evaluate() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "2").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1*10").unwrap();
    pair.sync();
    pair.ui.set_user_input(0, 1, 1, "3").unwrap();
    // Until the results arrive the UI keeps the previous value
    assert_eq!(pair.value(0, 1, 2), "20");
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "30");
}

#[test]
fn only_changed_cells_are_sent() {
    let mut pair = Pair::new();
    for row in 1..=50 {
        pair.ui.set_user_input(0, row, 1, &row.to_string()).unwrap();
        pair.ui
            .set_user_input(0, row, 2, &format!("=A{row}*2"))
            .unwrap();
    }
    pair.sync();
    pair.ui.set_user_input(0, 7, 1, "100").unwrap();
    let diffs = pair.send();
    let one_change = pair.evaluate(&[diffs]);
    let nothing = pair.evaluate(&[]);
    assert!(nothing.len() < one_change.len());
    pair.ui.apply_evaluation_results(&one_change).unwrap();
    assert_eq!(pair.value(0, 7, 2), "200");
    pair.ui.apply_evaluation_results(&nothing).unwrap();
    pair.assert_same();
}

#[test]
fn spill_cells_grow_and_shrink() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "3").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=SEQUENCE(A1)").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 3, 2), "3");
    assert_eq!(pair.value(0, 4, 2), "");

    pair.ui.set_user_input(0, 1, 1, "5").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 5, 2), "5");

    pair.ui.set_user_input(0, 1, 1, "2").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 2, 2), "2");
    assert_eq!(pair.value(0, 3, 2), "");
    assert_eq!(pair.value(0, 5, 2), "");
    pair.assert_same();
}

#[test]
fn blocked_spill_and_undo() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "=SEQUENCE(4)").unwrap();
    pair.sync();
    // Writing into the spill area blocks the array
    pair.ui.set_user_input(0, 3, 1, "Bong!").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 1), "#SPILL!");
    assert_eq!(pair.value(0, 3, 1), "Bong!");
    pair.assert_same();

    pair.ui.undo().unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 1), "1");
    assert_eq!(pair.value(0, 3, 1), "3");
    pair.assert_same();

    pair.ui.redo().unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 1), "#SPILL!");
    pair.assert_same();
}

#[test]
fn sheets_added_and_deleted() {
    let mut pair = Pair::new();
    pair.ui.new_sheet().unwrap();
    pair.ui.set_user_input(1, 1, 1, "7").unwrap();
    pair.ui.set_user_input(0, 1, 1, "=Sheet2!A1*3").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 1), "21");

    // Deleting the first sheet moves the second one to index 0: results are
    // matched by sheet id, not by index.
    pair.ui.new_sheet().unwrap();
    pair.ui.set_user_input(2, 1, 1, "=Sheet2!A1+1").unwrap();
    pair.ui.delete_sheet(0).unwrap();
    pair.ui.set_user_input(0, 1, 1, "10").unwrap();
    pair.sync();
    assert_eq!(pair.value(1, 1, 1), "11");
    pair.assert_same();
}

#[test]
fn errors_and_text_results() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "0").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=1/A1").unwrap();
    pair.ui
        .set_user_input(0, 1, 3, "=IF(A1>0,\"positive\",\"not positive\")")
        .unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "#DIV/0!");
    assert_eq!(pair.value(0, 1, 3), "not positive");
    pair.ui.set_user_input(0, 1, 1, "4").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "0.25");
    assert_eq!(pair.value(0, 1, 3), "positive");
    pair.assert_same();
}

#[test]
fn results_go_under_the_changes_made_meanwhile() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "2").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1*10").unwrap();
    pair.ui.set_user_input(0, 1, 3, "=SEQUENCE(3)*A1").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 2, 3), "4");

    // The worker evaluates a change that touches B1 and the spill cells...
    pair.ui.set_user_input(0, 1, 1, "4").unwrap();
    let diffs = pair.send();
    let late = pair.evaluate(&[diffs]);
    // ...while the user replaces the formula and writes over a spill cell,
    // which blocks the array and clears the rest of its spill.
    pair.ui.set_user_input(0, 1, 2, "typed").unwrap();
    pair.ui.set_user_input(0, 2, 3, "mine").unwrap();
    pair.ui.apply_evaluation_results(&late).unwrap();
    assert_eq!(pair.value(0, 1, 2), "typed");
    assert_eq!(pair.value(0, 2, 3), "mine");
    assert_eq!(pair.value(0, 3, 3), "");

    // What the user did meanwhile is still to be sent, and is sent next
    pair.sync();
    assert_eq!(pair.value(0, 1, 3), "#SPILL!");
    pair.assert_same();

    // Undoing it while the next results are on their way
    pair.ui.undo().unwrap();
    let diffs = pair.send();
    let results = pair.evaluate(&[diffs]);
    pair.ui.undo().unwrap();
    pair.ui.apply_evaluation_results(&results).unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "40");
    assert_eq!(pair.value(0, 3, 3), "12");
    pair.assert_same();
}

#[test]
fn autofill_on_a_paused_model() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "1").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1*2").unwrap();
    for row in 2..=5 {
        pair.ui.set_user_input(0, row, 1, &row.to_string()).unwrap();
    }
    pair.sync();
    let area = crate::expressions::types::Area {
        sheet: 0,
        row: 1,
        column: 2,
        width: 1,
        height: 1,
    };
    pair.ui.auto_fill_rows(&area, 5).unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 5, 2), "10");
    pair.assert_same();
}

#[test]
fn formula_replaced_and_typed_back_in_one_batch() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "2").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1*10").unwrap();
    pair.sync();
    assert_eq!(pair.value(0, 1, 2), "20");

    // The user replaces the formula and types it back before the worker gets
    // to evaluate: for the worker nothing changed, but the UI model holds the
    // formula unevaluated.
    pair.ui.set_user_input(0, 1, 2, "=A1*3").unwrap();
    let first = pair.send();
    pair.ui.set_user_input(0, 1, 2, "=A1*10").unwrap();
    let second = pair.send();
    let results = pair.evaluate(&[first, second]);
    pair.ui.apply_evaluation_results(&results).unwrap();
    assert_eq!(pair.value(0, 1, 2), "20");
    pair.assert_same();
}

#[test]
fn user_keeps_typing_while_the_worker_evaluates() {
    let mut pair = Pair::new();
    pair.ui.set_user_input(0, 1, 1, "1").unwrap();
    pair.ui.set_user_input(0, 1, 2, "=A1+1").unwrap();
    pair.ui.set_user_input(0, 2, 2, "=B1*2").unwrap();
    pair.sync();
    pair.ui.set_user_input(0, 1, 1, "10").unwrap();
    let first = pair.send();
    let first_results = pair.evaluate(&[first]);
    // Two more changes before the results arrive
    pair.ui.set_user_input(0, 1, 1, "100").unwrap();
    pair.ui.set_user_input(0, 3, 2, "=B2+1").unwrap();
    pair.ui.apply_evaluation_results(&first_results).unwrap();
    // Results for A1=10, the user's 100 kept
    assert_eq!(pair.value(0, 2, 2), "22");
    assert_eq!(pair.value(0, 1, 1), "100");
    pair.sync();
    assert_eq!(pair.value(0, 2, 2), "202");
    assert_eq!(pair.value(0, 3, 2), "203");
    pair.assert_same();
}

#[test]
fn wrong_results() {
    let mut model = new_empty_user_model();
    assert!(model.apply_evaluation_results(b"not results").is_err());
}

#[test]
fn last_evaluation_duration() {
    let mut model = new_empty_user_model();
    assert_eq!(model.last_evaluation_duration(), None);

    // A change made while paused is not evaluated
    model.pause_evaluation();
    model.set_user_input(0, 1, 1, "=1+1").unwrap();
    assert_eq!(model.last_evaluation_duration(), None);

    // The clock is mocked in tests: it does not move during the evaluation
    model.resume_evaluation();
    model.set_user_input(0, 1, 2, "=A1+1").unwrap();
    assert_eq!(model.last_evaluation_duration(), Some(0));

    let mut other = new_empty_user_model();
    other.evaluate();
    assert_eq!(other.last_evaluation_duration(), Some(0));
}
