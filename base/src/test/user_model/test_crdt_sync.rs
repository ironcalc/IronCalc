#![allow(clippy::unwrap_used)]

//! Two-replica convergence tests for the CRDT collaboration session.
//!
//! Pattern: both replicas start from the same (empty) workbook, perform
//! concurrent edits, exchange updates, and must end cell-by-cell identical —
//! including evaluation results, which are never shipped.

use crate::cf_types::{CfRule, CfRuleInput, ValueOperator};
use crate::crdt::CollabSession;
use crate::test::util::new_empty_model;
use crate::types::{Color, Dxf, DxfFont};
use crate::UserModel;

struct Replica {
    um: UserModel<'static>,
    session: CollabSession,
}

fn replica(client_id: u64) -> Replica {
    let mut um = UserModel::from_model(new_empty_model());
    let session = CollabSession::attach(&mut um, client_id).unwrap();
    Replica { um, session }
}

/// Exchange all pending updates in both directions.
fn sync(a: &mut Replica, b: &mut Replica) {
    let trace = std::env::var("CRDT_FUZZ_TRACE").is_ok();
    let from_a = a.session.flush_local(&mut a.um).unwrap();
    let from_b = b.session.flush_local(&mut b.um).unwrap();
    if trace {
        a.session.assert_model_matches_shadow(&a.um, "replica A after flush");
        b.session.assert_model_matches_shadow(&b.um, "replica B after flush");
    }
    a.session.apply_remote(&mut a.um, &from_b).unwrap();
    b.session.apply_remote(&mut b.um, &from_a).unwrap();
    if trace {
        a.session.assert_model_matches_shadow(&a.um, "replica A after apply");
        b.session.assert_model_matches_shadow(&b.um, "replica B after apply");
        assert_converged(a, b);
    }
}

const WINDOW_ROWS: i32 = 40;
const WINDOW_COLUMNS: i32 = 15;

/// Asserts both replicas are identical over a viewing window: sheet names,
/// cell contents, formatted (evaluated) values, row heights and hidden flags.
fn assert_converged(a: &Replica, b: &Replica) {
    assert_models_converged(&a.um, &b.um);
}

/// Same as [`assert_converged`], over bare models (shared with the sync-peer
/// tests, whose replicas carry a `SyncPeer` instead of a raw session).
fn assert_models_converged(a: &UserModel, b: &UserModel) {
    struct View<'a, 'b> {
        um: &'b UserModel<'a>,
    }
    let a = View { um: a };
    let b = View { um: b };
    let sheets_a = a.um.model.workbook.worksheets.len();
    let sheets_b = b.um.model.workbook.worksheets.len();
    assert_eq!(sheets_a, sheets_b, "sheet count differs");
    // The model keeps names in insertion order, which may legitimately
    // differ per replica; compare as sets.
    let mut names_a = a.um.get_defined_name_list();
    let mut names_b = b.um.get_defined_name_list();
    names_a.sort();
    names_b.sort();
    assert_eq!(names_a, names_b, "defined names differ");
    assert_eq!(a.um.get_theme(), b.um.get_theme(), "workbook theme differs");
    let mut styles_a = a.um.get_named_style_list();
    let mut styles_b = b.um.get_named_style_list();
    styles_a.sort();
    styles_b.sort();
    assert_eq!(styles_a, styles_b, "named style lists differ");
    for name in &styles_a {
        assert_eq!(
            a.um.get_named_style(name).unwrap(),
            b.um.get_named_style(name).unwrap(),
            "named style {name:?} differs"
        );
    }
    for sheet in 0..sheets_a as u32 {
        let name_a = a.um.model.workbook.worksheet(sheet).unwrap().get_name();
        let name_b = b.um.model.workbook.worksheet(sheet).unwrap().get_name();
        assert_eq!(name_a, name_b, "sheet {sheet} name differs");
        assert_eq!(
            cf_snapshot(a.um, sheet),
            cf_snapshot(b.um, sheet),
            "conditional formatting differs on sheet {sheet}"
        );
        for row in 1..=WINDOW_ROWS {
            for column in 1..=WINDOW_COLUMNS {
                let content_a = a.um.get_cell_content(sheet, row, column).unwrap();
                let content_b = b.um.get_cell_content(sheet, row, column).unwrap();
                assert_eq!(
                    content_a, content_b,
                    "content differs at sheet {sheet} R{row}C{column}"
                );
                let value_a = a.um.get_formatted_cell_value(sheet, row, column).unwrap();
                let value_b = b.um.get_formatted_cell_value(sheet, row, column).unwrap();
                assert_eq!(
                    value_a, value_b,
                    "value differs at sheet {sheet} R{row}C{column}"
                );
            }
            assert_eq!(
                a.um.get_row_height(sheet, row).unwrap(),
                b.um.get_row_height(sheet, row).unwrap(),
                "row {row} height differs on sheet {sheet}"
            );
        }
        for row in 1..=WINDOW_ROWS {
            for column in 1..=WINDOW_COLUMNS {
                assert_eq!(
                    a.um.get_cell_style(sheet, row, column).unwrap(),
                    b.um.get_cell_style(sheet, row, column).unwrap(),
                    "style differs at sheet {sheet} R{row}C{column}"
                );
            }
        }
        for column in 1..=WINDOW_COLUMNS {
            assert_eq!(
                a.um.get_column_width(sheet, column).unwrap(),
                b.um.get_column_width(sheet, column).unwrap(),
                "column {column} width differs on sheet {sheet}"
            );
        }
    }
}

/// Replica-comparable view of a sheet's CF rules: priority order with the
/// replica-local dxf ids replaced by the resolved dxf contents.
fn cf_snapshot(um: &UserModel, sheet: u32) -> Vec<(u32, String, CfRule, Option<Dxf>)> {
    um.get_conditional_formatting_list(sheet)
        .unwrap()
        .into_iter()
        .map(|view| {
            let dxf = um
                .get_dxf_for_conditional_formatting(sheet, view.index as u32)
                .unwrap();
            let mut rule = view.cf_rule;
            rule.set_dxf_id(0);
            (view.priority, view.range, rule, dxf)
        })
        .collect()
}

/// A simple bold-font dxf for CF tests.
fn bold_dxf() -> Dxf {
    Dxf {
        font: Some(DxfFont {
            b: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn cell_is_gt(threshold: &str) -> CfRuleInput {
    CfRuleInput::CellIs {
        operator: ValueOperator::GreaterThan,
        formula: threshold.to_string(),
        formula2: None,
        format: bold_dxf(),
        stop_if_true: false,
    }
}

#[test]
fn late_joiner_receives_full_state() {
    let mut a = replica(1);
    a.um.set_user_input(0, 1, 1, "Hello").unwrap();
    a.um.set_user_input(0, 2, 2, "=1+1").unwrap();

    let mut b = replica(2);
    let sv = b.session.state_vector();
    let update = {
        // A late joiner asks for everything it is missing.
        let _ = a.session.flush_local(&mut a.um).unwrap();
        a.session.encode_state_since(&sv).unwrap()
    };
    b.session.apply_remote(&mut b.um, &update).unwrap();

    assert_eq!(b.um.get_cell_content(0, 1, 1), Ok("Hello".to_string()));
    assert_eq!(b.um.get_formatted_cell_value(0, 2, 2), Ok("2".to_string()));
    assert_converged(&a, &b);
}

#[test]
fn same_cell_concurrent_edit_is_lww_and_order_independent() {
    // Pair 1: A's update applied to B after B's own edit, and vice versa.
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "from A").unwrap();
    b.um.set_user_input(0, 1, 1, "from B").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let winner = a.um.get_cell_content(0, 1, 1).unwrap();
    assert!(winner == "from A" || winner == "from B");

    // Pair 2: same edits, opposite delivery order — same winner.
    let mut c = replica(1);
    let mut d = replica(2);
    sync(&mut c, &mut d);
    c.um.set_user_input(0, 1, 1, "from A").unwrap();
    d.um.set_user_input(0, 1, 1, "from B").unwrap();
    let from_c = c.session.flush_local(&mut c.um).unwrap();
    let from_d = d.session.flush_local(&mut d.um).unwrap();
    // Reversed order of application compared to `sync`.
    d.session.apply_remote(&mut d.um, &from_c).unwrap();
    c.session.apply_remote(&mut c.um, &from_d).unwrap();
    assert_converged(&c, &d);
    assert_eq!(c.um.get_cell_content(0, 1, 1).unwrap(), winner);
}

#[test]
fn concurrent_edits_to_different_cells_both_survive() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "alpha").unwrap();
    b.um.set_user_input(0, 5, 3, "beta").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_cell_content(0, 1, 1), Ok("alpha".to_string()));
    assert_eq!(a.um.get_cell_content(0, 5, 3), Ok("beta".to_string()));
    assert_converged(&a, &b);
}

#[test]
fn edit_lands_on_logical_cell_despite_concurrent_row_insert() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 2, "marker").unwrap();
    sync(&mut a, &mut b);

    // A inserts a row above; B edits the marker cell.
    a.um.insert_rows(0, 2, 1).unwrap();
    b.um.set_user_input(0, 5, 2, "edited").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // The logical cell moved to row 6 and carries B's edit.
    assert_eq!(a.um.get_cell_content(0, 6, 2), Ok("edited".to_string()));
    assert_eq!(a.um.get_cell_content(0, 5, 2), Ok(String::new()));
}

