use std::collections::HashMap;

use crate::expressions::utils::{is_valid_column_number, is_valid_row};
use crate::types::Cell;

/// The spilled cells of one sheet, by ordinal `(row, column)`.
#[derive(Clone, Debug, Default)]
pub struct Spills(HashMap<(i32, i32), Cell>);

/// Derived state is never compared: two replicas agree when their *authored* state does.
impl PartialEq for Spills {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Spills {
    pub(crate) fn get(&self, row: i32, column: i32) -> Option<&Cell> {
        self.0.get(&(row, column))
    }

    pub(crate) fn insert(&mut self, row: i32, column: i32, cell: Cell) -> Result<(), String> {
        if !is_valid_row(row) || !is_valid_column_number(column) {
            return Err("Incorrect row or column".to_string());
        }
        self.0.insert((row, column), cell);
        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
pub(crate) mod test {
    #![allow(clippy::unwrap_used)]
    use crate::collab::model::{CollabModel, Stable};
    use crate::types::Position;
    use crate::user_model::CellArrayStructure;
    use crate::UserModel;

    pub(crate) type Peer = UserModel<'static, Stable>;

    pub(crate) fn peer(session: u32) -> Peer {
        UserModel::<Stable>::from_model(CollabModel::new(session))
    }

    pub(crate) fn deliver(from: &mut Peer, to: &mut Peer) {
        to.apply_external_diffs(&from.flush_send_queue()).unwrap();
    }

    /// Both replicas show the same values and the same array geometry over a small rectangle.
    pub(crate) fn converged(a: &Peer, b: &Peer) {
        for row in 1..=6 {
            for col in 1..=4 {
                assert_eq!(
                    b.get_formatted_cell_value(0, row, col),
                    a.get_formatted_cell_value(0, row, col),
                    "value at row {row} column {col}"
                );
                assert_eq!(
                    b.get_cell_array_structure(0, row, col),
                    a.get_cell_array_structure(0, row, col),
                    "structure at row {row} column {col}"
                );
            }
        }
    }

    fn structure(model: &Peer, row: i32, column: i32) -> CellArrayStructure {
        model.get_cell_array_structure(0, row, column).unwrap()
    }

    #[test]
    fn spill_and_block_round_trip() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        // Only row 1 is materialized: A2 and A3 have no keys of their own.
        a.set_user_input(0, 1, 1, "=SEQUENCE(3)").unwrap(); // A1=1, A2=2, A3=3
        a.set_user_input(0, 1, 3, "=SUM(A1#)").unwrap(); // C1=6
        deliver(&mut a, &mut b);

        for model in [&a, &b] {
            assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "1"); // A1=1
            assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "2"); // A2=2 (spill)
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3=3 (spill)
            assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "6"); // C1=SUM(A1#)
            assert_eq!(
                structure(model, 1, 1),
                CellArrayStructure::DynamicAnchor(1, 3)
            );
            assert_eq!(
                structure(model, 2, 1),
                CellArrayStructure::DynamicChild(1, 1, 1, 3)
            );
            let inner = model.get_model();
            assert!(inner.get_cell_formula(0, 1, 1).unwrap().is_some());
            assert_eq!(inner.get_cell_formula(0, 2, 1).unwrap(), None);
        }
        converged(&a, &b);

        b.set_user_input(0, 3, 1, "x").unwrap(); // Peer B: A3=x
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            // spill blocked by B's A3=x write
            assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#SPILL!");
            // A2 contents are results of A1 spill - they have no value on their own
            assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "");
            // A3 was written by peer B
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "x");
        }
        converged(&a, &b);

        b.set_user_input(0, 3, 1, "").unwrap(); // unblock A1 spill
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3=3
        }
        converged(&a, &b);

        a.set_user_input(0, 1, 1, "42").unwrap(); // A1=42
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "42");
            // A2 and A3 values were results of A1 spill, once it's gone they're empty
            assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "");
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "");
            assert_eq!(structure(model, 2, 1), CellArrayStructure::SingleCell);
        }
        converged(&a, &b);
    }

    #[test]
    fn concurrent_edit_versus_spill_both_orders() {
        fn execute(anchor_first: bool) {
            let (mut a, mut b) = (peer(1), peer(2));
            a.new_sheet().unwrap();
            deliver(&mut a, &mut b);

            a.set_user_input(0, 1, 1, "=SEQUENCE(2)").unwrap(); // A: A1=1, A2=2
            b.set_user_input(0, 2, 1, "5").unwrap(); // B: A2=5 (interferes with spill)
            let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
            if anchor_first {
                b.apply_external_diffs(&from_a).unwrap();
                a.apply_external_diffs(&from_b).unwrap();
            } else {
                a.apply_external_diffs(&from_b).unwrap();
                b.apply_external_diffs(&from_a).unwrap();
            }

            for model in [&a, &b] {
                // regardless of order of exchange, spill area is blocked by B's write
                assert_eq!(
                    model.get_formatted_cell_value(0, 1, 1).unwrap(),
                    "#SPILL!",
                    "anchor first: {anchor_first}"
                );
                assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "5");
            }
            converged(&a, &b);
        }

        execute(true);
        execute(false);
    }

    #[test]
    fn structural_moves_carry_the_spill() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        a.set_user_input(0, 2, 1, "=SEQUENCE(3)").unwrap(); // A2=1, A3=2, A4=3
        deliver(&mut a, &mut b);
        converged(&a, &b);

        b.insert_rows(0, 1, 1).unwrap(); // insert row before A1 (which holds array formula)
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            // after B's row insert, spills should shift down together with their array formula
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "1"); // A3=1
            assert_eq!(model.get_formatted_cell_value(0, 4, 1).unwrap(), "2"); // A4=2
            assert_eq!(model.get_formatted_cell_value(0, 5, 1).unwrap(), "3"); // A5=3
            assert_eq!(
                structure(model, 3, 1),
                CellArrayStructure::DynamicAnchor(1, 3)
            );
        }
        converged(&a, &b);

        b.delete_rows(0, 3, 1).unwrap(); // delete row which has array formula
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            // after deleting their anchor, spill cells should return to regular behavior
            for row in 1..=6 {
                assert_eq!(
                    structure(model, row, 1),
                    CellArrayStructure::SingleCell,
                    "row {row}"
                );
            }
        }
        converged(&a, &b);
    }

    #[test]
    fn snapshot_rebuilds_the_spill() {
        let mut author = UserModel::new_empty_with_session("book", "en", "UTC", "en", 1).unwrap();
        author.set_user_input(0, 1, 1, "=SEQUENCE(3)").unwrap(); // A1=1, A2=2, A3=3
        assert_eq!(author.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3=3

        let mut restored =
            UserModel::<Stable>::from_bytes_with_session(&author.to_bytes(), 2).unwrap();
        // The overlay is not in the payload; only an evaluation can put the spill back.
        restored.evaluate();
        assert_eq!(restored.get_formatted_cell_value(0, 1, 1).unwrap(), "1"); // A1=1
        assert_eq!(restored.get_formatted_cell_value(0, 3, 1).unwrap(), "3"); // A3=3
        let sheet = &restored.get_model().workbook.worksheets[0];
        // row 3 was never got explicit key, so it was not materialized
        // the only reason why it shows value, is because we track spill overlays
        assert_eq!(Stable::row_at(&sheet.index, 3), None);
    }
}
