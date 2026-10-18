#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{
    cf_types::{CfRuleInput, Cfvo, ColorScaleThreshold},
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::types::Area,
    test::user_model::util::new_empty_user_model,
    types::Color,
    user_model::{BorderArea, CellText, FillRun, UserModel},
};

fn cell(row: i32, column: i32) -> Area {
    Area {
        sheet: 0,
        row,
        column,
        width: 1,
        height: 1,
    }
}

// Every cell of the block has the style get_extended_cell_style gives it,
// and the fill runs cover the block in order, each in its cells' fill colour
fn assert_styles_match(model: &UserModel, row: i32, column: i32, width: i32, height: i32) {
    let block = model
        .get_cells_block(0, row, column, width, height)
        .unwrap();
    assert_eq!(block.style_indices.len(), (width * height) as usize);
    for r in 0..height {
        for c in 0..width {
            let index = block.style_indices[(r * width + c) as usize] as usize;
            let expected = model
                .get_extended_cell_style(0, row + r, column + c)
                .unwrap();
            assert_eq!(
                block.styles[index].style,
                expected.style,
                "cell ({}, {})",
                row + r,
                column + c
            );
        }
    }
    let mut runs = block.fills.iter();
    for r in row..row + height {
        let mut c = column;
        while c < column + width {
            let run = runs.next().expect("a run for every cell");
            assert_eq!((run.row, run.column), (r, c), "runs are contiguous");
            assert!(run.width >= 1 && c + run.width <= column + width);
            for cell_column in c..c + run.width {
                let style = model
                    .get_extended_cell_style(0, r, cell_column)
                    .unwrap()
                    .style;
                let expected = if style.fill.color.is_none() {
                    String::new()
                } else {
                    model.resolve_color(&style.fill.color)
                };
                assert_eq!(run.color, expected, "cell ({r}, {cell_column})");
            }
            c += run.width;
        }
    }
    assert!(runs.next().is_none());
}

#[test]
fn fills_are_runs_of_one_colour_along_a_row() {
    let mut model = new_empty_user_model();
    // A1:B1 red, C1 blue, D1 plain, E1 red and bold (another style, same fill)
    for c in [1, 2, 5] {
        model
            .update_range_style(&cell(1, c), "fill.fg_color", "#FF0000")
            .unwrap();
    }
    model
        .update_range_style(&cell(1, 3), "fill.fg_color", "#0000FF")
        .unwrap();
    model
        .update_range_style(&cell(1, 5), "font.b", "true")
        .unwrap();
    // A1:E2
    let block = model.get_cells_block(0, 1, 1, 5, 2).unwrap();
    let run = |row, column, width, color: &str| FillRun {
        row,
        column,
        width,
        color: color.to_string(),
    };
    assert_eq!(
        block.fills,
        vec![
            run(1, 1, 2, "#FF0000"),
            run(1, 3, 1, "#0000FF"),
            run(1, 4, 1, ""),
            run(1, 5, 1, "#FF0000"),
            run(2, 1, 5, ""),
        ]
    );
    assert_styles_match(&model, 1, 1, 5, 2);
}

#[test]
fn empty_sheet_shares_one_style() {
    let model = new_empty_user_model();
    let block = model.get_cells_block(0, 1, 1, 4, 3).unwrap();
    assert_eq!(
        (block.row, block.column, block.width, block.height),
        (1, 1, 4, 3)
    );
    assert_eq!(block.styles.len(), 1);
    assert_eq!(block.style_indices, vec![0; 12]);
    assert!(block.texts.is_empty());
}

#[test]
fn styles_are_deduplicated_and_indexed_row_major() {
    let mut model = new_empty_user_model();
    // B2 and D3 have a red fill, C2 is bold, the rest is plain
    model
        .update_range_style(&cell(2, 2), "fill.fg_color", "#FF0000")
        .unwrap();
    model
        .update_range_style(&cell(3, 4), "fill.fg_color", "#FF0000")
        .unwrap();
    model
        .update_range_style(&cell(2, 3), "font.b", "true")
        .unwrap();
    // B2:D3
    let block = model.get_cells_block(0, 2, 2, 3, 2).unwrap();
    let style = |i: usize| &block.styles[block.style_indices[i] as usize].style;
    // B2, C2, D2, B3, C3, D3
    assert_eq!(style(0).fill.color, Color::Rgb("#FF0000".to_string()));
    assert!(style(1).font.b);
    assert!(style(2).fill.color.is_none());
    assert_eq!(block.style_indices[0], block.style_indices[5]);
    assert_eq!(block.style_indices[2], block.style_indices[3]);
    assert_eq!(block.style_indices[2], block.style_indices[4]);
    assert_eq!(block.styles.len(), 3);
    assert_styles_match(&model, 2, 2, 3, 2);
}

