#![allow(clippy::unwrap_used, clippy::panic)]

use std::io::Cursor;

use ironcalc_base::charts::{
    Alignment, Bars, CategoryAxis, Chart, ColumnChart, DataLabelPosition, DataLabels, DataRange,
    DataSource, EmptyCells, Fill, Font, Gridlines, Legend, Line, LineStyle, Orientation, Position,
    Series, Stacking, Subtitle, TextDirection, TickMark, Title, ValueAxis, ValueFormat,
};
use ironcalc_base::drawing_types::{Anchor, AnchorBehavior, AnchorPoint, Drawing, DrawingContent};
use ironcalc_base::types::Color;
use ironcalc_base::Model;

use crate::export::save_xlsx_to_writer;
use crate::import::load_from_xlsx_bytes;

use super::test_export::new_empty_model;

fn font(size: u32) -> Font {
    Font {
        size,
        family: None,
        bold: false,
        italic: false,
        color: Color::Theme(1, 0.35),
    }
}

fn title(text: &str) -> Title {
    Title {
        text: text.to_string(),
        font: Font {
            size: 14,
            family: Some("Inter".to_string()),
            bold: true,
            italic: false,
            color: Color::Rgb("#336699".to_string()),
        },
        alignment: Alignment::Middle,
        subtitle: None,
        fill: Color::None,
        // Explicit, like the legend border: automatic does not survive a round trip
        border: Line {
            style: LineStyle::None,
            color: Color::None,
        },
    }
}

fn anchor(row: i32, column: i32) -> Anchor {
    Anchor {
        from: AnchorPoint {
            row,
            column,
            row_offset: 184150,
            column_offset: 501650,
        },
        to: AnchorPoint {
            row: row + 14,
            column: column + 6,
            row_offset: 82550,
            column_offset: 120650,
        },
        behavior: AnchorBehavior::MoveAndResize,
    }
}

fn drawing(id: u32, name: &str, anchor: Anchor, content: DrawingContent) -> Drawing {
    Drawing {
        id,
        name: name.to_string(),
        anchor,
        fill: Fill::Solid(Color::Theme(5, 0.6)),
        border: Line {
            style: LineStyle::Dashed,
            color: Color::Rgb("#FF0000".to_string()),
        },
        content,
    }
}