#[test]
fn concurrent_inserts_at_same_index_keep_both_rows() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 3, 1, "orig").unwrap();
    sync(&mut a, &mut b);

    a.um.insert_rows(0, 3, 1).unwrap();
    a.um.set_user_input(0, 3, 1, "a-row").unwrap();
    b.um.insert_rows(0, 3, 1).unwrap();
    b.um.set_user_input(0, 3, 1, "b-row").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // Both inserted rows exist (no duplication, no loss), original shifted by 2.
    let r3 = a.um.get_cell_content(0, 3, 1).unwrap();
    let r4 = a.um.get_cell_content(0, 4, 1).unwrap();
    assert_eq!(a.um.get_cell_content(0, 5, 1), Ok("orig".to_string()));
    let mut both = [r3.as_str(), r4.as_str()];
    both.sort_unstable();
    assert_eq!(both, ["a-row", "b-row"]);
}

#[test]
fn concurrent_delete_of_same_row_is_idempotent() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 4, 1, "goner").unwrap();
    a.um.set_user_input(0, 5, 1, "below").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_rows(0, 4, 1).unwrap();
    b.um.delete_rows(0, 4, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // Exactly one deletion happened.
    assert_eq!(a.um.get_cell_content(0, 4, 1), Ok("below".to_string()));
    assert_eq!(a.um.get_cell_content(0, 5, 1), Ok(String::new()));
}

#[test]
fn delete_row_vs_concurrent_edit_update_wins() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 4, 1, "val").unwrap();
    a.um.set_user_input(0, 5, 1, "below").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_rows(0, 4, 1).unwrap();
    b.um.set_user_input(0, 4, 2, "edited").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // Update-wins: the row survives with ALL its cells, on both replicas.
    assert_eq!(a.um.get_cell_content(0, 4, 1), Ok("val".to_string()));
    assert_eq!(a.um.get_cell_content(0, 4, 2), Ok("edited".to_string()));
    assert_eq!(a.um.get_cell_content(0, 5, 1), Ok("below".to_string()));
}

#[test]
fn delete_row_without_concurrent_edit_stays_deleted() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 4, 1, "goner").unwrap();
    a.um.set_user_input(0, 5, 1, "below").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_rows(0, 4, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(b.um.get_cell_content(0, 4, 1), Ok("below".to_string()));
}

#[test]
fn concurrent_row_height_race_converges() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_rows_height(0, 2, 2, 40.0).unwrap();
    b.um.set_rows_height(0, 2, 2, 55.0).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let height = a.um.get_row_height(0, 2).unwrap();
    assert!(
        (height - 40.0).abs() < 1e-9 || (height - 55.0).abs() < 1e-9,
        "unexpected height {height}"
    );
}

#[test]
fn duplicate_delivery_is_idempotent() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "once").unwrap();
    let update = a.session.flush_local(&mut a.um).unwrap();
    b.session.apply_remote(&mut b.um, &update).unwrap();
    b.session.apply_remote(&mut b.um, &update).unwrap();
    assert_eq!(b.um.get_cell_content(0, 1, 1), Ok("once".to_string()));
    assert_converged(&a, &b);
}

#[test]
fn formulas_are_recomputed_not_shipped() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 1, 1, "10").unwrap();
    a.um.set_user_input(0, 1, 2, "=A1*2").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 2), Ok("20".to_string()));

    // The other replica changes the input; the formula re-evaluates everywhere.
    b.um.set_user_input(0, 1, 1, "50").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(a.um.get_formatted_cell_value(0, 1, 2), Ok("100".to_string()));
    assert_converged(&a, &b);
}

#[test]
fn offline_divergence_converges_in_one_merge() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 10, 1, "anchor").unwrap();
    sync(&mut a, &mut b);

    // Both go "offline" and diverge substantially.
    for i in 1..=20 {
        a.um.set_user_input(0, i, 1, &format!("{}", i * 2)).unwrap();
        b.um.set_user_input(0, i, 3, &format!("b{i}")).unwrap();
    }
    a.um.insert_rows(0, 5, 2).unwrap();
    b.um.delete_rows(0, 8, 1).unwrap();
    b.um.set_rows_height(0, 3, 3, 42.0).unwrap();

    sync(&mut a, &mut b);
    assert_converged(&a, &b);
}

#[test]
fn new_sheet_and_concurrent_edit() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);

    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "second sheet").unwrap();
    b.um.set_user_input(0, 1, 1, "first sheet").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(
        b.um.get_cell_content(1, 1, 1),
        Ok("second sheet".to_string())
    );
    assert_eq!(
        a.um.get_cell_content(0, 1, 1),
        Ok("first sheet".to_string())
    );
}

#[test]
fn concurrent_sheet_rename_is_lww() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.rename_sheet(0, "From A").unwrap();
    b.um.rename_sheet(0, "From B").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let name = a.um.model.workbook.worksheet(0).unwrap().get_name();
    assert!(name == "From A" || name == "From B");
}

fn sheet_names(um: &UserModel) -> Vec<String> {
    um.model
        .workbook
        .worksheets
        .iter()
        .map(|ws| ws.get_name())
        .collect()
}

#[test]
fn sheet_move_syncs_to_remote() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.new_sheet().unwrap();
    a.um.set_user_input(0, 1, 1, "on first").unwrap();
    sync(&mut a, &mut b);

    a.um.move_sheet(0, 2).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(sheet_names(&b.um), ["Sheet2", "Sheet3", "Sheet1"]);
    // The content moved with the sheet, on both replicas.
    assert_eq!(b.um.get_cell_content(2, 1, 1), Ok("on first".to_string()));
}

#[test]
fn concurrent_sheet_moves_converge() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.new_sheet().unwrap();
    sync(&mut a, &mut b);

    a.um.move_sheet(0, 2).unwrap();
    b.um.move_sheet(2, 0).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(sheet_names(&a.um), sheet_names(&b.um));
}

#[test]
fn sheet_move_vs_concurrent_delete_update_wins() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "survivor").unwrap();
    sync(&mut a, &mut b);

    a.um.move_sheet(1, 2).unwrap();
    b.um.delete_sheet(1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    // The move is a positive op: it preempts the concurrent deletion.
    assert_eq!(sheet_names(&a.um), ["Sheet1", "Sheet3", "Sheet2"]);
    assert_eq!(a.um.get_cell_content(2, 1, 1), Ok("survivor".to_string()));
}

#[test]
fn undo_of_sheet_move_propagates() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.new_sheet().unwrap();
    sync(&mut a, &mut b);

    a.um.move_sheet(2, 0).unwrap();
    sync(&mut a, &mut b);
    assert_eq!(sheet_names(&b.um), ["Sheet3", "Sheet1", "Sheet2"]);

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(sheet_names(&b.um), ["Sheet1", "Sheet2", "Sheet3"]);
}

#[test]
fn undo_of_delete_rows_resurrects_same_rows() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "two").unwrap();
    a.um.set_user_input(0, 3, 1, "three").unwrap();
    a.um.set_user_input(0, 4, 1, "four").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_rows(0, 2, 2).unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_cell_content(0, 2, 1), Ok("four".to_string()));

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(b.um.get_cell_content(0, 2, 1), Ok("two".to_string()));
    assert_eq!(b.um.get_cell_content(0, 3, 1), Ok("three".to_string()));
    assert_eq!(b.um.get_cell_content(0, 4, 1), Ok("four".to_string()));
}

