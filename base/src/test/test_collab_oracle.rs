#![allow(clippy::unwrap_used)]

//! Differential oracle: the ordinal [`Model`] is the spec, so a scripted mutator sequence run on it
//! and on a [`CollabModel`] has to leave both showing the same values and the same formula texts.

use crate::collab::model::CollabModel;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::model::Model;

/// The pair the scenarios drive. Both start with a single `Sheet1`.
fn pair() -> (Model<'static>, CollabModel<'static>) {
    let ordinal = Model::new_empty("model", "en", "UTC", "en").unwrap();
    let mut collab = CollabModel::new(1);
    collab.new_sheet();
    (ordinal, collab)
}

/// Compares the two models cell for cell over `sheets` × rows `1..=rows` × columns `1..=cols`.
/// `step` names the mutation just applied, so a failure says which one broke.
fn compare(
    ordinal: &Model,
    collab: &CollabModel,
    sheets: &[u32],
    rows: i32,
    cols: i32,
    step: &str,
) {
    for &sheet in sheets {
        for row in 1..=rows {
            for col in 1..=cols {
                assert_eq!(
                    collab.get_formatted_cell_value(sheet, row, col),
                    ordinal.get_formatted_cell_value(sheet, row, col),
                    "value at sheet {sheet} row {row} column {col} after {step}"
                );
                assert_eq!(
                    collab.get_cell_formula(sheet, row, col),
                    ordinal.get_cell_formula(sheet, row, col),
                    "formula at sheet {sheet} row {row} column {col} after {step}"
                );
            }
        }
    }
}

/// Writes the same input to both models and evaluates them.
fn set(o: &mut Model, c: &mut CollabModel, sheet: u32, row: i32, col: i32, value: &str) {
    o.set_user_input(sheet, row, col, value.to_string())
        .unwrap();
    c.set_user_input(sheet, row, col, value.to_string())
        .unwrap();
    o.evaluate();
    c.evaluate();
}

#[test]
fn values_and_references() {
    let (mut o, mut c) = pair();
    // A second sheet, named so it needs quoting in a reference.
    o.new_sheet();
    c.new_sheet();
    o.rename_sheet_by_index(1, "My Sheet").unwrap();
    c.rename_sheet_by_index(1, "My Sheet").unwrap();

    for (row, col, value) in [
        (1, 1, "10"),
        (2, 1, "-2.5"),
        (3, 1, "hello"),
        (4, 1, "TRUE"),
        (1, 2, "1"),
        (2, 2, "2"),
        (3, 2, "3"),
    ] {
        set(&mut o, &mut c, 0, row, col, value);
    }
    set(&mut o, &mut c, 1, 1, 1, "42");
    set(&mut o, &mut c, 1, 2, 1, "world");

    for (row, col, formula) in [
        (1, 3, "=A1"),
        (2, 3, "=$A$1"),
        (3, 3, "=A$1"),
        (4, 3, "=$A1+B2"),
        (5, 3, "=SUM(A1:B3)"),
        (6, 3, "=A3&\" \"&A4"),
        (1, 4, "=Sheet1!A1*2"),
        (2, 4, "='My Sheet'!A1"),
        (3, 4, "='My Sheet'!A2"),
        (4, 4, "=SUM('My Sheet'!A1:A2)"),
        // Nothing was ever written there: the reference reads an unmaterialized cell.
        (5, 4, "=Z90+1"),
        (6, 4, "=COUNT(A1:A100)"),
    ] {
        set(&mut o, &mut c, 0, row, col, formula);
    }
    // A cross-sheet formula living on the other sheet too.
    set(&mut o, &mut c, 1, 3, 1, "=Sheet1!A1+Sheet1!B1");

    compare(&o, &c, &[0, 1], 8, 6, "seed");
}

