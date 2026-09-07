#![allow(clippy::unwrap_used)]

use crate::collab::model::{CollabModel, Stable};
use crate::types::Ordinal;
use crate::UserModel;

fn pair() -> (UserModel<'static, Ordinal>, UserModel<'static, Stable>) {
    let ordinal = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    let stable = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();
    (ordinal, stable)
}

/// Compares everything the two wrappers are expected to answer identically over `rows` × `cols`.
/// `step` names the call just made, so a failure says which one broke.
fn compare(o: &UserModel, c: &UserModel<Stable>, rows: i32, cols: i32, step: &str) {
    for row in 1..=rows {
        for col in 1..=cols {
            assert_eq!(
                c.get_formatted_cell_value(0, row, col),
                o.get_formatted_cell_value(0, row, col),
                "value at ({row}, {col}) after {step}"
            );
            assert_eq!(
                c.get_cell_content(0, row, col),
                o.get_cell_content(0, row, col),
                "content at ({row}, {col}) after {step}"
            );
            assert_eq!(
                c.get_cell_style(0, row, col),
                o.get_cell_style(0, row, col),
                "style at ({row}, {col}) after {step}"
            );
        }
        assert_eq!(
            c.get_row_height(0, row),
            o.get_row_height(0, row),
            "height of row {row} after {step}"
        );
    }
    for col in 1..=cols {
        assert_eq!(
            c.get_column_width(0, col),
            o.get_column_width(0, col),
            "width of column {col} after {step}"
        );
    }
    assert_eq!(
        c.get_frozen_rows_count(0),
        o.get_frozen_rows_count(0),
        "frozen rows after {step}"
    );
    assert_eq!(
        c.get_frozen_columns_count(0),
        o.get_frozen_columns_count(0),
        "frozen columns after {step}"
    );
    assert_eq!(
        c.get_worksheets_properties()
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>(),
        o.get_worksheets_properties()
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>(),
        "sheet names after {step}"
    );
}

#[test]
fn user_model_surface_matches() {
    let (mut o, mut c) = pair();

    for (row, col, value) in [
        (1, 1, "10"),
        (2, 1, "20"),
        (3, 1, "text"),
        (1, 2, "=A1+A2"),
        (2, 2, "=B1*2"),
        (3, 2, "=CONCAT(A3, \"!\")"),
    ] {
        o.set_user_input(0, row, col, value).unwrap();
        c.set_user_input(0, row, col, value).unwrap();
    }
    compare(&o, &c, 4, 3, "set_user_input");

    o.set_rows_height(0, 1, 3, 42.0).unwrap();
    c.set_rows_height(0, 1, 3, 42.0).unwrap();
    o.set_columns_width(0, 1, 2, 150.0).unwrap();
    c.set_columns_width(0, 1, 2, 150.0).unwrap();
    compare(&o, &c, 4, 3, "plural sizing");

    o.set_frozen_rows_count(0, 1).unwrap();
    c.set_frozen_rows_count(0, 1).unwrap();
    o.set_frozen_columns_count(0, 2).unwrap();
    c.set_frozen_columns_count(0, 2).unwrap();
    compare(&o, &c, 4, 3, "frozen counts");

    o.insert_rows(0, 2, 2).unwrap();
    c.insert_rows(0, 2, 2).unwrap();
    compare(&o, &c, 6, 3, "insert_rows");

    o.delete_rows(0, 2, 1).unwrap();
    c.delete_rows(0, 2, 1).unwrap();
    compare(&o, &c, 6, 3, "delete_rows");

    o.rename_sheet(0, "Data").unwrap();
    c.rename_sheet(0, "Data").unwrap();
    compare(&o, &c, 6, 3, "rename_sheet");

    o.new_defined_name("total", None, "Data!$A$1").unwrap();
    c.new_defined_name("total", None, "Data!$A$1").unwrap();
    o.set_user_input(0, 5, 3, "=total").unwrap();
    c.set_user_input(0, 5, 3, "=total").unwrap();
    compare(&o, &c, 6, 3, "defined name");
    // Something for the clear to remove. The wrapper's styling surface is a later round, so the
    // style is written on the models themselves.
    let mut style = o.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    for (row, col) in [(1, 1), (2, 2)] {
        o.model.set_cell_style(0, row, col, &style).unwrap();
        c.model.set_cell_style(0, row, col, &style).unwrap();
    }
    compare(&o, &c, 6, 3, "set_cell_style");

    let range = crate::expressions::types::Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 2,
        height: 3,
    };
    o.range_clear_formatting(&range).unwrap();
    c.range_clear_formatting(&range).unwrap();
    compare(&o, &c, 6, 3, "range_clear_formatting");
}