#[test]
fn undo_only_reverts_own_operation() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "mine").unwrap();
    sync(&mut a, &mut b);
    b.um.set_user_input(0, 2, 2, "theirs").unwrap();
    sync(&mut a, &mut b);

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    // A's edit is gone, B's edit survives.
    assert_eq!(b.um.get_cell_content(0, 1, 1), Ok(String::new()));
    assert_eq!(a.um.get_cell_content(0, 2, 2), Ok("theirs".to_string()));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn randomized_convergence_fuzz() {
    // Seeded and deterministic: any failure is reproducible by seed.
    // Set CRDT_FUZZ_SEEDS=n to stress with seeds 1..=n locally.
    let seeds: Vec<u64> = if let Ok(only) = std::env::var("CRDT_FUZZ_ONLY") {
        vec![only.parse().expect("CRDT_FUZZ_ONLY must be a number")]
    } else {
        match std::env::var("CRDT_FUZZ_SEEDS") {
            Ok(n) => (1..=n.parse::<u64>().expect("CRDT_FUZZ_SEEDS must be a number")).collect(),
            Err(_) => vec![1, 7, 42, 1234, 987_654],
        }
    };
    for seed in seeds {
        let result = std::panic::catch_unwind(|| fuzz_round(seed));
        assert!(result.is_ok(), "fuzz_round failed for seed {seed}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn fuzz_round(seed: u64) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(seed);
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);

    let trace = std::env::var("CRDT_FUZZ_TRACE").is_ok();
    for step in 0..120 {
        {
            let on_a = rng.gen_bool(0.5);
            let who = if on_a { "A" } else { "B" };
            let replica = if on_a { &mut a } else { &mut b };
            let row = rng.gen_range(1..=25);
            let column = rng.gen_range(1..=8);
            match rng.gen_range(0..22) {
                0..=3 => {
                    let value = format!("v{}", rng.gen::<u16>());
                    if trace {
                        eprintln!("{step}: {who} set R{row}C{column} = {value}");
                    }
                    replica.um.set_user_input(0, row, column, &value).unwrap();
                }
                4 => {
                    let target = rng.gen_range(1..=25);
                    let formula = match rng.gen_range(0..5) {
                        0 => format!("=A{target}*2"),
                        1 => format!("=$A${target}+B{}", rng.gen_range(1..=25)),
                        2 => {
                            let hi = target + rng.gen_range(0..=5);
                            format!("=SUM(A{target}:B{hi})")
                        }
                        3 => "=SUM(D:D)".to_string(),
                        _ => format!("=FUZZNAME{}+1", rng.gen_range(1..=2)),
                    };
                    if trace {
                        eprintln!("{step}: {who} set R{row}C{column} = {formula}");
                    }
                    replica
                        .um
                        .set_user_input(0, row, column, &formula)
                        .unwrap();
                }
                5 => {
                    let count = rng.gen_range(1..=2);
                    if trace {
                        eprintln!("{step}: {who} insert_rows at {row} x{count}");
                    }
                    replica.um.insert_rows(0, row, count).unwrap();
                }
                6 => {
                    if trace {
                        eprintln!("{step}: {who} delete_rows at {row}");
                    }
                    replica.um.delete_rows(0, row, 1).unwrap();
                }
                7 => {
                    let height = rng.gen_range(20..60) as f64;
                    if trace {
                        eprintln!("{step}: {who} row {row} height {height}");
                    }
                    replica.um.set_rows_height(0, row, row, height).unwrap();
                }
                8 => {
                    if trace {
                        eprintln!("{step}: {who} insert_columns at {column}");
                    }
                    replica.um.insert_columns(0, column, 1).unwrap();
                }
                9 => {
                    let n = rng.gen_range(1..=2);
                    let name = format!("FUZZNAME{n}");
                    let formula = format!("Sheet1!$A${row}");
                    if trace {
                        eprintln!("{step}: {who} define {name} = {formula}");
                    }
                    // Errors (already exists) are fine; names are also
                    // exercised via update/delete below.
                    let _ = replica.um.new_defined_name(&name, None, &formula);
                }
                10 => {
                    let n = rng.gen_range(1..=2);
                    let name = format!("FUZZNAME{n}");
                    if rng.gen_bool(0.5) {
                        if trace {
                            eprintln!("{step}: {who} delete name {name}");
                        }
                        let _ = replica.um.delete_defined_name(&name, None);
                    } else {
                        let formula = format!("Sheet1!$B${row}");
                        if trace {
                            eprintln!("{step}: {who} update name {name} = {formula}");
                        }
                        let _ = replica
                            .um
                            .update_defined_name(&name, None, &name, None, &formula);
                    }
                }
                11 => {
                    let count = rng.gen_range(1..=2);
                    let delta = if rng.gen_bool(0.5) {
                        rng.gen_range(1..=5)
                    } else {
                        -rng.gen_range(1..=5)
                    };
                    if trace {
                        eprintln!("{step}: {who} move_rows {row} x{count} by {delta}");
                    }
                    // Out-of-bounds targets error and are skipped.
                    let _ = replica.um.move_rows_action(0, row, count, delta);
                }
                12 => {
                    if trace {
                        eprintln!("{step}: {who} new_sheet");
                    }
                    let _ = replica.um.new_sheet();
                }
                13 => {
                    // Never sheet 0: the fuzz cells and names live there.
                    let count = replica.um.model.workbook.worksheets.len() as u32;
                    if count > 1 {
                        let index = rng.gen_range(1..count);
                        if trace {
                            eprintln!("{step}: {who} delete_sheet {index}");
                        }
                        let _ = replica.um.delete_sheet(index);
                    }
                }
                14 => {
                    let count = replica.um.model.workbook.worksheets.len() as u32;
                    if count > 1 {
                        let index = rng.gen_range(1..count);
                        let name = format!("R{}", rng.gen_range(1..=3));
                        if trace {
                            eprintln!("{step}: {who} rename sheet {index} -> {name}");
                        }
                        let _ = replica.um.rename_sheet(index, &name);
                    }
                }
                15 => {
                    use crate::expressions::types::Area;
                    let value = if rng.gen_bool(0.7) { "true" } else { "" };
                    let area = Area {
                        sheet: 0,
                        row,
                        column,
                        width: rng.gen_range(1..=2),
                        height: rng.gen_range(1..=2),
                    };
                    if trace {
                        eprintln!("{step}: {who} style R{row}C{column} font.b={value}");
                    }
                    let _ = replica.um.update_range_style(&area, "font.b", value);
                }
                16 => {
                    let hi = row + rng.gen_range(0..=5);
                    let range = format!("A{row}:C{hi}");
                    let rule = match rng.gen_range(0..3) {
                        0 => cell_is_gt(&format!("A{}", rng.gen_range(1..=25))),
                        1 => CfRuleInput::Formula {
                            formula: format!("=$A${}>0", rng.gen_range(1..=25)),
                            format: bold_dxf(),
                            stop_if_true: rng.gen_bool(0.3),
                        },
                        _ => CfRuleInput::DataBar {
                            min: None,
                            max: None,
                            positive_color: Color::Rgb("#00FF00".to_string()),
                            negative_color: Color::Rgb("#FF0000".to_string()),
                            is_gradient: false,
                            show_value: true,
                        },
                    };
                    if trace {
                        eprintln!("{step}: {who} add cf {range}");
                    }
                    let _ = replica.um.add_conditional_formatting(0, &range, rule);
                }
                17 => {
                    let count = replica
                        .um
                        .model
                        .workbook
                        .worksheet(0)
                        .unwrap()
                        .conditional_formatting
                        .len();
                    if count > 0 {
                        let index = rng.gen_range(0..count) as u32;
                        match rng.gen_range(0..4) {
                            0 => {
                                if trace {
                                    eprintln!("{step}: {who} delete cf {index}");
                                }
                                let _ = replica.um.delete_conditional_formatting(0, index);
                            }
                            1 => {
                                if trace {
                                    eprintln!("{step}: {who} raise cf {index}");
                                }
                                let _ =
                                    replica.um.raise_conditional_formatting_priority(0, index);
                            }
                            2 => {
                                if trace {
                                    eprintln!("{step}: {who} lower cf {index}");
                                }
                                let _ =
                                    replica.um.lower_conditional_formatting_priority(0, index);
                            }
                            _ => {
                                let range = format!("B{row}:D{}", row + 2);
                                if trace {
                                    eprintln!("{step}: {who} update cf {index} -> {range}");
                                }
                                let _ = replica.um.update_conditional_formatting(
                                    0,
                                    index,
                                    &range,
                                    cell_is_gt("3"),
                                );
                            }
                        }
                    }
                }
                18 => {
                    use crate::expressions::types::Area;
                    let kinds = ["All", "Outer", "Inner", "Top", "Left", "CenterH", "None"];
                    let kind = kinds[rng.gen_range(0..kinds.len())];
                    let styles = ["thin", "medium", "thick"];
                    let border_style = styles[rng.gen_range(0..styles.len())];
                    let height = rng.gen_range(1..=3);
                    let width = rng.gen_range(1..=3);
                    if trace {
                        eprintln!(
                            "{step}: {who} border {kind} {border_style} R{row}C{column} {height}x{width}"
                        );
                    }
                    let _ = replica.um.set_area_with_border(
                        &Area {
                            sheet: 0,
                            row,
                            column,
                            width,
                            height,
                        },
                        &border_area(border_style, kind),
                    );
                }
                19 => {
                    let mut theme = replica.um.get_theme();
                    theme.name = format!("Fuzz{}", rng.gen_range(1..=3));
                    theme.accent1 = format!("#{:06X}", rng.gen::<u32>() & 0xFF_FFFF);
                    if trace {
                        eprintln!("{step}: {who} set theme {} {}", theme.name, theme.accent1);
                    }
                    replica.um.set_theme(theme);
                }
                20 => {
                    // Keep sheet 0 in place: the fuzz cells and names live
                    // there.
                    let count = replica.um.model.workbook.worksheets.len() as u32;
                    if count > 2 {
                        let from = rng.gen_range(1..count);
                        let to = rng.gen_range(1..count);
                        if trace {
                            eprintln!("{step}: {who} move_sheet {from} -> {to}");
                        }
                        let _ = replica.um.move_sheet(from, to);
                    }
                }
                _ => {
                    if trace {
                        eprintln!("{step}: {who} undo");
                    }
                    let _ = replica.um.undo();
                }
            }
        }
        if rng.gen_ratio(1, 6) {
            if trace {
                eprintln!("{step}: sync");
            }
            sync(&mut a, &mut b);
        }
    }
    sync(&mut a, &mut b);
    sync(&mut a, &mut b);
    assert_eq!(
        a.session.shadow_for_tests(),
        b.session.shadow_for_tests(),
        "documents diverged (outbound bug)"
    );
    assert_converged(&a, &b);
}

#[test]
fn formula_edit_vs_concurrent_insert_preserves_both_intentions() {
    // The id-form payoff: a formula written concurrently with a structural
    // edit keeps pointing at its logical target — no LWW race, no data loss.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "7").unwrap();
    sync(&mut a, &mut b);

    a.um.set_user_input(0, 1, 2, "=A5*2").unwrap();
    b.um.insert_rows(0, 3, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // The marker moved to A6 and the formula follows it on both replicas.
    assert_eq!(a.um.get_cell_content(0, 1, 2), Ok("=A6*2".to_string()));
    assert_eq!(b.um.get_cell_content(0, 1, 2), Ok("=A6*2".to_string()));
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 2), Ok("14".to_string()));
}