#[test]
fn structural_displacement() {
    let (mut o, mut c) = pair();
    o.new_sheet();
    c.new_sheet();
    o.rename_sheet_by_index(1, "Data").unwrap();
    c.rename_sheet_by_index(1, "Data").unwrap();

    // Values live in rows 3..=8 and formulas in rows 1..=2, so the edits below displace what the
    // formulas point at instead of deleting the formulas themselves.
    for row in 3..=8 {
        set(&mut o, &mut c, 0, row, 1, &format!("{row}"));
        set(&mut o, &mut c, 0, row, 2, &format!("{}", row * 10));
    }
    for row in 1..=6 {
        set(&mut o, &mut c, 1, row, 1, &format!("{}", row * 100));
        set(&mut o, &mut c, 1, row, 2, &format!("{}", row * 3));
    }
    for (row, col, formula) in [
        (1, 4, "=A5*2"),
        (1, 5, "=SUM(A3:A8)"),
        (1, 6, "=$A$6+1"),
        (1, 7, "=Data!A4"),
        (1, 8, "=SUM(Data!B1:B3)"),
        (2, 4, "=A$7"),
        (2, 5, "=D1+1"),
        (2, 6, "=SUM(B3:B5)"),
    ] {
        set(&mut o, &mut c, 0, row, col, formula);
    }
    set(&mut o, &mut c, 1, 1, 4, "=Sheet1!A3+Sheet1!$B$4");
    let sheets = [0, 1];
    compare(&o, &c, &sheets, 14, 10, "seed");

    o.insert_rows(0, 5, 2).unwrap();
    c.insert_rows(0, 5, 2).unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 14, 10, "insert_rows");

    // Left of every formula: the formulas travel and their references to A and B shift with them.
    o.insert_columns(0, 2, 1).unwrap();
    c.insert_columns(0, 2, 1).unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 14, 10, "insert_columns");

    o.delete_rows(0, 4, 2).unwrap();
    c.delete_rows(0, 4, 2).unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 14, 10, "delete_rows");

    // Row 6 is what `=$A$6+1` names after those edits: deleting it has to break the reference.
    o.delete_rows(0, 6, 1).unwrap();
    c.delete_rows(0, 6, 1).unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 14, 10, "delete_rows onto a reference");

    // Column A of Data is what the cross-sheet formulas name.
    o.delete_columns(1, 1, 1).unwrap();
    c.delete_columns(1, 1, 1).unwrap();
    o.evaluate();
    c.evaluate();
    compare(
        &o,
        &c,
        &sheets,
        14,
        10,
        "delete_columns on the referenced sheet",
    );

    o.rename_sheet("Data", "Other Data").unwrap();
    c.rename_sheet("Data", "Other Data").unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 14, 10, "rename_sheet");
}

#[test]
fn defined_names() {
    let (mut o, mut c) = pair();
    o.new_sheet();
    c.new_sheet();
    o.rename_sheet_by_index(1, "Data").unwrap();
    c.rename_sheet_by_index(1, "Data").unwrap();

    for row in 1..=4 {
        set(&mut o, &mut c, 0, row, 1, &format!("{row}"));
        set(&mut o, &mut c, 1, row, 1, &format!("{}", row * 7));
    }

    let names = |o: &Model, c: &CollabModel, step: &str| {
        let (mut a, mut b) = (o.get_defined_name_list(), c.get_defined_name_list());
        a.sort();
        b.sort();
        assert_eq!(b, a, "defined names after {step}");
    };

    for (name, scope, formula) in [
        ("total", None, "Sheet1!$A$1:$A$4"),
        ("first", None, "Sheet1!$A$1"),
        ("local", Some(1), "Data!$A$2"),
        ("remote", None, "Data!$A$1:$A$4"),
    ] {
        o.new_defined_name(name, scope, formula).unwrap();
        c.new_defined_name(name, scope, formula).unwrap();
    }
    set(&mut o, &mut c, 0, 1, 3, "=SUM(total)");
    set(&mut o, &mut c, 0, 2, 3, "=first*10");
    set(&mut o, &mut c, 0, 3, 3, "=SUM(remote)");
    set(&mut o, &mut c, 1, 1, 3, "=local+1");
    let sheets = [0, 1];
    compare(&o, &c, &sheets, 6, 4, "create");
    names(&o, &c, "create");

    // A new body for an existing name.
    o.update_defined_name("first", None, "first", None, "Sheet1!$A$3")
        .unwrap();
    c.update_defined_name("first", None, "first", None, "Sheet1!$A$3")
        .unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 6, 4, "redefine");
    names(&o, &c, "redefine");

    // A rename, leaving the body alone.
    o.update_defined_name("total", None, "grand_total", None, "Sheet1!$A$1:$A$4")
        .unwrap();
    c.update_defined_name("total", None, "grand_total", None, "Sheet1!$A$1:$A$4")
        .unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 6, 4, "rename name");
    names(&o, &c, "rename name");

    // The sheet two bodies name changes: both bodies have to follow.
    o.rename_sheet("Data", "Data Two").unwrap();
    c.rename_sheet("Data", "Data Two").unwrap();
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 6, 4, "rename sheet");
    names(&o, &c, "rename sheet");
}

