//! Golden pictures: the SVG of a couple of charts, checked byte for byte.
//! Run with `UPDATE_GOLDEN=1` to rewrite them after a deliberate change,
//! then look at them.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use ironcalc_base::charts::{
    Alignment, Bars, CategoryAxis, ChartData, ColumnChart, DataLabelPosition, DataLabels,
    DataRange, DataSource, EmptyCells, Fill, Font, Gridlines, Legend, Line, LineStyle, Position,
    Series, SeriesData, Stacking, Subtitle, TextDirection, TickMark, Title, ValueAxis, ValueFormat,
};
use ironcalc_base::locale::get_locale;
use ironcalc_base::types::{Color, Theme};
use ironcharts::layout::Context;
use ironcharts::text::Inter;
use ironcharts::{layout, to_svg};

fn font(size: u32, bold: bool) -> Font {
    Font {
        size,
        family: None,
        bold,
        italic: false,
        color: Color::None,
    }
}

fn line(style: LineStyle) -> Line {
    Line {
        style,
        color: Color::None,
    }
}

fn title(text: &str) -> Title {
    Title {
        text: text.to_string(),
        font: font(14, false),
        alignment: Alignment::Middle,
        subtitle: None,
        fill: Color::None,
        border: Line {
            style: LineStyle::Automatic,
            color: Color::None,
        },
    }
}