#[test]
fn formula_reference_vs_concurrent_delete_shows_ref_error() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "7").unwrap();
    a.um.set_user_input(0, 1, 2, "=A5*2").unwrap();
    sync(&mut a, &mut b);

    b.um.delete_rows(0, 5, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 1, 2), Ok("=#REF!*2".to_string()));
}

#[test]
fn range_grows_and_clamps_under_concurrent_structural_edits() {
    let mut a = replica(1);
    let mut b = replica(2);
    for row in 1..=4 {
        a.um.set_user_input(0, row, 1, "10").unwrap();
    }
    a.um.set_user_input(0, 6, 2, "=SUM(A1:A4)").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_formatted_cell_value(0, 6, 2), Ok("40".to_string()));

    // A concurrent insert inside the range grows it; the new row's value is
    // included after the merge.
    b.um.insert_rows(0, 3, 1).unwrap();
    b.um.set_user_input(0, 3, 1, "5").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 7, 2), Ok("=SUM(A1:A5)".to_string()));
    assert_eq!(a.um.get_formatted_cell_value(0, 7, 2), Ok("45".to_string()));

    // Deleting the range's last row kills that endpoint (engine semantics:
    // =SUM(A1:#REF!), not Excel's clamping) — identically on both replicas.
    a.um.delete_rows(0, 5, 1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(
        b.um.get_cell_content(0, 6, 2),
        Ok("=SUM(A1:#REF!)".to_string())
    );
}

#[test]
fn cross_sheet_formula_rerenders_on_remote_structural_change() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(0, 5, 1, "9").unwrap();
    a.um.set_user_input(1, 1, 1, "=Sheet1!A5").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_formatted_cell_value(1, 1, 1), Ok("9".to_string()));

    // B inserts a row on Sheet1; the formula on Sheet2 must re-render on
    // both replicas even though Sheet2 itself did not change.
    b.um.insert_rows(0, 2, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(
        a.um.get_cell_content(1, 1, 1),
        Ok("=Sheet1!A6".to_string())
    );
    assert_eq!(a.um.get_formatted_cell_value(1, 1, 1), Ok("9".to_string()));
    assert_eq!(b.um.get_formatted_cell_value(1, 1, 1), Ok("9".to_string()));
}

#[test]
fn engine_displacement_matches_codec_render_matrix() {
    // The 3.4 contract: with the fan-out gone, the receiving replica's render
    // of the (unchanged) id-form must equal the text the originating engine
    // produced by displacement. Exercise formulas × structural ops and check
    // the model↔document invariant on both replicas after every exchange.
    let formulas = [
        "=A5",
        "=$A$5",
        "=A$5+$A5",
        "=SUM(A2:A8)",
        "=SUM($B$2:C9)",
        "=SUM(D:D)",
        "=SUM(3:4)",
        "=Sheet2!B4*2",
        "=SUM(Sheet2!A2:A6)",
    ];
    let ops: [fn(&mut Replica); 12] = [
        |r| r.um.insert_rows(0, 3, 1).unwrap(),
        |r| r.um.insert_rows(0, 1, 2).unwrap(),
        |r| r.um.delete_rows(0, 5, 1).unwrap(),
        |r| r.um.delete_rows(0, 2, 3).unwrap(),
        |r| r.um.insert_columns(0, 1, 1).unwrap(),
        |r| r.um.insert_columns(0, 3, 2).unwrap(),
        |r| r.um.delete_columns(0, 1, 1).unwrap(),
        |r| r.um.delete_columns(0, 2, 1).unwrap(),
        |r| r.um.move_rows_action(0, 2, 2, 5).unwrap(),
        |r| r.um.move_rows_action(0, 8, 1, -6).unwrap(),
        |r| r.um.move_columns_action(0, 1, 1, 3).unwrap(),
        |r| r.um.move_columns_action(0, 4, 2, -2).unwrap(),
    ];
    for formula in formulas {
        for (i, op) in ops.iter().enumerate() {
            let mut a = replica(1);
            let mut b = replica(2);
            a.um.new_sheet().unwrap();
            a.um.set_user_input(0, 10, 5, formula).unwrap();
            sync(&mut a, &mut b);

            op(&mut a);
            sync(&mut a, &mut b);

            a.session.assert_model_matches_shadow(&a.um, "replica A");
            b.session.assert_model_matches_shadow(&b.um, "replica B");
            let (ra, rb) = (
                a.um.get_cell_content(0, 10, 5).unwrap_or_default(),
                b.um.get_cell_content(0, 10, 5).unwrap_or_default(),
            );
            // The formula cell may itself have moved; compare the whole
            // window instead of guessing its new coordinates.
            assert_converged(&a, &b);
            assert_eq!(ra, rb, "formula {formula:?} op #{i}");
        }
    }
}

#[test]
fn resurrected_row_heals_references_to_it() {
    // Healing: the engine itself cannot restore a #REF! after delete+undo
    // (WrongReference nodes are never displaced back), but the id token still
    // points at the resurrected row, so the render heals it — on both sides.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "7").unwrap();
    a.um.set_user_input(0, 1, 2, "=A5*2").unwrap();
    sync(&mut a, &mut b);

    b.um.delete_rows(0, 5, 1).unwrap();
    sync(&mut a, &mut b);
    assert_eq!(a.um.get_cell_content(0, 1, 2), Ok("=#REF!*2".to_string()));

    b.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 1, 2), Ok("=A5*2".to_string()));
    assert_eq!(b.um.get_cell_content(0, 1, 2), Ok("=A5*2".to_string()));
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 2), Ok("14".to_string()));
    assert_eq!(b.um.get_cell_content(0, 5, 1), Ok("7".to_string()));
}

#[test]
fn cell_style_syncs_and_survives_concurrent_content_edit() {
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 2, "hello").unwrap();
    sync(&mut a, &mut b);

    // Style and content are independent registers: concurrent edits to the
    // same cell both survive.
    let area = Area {
        sheet: 0,
        row: 2,
        column: 2,
        width: 1,
        height: 1,
    };
    a.um.update_range_style(&area, "font.b", "true").unwrap();
    b.um.set_user_input(0, 2, 2, "world").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 2, 2), Ok("world".to_string()));
    assert!(a.um.get_cell_style(0, 2, 2).unwrap().font.b);
    assert!(b.um.get_cell_style(0, 2, 2).unwrap().font.b);
}

#[test]
fn style_travels_with_moved_row() {
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 3, 1, "styled").unwrap();
    let area = Area {
        sheet: 0,
        row: 3,
        column: 1,
        width: 1,
        height: 1,
    };
    a.um.update_range_style(&area, "font.i", "true").unwrap();
    sync(&mut a, &mut b);

    b.um.move_rows_action(0, 3, 1, 4).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert!(a.um.get_cell_style(0, 7, 1).unwrap().font.i);
    assert!(!a.um.get_cell_style(0, 3, 1).unwrap().font.i);
}

#[test]
fn range_clear_all_clears_style_remotely() {
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    let area = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 2,
        height: 2,
    };
    a.um.set_user_input(0, 1, 1, "x").unwrap();
    a.um.update_range_style(&area, "font.b", "true").unwrap();
    sync(&mut a, &mut b);
    assert!(b.um.get_cell_style(0, 2, 2).unwrap().font.b);

    b.um.range_clear_all(&area).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert!(!a.um.get_cell_style(0, 2, 2).unwrap().font.b);
    assert_eq!(a.um.get_cell_content(0, 1, 1), Ok(String::new()));
}

#[test]
fn full_column_style_syncs() {
    use crate::constants::LAST_ROW;
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    let full_column = Area {
        sheet: 0,
        row: 1,
        column: 3,
        width: 1,
        height: LAST_ROW,
    };
    a.um.update_range_style(&full_column, "font.b", "true").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    // Inherited by any cell of the column, including untouched ones.
    assert!(b.um.get_cell_style(0, 33, 3).unwrap().font.b);
}

