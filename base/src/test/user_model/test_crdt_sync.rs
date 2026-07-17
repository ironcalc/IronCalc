#![allow(clippy::unwrap_used)]

//! Two-replica convergence tests for the CRDT collaboration session.
//!
//! Pattern: both replicas start from the same (empty) workbook, perform
//! concurrent edits, exchange updates, and must end cell-by-cell identical —
//! including evaluation results, which are never shipped.

use crate::crdt::CollabSession;
use crate::test::util::new_empty_model;
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
    }
}

const WINDOW_ROWS: i32 = 40;
const WINDOW_COLUMNS: i32 = 15;

/// Asserts both replicas are identical over a viewing window: sheet names,
/// cell contents, formatted (evaluated) values, row heights and hidden flags.
fn assert_converged(a: &Replica, b: &Replica) {
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
    let seeds: Vec<u64> = match std::env::var("CRDT_FUZZ_SEEDS") {
        Ok(n) => (1..=n.parse::<u64>().expect("CRDT_FUZZ_SEEDS must be a number")).collect(),
        Err(_) => vec![1, 7, 42, 1234, 987_654],
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
            match rng.gen_range(0..16) {
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