/// FINDING: a range whose endpoint a delete turned into `#REF!` renders identically on both models
/// but evaluates differently — the ordinal model says `#VALUE!`, the collaborative one `#REF!`.
#[test]
fn broken_range_endpoint_value() {
    let (mut o, mut c) = pair();
    for row in 1..=3 {
        set(&mut o, &mut c, 0, row, 1, &format!("{}", row * 100));
        set(&mut o, &mut c, 0, row, 2, &format!("{}", row * 3));
    }
    set(&mut o, &mut c, 0, 1, 5, "=SUM(A1:B3)");
    o.delete_columns(0, 1, 1).unwrap();
    c.delete_columns(0, 1, 1).unwrap();
    o.evaluate();
    c.evaluate();
    // The formula text agrees.
    assert_eq!(
        c.get_cell_formula(0, 1, 4),
        o.get_cell_formula(0, 1, 4),
        "formula at sheet 0 row 1 column 4"
    );
    // atm. ordinal / stable models differ here:
    // - Ordinal is #VALUE!
    // - Stable is #REF!
    let ordinal = o.get_formatted_cell_value(0, 1, 4).unwrap();
    let stable = c.get_formatted_cell_value(0, 1, 4).unwrap();
    assert!(
        ordinal.starts_with('#') && stable.starts_with('#'),
        "both must error at sheet 0 row 1 column 4: ordinal {ordinal:?}, stable {stable:?}"
    );
}

#[test]
fn tail_moves() {
    let (mut o, mut c) = pair();
    for row in 1..=5 {
        set(&mut o, &mut c, 0, row, 1, &format!("{row}")); // A1:A5=1..5
    }
    set(&mut o, &mut c, 0, 3, 2, "=SUM(A1:A2)"); // B3=SUM(A1:A2)
    set(&mut o, &mut c, 0, 4, 2, "=SUM(A3:A5)"); // B4=SUM(A3:A5)
    set(&mut o, &mut c, 0, 5, 2, "=A2*10"); // B5=A2*10
    let sheets = [0];
    compare(&o, &c, &sheets, 16, 5, "seed");

    // A block of materialized rows landing past the tail.
    assert!(o.move_rows_action(0, 1, 2, 10).is_ok());
    assert!(c.move_rows_action(0, 1, 2, 10).is_ok());
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 16, 5, "move_rows past the tail");

    // The very last column: source, destination and everything between are unmaterialized.
    assert!(o.move_columns_action(0, LAST_COLUMN, 1, -1).is_ok());
    assert!(c.move_columns_action(0, LAST_COLUMN, 1, -1).is_ok());
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 16, 5, "move_columns at the last column");

    // The same shape on rows.
    assert!(o.move_rows_action(0, LAST_ROW, 1, -1).is_ok());
    assert!(c.move_rows_action(0, LAST_ROW, 1, -1).is_ok());
    o.evaluate();
    c.evaluate();
    compare(&o, &c, &sheets, 16, 5, "move_rows at the last row");
}