/// Two-browser repro: replica B, offline, fills column E yellow and column F
/// blue; replica A concurrently types "Adios" in E4 and "Hola" in F6. After
/// the merge those cells must show the column fill on both replicas — not a
/// white (default) background.
#[test]
fn concurrent_cell_write_inherits_offline_column_fill() {
    use crate::constants::LAST_ROW;
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);

    // B goes offline and fills the full columns E and F.
    let full_column = |column| Area {
        sheet: 0,
        row: 1,
        column,
        width: 1,
        height: LAST_ROW,
    };
    b.um.update_range_style(&full_column(5), "fill.bg_color", "#FFFF00")
        .unwrap();
    b.um.update_range_style(&full_column(6), "fill.bg_color", "#0000FF")
        .unwrap();

    // Meanwhile A writes values into those columns.
    a.um.set_user_input(0, 6, 6, "Hola").unwrap(); // F6
    a.um.set_user_input(0, 4, 5, "Adios").unwrap(); // E4

    // B comes back online.
    sync(&mut a, &mut b);
    assert_converged(&a, &b);

    let yellow = Color::Rgb("#FFFF00".to_string());
    let blue = Color::Rgb("#0000FF".to_string());
    for (replica, name) in [(&a, "A"), (&b, "B")] {
        assert_eq!(
            replica.um.get_cell_content(0, 4, 5),
            Ok("Adios".to_string()),
            "E4 content on replica {name}"
        );
        assert_eq!(
            replica.um.get_cell_content(0, 6, 6),
            Ok("Hola".to_string()),
            "F6 content on replica {name}"
        );
        // An untouched cell of the column carries the fill…
        assert_eq!(
            replica.um.get_cell_style(0, 20, 5).unwrap().fill.color,
            yellow,
            "E20 fill on replica {name}"
        );
        // …and so must the cells that were written concurrently.
        assert_eq!(
            replica.um.get_cell_style(0, 4, 5).unwrap().fill.color,
            yellow,
            "E4 fill on replica {name}"
        );
        assert_eq!(
            replica.um.get_cell_style(0, 6, 6).unwrap().fill.color,
            blue,
            "F6 fill on replica {name}"
        );
    }
}

/// Same inheritance rule on the catch-up path: a late joiner receives a
/// document whose cells were typed concurrently with a column fill (no cell
/// style register) and must render them with the column fill.
#[test]
fn late_joiner_inherits_column_fill_on_registerless_cells() {
    use crate::constants::LAST_ROW;
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    b.um.update_range_style(
        &Area {
            sheet: 0,
            row: 1,
            column: 5,
            width: 1,
            height: LAST_ROW,
        },
        "fill.bg_color",
        "#FFFF00",
    )
    .unwrap();
    a.um.set_user_input(0, 4, 5, "Adios").unwrap();
    sync(&mut a, &mut b);

    let mut c = replica(3);
    let sv = c.session.state_vector();
    let update = a.session.encode_state_since(&sv).unwrap();
    c.session.apply_remote(&mut c.um, &update).unwrap();

    assert_models_converged(&a.um, &c.um);
    assert_eq!(c.um.get_cell_content(0, 4, 5), Ok("Adios".to_string()));
    assert_eq!(
        c.um.get_cell_style(0, 4, 5).unwrap().fill.color,
        Color::Rgb("#FFFF00".to_string()),
        "late joiner shows a white E4"
    );
}

#[test]
fn named_style_definitions_sync_and_apply() {
    use crate::types::StyleIncludes;
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);

    let mut style = a.um.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    a.um.create_named_style("Bold Header", &style, StyleIncludes::default())
        .unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert!(b.um.get_named_style_list().contains(&"Bold Header".to_string()));

    // The other replica applies it; the resolved style replicates back.
    b.um.set_selected_range(1, 1, 1, 1).unwrap();
    b.um.on_apply_named_style("Bold Header").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert!(a.um.get_cell_style(0, 1, 1).unwrap().font.b);
}

fn border_area(style: &str, kind: &str) -> crate::BorderArea {
    serde_json::from_str(&format!(
        r##"{{"item": {{"style": "{style}", "color": "#333333"}}, "type": "{kind}"}}"##
    ))
    .unwrap()
}

fn set_border(um: &mut UserModel, row: i32, column: i32, height: i32, width: i32, style: &str, kind: &str) {
    use crate::expressions::types::Area;
    um.set_area_with_border(
        &Area {
            sheet: 0,
            row,
            column,
            width,
            height,
        },
        &border_area(style, kind),
    )
    .unwrap();
}

#[test]
fn border_syncs_to_remote() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    set_border(&mut a.um, 1, 1, 2, 2, "thin", "All");
    sync(&mut a, &mut b);

    let border = b.um.get_cell_style(0, 1, 1).unwrap().border;
    assert!(border.top.is_some(), "top border missing: {border:?}");
    assert!(border.left.is_some());
    assert!(border.right.is_some());
    assert!(border.bottom.is_some());
    assert_converged(&a, &b);
}

/// Test 18 of the design doc: the line between columns B and C is one
/// register, so concurrent borders around `A1:B2` and `C1:D2` converge with
/// both adjacent sides equal (no split-edge disagreement).
#[test]
fn shared_edge_between_adjacent_ranges_converges() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    set_border(&mut a.um, 1, 1, 2, 2, "thick", "Outer");
    set_border(&mut b.um, 1, 3, 2, 2, "thin", "Outer");
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    for row in 1..=2 {
        let right_of_b = a.um.get_cell_style(0, row, 2).unwrap().border.right;
        let left_of_c = a.um.get_cell_style(0, row, 3).unwrap().border.left;
        assert!(right_of_b.is_some(), "shared edge vanished");
        assert_eq!(
            right_of_b, left_of_c,
            "shared edge is incoherent at row {row}"
        );
    }
}

#[test]
fn border_survives_concurrent_style_edit_on_same_cell() {
    use crate::expressions::types::Area;
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    set_border(&mut a.um, 2, 2, 1, 1, "medium", "All");
    b.um.update_range_style(
        &Area {
            sheet: 0,
            row: 2,
            column: 2,
            width: 1,
            height: 1,
        },
        "font.b",
        "true",
    )
    .unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // The edge registers are independent of the style register, so the
    // border always survives (the bold flag resolves by LWW on the style).
    let border = a.um.get_cell_style(0, 2, 2).unwrap().border;
    assert!(border.top.is_some(), "border lost to concurrent style edit");
}

#[test]
fn border_travels_with_concurrent_row_insert() {
    let mut a = replica(1);
    let mut b = replica(2);
    set_border(&mut a.um, 5, 1, 1, 2, "thin", "All");
    sync(&mut a, &mut b);

    b.um.insert_rows(0, 3, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let border = a.um.get_cell_style(0, 6, 1).unwrap().border;
    assert!(border.top.is_some(), "border did not travel with its row");
    let border = a.um.get_cell_style(0, 5, 1).unwrap().border;
    assert!(border.top.is_none(), "stale border at the old position");
}

#[test]
fn border_clear_propagates() {
    let mut a = replica(1);
    let mut b = replica(2);
    set_border(&mut a.um, 1, 1, 2, 2, "thin", "All");
    sync(&mut a, &mut b);
    assert!(b.um.get_cell_style(0, 1, 1).unwrap().border.top.is_some());

    b.um.set_area_with_border(
        &crate::expressions::types::Area {
            sheet: 0,
            row: 1,
            column: 1,
            width: 2,
            height: 2,
        },
        &border_area("thick", "None"),
    )
    .unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let border = a.um.get_cell_style(0, 1, 1).unwrap().border;
    assert!(border.top.is_none(), "border not cleared: {border:?}");
}

#[test]
fn border_undo_removes_border_on_both_replicas() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    set_border(&mut a.um, 2, 2, 1, 1, "thin", "All");
    sync(&mut a, &mut b);
    assert!(b.um.get_cell_style(0, 2, 2).unwrap().border.top.is_some());

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let border = b.um.get_cell_style(0, 2, 2).unwrap().border;
    assert!(border.top.is_none(), "undone border survived: {border:?}");
    // The neighbouring cells must not have grown materialized borders.
    let border = b.um.get_cell_style(0, 2, 3).unwrap().border;
    assert!(border.left.is_none(), "neighbour kept the edge: {border:?}");
}

#[test]
fn cf_rule_syncs_to_remote_with_dxf() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "5").unwrap();
    a.um
        .add_conditional_formatting(0, "A1:B4", cell_is_gt("3"))
        .unwrap();
    sync(&mut a, &mut b);

    let rules = cf_snapshot(&b.um, 0);
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].1, "A1:B4");
    assert_eq!(rules[0].3, Some(bold_dxf()));
    assert_converged(&a, &b);
}

/// Test 19 of the design doc: raising a rule's priority concurrently with a
/// rule addition converges without index skew (priority is a fractional
/// position write, not an index-keyed swap).
#[test]
fn cf_raise_priority_vs_concurrent_add() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("1"))
        .unwrap();
    a.um
        .add_conditional_formatting(0, "B1:B5", cell_is_gt("2"))
        .unwrap();
    sync(&mut a, &mut b);

    // A raises the first rule above the second; B adds a third rule.
    a.um.raise_conditional_formatting_priority(0, 0).unwrap();
    b.um
        .add_conditional_formatting(0, "C1:C5", cell_is_gt("3"))
        .unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let rules = cf_snapshot(&a.um, 0);
    assert_eq!(rules.len(), 3);
    // The raise survives the merge: A1:A5 still outranks B1:B5.
    let rank = |range: &str| rules.iter().position(|r| r.1 == range).unwrap();
    assert!(rank("A1:A5") < rank("B1:B5"), "raise was lost: {rules:?}");
}

#[test]
fn cf_concurrent_adds_converge_deterministically() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um
        .add_conditional_formatting(0, "A1:A3", cell_is_gt("1"))
        .unwrap();
    b.um
        .add_conditional_formatting(0, "B1:B3", cell_is_gt("2"))
        .unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(cf_snapshot(&a.um, 0).len(), 2);
}