fn chart() -> ColumnChart {
    ColumnChart {
        title: Some(title("Visitors per city")),
        legend: Legend {
            position: Position::Bottom,
            vertical: false,
            padding: None,
            fill: Color::None,
            border: line(LineStyle::Automatic),
            font: font(9, false),
            overlay: false,
        },
        stacking: Stacking::None,
        bars: Bars::default(),
        category_axis: CategoryAxis {
            title: None,
            labels: font(9, false),
            rotation: None,
            line: line(LineStyle::Automatic),
            gridlines: Gridlines {
                major: line(LineStyle::None),
                minor: line(LineStyle::None),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        value_axis: ValueAxis {
            title: None,
            labels: ValueFormat {
                number_format: None,
                font: font(9, false),
            },
            line: line(LineStyle::Automatic),
            gridlines: Gridlines {
                major: line(LineStyle::Automatic),
                minor: line(LineStyle::None),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        data: DataSource {
            range: DataRange::Manual {
                categories: "Sheet1!$B$2:$B$6".to_string(),
                series: vec![Series {
                    name: "Sheet1!$C$1".to_string(),
                    values: "Sheet1!$C$2:$C$6".to_string(),
                }],
            },
            empty_cells: EmptyCells::Gaps,
            na_as_empty: false,
            show_hidden_data: false,
        },
        colors: vec![],
        data_labels: vec![],
    }
}

fn data() -> ChartData {
    let series = |name: &str, values: &[Option<f64>]| SeriesData {
        name: name.to_string(),
        values: values.to_vec(),
        number_format: "#,##0".to_string(),
    };
    ChartData {
        categories: ["London", "Madrid", "Rome", "Paris", "Amsterdam"]
            .iter()
            .map(|c| c.to_string())
            .collect(),
        series: vec![
            series(
                "September",
                &[
                    Some(123.0),
                    Some(23.0),
                    Some(234.0),
                    Some(156.0),
                    Some(123.0),
                ],
            ),
            series(
                "October",
                &[Some(167.0), Some(456.0), None, Some(300.0), Some(345.0)],
            ),
            series(
                "November",
                &[
                    Some(120.0),
                    Some(170.0),
                    Some(200.0),
                    Some(250.0),
                    Some(321.0),
                ],
            ),
        ],
    }
}

fn check(name: &str, chart: &ColumnChart, fill: &Fill, border: &Line, width: f64, height: f64) {
    let theme = Theme::default();
    let context = Context {
        width,
        height,
        theme: &theme,
        locale: get_locale("en").unwrap(),
        measurer: &Inter,
    };
    let svg = to_svg(&layout(chart, &data(), fill, border, &context));
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/golden/{name}.svg"));
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(&path, &svg).unwrap();
    }
    let expected = fs::read_to_string(&path).unwrap();
    assert_eq!(svg, expected, "{name}.svg differs from the golden");
}

#[test]
fn clustered_with_excel_defaults() {
    check(
        "clustered",
        &chart(),
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn stacked_with_styled_frame_and_axis_titles() {
    let mut chart = chart();
    chart.title = Some(Title {
        text: "Autumn".to_string(),
        font: Font {
            size: 16,
            family: None,
            bold: true,
            italic: false,
            color: Color::Rgb("#C00000".to_string()),
        },
        alignment: Alignment::Left,
        subtitle: None,
        fill: Color::None,
        border: Line {
            style: LineStyle::Automatic,
            color: Color::None,
        },
    });
    chart.stacking = Stacking::Standard;
    chart.bars = Bars::stacked();
    chart.legend.position = Position::Right;
    chart.legend.vertical = true;
    chart.legend.fill = Color::Rgb("#FFFFFF".to_string());
    chart.legend.border = line(LineStyle::Thin);
    chart.value_axis.title = Some(title("Visitors"));
    chart.category_axis.title = Some(title("City"));
    chart.value_axis.labels.number_format = Some("0.0".to_string());
    chart.value_axis.line = line(LineStyle::Thick);
    chart.value_axis.gridlines.major = line(LineStyle::Dashed);
    chart.colors = vec![Color::Theme(5, 0.4), Color::Rgb("#336699".to_string())];
    check(
        "stacked",
        &chart,
        &Fill::Solid(Color::Theme(2, 0.0)),
        &Line {
            style: LineStyle::Dashed,
            color: Color::Rgb("#FF0000".to_string()),
        },
        640.0,
        400.0,
    );
}

#[test]
fn clustered_with_major_and_minor_gridlines() {
    // Chart 16 of AssortedColumns.xlsx: both axes, Excel's default styles
    let mut chart = chart();
    chart.title = Some(title("With gridlines"));
    chart.category_axis.gridlines = Gridlines {
        major: line(LineStyle::Automatic),
        minor: line(LineStyle::Automatic),
        major_ticks: TickMark::None,
        minor_ticks: TickMark::None,
    };
    chart.value_axis.gridlines.minor = line(LineStyle::Automatic);
    check(
        "gridlines",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_boxed_titles() {
    let mut chart = chart();
    chart.title = Some(Title {
        text: "Visitors per city".to_string(),
        font: font(14, true),
        alignment: Alignment::Middle,
        subtitle: None,
        fill: Color::Theme(4, 0.8),
        border: Line {
            style: LineStyle::Thin,
            color: Color::Theme(4, 0.0),
        },
    });
    let mut value_title = title("Visitors");
    value_title.font = font(10, false);
    value_title.fill = Color::Rgb("#FFF2CC".to_string());
    value_title.border = Line {
        style: LineStyle::Dashed,
        color: Color::Rgb("#BF9000".to_string()),
    };
    chart.value_axis.title = Some(value_title);
    let mut category_title = title("City");
    category_title.font = font(10, false);
    category_title.border = Line {
        style: LineStyle::Thick,
        color: Color::Theme(1, 0.5),
    };
    chart.category_axis.title = Some(category_title);
    check(
        "titles",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_subtitles() {
    // Chart 17 of AssortedColumns.xlsx: a big red title over a small second
    // line; the axis titles get one too, the value axis's in a box
    let subtitle = |text: &str, size: u32| Subtitle {
        text: text.to_string(),
        font: Font {
            size,
            family: None,
            bold: false,
            italic: true,
            color: Color::Theme(1, 0.5),
        },
    };
    let mut chart = chart();
    chart.title = Some(Title {
        text: "The main title is here".to_string(),
        font: Font {
            size: 28,
            family: None,
            bold: false,
            italic: false,
            color: Color::Rgb("#FF0000".to_string()),
        },
        alignment: Alignment::Middle,
        fill: Color::None,
        border: line(LineStyle::Automatic),
        subtitle: Some(subtitle("and another line", 10)),
    });
    let mut value_title = title("Visitors");
    value_title.font = font(10, true);
    value_title.border = line(LineStyle::Thin);
    value_title.subtitle = Some(subtitle("thousands", 8));
    chart.value_axis.title = Some(value_title);
    let mut category_title = title("City");
    category_title.font = font(10, true);
    category_title.subtitle = Some(subtitle("autumn 2024", 8));
    chart.category_axis.title = Some(category_title);
    check(
        "subtitles",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_data_labels() {
    let mut chart = chart();
    chart.title = None;
    chart.data_labels = vec![
        Some(DataLabels {
            position: DataLabelPosition::OutsideEnd,
            direction: TextDirection::Horizontal,
            number_format: None,
            font: font(9, false),
            fill: Color::None,
            border: line(LineStyle::Automatic),
        }),
        Some(DataLabels {
            position: DataLabelPosition::InsideEnd,
            direction: TextDirection::Horizontal,
            number_format: Some("#,##0.0".to_string()),
            font: Font {
                size: 8,
                family: None,
                bold: true,
                italic: false,
                color: Color::Rgb("#FFFFFF".to_string()),
            },
            fill: Color::Theme(5, -0.25),
            border: line(LineStyle::None),
        }),
        None,
    ];
    check(
        "labels",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        560.0,
        339.0,
    );
}

#[test]
fn clustered_with_narrow_gaps_and_wide_overlap() {
    // Chart 17 of AssortedColumns.xlsx: a gap of 100% and an overlap of -50%
    let mut chart = chart();
    chart.title = Some(title("Gap 100%, overlap -50%"));
    chart.bars = Bars {
        gap: 100,
        overlap: -50,
    };
    check(
        "bars",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_tick_marks() {
    // Chart 19 of AssortedColumns.xlsx: a visible value axis line with
    // major ticks across it and minor ones outside; category major ticks
    // inside and minor ones across
    let mut chart = chart();
    chart.title = Some(title("Tick marks"));
    chart.value_axis.line = line(LineStyle::Thin);
    chart.value_axis.gridlines.major_ticks = TickMark::Cross;
    chart.value_axis.gridlines.minor_ticks = TickMark::Outside;
    chart.category_axis.gridlines.major_ticks = TickMark::Inside;
    chart.category_axis.gridlines.minor_ticks = TickMark::Cross;
    check(
        "ticks",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_turned_data_labels() {
    // Chart 18 of AssortedColumns.xlsx: labels reading up outside the end,
    // labels reading down in white in the middle, and a boxed turned one
    let mut chart = chart();
    chart.title = None;
    chart.data_labels = vec![
        Some(DataLabels {
            position: DataLabelPosition::OutsideEnd,
            direction: TextDirection::Rotate270,
            number_format: None,
            font: font(9, false),
            fill: Color::None,
            border: line(LineStyle::Automatic),
        }),
        Some(DataLabels {
            position: DataLabelPosition::Center,
            direction: TextDirection::Rotate90,
            number_format: None,
            font: Font {
                size: 9,
                family: None,
                bold: true,
                italic: false,
                color: Color::Rgb("#FFFFFF".to_string()),
            },
            fill: Color::None,
            border: line(LineStyle::Automatic),
        }),
        Some(DataLabels {
            position: DataLabelPosition::InsideBase,
            direction: TextDirection::Rotate270,
            number_format: Some("0.0".to_string()),
            font: font(8, false),
            fill: Color::Theme(0, 0.0),
            border: line(LineStyle::Thin),
        }),
    ];
    check(
        "turned_labels",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        500.0,
        339.0,
    );
}

#[test]
fn clustered_with_rotated_category_labels() {
    let mut chart = chart();
    chart.category_axis.rotation = Some(45);
    check(
        "rotated",
        &chart,
        &Fill::Automatic,
        &line(LineStyle::Automatic),
        400.0,
        300.0,
    );
}
