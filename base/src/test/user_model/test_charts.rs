#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::charts::{
    Alignment, Bars, CategoryAxis, Chart, ColumnChart, DataRange, DataSource, EmptyCells, Fill,
    Font, Gridlines, Legend, Line, LineStyle, Orientation, Position, Stacking, TickMark, Title,
    ValueAxis, ValueFormat,
};
use crate::drawing_types::{Anchor, AnchorBehavior, AnchorPoint};
use crate::test::user_model::util::new_empty_user_model;
use crate::types::Color;
use crate::UserModel;

fn font() -> Font {
    Font {
        size: 9,
        family: None,
        bold: false,
        italic: false,
        color: Color::None,
    }
}

fn line() -> Line {
    Line {
        style: LineStyle::Automatic,
        color: Color::None,
    }
}

fn chart(title: &str, stacking: Stacking) -> Chart {
    Chart::ColumnChart(ColumnChart {
        title: Some(Title {
            text: title.to_string(),
            font: font(),
            alignment: Alignment::Middle,
            subtitle: None,
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
        }),
        legend: Legend {
            position: Position::Bottom,
            vertical: false,
            padding: None,
            fill: Color::None,
            border: line(),
            font: font(),
            overlay: false,
        },
        stacking,
        bars: Bars::default(),
        category_axis: CategoryAxis {
            title: None,
            labels: font(),
            rotation: None,
            line: line(),
            gridlines: Gridlines {
                major: line(),
                minor: line(),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        value_axis: ValueAxis {
            title: None,
            labels: ValueFormat {
                number_format: None,
                font: font(),
            },
            line: line(),
            gridlines: Gridlines {
                major: line(),
                minor: Line {
                    style: LineStyle::None,
                    color: Color::None,
                },
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        data: DataSource {
            range: DataRange::Range {
                range: "Sheet1!$A$1:$C$5".to_string(),
                orientation: Orientation::Columns,
            },
            empty_cells: EmptyCells::Gaps,
            na_as_empty: false,
            show_hidden_data: false,
        },
        colors: vec![],
        data_labels: vec![],
    })
}

fn anchor(row: i32, column: i32) -> Anchor {
    Anchor {
        from: AnchorPoint {
            row,
            column,
            row_offset: 0,
            column_offset: 0,
        },
        to: AnchorPoint {
            row: row + 15,
            column: column + 7,
            row_offset: 0,
            column_offset: 0,
        },
        behavior: AnchorBehavior::MoveAndResize,
    }
}

fn title_of(model: &UserModel, id: u32) -> String {
    let Chart::ColumnChart(chart) = model.get_chart(0, id).unwrap();
    chart.title.as_ref().unwrap().text.clone()
}

#[test]
fn new_chart_creates_a_drawing_and_can_be_undone() {
    let mut model = new_empty_user_model();
    assert!(model.get_chart_ids(0).unwrap().is_empty());

    let id = model
        .new_chart(0, anchor(2, 2), chart("Sales", Stacking::None))
        .unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![id]);
    assert_eq!(title_of(&model, id), "Sales");
    let drawing = &model.model.workbook.worksheets[0].drawings[0];
    assert_eq!(drawing.id, id);
    assert_eq!(drawing.name, "Chart 1");
    assert_eq!(drawing.anchor, anchor(2, 2));
    assert_eq!(drawing.fill, Fill::Automatic);
    assert_eq!(drawing.border.style, LineStyle::Automatic);

    model.undo().unwrap();
    assert!(model.get_chart_ids(0).unwrap().is_empty());
    assert!(model.get_chart(0, id).is_err());

    model.redo().unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![id]);
    assert_eq!(title_of(&model, id), "Sales");
}

#[test]
fn ids_are_unique_and_names_count_charts() {
    let mut model = new_empty_user_model();
    let first = model
        .new_chart(0, anchor(1, 1), chart("One", Stacking::None))
        .unwrap();
    let second = model
        .new_chart(0, anchor(20, 1), chart("Two", Stacking::None))
        .unwrap();
    assert_ne!(first, second);
    model.delete_chart(0, first).unwrap();
    // A new chart never reuses the id of a live one
    let third = model
        .new_chart(0, anchor(40, 1), chart("Three", Stacking::None))
        .unwrap();
    assert_ne!(third, second);
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![second, third]);
    let names: Vec<&str> = model.model.workbook.worksheets[0]
        .drawings
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert_eq!(names, vec!["Chart 2", "Chart 2"]);
}

#[test]
fn delete_chart_undo_restores_it_in_its_place() {
    let mut model = new_empty_user_model();
    let first = model
        .new_chart(0, anchor(1, 1), chart("One", Stacking::None))
        .unwrap();
    let second = model
        .new_chart(0, anchor(20, 1), chart("Two", Stacking::None))
        .unwrap();

    model.delete_chart(0, first).unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![second]);
    assert!(model.get_chart(0, first).is_err());

    model.undo().unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![first, second]);
    assert_eq!(title_of(&model, first), "One");
    assert_eq!(
        model.model.workbook.worksheets[0].drawings[0].anchor,
        anchor(1, 1)
    );

    model.redo().unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![second]);

    assert!(model.delete_chart(0, 999).is_err());
}