fn column_chart(data: DataRange) -> ColumnChart {
    let mut chart_title = title("Population");
    chart_title.subtitle = Some(Subtitle {
        text: "by city".to_string(),
        font: Font {
            size: 10,
            family: Some("Inter".to_string()),
            bold: false,
            italic: true,
            color: Color::Rgb("#808080".to_string()),
        },
    });
    ColumnChart {
        title: Some(chart_title),
        legend: Legend {
            position: Position::Bottom,
            vertical: false,
            padding: None,
            fill: Color::None,
            border: Line {
                style: LineStyle::None,
                color: Color::None,
            },
            font: Font {
                size: 20,
                family: None,
                bold: false,
                italic: true,
                color: Color::Theme(5, -0.5),
            },
            overlay: false,
        },
        stacking: Stacking::None,
        bars: Bars {
            gap: 100,
            overlap: -50,
        },
        category_axis: CategoryAxis {
            title: None,
            labels: font(9),
            rotation: None,
            line: Line {
                style: LineStyle::Thin,
                color: Color::Theme(1, 0.85),
            },
            gridlines: Gridlines {
                major: Line {
                    style: LineStyle::None,
                    color: Color::None,
                },
                minor: Line {
                    style: LineStyle::Dashed,
                    color: Color::Theme(4, 0.0),
                },
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        value_axis: ValueAxis {
            title: Some(title("People")),
            labels: ValueFormat {
                number_format: Some("#,##0".to_string()),
                font: font(9),
            },
            line: Line {
                style: LineStyle::Thick,
                color: Color::Rgb("#C00000".to_string()),
            },
            gridlines: Gridlines {
                major: Line {
                    style: LineStyle::Dashed,
                    color: Color::Theme(7, 0.4),
                },
                minor: Line {
                    style: LineStyle::Thin,
                    color: Color::Rgb("#00FF00".to_string()),
                },
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        data: DataSource {
            range: data,
            empty_cells: EmptyCells::Gaps,
            na_as_empty: true,
            show_hidden_data: false,
        },
        colors: vec![Color::Theme(4, 0.0), Color::Rgb("#FF0000".to_string())],
        data_labels: vec![
            None,
            Some(DataLabels {
                position: DataLabelPosition::InsideEnd,
                direction: TextDirection::Rotate270,
                number_format: Some("#,##0.00".to_string()),
                font: Font {
                    size: 10,
                    family: None,
                    bold: true,
                    italic: false,
                    color: Color::Rgb("#FF0000".to_string()),
                },
                fill: Color::Theme(4, 0.8),
                border: Line {
                    style: LineStyle::Thin,
                    color: Color::Theme(1, 0.5),
                },
            }),
        ],
    }
}

fn round_trip(model: &Model) -> Model<'static> {
    let cursor = save_xlsx_to_writer(model, Cursor::new(Vec::new())).unwrap();
    let workbook = load_from_xlsx_bytes(&cursor.into_inner(), "rt", "en", "UTC").unwrap();
    Model::from_workbook(workbook, "en").unwrap()
}

fn chart_of(drawing: &Drawing) -> &ColumnChart {
    match &drawing.content {
        DrawingContent::Chart(chart) => match chart.as_ref() {
            Chart::ColumnChart(column_chart) => column_chart,
        },
        DrawingContent::Unsupported => panic!("expected a chart"),
    }
}

#[test]
fn a_manual_column_chart_survives_a_round_trip() {
    let mut model = new_empty_model();
    let manual = DataRange::Manual {
        categories: "Sheet1!$B$2:$B$5".to_string(),
        series: vec![
            Series {
                name: "Sheet1!$C$1".to_string(),
                values: "Sheet1!$C$2:$C$5".to_string(),
            },
            Series {
                name: "\"Say \"\"hi\"\"\"".to_string(),
                values: "Sheet1!$D$2:$D$5".to_string(),
            },
        ],
    };
    let drawing = drawing(
        2,
        "Chart 1",
        anchor(10, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(column_chart(manual)))),
    );
    model.workbook.worksheets[0].drawings.push(drawing.clone());

    let model = round_trip(&model);
    assert_eq!(model.workbook.worksheets[0].drawings, vec![drawing]);
}

#[test]
fn stacked_columns_keep_their_gap_and_always_overlap() {
    let mut model = new_empty_model();
    let mut chart = column_chart(DataRange::Range {
        range: "Sheet1!$B$1:$D$5".to_string(),
        orientation: Orientation::Columns,
    });
    chart.stacking = Stacking::Standard;
    model.workbook.worksheets[0].drawings.push(drawing(
        1,
        "Chart 1",
        anchor(10, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(chart))),
    ));
    let model = round_trip(&model);
    // The helper's gap (100%) survives; its overlap (-50%) is not written
    // for a stacked chart, which always overlaps fully
    assert_eq!(
        chart_of(&model.workbook.worksheets[0].drawings[0]).bars,
        Bars {
            gap: 100,
            overlap: 100
        }
    );
}

#[test]
fn the_alignment_of_the_title_survives_a_round_trip() {
    let mut model = new_empty_model();
    let range = DataRange::Range {
        range: "Sheet1!$B$1:$D$5".to_string(),
        orientation: Orientation::Columns,
    };
    let mut drawings = Vec::new();
    for (id, alignment) in [Alignment::Left, Alignment::Middle, Alignment::Right]
        .into_iter()
        .enumerate()
    {
        let mut chart = column_chart(range.clone());
        chart.title.as_mut().unwrap().alignment = alignment;
        drawings.push(drawing(
            id as u32 + 1,
            &format!("Chart {}", id + 1),
            anchor(10 + 20 * id as i32, 1),
            DrawingContent::Chart(Box::new(Chart::ColumnChart(chart))),
        ));
    }
    model.workbook.worksheets[0].drawings = drawings;
    let model = round_trip(&model);
    let alignments: Vec<Alignment> = model.workbook.worksheets[0]
        .drawings
        .iter()
        .map(|drawing| chart_of(drawing).title.as_ref().unwrap().alignment.clone())
        .collect();
    assert_eq!(
        alignments,
        vec![Alignment::Left, Alignment::Middle, Alignment::Right]
    );
}

#[test]
fn the_angle_of_the_category_labels_survives_a_round_trip() {
    let mut model = new_empty_model();
    let manual = DataRange::Manual {
        categories: "Sheet1!$B$2:$B$5".to_string(),
        series: vec![
            Series {
                name: "Sheet1!$C$1".to_string(),
                values: "Sheet1!$C$2:$C$5".to_string(),
            },
            Series {
                name: "Sheet1!$D$1".to_string(),
                values: "Sheet1!$D$2:$D$5".to_string(),
            },
        ],
    };
    let mut chart = column_chart(manual);
    chart.category_axis.rotation = Some(-60);
    let drawing = drawing(
        2,
        "Chart 1",
        anchor(1, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(chart))),
    );
    model.workbook.worksheets[0].drawings.push(drawing.clone());

    let model = round_trip(&model);
    assert_eq!(model.workbook.worksheets[0].drawings, vec![drawing]);
}

