#![allow(clippy::unwrap_used)]

//! The `Ordinal` ⇄ `Stable` projection. An ordinal worksheet imported into the collaborative model
//! and projected straight back has to be the worksheet we started with, field for field.

use crate::cf_types::{CfRuleInput, ValueOperator};
use crate::collab::model::CollabModel;
use crate::test::util::new_empty_model;
use crate::types::{Col, Color, Comment, Dxf, Fill, MergedCell, RangeRef, Worksheet};

/// Rows and columns without the style index, which is local to each workbook's style table.
fn row_shape(ws: &Worksheet) -> Vec<(i32, f64, bool, bool, bool)> {
    let mut rows: Vec<_> = ws
        .rows
        .iter()
        .map(|r| (r.r, r.height, r.hidden, r.custom_height, r.custom_format))
        .collect();
    rows.sort_by_key(|r| r.0);
    rows
}

fn col_shape(ws: &Worksheet) -> Vec<(i32, i32, f64, bool, bool)> {
    let mut cols: Vec<_> = ws
        .cols
        .iter()
        .map(|c| (c.min, c.max, c.width, c.custom_width, c.hidden))
        .collect();
    cols.sort_by_key(|c| (c.0, c.1));
    cols
}

#[test]
fn projection_identity() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 1, "42".to_string()).unwrap();
    model.set_user_input(0, 1, 2, "text".to_string()).unwrap();
    model.set_user_input(0, 2, 1, "=A1*2".to_string()).unwrap();
    model.set_row_height(0, 3, 30.0).unwrap();
    let mut style = model.get_style_for_cell(0, 1, 1).unwrap();
    style.font.b = true;
    model.set_cell_style(0, 1, 1, &style).unwrap();
    model
        .add_conditional_formatting(
            0,
            "A1:B5",
            CfRuleInput::CellIs {
                operator: ValueOperator::GreaterThan,
                formula: "5".to_string(),
                formula2: None,
                format: Dxf {
                    fill: Some(Fill {
                        color: Color::Rgb("#FF0000".to_string()),
                    }),
                    ..Default::default()
                },
                stop_if_true: false,
            },
        )
        .unwrap();
    model.evaluate();
    {
        let ws = model.workbook.worksheet_mut(0).unwrap();
        ws.cols.push(Col {
            min: 2,
            max: 4,
            width: 20.0,
            custom_width: true,
            hidden: false,
            style: None,
        });
        ws.merged_cells.push(MergedCell {
            row: 3,
            column: 3,
            width: 2,
            height: 2,
        });
        ws.comments.push(Comment {
            text: "hi".to_string(),
            author_name: "me".to_string(),
            author_id: None,
            cell_ref: (5, 2),
        });
    }
    let ws = &model.workbook.worksheets[0];

    let mut a = CollabModel::from_workbook_with_session(model.workbook.clone(), "en", 1).unwrap();
    a.evaluate();
    let projected = a.to_ordinal_workbook();
    let out = &projected.worksheets[0];

    // Everything the projection carries. `dimension`, `views`, `links` and the order of
    // `shared_formulas` are the model's own bookkeeping, not the sheet's shape.
    assert_eq!(out.name, ws.name);
    assert_eq!(out.sheet_id, ws.sheet_id);
    assert_eq!(out.state, ws.state);
    assert_eq!(out.color, ws.color);
    assert_eq!(out.show_grid_lines, ws.show_grid_lines);
    assert_eq!(out.frozen_rows, ws.frozen_rows);
    assert_eq!(out.frozen_columns, ws.frozen_columns);
    assert_eq!(row_shape(out), row_shape(ws));
    assert_eq!(col_shape(out), col_shape(ws));
    assert_eq!(out.merged_cells, ws.merged_cells);
    assert_eq!(out.comments, ws.comments);
    let ranges: Vec<_> = out
        .conditional_formatting
        .iter()
        .map(|cf| cf.ranges.clone())
        .collect();
    let expected: Vec<_> = ws
        .conditional_formatting
        .iter()
        .map(|cf| cf.ranges.clone())
        .collect();
    assert_eq!(ranges, expected);

    // Cells come back as they went in. A formula cell's stream is bound, not copied, so it is
    // compared by the text the two models read out of it.
    for (r, cells) in &ws.sheet_data {
        for (c, cell) in cells {
            let back = out.sheet_data.get(r).and_then(|row| row.get(c));
            if cell.has_formula() {
                assert!(back.unwrap().has_formula(), "formula lost at {r}:{c}");
                assert_eq!(
                    a.get_cell_formula(0, *r, *c),
                    model.get_cell_formula(0, *r, *c),
                    "formula text at {r}:{c}"
                );
            } else {
                assert_eq!(
                    back.map(|b| (b.get_type(), b.get_style() != 0)),
                    Some((cell.get_type(), cell.get_style() != 0)),
                    "cell at {r}:{c}"
                );
            }
            assert_eq!(
                a.get_formatted_cell_value(0, *r, *c),
                model.get_formatted_cell_value(0, *r, *c),
                "value at {r}:{c}"
            );
        }
    }
    assert!(a.get_style_for_cell(0, 1, 1).unwrap().font.b);

    // The identity above must come from the mapping, not from the projection being a no-op: reorder
    // the rows on the stable side and every ordinal the projection reports has to follow. Nothing
    // public moves a row without also displacing what points at it, so the index is moved directly.
    a.workbook.worksheets[0].index.rows.move_to(0..1, 5); // first row of five to the end
    let projected = a.to_ordinal_workbook();
    let moved = &projected.worksheets[0];
    assert_ne!(moved.sheet_data, out.sheet_data);
    for (before, after) in [(1, 5), (2, 1), (3, 2), (4, 3), (5, 4)] {
        assert_eq!(
            moved.sheet_data.get(&after),
            out.sheet_data.get(&before),
            "row {before} should now read at {after}"
        );
    }
    // The one custom row sat at 3 and reads at 2 now that the first row went to the end.
    let mut expected_rows = row_shape(out);
    expected_rows[0].0 -= 1;
    assert_eq!(row_shape(moved), expected_rows);
    assert_eq!(moved.comments[0].cell_ref, (4, 2));
    assert_eq!(
        moved.merged_cells,
        vec![MergedCell {
            row: 2,
            column: 3,
            width: 2,
            height: 2,
        }]
    );
    // The projection maps corners, it does not re-normalize them: A1:B5 comes back inverted because
    // its top row is now the bottom one.
    assert_eq!(
        moved.conditional_formatting[0].ranges,
        vec![RangeRef {
            rows: Some((5, 4)),
            cols: Some((1, 2))
        }]
    );
    // Nothing on the column axis moved.
    assert_eq!(col_shape(moved), col_shape(out));
}