#[test]
fn update_chart_keeps_the_frame_and_can_be_undone() {
    let mut model = new_empty_user_model();
    let id = model
        .new_chart(0, anchor(1, 1), chart("Before", Stacking::None))
        .unwrap();
    model
        .update_chart(0, id, chart("After", Stacking::Standard))
        .unwrap();
    assert_eq!(title_of(&model, id), "After");
    let Chart::ColumnChart(column) = model.get_chart(0, id).unwrap();
    assert_eq!(column.stacking, Stacking::Standard);
    assert_eq!(
        model.model.workbook.worksheets[0].drawings[0].anchor,
        anchor(1, 1)
    );

    model.undo().unwrap();
    assert_eq!(title_of(&model, id), "Before");
    model.redo().unwrap();
    assert_eq!(title_of(&model, id), "After");

    assert!(model
        .update_chart(0, 999, chart("Nowhere", Stacking::None))
        .is_err());
}

#[test]
fn move_and_style_the_drawing_with_undo() {
    let mut model = new_empty_user_model();
    let id = model
        .new_chart(0, anchor(1, 1), chart("Chart", Stacking::None))
        .unwrap();

    let drawing = |model: &UserModel| model.model.workbook.worksheets[0].drawings[0].clone();
    model.move_drawing(0, id, anchor(10, 4)).unwrap();
    assert_eq!(drawing(&model).anchor, anchor(10, 4));

    let red = Line {
        style: LineStyle::Dashed,
        color: Color::Rgb("#FF0000".to_string()),
    };
    model
        .set_drawing_style(0, id, Fill::Solid(Color::Theme(5, 0.6)), red.clone())
        .unwrap();
    assert_eq!(drawing(&model).fill, Fill::Solid(Color::Theme(5, 0.6)));
    assert_eq!(drawing(&model).border, red);

    // Undo the style, then the move
    model.undo().unwrap();
    assert_eq!(drawing(&model).fill, Fill::Automatic);
    assert_eq!(drawing(&model).border.style, LineStyle::Automatic);
    assert_eq!(drawing(&model).anchor, anchor(10, 4));
    model.undo().unwrap();
    assert_eq!(drawing(&model).anchor, anchor(1, 1));

    model.redo().unwrap();
    model.redo().unwrap();
    assert_eq!(drawing(&model).anchor, anchor(10, 4));
    assert_eq!(drawing(&model).border, red);
    // The chart itself is untouched
    assert_eq!(title_of(&model, id), "Chart");

    assert!(model.move_drawing(0, 999, anchor(1, 1)).is_err());
    assert!(model.get_chart_ids(7).is_err());
}

#[test]
fn deleting_a_sheet_takes_its_charts_and_undo_brings_them_back() {
    let mut model = new_empty_user_model();
    model.new_sheet().unwrap();
    let id = model
        .new_chart(1, anchor(1, 1), chart("Gone", Stacking::None))
        .unwrap();
    model.delete_sheet(1).unwrap();
    assert!(model.get_chart_ids(1).is_err());
    model.undo().unwrap();
    assert_eq!(model.get_chart_ids(1).unwrap(), vec![id]);
    assert_eq!(
        {
            let Chart::ColumnChart(chart) = model.get_chart(1, id).unwrap();
            chart.title.as_ref().unwrap().text.clone()
        },
        "Gone"
    );
}

