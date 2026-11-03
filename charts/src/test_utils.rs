//! Fixtures shared by the tests of this crate.

use ironcalc_base::charts::{
    Alignment, Bars, CategoryAxis, ColumnChart, DataRange, DataSource, EmptyCells, Font, Gridlines,
    Legend, Line, LineStyle, Position, Series, Stacking, TickMark, Title, ValueAxis, ValueFormat,
};
use ironcalc_base::types::Color;

pub fn font(size: u32) -> Font {
    Font {
        size,
        family: None,
        bold: false,
        italic: false,
        color: Color::Theme(1, 0.35),
    }
}

pub fn title(text: &str) -> Title {
    Title {
        text: text.to_string(),
        font: Font {
            size: 14,
            family: None,
            bold: false,
            italic: false,
            color: Color::Theme(1, 0.35),
        },
        alignment: Alignment::Middle,
        subtitle: None,
        fill: Color::None,
        border: Line {
            style: LineStyle::Automatic,
            color: Color::None,
        },
    }
}

fn line(style: LineStyle) -> Line {
    Line {
        style,
        color: Color::Theme(1, 0.85),
    }
}

/// A clustered column chart styled like Excel's default, with a title, a
/// legend at the bottom and the value axis linked to the source format.
pub fn column_chart() -> ColumnChart {
    ColumnChart {
        title: Some(title("Population")),
        legend: Legend {
            position: Position::Bottom,
            vertical: false,
            padding: None,
            fill: Color::None,
            border: line(LineStyle::None),
            font: font(9),
            overlay: false,
        },
        stacking: Stacking::None,
        bars: Bars::default(),
        category_axis: CategoryAxis {
            title: None,
            labels: font(9),
            rotation: None,
            line: line(LineStyle::Thin),
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
                font: font(9),
            },
            line: line(LineStyle::None),
            gridlines: Gridlines {
                major: line(LineStyle::Thin),
                minor: line(LineStyle::None),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            },
        },
        data: DataSource {
            range: DataRange::Manual {
                categories: "Sheet1!$B$2:$B$5".to_string(),
                series: vec![Series {
                    name: "Sheet1!$C$1".to_string(),
                    values: "Sheet1!$C$2:$C$5".to_string(),
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