#[test]
fn a_range_data_source_is_written_as_series() {
    let mut model = new_empty_model();
    let range = DataRange::Range {
        range: "Sheet1!$B$1:$D$5".to_string(),
        orientation: Orientation::Columns,
    };
    let mut chart = column_chart(range);
    chart.stacking = Stacking::Full;
    // Stacked columns always overlap fully
    chart.bars.overlap = 100;
    chart.title = None;
    chart.legend.position = Position::Float(0.25, 0.5);
    chart.legend.overlay = true;
    chart.legend.fill = Color::Theme(9, 0.8);
    chart.legend.border = Line {
        style: LineStyle::Thick,
        color: Color::Rgb("#000000".to_string()),
    };
    // No gridlines and no axis line are written as explicit "no line"
    chart.value_axis.gridlines.major = Line {
        style: LineStyle::None,
        color: Color::None,
    };
    chart.value_axis.gridlines.minor = chart.value_axis.gridlines.major.clone();
    chart.value_axis.line = chart.value_axis.gridlines.major.clone();
    chart.data.empty_cells = EmptyCells::Zero;
    chart.data.show_hidden_data = true;
    chart.data.na_as_empty = false;
    chart.value_axis.labels.number_format = None;
    let mut drawing = drawing(
        2,
        "Chart 1",
        anchor(1, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(chart.clone()))),
    );
    drawing.fill = Fill::None;
    drawing.border = Line {
        style: LineStyle::Thick,
        color: Color::Theme(9, -0.25),
    };
    model.workbook.worksheets[0].drawings.push(drawing.clone());

    let model = round_trip(&model);
    let drawings = &model.workbook.worksheets[0].drawings;
    assert_eq!(drawings.len(), 1);
    assert_eq!(drawings[0].fill, drawing.fill);
    assert_eq!(drawings[0].border, drawing.border);
    let read = chart_of(&drawings[0]);

    // The file only stores series, so the range comes back expanded.
    assert_eq!(
        read.data.range,
        DataRange::Manual {
            categories: "Sheet1!$B$2:$B$5".to_string(),
            series: vec![
                Series {
                    name: "Sheet1!$C$1".to_string(),
                    values: "Sheet1!$C$2:$C$5".to_string(),
                },
                Series {
                    name: "Sheet1!$D$1".to_string(),
                    values: "Sheet1!$D$2:$D$5".to_string(),
                },
            ],
        }
    );
    // Everything else is the same
    let mut expected = chart.clone();
    expected.data.range = read.data.range.clone();
    assert_eq!(read, &expected);
}