#[test]
fn cf_update_vs_concurrent_delete_converges_order_independently() {
    // Pair 1.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("1"))
        .unwrap();
    sync(&mut a, &mut b);
    a.um
        .update_conditional_formatting(0, 0, "A1:C4", cell_is_gt("7"))
        .unwrap();
    b.um.delete_conditional_formatting(0, 0).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let outcome = cf_snapshot(&a.um, 0);

    // Pair 2: reversed delivery order — same outcome.
    let mut c = replica(1);
    let mut d = replica(2);
    c.um
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("1"))
        .unwrap();
    sync(&mut c, &mut d);
    c.um
        .update_conditional_formatting(0, 0, "A1:C4", cell_is_gt("7"))
        .unwrap();
    d.um.delete_conditional_formatting(0, 0).unwrap();
    let from_c = c.session.flush_local(&mut c.um).unwrap();
    let from_d = d.session.flush_local(&mut d.um).unwrap();
    d.session.apply_remote(&mut d.um, &from_c).unwrap();
    c.session.apply_remote(&mut c.um, &from_d).unwrap();
    assert_converged(&c, &d);
    assert_eq!(cf_snapshot(&c.um, 0), outcome);
}

#[test]
fn cf_range_follows_concurrent_row_insert() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um
        .add_conditional_formatting(0, "A5:A10", cell_is_gt("0"))
        .unwrap();
    sync(&mut a, &mut b);

    b.um.insert_rows(0, 3, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let rules = cf_snapshot(&a.um, 0);
    assert_eq!(rules[0].1, "A6:A11");
}

#[test]
fn cf_undo_of_add_propagates() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("1"))
        .unwrap();
    sync(&mut a, &mut b);
    assert_eq!(cf_snapshot(&b.um, 0).len(), 1);

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(cf_snapshot(&b.um, 0).len(), 0);

    a.um.redo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(cf_snapshot(&b.um, 0).len(), 1);
}

#[test]
fn cf_travels_with_duplicated_sheet() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um
        .add_conditional_formatting(0, "A1:A5", cell_is_gt("1"))
        .unwrap();
    sync(&mut a, &mut b);

    a.um.duplicate_sheet(0).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(cf_snapshot(&b.um, 1).len(), 1);
    assert_eq!(cf_snapshot(&b.um, 1)[0].1, "A1:A5");
}

#[test]
fn sheet_survives_concurrent_delete_when_edited() {
    // Update-wins at sheet granularity: an edit inside a concurrently deleted
    // sheet resurrects the whole sheet with all its content.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "existing").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_sheet(1).unwrap();
    b.um.set_user_input(1, 2, 2, "concurrent edit").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheets.len(), 2);
    assert_eq!(a.um.get_cell_content(1, 1, 1), Ok("existing".to_string()));
    assert_eq!(
        a.um.get_cell_content(1, 2, 2),
        Ok("concurrent edit".to_string())
    );
}

#[test]
fn sheet_rename_vs_concurrent_delete_resurrects() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "keep").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_sheet(1).unwrap();
    b.um.rename_sheet(1, "Renamed").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheets.len(), 2);
    assert_eq!(
        a.um.model.workbook.worksheet(1).unwrap().get_name(),
        "Renamed"
    );
    assert_eq!(a.um.get_cell_content(1, 1, 1), Ok("keep".to_string()));
}

#[test]
fn deleted_sheet_stays_deleted_without_concurrent_ops() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "gone").unwrap();
    sync(&mut a, &mut b);

    b.um.delete_sheet(1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheets.len(), 1);
}

#[test]
fn concurrent_new_sheets_get_deterministic_names() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 1, 1, "from A").unwrap();
    b.um.new_sheet().unwrap();
    b.um.set_user_input(1, 1, 1, "from B").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheets.len(), 3);
    // Both were locally "Sheet2"; the later one (by position/id) gets a
    // deterministic suffix on every replica.
    let names: Vec<String> = (0..3)
        .map(|i| a.um.model.workbook.worksheet(i).unwrap().get_name())
        .collect();
    assert!(names.contains(&"Sheet2".to_string()));
    assert!(names.contains(&"Sheet2 (2)".to_string()));
}

#[test]
fn concurrent_deletes_of_all_sheets_keep_one_deterministically() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    sync(&mut a, &mut b);

    // Each replica deletes a different sheet; merged, everything would be
    // tombstoned — the render-time fixup keeps one, the same one everywhere.
    a.um.delete_sheet(0).unwrap();
    b.um.delete_sheet(1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheets.len(), 1);
    assert_eq!(
        a.um.model.workbook.worksheet(0).unwrap().get_name(),
        b.um.model.workbook.worksheet(0).unwrap().get_name()
    );
}

#[test]
fn sheet_settings_sync() {
    use crate::types::{Color, SheetState};
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    sync(&mut a, &mut b);

    a.um.set_sheet_color(0, &Color::Rgb("#FFAA00".to_string())).unwrap();
    a.um.set_show_grid_lines(0, false).unwrap();
    a.um.hide_sheet(1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let ws = b.um.model.workbook.worksheet(0).unwrap();
    assert_eq!(ws.color, Color::Rgb("#FFAA00".to_string()));
    assert!(!ws.show_grid_lines);
    assert_eq!(
        b.um.model.workbook.worksheet(1).unwrap().state,
        SheetState::Hidden
    );

    // And back to defaults.
    b.um.set_sheet_color(0, &Color::None).unwrap();
    b.um.unhide_sheet(1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(a.um.model.workbook.worksheet(0).unwrap().color, Color::None);
    assert_eq!(
        a.um.model.workbook.worksheet(1).unwrap().state,
        SheetState::Visible
    );
}

#[test]
fn workbook_timezone_syncs() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_timezone("Europe/Berlin").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.model.workbook.settings.tz, "Europe/Berlin");
    assert_converged(&a, &b);
}

#[test]
fn workbook_theme_syncs() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    let mut theme = a.um.get_theme();
    theme.name = "Midnight".to_string();
    theme.accent1 = "#FF0000".to_string();
    a.um.set_theme(theme.clone());
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_theme(), theme);
    assert_converged(&a, &b);

    // Undo on the originator reverts both replicas.
    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_theme().name, "Office");
    assert_converged(&a, &b);
}

#[test]
fn workbook_name_syncs() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.set_name("Budget 2026");
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_name(), "Budget 2026");
    assert_converged(&a, &b);
}

/// A joiner attaches a blank workbook with an empty name (the webapp's
/// `?room=` flow): it must adopt the host's name instead of competing.
#[test]
fn joiner_with_empty_name_adopts_host_name() {
    let mut um_a = UserModel::from_model(new_empty_model());
    um_a.set_name("Quarterly report");
    let session_a = CollabSession::attach(&mut um_a, 1).unwrap();
    let mut a = Replica {
        um: um_a,
        session: session_a,
    };

    let mut um_b = UserModel::from_model(new_empty_model());
    um_b.set_name("");
    let session_b = CollabSession::attach(&mut um_b, 2).unwrap();
    let mut b = Replica {
        um: um_b,
        session: session_b,
    };

    sync(&mut a, &mut b);
    assert_eq!(b.um.get_name(), "Quarterly report");
    assert_converged(&a, &b);

    // A later rename on either side propagates.
    b.um.set_name("Quarterly report v2");
    sync(&mut a, &mut b);
    assert_eq!(a.um.get_name(), "Quarterly report v2");
    assert_converged(&a, &b);
}

#[test]
fn undo_of_sheet_delete_restores_content_on_both() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.new_sheet().unwrap();
    a.um.set_user_input(1, 3, 3, "buried treasure").unwrap();
    sync(&mut a, &mut b);

    a.um.delete_sheet(1).unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.model.workbook.worksheets.len(), 1);

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(b.um.model.workbook.worksheets.len(), 2);
    assert_eq!(
        b.um.get_cell_content(1, 3, 3),
        Ok("buried treasure".to_string())
    );
}

#[test]
fn edit_travels_with_concurrently_moved_row() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "payload").unwrap();
    a.um.set_user_input(0, 3, 1, "second").unwrap();
    sync(&mut a, &mut b);

    // A moves rows 2–3 down to 7–8 while B edits a cell inside the block.
    a.um.move_rows_action(0, 2, 2, 5).unwrap();
    b.um.set_user_input(0, 2, 2, "edited").unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 7, 1), Ok("payload".to_string()));
    assert_eq!(a.um.get_cell_content(0, 7, 2), Ok("edited".to_string()));
    assert_eq!(a.um.get_cell_content(0, 8, 1), Ok("second".to_string()));
    assert_eq!(a.um.get_cell_content(0, 2, 2), Ok(String::new()));
}

#[test]
fn concurrent_moves_of_same_row_resolve_to_one_target() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "traveler").unwrap();
    sync(&mut a, &mut b);

    a.um.move_rows_action(0, 2, 1, 3).unwrap();
    b.um.move_rows_action(0, 2, 1, 6).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    // Exactly one copy, at one of the two targets.
    let mut hits = Vec::new();
    for row in 1..=WINDOW_ROWS {
        if a.um.get_cell_content(0, row, 1).unwrap() == "traveler" {
            hits.push(row);
        }
    }
    assert_eq!(hits.len(), 1, "row duplicated or lost: {hits:?}");
    assert!(hits[0] == 5 || hits[0] == 8, "unexpected target {hits:?}");
}