#[test]
fn navigation_reads_stable_metrics() {
    let (mut o, mut c) = pair();

    o.set_rows_height(0, 1, 1, 200.0).unwrap();
    c.set_rows_height(0, 1, 1, 200.0).unwrap();
    o.set_columns_hidden(0, 2, 3, true).unwrap();
    c.set_columns_hidden(0, 2, 3, true).unwrap();

    // The hidden columns are skipped: B and C are gone, so the right arrow lands on D.
    o.set_selected_cell(1, 1).unwrap();
    c.set_selected_cell(1, 1).unwrap();
    o.on_arrow_right().unwrap();
    c.on_arrow_right().unwrap();
    assert_eq!(o.get_selected_cell(), (0, 1, 4));
    assert_eq!(c.get_selected_cell(), o.get_selected_cell());

    o.on_arrow_down().unwrap();
    c.on_arrow_down().unwrap();
    assert_eq!(o.get_selected_cell(), (0, 2, 4));
    assert_eq!(c.get_selected_cell(), o.get_selected_cell());

    // The tall first row is what the scroll offset is made of.
    assert_eq!(c.get_scroll_y(), o.get_scroll_y());
    assert_eq!(c.get_column_width(0, 2), Ok(0.0));
}

#[test]
fn plural_setters_are_idempotent() {
    let mut model = CollabModel::new(1);
    model.new_sheet();
    let mut user_model = UserModel::<Stable>::from_model(model);

    user_model.set_rows_height(0, 1, 5, 30.0).unwrap();
    user_model.set_columns_width(0, 1, 4, 120.0).unwrap();
    user_model.set_rows_hidden(0, 1, 5, true).unwrap();

    assert_eq!(user_model.get_model().local.pending.len(), 4);
    let pending = user_model.get_model().local.pending.len();

    // The same calls again change nothing
    user_model.set_rows_height(0, 1, 5, 30.0).unwrap();
    user_model.set_columns_width(0, 1, 4, 120.0).unwrap();
    user_model.set_rows_hidden(0, 1, 5, true).unwrap();
    assert_eq!(user_model.get_model().local.pending.len(), pending);
}

/// Undoes on both wrappers and checks they still agree — including on what is undoable next.
fn undo_both(o: &mut UserModel<'_, Ordinal>, c: &mut UserModel<'_, Stable>, step: &str) {
    assert_eq!(c.can_undo(), o.can_undo(), "can_undo before {step}");
    o.undo().unwrap();
    c.undo().unwrap();
    compare(o, c, 8, 4, step);
}

/// The mirror of [`undo_both`].
fn redo_both(o: &mut UserModel<'_, Ordinal>, c: &mut UserModel<'_, Stable>, step: &str) {
    assert_eq!(c.can_redo(), o.can_redo(), "can_redo before {step}");
    o.redo().unwrap();
    c.redo().unwrap();
    compare(o, c, 8, 4, step);
}

/// Runs the same call on both wrappers and compares.
macro_rules! both {
    ($o:ident, $c:ident, $method:ident($($arg:expr),*), $step:expr) => {{
        $o.$method($($arg),*).unwrap();
        $c.$method($($arg),*).unwrap();
        compare(&$o, &$c, 8, 4, $step);
    }};
}