#[test]
fn unsupported_drawings_are_dropped_and_sheets_without_charts_have_no_drawing_part() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.workbook.worksheets[1].drawings.push(drawing(
        2,
        "Picture 1",
        anchor(1, 1),
        DrawingContent::Unsupported,
    ));
    let manual = DataRange::Manual {
        categories: "Sheet1!$B$2:$B$5".to_string(),
        series: vec![Series {
            name: "".to_string(),
            values: "Sheet1!$C$2:$C$5".to_string(),
        }],
    };
    model.workbook.worksheets[1].drawings.push(drawing(
        3,
        "Chart 2",
        anchor(20, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(column_chart(manual)))),
    ));

    let model = round_trip(&model);
    assert!(model.workbook.worksheets[0].drawings.is_empty());
    let drawings = &model.workbook.worksheets[1].drawings;
    assert_eq!(drawings.len(), 1);
    assert_eq!(drawings[0].id, 3);
    assert_eq!(drawings[0].name, "Chart 2");
    assert!(matches!(drawings[0].content, DrawingContent::Chart(_)));
}

#[test]
fn automatic_styling_is_exported_as_excel_defaults() {
    let mut model = new_empty_model();
    let manual = DataRange::Manual {
        categories: "Sheet1!$B$2:$B$5".to_string(),
        series: vec![Series {
            name: "Sheet1!$C$1".to_string(),
            values: "Sheet1!$C$2:$C$5".to_string(),
        }],
    };
    let automatic = Line {
        style: LineStyle::Automatic,
        color: Color::None,
    };
    let mut chart = column_chart(manual);
    chart.category_axis.line = automatic.clone();
    chart.value_axis.line = automatic.clone();
    chart.value_axis.gridlines.major = automatic.clone();
    chart.value_axis.gridlines.minor = automatic.clone();
    chart.category_axis.gridlines.major = automatic.clone();
    chart.legend.border = automatic.clone();
    let mut drawing = drawing(
        2,
        "Chart 1",
        anchor(1, 1),
        DrawingContent::Chart(Box::new(Chart::ColumnChart(chart))),
    );
    drawing.fill = Fill::Automatic;
    drawing.border = automatic;
    model.workbook.worksheets[0].drawings.push(drawing);

    let model = round_trip(&model);
    let drawings = &model.workbook.worksheets[0].drawings;
    let default_line = Line {
        style: LineStyle::Thin,
        color: Color::Theme(1, 0.85),
    };
    let no_line = Line {
        style: LineStyle::None,
        color: Color::None,
    };
    // Excel needs the values spelled out, so automatic reads back explicit.
    assert_eq!(drawings[0].fill, Fill::Solid(Color::Theme(0, 0.0)));
    assert_eq!(drawings[0].border, default_line);
    let read = chart_of(&drawings[0]);
    assert_eq!(read.category_axis.line, default_line);
    assert_eq!(read.value_axis.gridlines.major, default_line);
    assert_eq!(read.category_axis.gridlines.major, default_line);
    assert_eq!(
        read.value_axis.gridlines.minor,
        Line {
            style: LineStyle::Thin,
            color: Color::Theme(1, 0.95),
        }
    );
    // Explicit lines come back as they were
    assert_eq!(
        read.category_axis.gridlines.minor,
        Line {
            style: LineStyle::Dashed,
            color: Color::Theme(4, 0.0),
        }
    );
    assert_eq!(read.value_axis.line, no_line);
    assert_eq!(read.legend.border, no_line);
}