#[test]
fn move_with_hidden_rows_converges() {
    // move_rows_action pre-adjusts the delta for locally hidden rows; the
    // diff carries the resolved delta, so replicas agree regardless of what
    // is hidden where.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "mover").unwrap();
    a.um.set_user_input(0, 5, 1, "below").unwrap();
    a.um.set_rows_hidden(0, 3, 4, true).unwrap();
    sync(&mut a, &mut b);

    a.um.move_rows_action(0, 2, 1, 1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);

    // The adjusted move skipped the hidden block.
    let mut position = None;
    for row in 1..=WINDOW_ROWS {
        if b.um.get_cell_content(0, row, 1).unwrap() == "mover" {
            position = Some(row);
        }
    }
    assert_eq!(position, Some(5));
}

#[test]
fn moved_row_survives_concurrent_delete() {
    // AegisSheet semantics: moving a row is a positive op that preempts a
    // concurrent deletion of it (update-wins).
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "keep me").unwrap();
    sync(&mut a, &mut b);

    a.um.move_rows_action(0, 2, 1, 4).unwrap();
    b.um.delete_rows(0, 2, 1).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    let mut hits = 0;
    for row in 1..=WINDOW_ROWS {
        if a.um.get_cell_content(0, row, 1).unwrap() == "keep me" {
            hits += 1;
        }
    }
    assert_eq!(hits, 1, "moved row should survive the concurrent delete");
}

#[test]
fn formula_follows_concurrently_moved_target() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "9").unwrap();
    a.um.set_user_input(0, 1, 3, "=A5*2").unwrap();
    sync(&mut a, &mut b);

    b.um.move_rows_action(0, 5, 1, 4).unwrap();
    sync(&mut a, &mut b);

    assert_converged(&a, &b);
    assert_eq!(a.um.get_cell_content(0, 1, 3), Ok("=A9*2".to_string()));
    assert_eq!(a.um.get_formatted_cell_value(0, 1, 3), Ok("18".to_string()));
}

#[test]
fn undo_of_move_returns_row_on_both_replicas() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 2, 1, "boomerang").unwrap();
    sync(&mut a, &mut b);

    a.um.move_rows_action(0, 2, 1, 5).unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_cell_content(0, 7, 1), Ok("boomerang".to_string()));

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(b.um.get_cell_content(0, 2, 1), Ok("boomerang".to_string()));
    assert_eq!(b.um.get_cell_content(0, 7, 1), Ok(String::new()));
}

#[test]
fn defined_name_syncs_and_reevaluates() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 1, 1, "21").unwrap();
    a.um.new_defined_name("DOUBLE_ME", None, "Sheet1!$A$1").unwrap();
    a.um.set_user_input(0, 2, 2, "=DOUBLE_ME*2").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_formatted_cell_value(0, 2, 2), Ok("42".to_string()));

    // The other replica points the name somewhere else; everyone re-evaluates.
    b.um.set_user_input(0, 5, 1, "100").unwrap();
    b.um.update_defined_name("DOUBLE_ME", None, "DOUBLE_ME", None, "Sheet1!$A$5")
        .unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(a.um.get_formatted_cell_value(0, 2, 2), Ok("200".to_string()));
}

#[test]
fn concurrent_same_name_creation_is_lww() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.new_defined_name("TOTAL", None, "Sheet1!$A$1").unwrap();
    b.um.new_defined_name("TOTAL", None, "Sheet1!$B$2").unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    let names_a = a.um.get_defined_name_list();
    let names_b = b.um.get_defined_name_list();
    assert_eq!(names_a.len(), 1);
    assert_eq!(names_a, names_b);
    assert!(names_a[0].2 == "Sheet1!$A$1" || names_a[0].2 == "Sheet1!$B$2");
}

#[test]
fn defined_name_stays_positional_under_structural_edits() {
    // The engine does NOT displace defined-name formulas on insert/delete
    // (unlike Excel — an engine-level divergence): Sheet1!$A$5 keeps saying
    // $A$5 while the content moves. The collab layer replicates the engine
    // faithfully; both replicas must agree. If the engine gains name
    // displacement, `sync_names` re-encodes the displaced text and this test
    // flips expectations along with it.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "7").unwrap();
    a.um.new_defined_name("TARGET", None, "Sheet1!$A$5").unwrap();
    a.um.set_user_input(0, 1, 3, "=TARGET+1").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 3), Ok("8".to_string()));

    // A concurrent insert above the target: the "7" moves to A6, the name
    // keeps pointing at (the now empty) A5 — identically on both replicas.
    b.um.insert_rows(0, 2, 1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(a.um.get_defined_name_list()[0].2, "Sheet1!$A$5");
    assert_eq!(b.um.get_defined_name_list()[0].2, "Sheet1!$A$5");
    assert_eq!(a.um.get_formatted_cell_value(0, 1, 3), Ok("1".to_string()));
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 3), Ok("1".to_string()));
}

#[test]
fn renaming_defined_name_updates_dependent_formulas() {
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 1, 1, "5").unwrap();
    a.um.new_defined_name("OLD_NAME", None, "Sheet1!$A$1").unwrap();
    a.um.set_user_input(0, 3, 3, "=OLD_NAME*3").unwrap();
    sync(&mut a, &mut b);

    a.um.update_defined_name("OLD_NAME", None, "NEW_NAME", None, "Sheet1!$A$1")
        .unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(
        b.um.get_cell_content(0, 3, 3),
        Ok("=NEW_NAME*3".to_string())
    );
    assert_eq!(b.um.get_formatted_cell_value(0, 3, 3), Ok("15".to_string()));
}

#[test]
fn undo_of_defined_name_creation_propagates() {
    let mut a = replica(1);
    let mut b = replica(2);
    sync(&mut a, &mut b);
    a.um.new_defined_name("EPHEMERAL", None, "Sheet1!$A$1").unwrap();
    sync(&mut a, &mut b);
    assert_eq!(b.um.get_defined_name_list().len(), 1);

    a.um.undo().unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert!(a.um.get_defined_name_list().is_empty());
    assert!(b.um.get_defined_name_list().is_empty());
}

#[test]
fn formula_displacement_syncs_when_sequential() {
    // Sequential (not concurrent) structural edit: the displaced formula is
    // re-derived on the peer by replaying against converged state.
    let mut a = replica(1);
    let mut b = replica(2);
    a.um.set_user_input(0, 5, 1, "7").unwrap();
    a.um.set_user_input(0, 1, 2, "=A5").unwrap();
    sync(&mut a, &mut b);

    a.um.insert_rows(0, 3, 1).unwrap();
    sync(&mut a, &mut b);
    assert_converged(&a, &b);
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 2), Ok("7".to_string()));
}

// ---- sync-peer (transport protocol) tests ----------------------------------

use crate::crdt::SyncPeer;
use std::collections::VecDeque;

struct PeerReplica {
    um: UserModel<'static>,
    peer: SyncPeer,
}

fn peer_replica(client_id: u64) -> PeerReplica {
    let mut um = UserModel::from_model(new_empty_model());
    let peer = SyncPeer::attach(&mut um, client_id).unwrap();
    PeerReplica { um, peer }
}

/// Run both peers' connection handshakes and pump frames until quiescent.
fn connect(a: &mut PeerReplica, b: &mut PeerReplica) {
    let mut to_b: VecDeque<Vec<u8>> = a.peer.start_sync().into();
    let mut to_a: VecDeque<Vec<u8>> = b.peer.start_sync().into();
    let mut guard = 0;
    while !(to_a.is_empty() && to_b.is_empty()) {
        guard += 1;
        assert!(guard < 100, "handshake does not quiesce");
        if let Some(frame) = to_b.pop_front() {
            let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
            to_a.extend(outcome.replies);
        }
        if let Some(frame) = to_a.pop_front() {
            let outcome = a.peer.handle_frame(&mut a.um, &frame).unwrap();
            to_b.extend(outcome.replies);
        }
    }
}

/// Flush both peers' local edits and deliver the update frames, including any
/// replies they trigger.
fn flush_peers(a: &mut PeerReplica, b: &mut PeerReplica) {
    if let Some(frame) = a.peer.flush_local(&mut a.um).unwrap() {
        let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
        for reply in outcome.replies {
            a.peer.handle_frame(&mut a.um, &reply).unwrap();
        }
    }
    if let Some(frame) = b.peer.flush_local(&mut b.um).unwrap() {
        let outcome = a.peer.handle_frame(&mut a.um, &frame).unwrap();
        for reply in outcome.replies {
            b.peer.handle_frame(&mut b.um, &reply).unwrap();
        }
    }
}

#[test]
fn peer_handshake_syncs_offline_edits() {
    // Both sides edit before ever connecting; the y-sync handshake alone
    // (SyncStep1/SyncStep2 both ways) must converge them, including edits
    // still sitting untranslated in the local queue.
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    a.um.set_user_input(0, 1, 1, "10").unwrap();
    a.um.set_user_input(0, 2, 1, "=A1*2").unwrap();
    b.um.set_user_input(0, 1, 2, "offline").unwrap();
    connect(&mut a, &mut b);
    assert_models_converged(&a.um, &b.um);
    assert_eq!(b.um.get_formatted_cell_value(0, 2, 1), Ok("20".to_string()));
    assert_eq!(a.um.get_cell_content(0, 1, 2), Ok("offline".to_string()));
}