#[test]
fn undo_redo_matches_ordinal() {
    let (mut o, mut c) = pair();

    // Undo and redo on an empty history: a no-op on both, not an error.
    undo_both(&mut o, &mut c, "undo on an empty stack");
    redo_both(&mut o, &mut c, "redo on an empty stack");

    // A grid, including a multiline value: that one action is a write plus an auto-fit.
    both!(o, c, set_user_input(0, 1, 1, "10"), "A1");
    both!(o, c, set_user_input(0, 2, 1, "20"), "A2");
    both!(o, c, set_user_input(0, 1, 2, "=A1+A2"), "B1");
    both!(o, c, set_user_input(0, 3, 3, "line1\nline2"), "C3");
    both!(o, c, insert_rows(0, 2, 2), "insert_rows");
    both!(o, c, rename_sheet(0, "Data"), "rename_sheet");
    both!(o, c, set_rows_height(0, 1, 3, 42.0), "set_rows_height");
    both!(
        o,
        c,
        new_defined_name("total", None, "Data!$A$1"),
        "new_defined_name"
    );

    let steps = [
        "undo new_defined_name",
        "undo set_rows_height",
        "undo rename_sheet",
        "undo insert_rows",
        "undo C3",
        "undo B1",
        "undo A2",
        "undo A1",
    ];
    for step in steps {
        undo_both(&mut o, &mut c, step);
    }
    for step in steps.iter().rev() {
        redo_both(&mut o, &mut c, step);
    }
    assert!(!c.can_redo());
    assert_eq!(c.can_redo(), o.can_redo());
    redo_both(&mut o, &mut c, "redo past the top");

    // Partial unwind, then a fresh edit: the redo branch is dropped on both.
    for step in ["undo 1 of 3", "undo 2 of 3", "undo 3 of 3"] {
        undo_both(&mut o, &mut c, step);
    }
    assert!(c.can_redo() && o.can_redo());
    both!(o, c, set_user_input(0, 8, 4, "99"), "fresh edit");
    assert!(!c.can_redo(), "a fresh edit clears the redo branch");
    assert_eq!(c.can_redo(), o.can_redo());
    undo_both(&mut o, &mut c, "undo the fresh edit");
    redo_both(&mut o, &mut c, "redo the fresh edit");

    // Something for the clear to remove. The wrapper's styling surface is a later round, so the
    // style is written on the models themselves — outside either wrapper's history.
    let mut style = o.get_cell_style(0, 1, 1).unwrap();
    style.font.b = true;
    for (row, col) in [(6, 1), (7, 2)] {
        o.model.set_cell_style(0, row, col, &style).unwrap();
        c.model.set_cell_style(0, row, col, &style).unwrap();
    }
    let range = crate::expressions::types::Area {
        sheet: 0,
        row: 6,
        column: 1,
        width: 2,
        height: 2,
    };
    both!(
        o,
        c,
        range_clear_formatting(&range),
        "range_clear_formatting"
    );
    undo_both(&mut o, &mut c, "undo range_clear_formatting");
    redo_both(&mut o, &mut c, "redo range_clear_formatting");
    // The unwind stops here: undoing a value write un-materializes its cell under stable
    // addressing, so a style that outlives the value — and the row itself — does not survive it.
}

#[test]
fn redo_relowers_formulas_on_revived_rows() {
    let mut c = UserModel::new_empty_with_session("model", "en", "UTC", "en", 1).unwrap();

    c.set_user_input(0, 5, 1, "7").unwrap(); // A5=7
    c.set_user_input(0, 1, 2, "=A5").unwrap(); // B1=A5
    assert_eq!(c.get_formatted_cell_value(0, 1, 2).unwrap(), "7");

    c.undo().unwrap(); // clears B1
    c.undo().unwrap(); // clears A5 and deletes row 5's identity
    let before = c.get_model().local.full_resyncs;
    c.redo().unwrap(); // revives row 5
    c.redo().unwrap(); // restores B1's formula over the revived key
    assert!(
        c.get_model().local.full_resyncs > before,
        "a revival must take the structural resync path"
    );
    assert_eq!(c.get_formatted_cell_value(0, 1, 2).unwrap(), "7");

    // The same commits arrive at a peer through `apply`: the revival is caught there too.
    let mut b = UserModel::<Stable>::from_model(CollabModel::new(2));
    b.apply_external_diffs(&c.flush_send_queue()).unwrap();
    assert_eq!(b.get_cell_content(0, 1, 2).unwrap(), "=A5");
    assert_eq!(b.get_formatted_cell_value(0, 1, 2).unwrap(), "7");
}