#[test]
fn a_neighbours_border_tells_cells_apart() {
    let mut model = new_empty_user_model();
    // Bottom border on B2: B3 shows it as its top border, C3 does not
    let border_area: BorderArea = serde_json::from_str(
        r##"{"item": {"style": "thin", "color": "#FF5566"}, "type": "Bottom"}"##,
    )
    .unwrap();
    model
        .set_area_with_border(&cell(2, 2), &border_area)
        .unwrap();
    // B3:C3
    let block = model.get_cells_block(0, 3, 2, 2, 1).unwrap();
    assert_ne!(block.style_indices[0], block.style_indices[1]);
    let b3 = &block.styles[block.style_indices[0] as usize].style;
    let c3 = &block.styles[block.style_indices[1] as usize].style;
    assert!(b3.border.top.is_some());
    assert!(c3.border.top.is_none());
    assert_styles_match(&model, 1, 1, 4, 4);
    // The block does not need to include the bordered cell for B3 to show
    // the border: the neighbour's style is read whether it is in the
    // block or not
    assert_styles_match(&model, 3, 2, 1, 1);
}

#[test]
fn texts_list_the_non_empty_cells_in_row_major_order() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "hello").unwrap();
    model.set_user_input(0, 1, 3, "=1+1").unwrap();
    model.set_user_input(0, 2, 2, "TRUE").unwrap();
    // Outside the block
    model.set_user_input(0, 1, 4, "out").unwrap();
    model.set_user_input(0, 4, 1, "out").unwrap();
    // Styled but empty: not a text
    model
        .update_range_style(&cell(2, 1), "fill.fg_color", "#FF0000")
        .unwrap();
    let block = model.get_cells_block(0, 1, 1, 3, 3).unwrap();
    assert_eq!(
        block.texts,
        vec![
            CellText {
                row: 1,
                column: 1,
                text: "hello".to_string(),
                cell_type: 2,
            },
            CellText {
                row: 1,
                column: 3,
                text: "2".to_string(),
                cell_type: 1,
            },
            CellText {
                row: 2,
                column: 2,
                text: "TRUE".to_string(),
                cell_type: 4,
            },
        ]
    );
}

#[test]
fn texts_are_formatted_with_the_cell_number_format() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "1234.5").unwrap();
    model
        .update_range_style(&cell(1, 1), "num_fmt", "#,##0.00")
        .unwrap();
    let block = model.get_cells_block(0, 1, 1, 1, 1).unwrap();
    assert_eq!(block.texts[0].text, "1,234.50");
}

#[test]
fn conditionally_formatted_cells_get_their_own_style() {
    let mut model = new_empty_user_model();
    for i in 1..=3 {
        model.set_user_input(0, i, 1, &i.to_string()).unwrap();
    }
    let color_scale = CfRuleInput::ColorScale {
        thresholds: vec![
            ColorScaleThreshold {
                cfvo: Cfvo::Min,
                color: Color::Rgb("#FF0000".to_string()),
            },
            ColorScaleThreshold {
                cfvo: Cfvo::Max,
                color: Color::Rgb("#00FF00".to_string()),
            },
        ],
    };
    model
        .add_conditional_formatting(0, "A1:A3", color_scale)
        .unwrap();
    // A1:B3
    let block = model.get_cells_block(0, 1, 1, 2, 3).unwrap();
    let fill = |i: usize| {
        block.styles[block.style_indices[i] as usize]
            .style
            .fill
            .color
            .clone()
    };
    // A1, A2 and A3 get three different fills from the scale
    assert!(fill(0).is_some());
    assert_ne!(fill(0), fill(2));
    assert_ne!(fill(2), fill(4));
    assert_ne!(fill(0), fill(4));
    // The B cells are plain and share one style
    assert!(fill(1).is_none());
    assert_eq!(block.style_indices[1], block.style_indices[3]);
    assert_eq!(block.style_indices[3], block.style_indices[5]);
    assert_styles_match(&model, 1, 1, 2, 3);
}

#[test]
fn blocks_reach_the_edges_of_the_sheet() {
    let model = new_empty_user_model();
    assert_styles_match(&model, 1, 1, 2, 2);
    assert_styles_match(&model, LAST_ROW - 1, LAST_COLUMN - 1, 2, 2);
    let block = model
        .get_cells_block(0, LAST_ROW, LAST_COLUMN, 1, 1)
        .unwrap();
    assert_eq!(block.style_indices, vec![0]);
}

#[test]
fn invalid_blocks_are_errors() {
    let model = new_empty_user_model();
    assert!(model.get_cells_block(0, 1, 1, 0, 1).is_err());
    assert!(model.get_cells_block(0, 1, 1, 1, 0).is_err());
    assert!(model.get_cells_block(0, 0, 1, 1, 1).is_err());
    assert!(model.get_cells_block(0, 1, 0, 1, 1).is_err());
    assert!(model.get_cells_block(0, LAST_ROW, 1, 1, 2).is_err());
    assert!(model.get_cells_block(0, 1, LAST_COLUMN, 2, 1).is_err());
    assert!(model.get_cells_block(7, 1, 1, 1, 1).is_err());
}