fn anchor_of(model: &UserModel, id: u32) -> Anchor {
    model.model.get_drawing_anchor(0, id).unwrap()
}

#[test]
fn charts_follow_row_and_column_inserts_and_deletes() {
    let mut model = new_empty_user_model();
    // Rows 10 to 25, columns 5 to 12
    let id = model
        .new_chart(0, anchor(10, 5), chart("Follow", Stacking::None))
        .unwrap();

    // Rows above: the chart moves down
    model.insert_rows(0, 3, 2).unwrap();
    assert_eq!(anchor_of(&model, id).from.row, 12);
    assert_eq!(anchor_of(&model, id).to.row, 27);
    // A column inside: the chart grows
    model.insert_columns(0, 7, 1).unwrap();
    assert_eq!(anchor_of(&model, id).from.column, 5);
    assert_eq!(anchor_of(&model, id).to.column, 13);
    // Rows above deleted: the chart moves up
    model.delete_rows(0, 1, 5).unwrap();
    assert_eq!(anchor_of(&model, id).from.row, 7);
    assert_eq!(anchor_of(&model, id).to.row, 22);
    // Rows cutting the bottom: the chart ends where the deletion starts
    model.delete_rows(0, 20, 10).unwrap();
    assert_eq!(anchor_of(&model, id).from.row, 7);
    assert_eq!(anchor_of(&model, id).to.row, 20);
    assert_eq!(anchor_of(&model, id).to.row_offset, 0);
    // A column cutting the left: the chart starts where the deletion was
    model.delete_columns(0, 4, 3).unwrap();
    assert_eq!(anchor_of(&model, id).from.column, 4);
    assert_eq!(anchor_of(&model, id).from.column_offset, 0);
    assert_eq!(anchor_of(&model, id).to.column, 10);

    // All the way back
    for _ in 0..5 {
        model.undo().unwrap();
    }
    assert_eq!(anchor_of(&model, id), anchor(10, 5));
    // And forward again
    for _ in 0..5 {
        model.redo().unwrap();
    }
    assert_eq!(anchor_of(&model, id).from.column, 4);
    assert_eq!(anchor_of(&model, id).to.row, 20);
}

#[test]
fn a_chart_inside_deleted_rows_goes_and_comes_back_on_undo() {
    let mut model = new_empty_user_model();
    let id = model
        .new_chart(0, anchor(10, 5), chart("Gone", Stacking::None))
        .unwrap();
    model.delete_rows(0, 8, 20).unwrap();
    assert!(model.get_chart_ids(0).unwrap().is_empty());
    model.undo().unwrap();
    assert_eq!(model.get_chart_ids(0).unwrap(), vec![id]);
    assert_eq!(anchor_of(&model, id), anchor(10, 5));
    assert_eq!(title_of(&model, id), "Gone");
}

#[test]
fn move_only_and_fixed_charts_keep_their_size() {
    let mut model = new_empty_user_model();
    let id = model
        .new_chart(0, anchor(10, 5), chart("Span", Stacking::None))
        .unwrap();
    let mut move_only = anchor(10, 5);
    move_only.behavior = AnchorBehavior::MoveOnly;
    model.move_drawing(0, id, move_only.clone()).unwrap();
    // Rows inside the chart: it does not grow
    model.insert_rows(0, 15, 3).unwrap();
    assert_eq!(anchor_of(&model, id), move_only);
    // Rows above: it moves as a whole
    model.insert_rows(0, 2, 4).unwrap();
    assert_eq!(anchor_of(&model, id).from.row, 14);
    assert_eq!(anchor_of(&model, id).to.row, 29);

    let mut fixed = anchor(10, 5);
    fixed.behavior = AnchorBehavior::Fixed;
    model.move_drawing(0, id, fixed.clone()).unwrap();
    model.insert_rows(0, 2, 4).unwrap();
    model.delete_columns(0, 1, 2).unwrap();
    assert_eq!(anchor_of(&model, id), fixed);
}
