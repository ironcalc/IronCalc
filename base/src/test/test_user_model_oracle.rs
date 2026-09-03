#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use crate::collab::model::{CollabModel, Stable};
use crate::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::types::{Ordinal, WorkbookView, WorksheetView};
use crate::UserModel;

fn pair() -> (UserModel<'static, Ordinal>, UserModel<'static, Stable>) {
    let ordinal = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    let mut model = CollabModel::new(1);
    model.new_sheet();
    seed_views(&mut model);
    (ordinal, UserModel::from_model(model))
}

fn seed_views(model: &mut CollabModel<'_>) {
    model.workbook.views.insert(
        0,
        WorkbookView {
            sheet: 0,
            window_width: DEFAULT_WINDOW_WIDTH,
            window_height: DEFAULT_WINDOW_HEIGHT,
        },
    );
    for worksheet in &mut model.workbook.worksheets {
        worksheet.views = HashMap::from([(
            0,
            WorksheetView {
                row: 1,
                column: 1,
                range: [1, 1, 1, 1],
                focus_row: 1,
                focus_column: 1,
                top_row: 1,
                left_column: 1,
            },
        )]);
    }
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