#[test]
fn peers_converge_through_the_wire() {
    let mut a = UserModel::<Stable>::from_model(CollabModel::new(1));
    let mut b = UserModel::<Stable>::from_model(CollabModel::new(2));

    a.new_sheet().unwrap();
    a.set_user_input(0, 1, 1, "10").unwrap(); // A1=10
    b.apply_external_diffs(&a.flush_send_queue()).unwrap();
    assert_eq!(b.get_formatted_cell_value(0, 1, 1).unwrap(), "10");
    // B has nothing of its own to undo.
    assert!(!b.can_undo() && !b.can_redo());

    a.undo().unwrap(); // clear A1
    b.apply_external_diffs(&a.flush_send_queue()).unwrap();
    assert_eq!(b.get_formatted_cell_value(0, 1, 1).unwrap(), "");
    assert_eq!(a.get_model().workbook, b.get_model().workbook);

    // The race: B writes A1 while A undoes its own earlier write of A1.
    a.set_user_input(0, 1, 1, "1").unwrap(); // A: A1=1
    b.apply_external_diffs(&a.flush_send_queue()).unwrap();
    b.set_user_input(0, 1, 1, "2").unwrap(); // B: A1=2
    let b_edit = b.flush_send_queue();
    a.undo().unwrap();
    let a_undo = a.flush_send_queue();
    // Delivered in opposite orders; last stamp wins on both.
    a.apply_external_diffs(&b_edit).unwrap();
    b.apply_external_diffs(&a_undo).unwrap();
    assert_eq!(
        a.get_formatted_cell_value(0, 1, 1),
        b.get_formatted_cell_value(0, 1, 1)
    );
    assert_eq!(a.get_model().workbook, b.get_model().workbook);
}

#[test]
fn snapshot_round_trips_into_a_working_replica() {
    let mut author = UserModel::new_empty_with_session("book", "en", "UTC", "en", 1).unwrap();
    author.set_user_input(0, 1, 1, "7").unwrap(); // A1=7
    author.set_user_input(0, 1, 2, "=A1").unwrap(); // B1=A1
    author.set_rows_height(0, 1, 1, 42.0).unwrap();
    author.rename_sheet(0, "Data").unwrap();
    // Nobody was listening: the author's own state is complete without the queue.
    author.flush_send_queue();

    let mut restored = UserModel::<Stable>::from_bytes_with_session(&author.to_bytes(), 2).unwrap();
    assert_eq!(restored.get_formatted_cell_value(0, 1, 2).unwrap(), "7");
    assert_eq!(restored.get_row_height(0, 1).unwrap(), 42.0);
    assert_eq!(restored.get_worksheets_properties()[0].name, "Data");
    // Views are local state, absent from the payload, so the restore seeds them.
    restored.set_selected_cell(3, 2).unwrap();
    assert_eq!(restored.get_selected_cell(), (0, 3, 2));

    restored.insert_rows(0, 1, 1).unwrap();
    restored.set_user_input(0, 1, 1, "99").unwrap();
    author
        .apply_external_diffs(&restored.flush_send_queue())
        .unwrap();

    for row in 1..=3 {
        for col in 1..=2 {
            assert_eq!(
                author.get_formatted_cell_value(0, row, col).unwrap(),
                restored.get_formatted_cell_value(0, row, col).unwrap(),
                "value at ({row}, {col})"
            );
        }
    }
    assert_eq!(author.get_formatted_cell_value(0, 1, 1).unwrap(), "99");
    assert_eq!(author.get_formatted_cell_value(0, 2, 2).unwrap(), "7");

    // The restoring replica mints under its own session, never the author's.
    let sheet = &restored.get_model().workbook.worksheets[0];
    let key = <Stable as crate::types::Position>::row_at(&sheet.index, 1).unwrap();
    assert_eq!(key.split().1, 2u32.to_be_bytes());
}
