#![allow(clippy::unwrap_used)]

//! The `Ordinal` ⇄ `Stable` projection. Building a stable sheet out of an ordinal one and projecting
//! it straight back has to give the sheet we started with, field for field.

use std::collections::HashMap;

use crate::cf_types::{CfRuleInput, ConditionalFormatting, ValueOperator};
use crate::collab::fractional_index::{virtual_key, FractionalKey};
use crate::collab::model::{SheetIndexes, Stable, StableRange};
use crate::test::util::new_empty_model;
use crate::types::{Col, Color, Comment, Dxf, Fill, MergedCell, Position, RangeRef, Row, Worksheet};

/// How far the sheet reaches on each axis. Only the used range gets keys — a stable sheet has no
/// notion of the million rows an ordinal one addresses implicitly.
fn used_extent(ws: &Worksheet) -> (i32, i32) {
    let (mut rows, mut cols) = (0, 0);
    for (r, row) in &ws.sheet_data {
        rows = rows.max(*r);
        for c in row.keys() {
            cols = cols.max(*c);
        }
    }
    for r in &ws.rows {
        rows = rows.max(r.r);
    }
    for c in &ws.cols {
        cols = cols.max(c.max);
    }
    for m in &ws.merged_cells {
        rows = rows.max(m.last_row());
        cols = cols.max(m.last_column());
    }
    for range in ws.conditional_formatting.iter().flat_map(|cf| &cf.ranges) {
        if let Some((_, hi)) = range.rows {
            rows = rows.max(hi);
        }
        if let Some((_, hi)) = range.cols {
            cols = cols.max(hi);
        }
    }
    for c in &ws.comments {
        rows = rows.max(c.cell_ref.0);
        cols = cols.max(c.cell_ref.1);
    }
    (rows, cols)
}

fn stable_from_ordinal(ws: &Worksheet) -> Worksheet<Stable> {
    let (row_count, column_count) = used_extent(ws);
    let mut index = SheetIndexes::default();
    for i in 1..=row_count as u32 {
        index.rows.insert_key(virtual_key(i));
    }
    for i in 1..=column_count as u32 {
        index.cols.insert_key(virtual_key(i));
    }
    let row_key = |o: i32| Stable::row_at(&index, o).unwrap();
    let col_key = |o: i32| Stable::col_at(&index, o).unwrap();
    let range = |r: &RangeRef| StableRange {
        rows: r.rows.map(|(lo, hi)| (row_key(lo), row_key(hi))),
        cols: r.cols.map(|(lo, hi)| (col_key(lo), col_key(hi))),
    };

    let mut sheet_data = crate::types::SheetData::<Stable>::default();
    for (r, row) in &ws.sheet_data {
        let entry = sheet_data.entry(row_key(*r)).or_default();
        for (c, cell) in row {
            entry.insert(col_key(*c), cell.clone());
        }
    }

    let cols = ws
        .cols
        .iter()
        .map(|c| Col {
            min: col_key(c.min),
            max: col_key(c.max),
            width: c.width,
            custom_width: c.custom_width,
            hidden: c.hidden,
            style: c.style,
        })
        .collect();

    Worksheet {
        dimension: ws.dimension.clone(),
        cols,
        rows: ws
            .rows
            .iter()
            .map(|r| Row {
                r: row_key(r.r),
                height: r.height,
                custom_format: r.custom_format,
                custom_height: r.custom_height,
                s: r.s,
                hidden: r.hidden,
            })
            .collect(),
        name: ws.name.clone(),
        sheet_data,
        shared_formulas: ws.shared_formulas.clone(),
        sheet_id: ws.sheet_id,
        state: ws.state.clone(),
        color: ws.color.clone(),
        merged_cells: ws
            .merged_cells
            .iter()
            .map(|m| range(&RangeRef::from(m)))
            .collect(),
        comments: ws
            .comments
            .iter()
            .map(|c| Comment {
                text: c.text.clone(),
                author_name: c.author_name.clone(),
                author_id: c.author_id.clone(),
                cell_ref: (row_key(c.cell_ref.0), col_key(c.cell_ref.1)),
            })
            .collect(),
        links: ws
            .links
            .iter()
            .map(|(&(r, c), link)| ((row_key(r), col_key(c)), link.clone()))
            .collect(),
        frozen_rows: ws.frozen_rows,
        frozen_columns: ws.frozen_columns,
        views: ws.views.clone(),
        show_grid_lines: ws.show_grid_lines,
        conditional_formatting: ws
            .conditional_formatting
            .iter()
            .map(|cf| ConditionalFormatting {
                ranges: cf.ranges.iter().map(&range).collect(),
                cf_rule: cf.cf_rule.clone(),
                priority: cf.priority,
            })
            .collect(),
        index,
    }
}

