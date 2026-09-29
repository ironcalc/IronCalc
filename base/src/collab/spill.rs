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

    /// Drops the cells `anchor` spilled over its `(width, height)` range; whatever another anchor
    /// spilled there stays.
    pub(crate) fn remove_spill(&mut self, anchor: (i32, i32), (width, height): (i32, i32)) {
        for row in anchor.0..anchor.0 + height {
            for column in anchor.1..anchor.1 + width {
                if let Some(Cell::SpillCell { a, .. }) = self.0.get(&(row, column)) {
                    if *a == anchor {
                        self.0.remove(&(row, column));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    #![allow(clippy::unwrap_used)]
    use crate::collab::model::{CollabModel, Stable};
    use crate::types::{ArrayKind, Cell, Position};
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
    fn markup_is_the_same_on_every_replica() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        a.set_user_input(0, 1, 1, "=SEQUENCE(3)").unwrap(); // A: A1:A3=1,2,3
        a.set_user_input(0, 1, 3, "=SUM(A:A)").unwrap(); // A: C1
        deliver(&mut a, &mut b);
        b.set_user_array_formula(0, 5, 1, 2, 2, "=1+1").unwrap(); // B: A5:B6=2
        deliver(&mut b, &mut a);

        let expected = "=SEQUENCE(3)||=SUM(A:A)\n2||\n3||\n||\n=1+1|2|\n2|2|";
        for model in [&a, &b] {
            assert_eq!(model.get_model().get_sheet_markup(0).unwrap(), expected);
        }

        // Blocked, an array is its anchor alone: what blocks it shows instead.
        a.set_user_input(0, 2, 1, "x").unwrap(); // A: A2=x (blocks sequence)
        deliver(&mut a, &mut b);
        let expected = "=SEQUENCE(3)||=SUM(A:A)\nx||\n||\n||\n=1+1|2|\n2|2|";
        for model in [&a, &b] {
            assert_eq!(model.get_model().get_sheet_markup(0).unwrap(), expected);
        }
    }

    #[test]
    fn dimension_covers_spilled_cells() {
        use crate::worksheet::WorksheetDimension;

        fn dimension(model: &Peer) -> WorksheetDimension {
            model.get_model().workbook.worksheets[0].dimension()
        }
        fn rows_and_columns(max_row: i32, max_column: i32) -> WorksheetDimension {
            WorksheetDimension {
                min_row: 1,
                min_column: 1,
                max_row,
                max_column,
            }
        }
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        a.set_user_input(0, 1, 1, "=SEQUENCE(3)").unwrap(); // A1:A3=1,2,3 (3x1)
        a.set_user_input(0, 1, 3, "=SUM(A:A)").unwrap(); // C1 (3x3)
        a.set_user_input(0, 2, 3, "=COUNT(A:A)").unwrap(); // C2 (3x3)
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(dimension(model), rows_and_columns(3, 3));
            assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "6");
            assert_eq!(model.get_formatted_cell_value(0, 2, 3).unwrap(), "3");
        }

        b.set_user_array_formula(0, 5, 4, 2, 3, "=1+1").unwrap(); // D5:E7=2 (7x5)
        b.set_user_input(0, 3, 3, "=SUM(E:E)").unwrap(); // C3
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            assert_eq!(dimension(model), rows_and_columns(7, 5));
            assert_eq!(model.get_formatted_cell_value(0, 3, 3).unwrap(), "6");
        }

        // Blocked, the dynamic array draws its anchor alone. Freed, it reaches as far again.
        b.set_user_input(0, 2, 1, "7").unwrap(); // A2=7
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#SPILL!");
            assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "#SPILL!");
        }
        b.set_user_input(0, 2, 1, "").unwrap();
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            assert_eq!(dimension(model), rows_and_columns(7, 5));
            assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "6");
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

    /// B1:B3 = 5, 10, 15 and the CSE array A1:A3 = B1:B3*2 on `a`, flushed.
    fn cse_init(a: &mut Peer) {
        for row in 1..=3 {
            a.set_user_input(0, row, 2, &format!("{}", row * 5))
                .unwrap();
        }
        a.set_user_array_formula(0, 1, 1, 1, 3, "=B1:B3*2").unwrap(); // A1:A3 = B1:B3*2
    }

    #[test]
    fn cse_replicates() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        cse_init(&mut a);
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10"); // A1=10
            assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "20"); // A2=20
            assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "30"); // A3=30
            assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=B1:B3*2");
            assert_eq!(
                structure(model, 1, 1),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            assert_eq!(
                structure(model, 3, 1),
                CellArrayStructure::ArrayChild(1, 1, 1, 3)
            );
        }
        converged(&a, &b);
        b.flush_send_queue();

        assert!(b.set_user_input(0, 2, 1, "7").is_err()); // A2=7 (denied)
        assert!(b.get_model().local.pending.is_empty()); // write denied, no updates

        // A scalar formula over a range is still an array: it fills the range.
        a.set_user_array_formula(0, 5, 3, 2, 2, "=1+1").unwrap(); // C5:D6

        // Read before the anchor in sheet order: the covered cell is there all the same.
        a.set_user_input(0, 1, 3, "=D6*3").unwrap(); // C1=6
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            for (row, col) in [(5, 3), (5, 4), (6, 3), (6, 4)] {
                assert_eq!(model.get_formatted_cell_value(0, row, col).unwrap(), "2");
            }
            assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "6");
        }
        converged(&a, &b);
    }

    #[test]
    fn cse_blocked_by_concurrent_write() {
        fn execute(clear_on_author: bool) {
            let (mut a, mut b) = (peer(1), peer(2));
            a.new_sheet().unwrap();
            deliver(&mut a, &mut b);

            cse_init(&mut a); // A: A1:A3=B1:B3*2
            b.set_user_input(0, 2, 1, "7").unwrap(); // B: A2=7, before A's array arrives
            let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
            b.apply_external_diffs(&from_a).unwrap();
            a.apply_external_diffs(&from_b).unwrap();
            for model in [&a, &b] {
                // on collab, CSE conflict works like dynamic array
                assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "#SPILL!");
                assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "7");
                assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "");
            }
            converged(&a, &b);

            let (from, to) = if clear_on_author {
                (&mut a, &mut b)
            } else {
                (&mut b, &mut a)
            };
            from.set_user_input(0, 2, 1, "").unwrap(); // unblock the array
            deliver(from, to);
            for model in [&a, &b] {
                assert_eq!(model.get_formatted_cell_value(0, 1, 1).unwrap(), "10");
                assert_eq!(model.get_formatted_cell_value(0, 2, 1).unwrap(), "20");
                assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "30");
            }
            converged(&a, &b);
        }

        execute(true);
        execute(false);
    }

    /// An array against a plain value written to its anchor concurrently. The anchor is one
    /// register: the later write stands, and nothing of the other is left.
    #[test]
    fn cse_anchor_is_one_register() {
        fn execute(array_last: bool) {
            let (mut a, mut b) = (peer(1), peer(2));
            a.new_sheet().unwrap();
            deliver(&mut a, &mut b);

            // The peers share this process's clock: the later call carries the later stamp.
            if array_last {
                b.set_user_input(0, 1, 1, "x").unwrap(); // B: A1=x
                a.set_user_array_formula(0, 1, 1, 1, 2, "=1+1").unwrap(); // A: A1:A2=1+1 (CSE)
            } else {
                a.set_user_array_formula(0, 1, 1, 1, 2, "=1+1").unwrap(); // A: A1:A2=1+1 (CSE)
                b.set_user_input(0, 1, 1, "x").unwrap(); // B: A1=x
            }
            let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
            b.apply_external_diffs(&from_a).unwrap();
            a.apply_external_diffs(&from_b).unwrap();

            let (values, anchor, below) = if array_last {
                (
                    ["2", "2"],
                    CellArrayStructure::ArrayAnchor(1, 2),
                    CellArrayStructure::ArrayChild(1, 1, 1, 2),
                )
            } else {
                (
                    ["x", ""],
                    CellArrayStructure::SingleCell,
                    CellArrayStructure::SingleCell,
                )
            };
            for model in [&a, &b] {
                let shown = [
                    model.get_formatted_cell_value(0, 1, 1).unwrap(),
                    model.get_formatted_cell_value(0, 2, 1).unwrap(),
                ];
                assert_eq!(shown, values, "array last: {array_last}");
                assert_eq!(structure(model, 1, 1), anchor, "array last: {array_last}");
                assert_eq!(structure(model, 2, 1), below, "array last: {array_last}");
            }
            converged(&a, &b);
        }

        execute(true);
        execute(false);
    }

    #[test]
    fn cse_undo() {
        use crate::collab::patch::{invert_patches, Patch};

        let mut model = CollabModel::new(1);
        model.new_sheet();
        for row in 1..=3 {
            model.set_user_input(0, row, 1, format!("{row}")).unwrap(); // Ax=x
        }
        model.evaluate();
        model.flush();

        model.set_user_array_formula(0, 1, 1, 1, 3, "=10").unwrap(); // A1:A3=10
        model.evaluate();
        assert_eq!(model.get_formatted_cell_value(0, 3, 1).unwrap(), "10"); // A3=10
        let commits = model.flush();
        assert_eq!(commits.len(), 1);

        let inverse: Vec<Patch> = commits
            .iter()
            .rev()
            .flat_map(|commit| invert_patches(&commit.patches))
            .collect();
        model.commit_local(inverse);
        model.evaluate();
        for row in 1..=3 {
            assert_eq!(
                model.get_formatted_cell_value(0, row, 1).unwrap(),
                format!("{row}")
            ); // Ax == x
            assert!(matches!(
                model.workbook.worksheets[0].cell(row, 1),
                Some(Cell::NumberCell { .. })
            ));
        }
    }

    #[test]
    fn cse_survives_structural_edits() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        cse_init(&mut a);
        deliver(&mut a, &mut b);

        // Concurrently: A inserts a row inside the array, B deletes one.
        a.insert_rows(0, 2, 1).unwrap();
        b.delete_rows(0, 3, 1).unwrap();
        let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
        b.apply_external_diffs(&from_a).unwrap();
        a.apply_external_diffs(&from_b).unwrap();
        a.evaluate();
        b.evaluate();
        converged(&a, &b);
        for model in [&a, &b] {
            // The array is `height` rows from wherever its anchor sits now.
            assert_eq!(
                structure(model, 1, 1),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            for row in 2..=3 {
                assert_eq!(
                    structure(model, row, 1),
                    CellArrayStructure::ArrayChild(1, 1, 1, 3)
                );
            }
            assert_eq!(structure(model, 4, 1), CellArrayStructure::SingleCell);
        }
    }

    #[test]
    fn cse_survives_snapshot_and_sheet_copy() {
        let mut author = UserModel::new_empty_with_session("book", "en", "UTC", "en", 1).unwrap();
        cse_init(&mut author);
        author.duplicate_sheet(0).unwrap();
        for sheet in 0..2 {
            assert_eq!(
                author.get_cell_array_structure(sheet, 1, 1).unwrap(),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            assert_eq!(author.get_formatted_cell_value(sheet, 3, 1).unwrap(), "30");
        }

        let mut restored =
            UserModel::<Stable>::from_bytes_with_session(&author.to_bytes(), 2).unwrap();
        restored.evaluate();
        for sheet in 0..2 {
            assert!(matches!(
                restored.get_model().workbook.worksheets[sheet as usize].cell(1, 1),
                Some(Cell::ArrayFormula {
                    kind: ArrayKind::Cse,
                    r: (1, 3),
                    ..
                })
            ));
            assert_eq!(
                restored.get_formatted_cell_value(sheet, 3, 1).unwrap(),
                "30"
            );
        }
    }

    #[test]
    fn cse_extent_from_the_wire_is_clamped() {
        use crate::collab::patch::{CellInput, Patch};

        let mut a = CollabModel::new(1);
        a.new_sheet();
        a.set_user_array_formula(0, 1, 1, 1, 2, "=1+1").unwrap(); // A1:A2=1+1
        let mut commits = a.flush();
        for commit in &mut commits {
            for patch in &mut commit.patches {
                if let Patch::SetCellValue {
                    value: Some(CellInput::Cse { width, height, .. }),
                    ..
                } = patch
                {
                    (*width, *height) = (0, -3);
                }
            }
        }
        let mut b = CollabModel::new(2);
        b.apply_batch(&commits).unwrap();
        b.evaluate();
        assert!(matches!(
            b.workbook.worksheets[0].cell(1, 1),
            Some(Cell::ArrayFormula {
                kind: ArrayKind::Cse,
                r: (1, 1),
                ..
            })
        ));
        assert_eq!(b.get_formatted_cell_value(0, 1, 1).unwrap(), "2"); // A1 == 2
    }

    /// The values of A1..A4, as shown.
    fn a1_to_a4(model: &Peer) -> Vec<String> {
        (1..=4)
            .map(|row| model.get_formatted_cell_value(0, row, 1).unwrap())
            .collect()
    }

    /// Exchanges what both peers did concurrently: each applies its own work first.
    fn exchange(a: &mut Peer, b: &mut Peer) {
        let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
        b.apply_external_diffs(&from_a).unwrap();
        a.apply_external_diffs(&from_b).unwrap();
    }

    #[test]
    fn cse_undo_redo_converges() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        for row in 1..=3 {
            a.set_user_input(0, row, 1, &format!("{row}")).unwrap(); // A1:A3=1,2,3
        }
        cse_init(&mut a); // over what A1:A3 held
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["10", "20", "30", ""]);
        }

        // One step takes the array away and puts back what it covered, on both.
        a.undo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["1", "2", "3", ""]);
            for row in 1..=3 {
                assert_eq!(structure(model, row, 1), CellArrayStructure::SingleCell);
            }
            // Nothing is locked any more.
            assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "1");
        }
        converged(&a, &b);

        a.redo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["10", "20", "30", ""]);
            assert_eq!(
                structure(model, 1, 1),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            assert_eq!(
                structure(model, 2, 1),
                CellArrayStructure::ArrayChild(1, 1, 1, 3)
            );
        }
        converged(&a, &b);
        assert!(b.set_user_input(0, 2, 1, "7").is_err()); // locked again

        a.undo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["1", "2", "3", ""]);
        }
        converged(&a, &b);
    }

    #[test]
    fn cse_undo_keeps_a_concurrent_write_into_the_range() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        deliver(&mut a, &mut b);

        cse_init(&mut a); // A: A1:A3=B1:B3*2
        b.set_user_input(0, 2, 1, "7").unwrap(); // B: A2=7, before the array arrives
        exchange(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["#SPILL!", "7", "", ""]);
        }

        a.undo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["", "7", "", ""]);
            assert_eq!(structure(model, 1, 1), CellArrayStructure::SingleCell);
        }
        converged(&a, &b);

        // Redone, the array is back and blocked by the same cell.
        a.redo().unwrap();
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["#SPILL!", "7", "", ""]);
        }
        converged(&a, &b);
    }

    #[test]
    fn undo_of_the_blocking_write_restores_the_cse() {
        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        deliver(&mut a, &mut b);

        cse_init(&mut a);
        b.set_user_input(0, 2, 1, "7").unwrap(); // A2=7
        exchange(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["#SPILL!", "7", "", ""]);
        }

        b.undo().unwrap(); // undo A2=7, unlocks CSE
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            // CSE works back
            assert_eq!(a1_to_a4(model), ["10", "20", "30", ""]);
            assert_eq!(
                structure(model, 2, 1),
                CellArrayStructure::ArrayChild(1, 1, 1, 3)
            );
        }
        converged(&a, &b);
    }

    #[test]
    fn cse_undo_after_a_peer_cleared_the_range() {
        use crate::expressions::types::Area;

        let (mut a, mut b) = (peer(1), peer(2));
        a.new_sheet().unwrap();
        for row in 1..=3 {
            a.set_user_input(0, row, 1, &format!("{row}")).unwrap(); // A1,A2,A3=1,2,3
        }
        cse_init(&mut a);
        deliver(&mut a, &mut b);

        let area = Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 1,
            height: 3,
        };
        b.range_clear_contents(&area).unwrap(); // clear A1:A3 (wins)
        deliver(&mut b, &mut a);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["", "", "", ""]);
            assert_eq!(structure(model, 1, 1), CellArrayStructure::SingleCell);
        }

        a.undo().unwrap(); // undo CSE, brings back A1,A2,A3=1,2,3
        deliver(&mut a, &mut b);
        for model in [&a, &b] {
            assert_eq!(a1_to_a4(model), ["1", "2", "3", ""]);
            assert_eq!(structure(model, 1, 1), CellArrayStructure::SingleCell);
        }
        converged(&a, &b);
    }

    #[test]
    fn cse_overlapping_arrays_converge() {
        let (mut a, mut b, mut c) = (peer(1), peer(2), peer(3));
        a.new_sheet().unwrap();
        let sheet = a.flush_send_queue();
        b.apply_external_diffs(&sheet).unwrap();
        c.apply_external_diffs(&sheet).unwrap();

        a.set_user_array_formula(0, 1, 1, 1, 3, "=1+1").unwrap(); // A: A1:A3=1+1
        b.set_user_array_formula(0, 2, 1, 1, 3, "=2+2").unwrap(); // B: A2:A4=2+2
        let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
        b.apply_external_diffs(&from_a).unwrap();
        a.apply_external_diffs(&from_b).unwrap();
        // The third peer sees B's first.
        c.apply_external_diffs(&from_b).unwrap();
        c.apply_external_diffs(&from_a).unwrap();

        for model in [&a, &b, &c] {
            // A1:A3 is locked by A2:A4, since A2 (anchor) is in A1's spill range
            assert_eq!(a1_to_a4(model), ["#SPILL!", "4", "4", "4"]);
            assert_eq!(
                structure(model, 1, 1),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            assert_eq!(
                structure(model, 2, 1),
                CellArrayStructure::ArrayAnchor(1, 3)
            );
            assert_eq!(
                structure(model, 4, 1),
                CellArrayStructure::ArrayChild(2, 1, 1, 3)
            );
        }
        converged(&a, &b);
        converged(&a, &c);
    }

    #[test]
    fn cse_same_anchor_converges_on_one_extent() {
        fn execute(vertical_last: bool) {
            let (mut a, mut b, mut c) = (peer(1), peer(2), peer(3));
            a.new_sheet().unwrap();
            let sheet = a.flush_send_queue();
            b.apply_external_diffs(&sheet).unwrap();
            c.apply_external_diffs(&sheet).unwrap();

            // The peers share this process's clock: the later call carries the later stamp.
            if vertical_last {
                b.set_user_array_formula(0, 1, 1, 3, 1, "=2+2").unwrap(); // B: A1:C1, across
                a.set_user_array_formula(0, 1, 1, 1, 3, "=1+1").unwrap(); // A: A1:A3, down
            } else {
                a.set_user_array_formula(0, 1, 1, 1, 3, "=1+1").unwrap(); // A: A1:A3, down
                b.set_user_array_formula(0, 1, 1, 3, 1, "=2+2").unwrap(); // B: A1:C1, across
            }
            let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
            b.apply_external_diffs(&from_a).unwrap();
            a.apply_external_diffs(&from_b).unwrap();
            // The third peer sees B's first.
            c.apply_external_diffs(&from_b).unwrap();
            c.apply_external_diffs(&from_a).unwrap();

            let (anchor, down, across, below, beside) = if vertical_last {
                (
                    CellArrayStructure::ArrayAnchor(1, 3),
                    ["2", "2", "2", ""],
                    ["2", "", ""],
                    CellArrayStructure::ArrayChild(1, 1, 1, 3),
                    CellArrayStructure::SingleCell,
                )
            } else {
                (
                    CellArrayStructure::ArrayAnchor(3, 1),
                    ["4", "", "", ""],
                    ["4", "4", "4"],
                    CellArrayStructure::SingleCell,
                    CellArrayStructure::ArrayChild(1, 1, 3, 1),
                )
            };
            for model in [&a, &b, &c] {
                let a1_to_c1: Vec<String> = (1..=3)
                    .map(|column| model.get_formatted_cell_value(0, 1, column).unwrap())
                    .collect();
                assert_eq!(structure(model, 1, 1), anchor, "down last: {vertical_last}");
                assert_eq!(a1_to_a4(model), down, "down last: {vertical_last}");
                assert_eq!(a1_to_c1, across, "down last: {vertical_last}");
                assert_eq!(structure(model, 2, 1), below, "down last: {vertical_last}");
                assert_eq!(structure(model, 1, 2), beside, "down last: {vertical_last}");
            }
            converged(&a, &b);
            converged(&a, &c);
        }

        execute(true);
        execute(false);
    }

    #[test]
    fn cse_versus_dynamic_spill_converges() {
        let (mut a, mut b, mut c) = (peer(1), peer(2), peer(3));
        a.new_sheet().unwrap();
        for row in 1..=3 {
            a.set_user_input(0, row, 4, &format!("{row}")).unwrap(); // D1,D2,D3=1,2,3
        }
        let seed = a.flush_send_queue();
        b.apply_external_diffs(&seed).unwrap();
        c.apply_external_diffs(&seed).unwrap();

        a.set_user_array_formula(0, 2, 1, 2, 1, "=1+1").unwrap(); // A: A2:B2=1+1
        b.set_user_input(0, 1, 2, "=D1:D3").unwrap(); // B: B1 spills over B1:B3
        let (from_a, from_b) = (a.flush_send_queue(), b.flush_send_queue());
        b.apply_external_diffs(&from_a).unwrap();
        a.apply_external_diffs(&from_b).unwrap();
        c.apply_external_diffs(&from_b).unwrap();
        c.apply_external_diffs(&from_a).unwrap();
        converged(&a, &b);
        converged(&a, &c);
        // Neither holds the other's anchor, so which one gives way is evaluation order — the same
        // on every replica — and exactly one of them does.
        for model in [&a, &b, &c] {
            let cse = model.get_formatted_cell_value(0, 2, 1).unwrap();
            let dynamic = model.get_formatted_cell_value(0, 1, 2).unwrap();
            assert!(
                (cse == "#SPILL!") != (dynamic == "#SPILL!"),
                "A2 = {cse}, B1 = {dynamic}"
            );
        }
    }
}