#[test]
fn peer_update_frames_flow_after_connect() {
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "5").unwrap();
    let frame = a.peer.flush_local(&mut a.um).unwrap().expect("an update");
    let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
    assert!(outcome.applied_update, "update frame must mark the model dirty");
    assert!(outcome.replies.is_empty(), "plain update needs no reply");
    assert_eq!(b.um.get_formatted_cell_value(0, 1, 1), Ok("5".to_string()));

    b.um.set_user_input(0, 1, 2, "=A1+1").unwrap();
    flush_peers(&mut a, &mut b);
    assert_models_converged(&a.um, &b.um);
    assert_eq!(a.um.get_formatted_cell_value(0, 1, 2), Ok("6".to_string()));
}

#[test]
fn peer_flush_without_edits_is_none() {
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);
    assert!(a.peer.flush_local(&mut a.um).unwrap().is_none());
}

#[test]
fn peer_does_not_echo_received_updates() {
    // Applying a remote update marks its blocks as sent: the receiver's next
    // flush must not re-broadcast them (star topology: the relay already
    // fanned them out).
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "5").unwrap();
    let frame = a.peer.flush_local(&mut a.um).unwrap().expect("an update");
    b.peer.handle_frame(&mut b.um, &frame).unwrap();
    assert!(
        b.peer.flush_local(&mut b.um).unwrap().is_none(),
        "receiving an update must not produce an outbound echo"
    );
}

#[test]
fn peer_idle_session_flush_is_the_empty_update() {
    // Guards the EMPTY_UPDATE_V1 constant the peer uses to suppress empty
    // frames: an idle session flush must encode as exactly [0, 0].
    let mut um = UserModel::from_model(new_empty_model());
    let mut session = CollabSession::attach(&mut um, 9).unwrap();
    let _bootstrap = session.flush_local(&mut um).unwrap();
    assert_eq!(session.flush_local(&mut um).unwrap(), vec![0u8, 0u8]);
}

#[test]
fn peer_tolerates_duplicated_and_reordered_frames() {
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "first").unwrap();
    let frame1 = a.peer.flush_local(&mut a.um).unwrap().expect("an update");
    a.um.set_user_input(0, 2, 1, "second").unwrap();
    let frame2 = a.peer.flush_local(&mut a.um).unwrap().expect("an update");

    // Deliver out of order: frame2 has a causal gap and parks in the pending
    // queue; frame1 fills the gap and both integrate; the duplicate is a
    // no-op.
    let outcome = b.peer.handle_frame(&mut b.um, &frame2).unwrap();
    assert!(!outcome.applied_update, "gapped update must be held back");
    assert!(
        !outcome.replies.is_empty(),
        "a gapped update must trigger a resync request"
    );
    let outcome = b.peer.handle_frame(&mut b.um, &frame1).unwrap();
    assert!(outcome.applied_update);
    b.peer.handle_frame(&mut b.um, &frame1).unwrap();
    assert_eq!(b.um.get_cell_content(0, 1, 1), Ok("first".to_string()));
    assert_eq!(b.um.get_cell_content(0, 2, 1), Ok("second".to_string()));
    assert_models_converged(&a.um, &b.um);
}

#[test]
fn peer_reconnect_handshake_heals_gaps() {
    // a↔b connected; c syncs with a only. b misses c's blocks (a does not
    // relay — that is the server's job); a fresh handshake heals b.
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    let mut c = peer_replica(3);
    connect(&mut a, &mut b);
    a.um.set_user_input(0, 1, 1, "from a").unwrap();
    flush_peers(&mut a, &mut b);

    c.um.set_user_input(0, 5, 5, "from c").unwrap();
    connect(&mut c, &mut a);
    assert_models_converged(&a.um, &c.um);

    assert_eq!(b.um.get_cell_content(0, 5, 5), Ok(String::new()));
    connect(&mut a, &mut b);
    assert_models_converged(&a.um, &b.um);
    assert_eq!(b.um.get_cell_content(0, 5, 5), Ok("from c".to_string()));
}

#[test]
fn peer_presence_exchange_and_clear() {
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);

    let frame = a.peer.set_presence(r#"{"name":"ana","cell":"A1"}"#).unwrap();
    let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
    assert!(outcome.presence_changed);
    assert!(!outcome.applied_update);
    assert_eq!(
        b.peer.presence(),
        vec![(1, r#"{"name":"ana","cell":"A1"}"#.to_string())]
    );

    let frame = b.peer.set_presence(r#"{"name":"bob"}"#).unwrap();
    a.peer.handle_frame(&mut a.um, &frame).unwrap();
    assert_eq!(a.peer.presence().len(), 2);

    let frame = a.peer.clear_presence().unwrap();
    let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
    assert!(outcome.presence_changed);
    assert_eq!(
        b.peer.presence(),
        vec![(2, r#"{"name":"bob"}"#.to_string())]
    );
}

#[test]
fn peer_presence_set_before_connect_travels_in_handshake() {
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    let _unsent = a.peer.set_presence(r#"{"name":"ana"}"#).unwrap();
    connect(&mut a, &mut b);
    assert_eq!(b.peer.presence(), vec![(1, r#"{"name":"ana"}"#.to_string())]);
}

#[cfg(not(target_arch = "wasm32"))]
fn peer_fuzz_round(seed: u64) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(seed);
    let mut a = peer_replica(1);
    let mut b = peer_replica(2);
    connect(&mut a, &mut b);

    // Frames in flight, either direction. Delivery may duplicate frames but
    // preserves order (a websocket pipe): yrs handles duplicates; reconnect
    // handshakes heal anything a partition dropped.
    let mut to_b: VecDeque<Vec<u8>> = VecDeque::new();
    let mut to_a: VecDeque<Vec<u8>> = VecDeque::new();
    for _step in 0..80 {
        let on_a = rng.gen_bool(0.5);
        let row = rng.gen_range(1..=20);
        let column = rng.gen_range(1..=6);
        match rng.gen_range(0..10) {
            0..=3 => {
                let value = format!("v{}", rng.gen::<u16>());
                let um = if on_a { &mut a.um } else { &mut b.um };
                um.set_user_input(0, row, column, &value).unwrap();
            }
            4 => {
                let target = rng.gen_range(1..=20);
                let um = if on_a { &mut a.um } else { &mut b.um };
                um.set_user_input(0, row, column, &format!("=A{target}+1"))
                    .unwrap();
            }
            5 => {
                let um = if on_a { &mut a.um } else { &mut b.um };
                um.insert_rows(0, row, 1).unwrap();
            }
            6 => {
                let um = if on_a { &mut a.um } else { &mut b.um };
                um.delete_rows(0, row, 1).unwrap();
            }
            7 => {
                let (peer, queue) = if on_a {
                    (&mut a.peer, &mut to_b)
                } else {
                    (&mut b.peer, &mut to_a)
                };
                let json = format!(r#"{{"cell":"R{row}C{column}"}}"#);
                queue.push_back(peer.set_presence(&json).unwrap());
            }
            8 => {
                // Flush pending edits into the pipe, sometimes duplicated.
                let (peer, um, queue) = if on_a {
                    (&mut a.peer, &mut a.um, &mut to_b)
                } else {
                    (&mut b.peer, &mut b.um, &mut to_a)
                };
                if let Some(frame) = peer.flush_local(um).unwrap() {
                    if rng.gen_bool(0.2) {
                        queue.push_back(frame.clone());
                    }
                    queue.push_back(frame);
                }
            }
            _ => {
                // Deliver one in-flight frame to the other side.
                let (peer, um, queue, back) = if on_a {
                    (&mut a.peer, &mut a.um, &mut to_a, &mut to_b)
                } else {
                    (&mut b.peer, &mut b.um, &mut to_b, &mut to_a)
                };
                if let Some(frame) = queue.pop_front() {
                    let outcome = peer.handle_frame(um, &frame).unwrap();
                    back.extend(outcome.replies);
                }
            }
        }
    }
    // Drain the pipes, then a final reconnect handshake and convergence check.
    while !(to_a.is_empty() && to_b.is_empty()) {
        if let Some(frame) = to_b.pop_front() {
            let outcome = b.peer.handle_frame(&mut b.um, &frame).unwrap();
            to_a.extend(outcome.replies);
        }
        if let Some(frame) = to_a.pop_front() {
            let outcome = a.peer.handle_frame(&mut a.um, &frame).unwrap();
            to_b.extend(outcome.replies);
        }
    }
    connect(&mut a, &mut b);
    assert_models_converged(&a.um, &b.um);
    assert_eq!(a.peer.presence().len(), b.peer.presence().len());
}

#[test]
fn randomized_peer_protocol_fuzz() {
    // Seeded and deterministic; CRDT_FUZZ_SEEDS=n stresses seeds 1..=n.
    let seeds: Vec<u64> = match std::env::var("CRDT_FUZZ_SEEDS") {
        Ok(n) => (1..=n.parse::<u64>().expect("CRDT_FUZZ_SEEDS must be a number")).collect(),
        Err(_) => vec![3, 11, 77, 4321, 123_456],
    };
    for seed in seeds {
        let result = std::panic::catch_unwind(|| peer_fuzz_round(seed));
        assert!(result.is_ok(), "peer_fuzz_round failed for seed {seed}");
    }
}