fn project(ws: &Worksheet<Stable>) -> Worksheet {
    let index = &ws.index;
    let row = |k: &FractionalKey| Stable::row_ordinal(index, k).unwrap();
    let col = |k: &FractionalKey| Stable::col_ordinal(index, k).unwrap();
    let range = |r: &StableRange| RangeRef {
        rows: r.rows.as_ref().map(|(lo, hi)| (row(lo), row(hi))),
        cols: r.cols.as_ref().map(|(lo, hi)| (col(lo), col(hi))),
    };

    let mut sheet_data = HashMap::new();
    for (r, cells) in &ws.sheet_data {
        let entry: &mut HashMap<i32, _> = sheet_data.entry(row(r)).or_default();
        for (c, cell) in cells {
            entry.insert(col(c), cell.clone());
        }
    }

    let cols = ws
        .cols
        .iter()
        .map(|c| Col {
            min: col(&c.min),
            max: col(&c.max),
            width: c.width,
            custom_width: c.custom_width,
            hidden: c.hidden,
            style: c.style,
        })
        .collect();

    Worksheet {
        dimension: ws.dimension.clone(),
        cols,
        rows: ws
            .rows
            .iter()
            .map(|r| Row {
                r: row(&r.r),
                height: r.height,
                custom_format: r.custom_format,
                custom_height: r.custom_height,
                s: r.s,
                hidden: r.hidden,
            })
            .collect(),
        name: ws.name.clone(),
        sheet_data,
        shared_formulas: ws.shared_formulas.clone(),
        sheet_id: ws.sheet_id,
        state: ws.state.clone(),
        color: ws.color.clone(),
        merged_cells: ws
            .merged_cells
            .iter()
            .map(|r| MergedCell::from(&range(r)))
            .collect(),
        comments: ws
            .comments
            .iter()
            .map(|c| Comment {
                text: c.text.clone(),
                author_name: c.author_name.clone(),
                author_id: c.author_id.clone(),
                cell_ref: (row(&c.cell_ref.0), col(&c.cell_ref.1)),
            })
            .collect(),
        links: ws
            .links
            .iter()
            .map(|((r, c), link)| ((row(r), col(c)), link.clone()))
            .collect(),
        frozen_rows: ws.frozen_rows,
        frozen_columns: ws.frozen_columns,
        views: ws.views.clone(),
        show_grid_lines: ws.show_grid_lines,
        conditional_formatting: ws
            .conditional_formatting
            .iter()
            .map(|cf| ConditionalFormatting {
                ranges: cf.ranges.iter().map(&range).collect(),
                cf_rule: cf.cf_rule.clone(),
                priority: cf.priority,
            })
            .collect(),
        index: (),
    }
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

    let stable = stable_from_ordinal(ws);
    assert_eq!(project(&stable), *ws);

    // The identity above must come from the mapping, not from the projection being a no-op: reorder
    // the rows on the stable side and every ordinal the projection reports has to follow.
    let mut moved = stable.clone();
    moved.index.rows.move_to(0..1, 5); // first row of five to the end
    let projected = project(&moved);
    assert_ne!(projected, *ws);
    for (before, after) in [(1, 5), (2, 1), (3, 2), (4, 3), (5, 4)] {
        assert_eq!(
            projected.sheet_data.get(&after),
            ws.sheet_data.get(&before),
            "row {before} should now read at {after}"
        );
    }
    assert_eq!(
        projected.rows,
        vec![Row {
            r: 2,
            ..ws.rows[0].clone()
        }]
    );
    assert_eq!(projected.comments[0].cell_ref, (4, 2));
    assert_eq!(
        projected.merged_cells,
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
        projected.conditional_formatting[0].ranges,
        vec![RangeRef {
            rows: Some((5, 4)),
            cols: Some((1, 2))
        }]
    );
    // Nothing on the column axis moved.
    assert_eq!(projected.cols, ws.cols);
}
