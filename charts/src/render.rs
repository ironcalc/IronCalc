//! Fills the plot area: gridlines and the columns of every series.
//!
//! The columns follow Excel's geometry. Every category gets an equal band of
//! the plot width. Within it, clustered series sit side by side, with the
//! chart's gap between bands and overlap between neighbours (Excel's
//! defaults: 219% of a column and -27%); stacked series share one column,
//! with the chart's gap between bands (Excel's default: 150%), positive
//! values piling up from zero and negative ones down.

use ironcalc_base::charts::{
    ChartData, ColumnChart, DataLabelPosition, DataLabels, Fill, Gridlines, Line, SeriesData,
    Stacking, TextDirection, TickMark,
};
use ironcalc_base::formatter::format::format_number;
use ironcalc_base::types::Color;

use crate::layout::{
    automatic_minor_stroke, automatic_stroke, layout_frame, resolve_font, series_color, stroke,
    Context, Frame, Rect, MAJOR_TICK_LENGTH, MINOR_TICK_LENGTH,
};
use crate::scene::{Anchor, Element, Role, Scene, Shape, Stroke};

/// Excel's default minor unit: a fifth of the major one.
const MINOR_DIVISIONS: usize = 5;

/// Space between a data label and the end of its column.
const LABEL_PADDING: f64 = 3.0;
/// Padding of the box drawn behind a data label that has a fill or a border.
const LABEL_BOX_PADDING_X: f64 = 3.0;
const LABEL_BOX_PADDING_Y: f64 = 1.0;

/// A column as drawn, with the value it stands for.
struct Column {
    /// The series it belongs to
    index: usize,
    /// The value of the cell, before any stacking
    value: f64,
    /// Whether the end of the column is its top (a positive value)
    positive: bool,
    rect: Rect,
}

/// The label of a column: its value, formatted, at the position the series
/// asks for. Returns the box behind the text, when it has a fill or a
/// border, and the text.
fn data_label(
    chart: &ColumnChart,
    labels: &DataLabels,
    series: &SeriesData,
    column: &Column,
    context: &Context,
) -> Vec<Element> {
    let Column {
        index,
        value,
        positive,
        rect,
    } = *column;
    let column = rect;
    let format = labels
        .number_format
        .as_deref()
        .unwrap_or(&series.number_format);
    let text = format_number(value, format, context.locale).text;
    let mut font = resolve_font(&labels.font, context.theme);
    if labels.font.color == Color::None {
        // Excel's labels are a little darker than its axis text
        font.color = context.theme.resolve(1, 0.25);
    }
    let extent = context.measurer.measure(&text, font.size, font.bold);
    // The text as it lies on the chart: a turned label is as tall as its
    // text is long, and its box turns with it
    let turned = labels.direction != TextDirection::Horizontal;
    let (width, height, pad_x, pad_y) = if turned {
        (
            extent.height(),
            extent.width,
            LABEL_BOX_PADDING_Y,
            LABEL_BOX_PADDING_X,
        )
    } else {
        (
            extent.width,
            extent.height(),
            LABEL_BOX_PADDING_X,
            LABEL_BOX_PADDING_Y,
        )
    };
    // Outside end does not exist for stacked columns
    let position = match (&labels.position, &chart.stacking) {
        (DataLabelPosition::OutsideEnd, Stacking::None) => DataLabelPosition::OutsideEnd,
        (DataLabelPosition::OutsideEnd, _) => DataLabelPosition::Center,
        (position, _) => position.clone(),
    };
    let (end, base) = if positive {
        (column.y, column.bottom())
    } else {
        (column.bottom(), column.y)
    };
    // The center of the label's box
    let above = |y: f64| y - LABEL_PADDING - height / 2.0;
    let below = |y: f64| y + LABEL_PADDING + height / 2.0;
    let center_y = match position {
        DataLabelPosition::Center => column.y + column.height / 2.0,
        DataLabelPosition::InsideEnd => {
            if positive {
                below(end)
            } else {
                above(end)
            }
        }
        DataLabelPosition::InsideBase => {
            if positive {
                above(base)
            } else {
                below(base)
            }
        }
        DataLabelPosition::OutsideEnd => {
            if positive {
                above(end)
            } else {
                below(end)
            }
        }
    };
    let center_x = column.x + column.width / 2.0;
    let mut elements = Vec::new();
    let fill = match &labels.fill {
        Color::None => None,
        color => Some(color.to_rgb(context.theme)),
    };
    let border = stroke(&labels.border, None, context.theme);
    if fill.is_some() || border.is_some() {
        elements.push(Element {
            role: Role::DataLabel(index),
            shape: Shape::Rect {
                x: center_x - width / 2.0 - pad_x,
                y: center_y - height / 2.0 - pad_y,
                width: width + 2.0 * pad_x,
                height: height + 2.0 * pad_y,
                fill,
                stroke: border,
            },
        });
    }
    // The middle of the baseline sits half the difference of ascent and
    // descent below the center of the glyphs, along the text's own down;
    // turned text rotates around that anchor
    let lift = (extent.ascent - extent.descent) / 2.0;
    let (x, y, rotation) = match labels.direction {
        TextDirection::Horizontal => (center_x, center_y + lift, 0.0),
        TextDirection::Rotate90 => (center_x - lift, center_y, 90.0),
        TextDirection::Rotate270 => (center_x + lift, center_y, -90.0),
    };
    elements.push(Element {
        role: Role::DataLabel(index),
        shape: Shape::Text {
            x,
            y,
            text,
            font,
            anchor: Anchor::Middle,
            rotation,
        },
    });
    elements
}

/// The x and width of the column of series `index` within the band starting
/// at `band_x`.
fn column_span(
    chart: &ColumnChart,
    index: usize,
    series: usize,
    band_x: f64,
    band: f64,
) -> (f64, f64) {
    let series = series.max(1) as f64;
    match chart.stacking {
        Stacking::None => {
            // n columns, n-1 overlaps between them and a gap, all in a band
            let gap = f64::from(chart.bars.gap.min(500)) / 100.0;
            let overlap = f64::from(chart.bars.overlap.clamp(-100, 100)) / 100.0;
            let width = band / (series - (series - 1.0) * overlap + gap);
            let cluster = width * (series - (series - 1.0) * overlap);
            let start = band_x + (band - cluster) / 2.0;
            (start + index as f64 * width * (1.0 - overlap), width)
        }
        Stacking::Standard | Stacking::Full => {
            let gap = f64::from(chart.bars.gap.min(500)) / 100.0;
            let width = band / (1.0 + gap);
            (band_x + (band - width) / 2.0, width)
        }
    }
}

/// The sum of the absolute values of a category, what a percent stacked
/// chart divides by.
fn category_total(data: &ChartData, category: usize) -> f64 {
    data.series
        .iter()
        .filter_map(|s| s.values.get(category).copied().flatten())
        .map(f64::abs)
        .sum()
}

/// Where the gridlines of the value axis cross it, as fractions of the plot
/// height: the major ticks, or the minor ones between them (Excel's default
/// minor unit is a fifth of the major unit).
fn value_fractions(frame: &Frame, minor: bool) -> Vec<f64> {
    let scale = &frame.scale;
    let intervals = ((scale.max - scale.min) / scale.step).round() as usize;
    let divisions = if minor { MINOR_DIVISIONS } else { 1 };
    (0..=intervals * divisions)
        .filter(|index| !minor || index % divisions != 0)
        .map(|index| index as f64 / (intervals * divisions) as f64)
        .collect()
}

/// Where the gridlines of the category axis cross it, as fractions of the
/// plot width: the major ones at the category boundaries, the minor ones
/// through the category centers, as Excel draws them when the value axis
/// crosses between categories.
fn category_fractions(frame: &Frame, minor: bool) -> Vec<f64> {
    let count = frame.categories.max(1) as f64;
    if minor {
        (0..frame.categories)
            .map(|index| (index as f64 + 0.5) / count)
            .collect()
    } else {
        (0..=frame.categories)
            .map(|index| index as f64 / count)
            .collect()
    }
}

/// The gridlines of both axes, minor under major, so a major line wins
/// where the two would coincide.
fn gridlines(chart: &ColumnChart, frame: &Frame, context: &Context) -> Vec<Element> {
    let theme = context.theme;
    let plot = frame.plot;
    let horizontal = |fraction: f64, stroke: &Stroke| {
        let y = plot.bottom() - fraction * plot.height;
        Shape::Line {
            x1: plot.x,
            y1: y,
            x2: plot.right(),
            y2: y,
            stroke: stroke.clone(),
        }
    };
    let vertical = |fraction: f64, stroke: &Stroke| {
        let x = plot.x + fraction * plot.width;
        Shape::Line {
            x1: x,
            y1: plot.y,
            x2: x,
            y2: plot.bottom(),
            stroke: stroke.clone(),
        }
    };
    let mut elements = Vec::new();
    for minor in [true, false] {
        let automatic = if minor {
            automatic_minor_stroke(theme)
        } else {
            automatic_stroke(theme)
        };
        let line = |gridlines: &Gridlines| {
            if minor {
                gridlines.minor.clone()
            } else {
                gridlines.major.clone()
            }
        };
        if let Some(stroke) = stroke(
            &line(&chart.category_axis.gridlines),
            Some(automatic.clone()),
            theme,
        ) {
            elements.extend(
                category_fractions(frame, minor)
                    .into_iter()
                    .map(|f| Element {
                        role: Role::Gridlines,
                        shape: vertical(f, &stroke),
                    }),
            );
        }
        if let Some(stroke) = stroke(&line(&chart.value_axis.gridlines), Some(automatic), theme) {
            elements.extend(value_fractions(frame, minor).into_iter().map(|f| Element {
                role: Role::Gridlines,
                shape: horizontal(f, &stroke),
            }));
        }
    }
    elements
}

/// The tick marks of both axes, drawn with the axis line at the major and
/// minor steps: into the plot, out of it towards the labels, or both ways.
fn tick_marks(chart: &ColumnChart, frame: &Frame, context: &Context) -> Vec<Element> {
    let theme = context.theme;
    let plot = frame.plot;
    // How far a tick reaches into the plot and out of it
    let reach = |tick: &TickMark, length: f64| match tick {
        TickMark::None => None,
        TickMark::Inside => Some((length, 0.0)),
        TickMark::Outside => Some((0.0, length)),
        TickMark::Cross => Some((length, length)),
    };
    let steps = |gridlines: &Gridlines| {
        [
            (false, reach(&gridlines.major_ticks, MAJOR_TICK_LENGTH)),
            (true, reach(&gridlines.minor_ticks, MINOR_TICK_LENGTH)),
        ]
    };
    let mut elements = Vec::new();
    let category_stroke = stroke(
        &chart.category_axis.line,
        Some(automatic_stroke(theme)),
        theme,
    );
    if let Some(stroke) = category_stroke {
        // On the category axis, where zero is; outside is downwards
        let y = plot.bottom() - frame.scale.baseline_fraction() * plot.height;
        for (minor, reach) in steps(&chart.category_axis.gridlines) {
            let Some((inside, outside)) = reach else {
                continue;
            };
            for fraction in category_fractions(frame, minor) {
                let x = plot.x + fraction * plot.width;
                elements.push(Element {
                    role: Role::CategoryAxis,
                    shape: Shape::Line {
                        x1: x,
                        y1: y - inside,
                        x2: x,
                        y2: y + outside,
                        stroke: stroke.clone(),
                    },
                });
            }
        }
    }
    // Excel's automatic value axis has no line, and so no tick marks
    if let Some(stroke) = stroke(&chart.value_axis.line, None, theme) {
        for (minor, reach) in steps(&chart.value_axis.gridlines) {
            let Some((inside, outside)) = reach else {
                continue;
            };
            for fraction in value_fractions(frame, minor) {
                let y = plot.bottom() - fraction * plot.height;
                elements.push(Element {
                    role: Role::ValueAxis,
                    shape: Shape::Line {
                        x1: plot.x - outside,
                        y1: y,
                        x2: plot.x + inside,
                        y2: y,
                        stroke: stroke.clone(),
                    },
                });
            }
        }
    }
    elements
}

fn columns(
    chart: &ColumnChart,
    data: &ChartData,
    frame: &Frame,
    context: &Context,
) -> Vec<Element> {
    let plot = frame.plot;
    let band = plot.width / frame.categories as f64;
    let series_count = data.series.len();
    let mut elements = Vec::new();
    // Painted after every column, so a label never hides under a neighbour
    let mut labels = Vec::new();
    for category in 0..frame.categories {
        let band_x = plot.x + category as f64 * band;
        let total = category_total(data, category);
        // Where the next positive and negative values of a stack start
        let (mut positive, mut negative) = (0.0, 0.0);
        for (index, series) in data.series.iter().enumerate() {
            let Some(Some(value)) = series.values.get(category) else {
                continue;
            };
            let raw = *value;
            let value = match chart.stacking {
                Stacking::Full if total > 0.0 => value / total,
                Stacking::Full => 0.0,
                _ => *value,
            };
            let (base, top) = match chart.stacking {
                Stacking::None => (0.0, value),
                _ if value >= 0.0 => {
                    positive += value;
                    (positive - value, positive)
                }
                _ => {
                    negative += value;
                    (negative - value, negative)
                }
            };
            let (x, width) = column_span(chart, index, series_count, band_x, band);
            let y_base = plot.bottom() - frame.scale.fraction(base) * plot.height;
            let y_top = plot.bottom() - frame.scale.fraction(top) * plot.height;
            let height = (y_base - y_top).abs();
            if height == 0.0 {
                continue;
            }
            let column = Rect {
                x,
                y: y_base.min(y_top),
                width,
                height,
            };
            elements.push(Element {
                role: Role::Series(index),
                shape: Shape::Rect {
                    x: column.x,
                    y: column.y,
                    width: column.width,
                    height: column.height,
                    fill: Some(series_color(chart, index, context.theme)),
                    stroke: None,
                },
            });
            if let Some(Some(series_labels)) = chart.data_labels.get(index) {
                labels.extend(data_label(
                    chart,
                    series_labels,
                    series,
                    &Column {
                        index,
                        value: raw,
                        positive: value >= 0.0,
                        rect: column,
                    },
                    context,
                ));
            }
        }
    }
    elements.extend(labels);
    elements
}

/// Lays out the whole chart: the frame, then gridlines and columns in the
/// plot area, painted right after the chart area so titles, labels and a
/// floating legend stay on top.
pub fn layout(
    chart: &ColumnChart,
    data: &ChartData,
    fill: &Fill,
    border: &Line,
    context: &Context,
) -> Scene {
    let (mut scene, frame) = layout_frame(chart, data, fill, border, context);
    let mut plot_elements = gridlines(chart, &frame, context);
    plot_elements.extend(columns(chart, data, &frame, context));
    plot_elements.extend(tick_marks(chart, &frame, context));
    let after_chart_area = scene.elements.len().min(1);
    scene
        .elements
        .splice(after_chart_area..after_chart_area, plot_elements);
    scene
}

/// The plot rectangle of a scene, for tests and tools.
pub fn plot_rect(scene: &Scene) -> Option<Rect> {
    scene
        .elements
        .iter()
        .find_map(|e| match (&e.role, &e.shape) {
            (
                Role::PlotArea,
                Shape::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                },
            ) => Some(Rect {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
            }),
            _ => None,
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::test_utils::column_chart;
    use crate::text::{Inter, TextMeasurer};
    use ironcalc_base::charts::Bars;
    use ironcalc_base::charts::LineStyle;
    use ironcalc_base::locale::get_locale;
    use ironcalc_base::types::{Color, Theme};

    fn data(series: &[&[Option<f64>]]) -> ChartData {
        ChartData {
            categories: (0..series[0].len()).map(|i| format!("C{i}")).collect(),
            series: series
                .iter()
                .map(|values| SeriesData {
                    name: "".to_string(),
                    values: values.to_vec(),
                    number_format: "General".to_string(),
                })
                .collect(),
        }
    }

    fn scene_of(chart: &ColumnChart, data: &ChartData) -> Scene {
        let theme = Theme::default();
        let context = Context {
            width: 480.0,
            height: 288.0,
            theme: &theme,
            locale: get_locale("en").unwrap(),
            measurer: &Inter,
        };
        layout(
            chart,
            data,
            &Fill::Automatic,
            &Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
            &context,
        )
    }

    fn bars(scene: &Scene, series: usize) -> Vec<Rect> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == Role::Series(series))
            .filter_map(|e| match &e.shape {
                Shape::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => Some(Rect {
                    x: *x,
                    y: *y,
                    width: *width,
                    height: *height,
                }),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn clustered_columns_share_a_band_with_excels_gaps() {
        let chart = column_chart();
        let data = data(&[&[Some(100.0), Some(50.0)], &[Some(200.0), None]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let band = plot.width / 2.0;
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        assert_eq!(first.len(), 2);
        // The empty cell has no column
        assert_eq!(second.len(), 1);

        // Two columns, one overlap of -27% and a gap of 219% fill the band
        let width = band / (2.0 + 0.27 + 2.19);
        assert!((first[0].width - width).abs() < 1e-9);
        assert!((second[0].x - (first[0].x + width * 1.27)).abs() < 1e-9);
        // and the cluster is centered in the band
        let cluster = width * 2.27;
        assert!((first[0].x - (plot.x + (band - cluster) / 2.0)).abs() < 1e-9);
        assert!((first[1].x - (plot.x + band + (band - cluster) / 2.0)).abs() < 1e-9);

        // Heights follow the scale: 0 to 250
        assert!((first[0].height - plot.height * 100.0 / 250.0).abs() < 1e-9);
        assert!((second[0].height - plot.height * 200.0 / 250.0).abs() < 1e-9);
        assert_eq!(first[0].y + first[0].height, plot.bottom());
    }

    #[test]
    fn the_bars_set_the_gap_between_bands_and_the_overlap_within() {
        // Chart 17 of AssortedColumns.xlsx: a gap of 100% and an overlap of -50%
        let mut chart = column_chart();
        chart.bars = Bars {
            gap: 100,
            overlap: -50,
        };
        let data = data(&[&[Some(100.0), Some(50.0)], &[Some(200.0), Some(20.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let band = plot.width / 2.0;
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        let width = band / (2.0 + 0.5 + 1.0);
        assert!((first[0].width - width).abs() < 1e-9);
        assert!((second[0].x - (first[0].x + width * 1.5)).abs() < 1e-9);
        let cluster = width * 2.5;
        assert!((first[0].x - (plot.x + (band - cluster) / 2.0)).abs() < 1e-9);

        // Overlapping columns: 100% puts them on the same spot
        chart.bars = Bars {
            gap: 50,
            overlap: 100,
        };
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        assert!((first[0].width - plot.width / 2.0 / 1.5).abs() < 1e-9);
        assert!((second[0].x - first[0].x).abs() < 1e-9);

        // Stacked columns take the gap and ignore the overlap
        chart.stacking = Stacking::Standard;
        chart.bars = Bars {
            gap: 50,
            overlap: -50,
        };
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        assert!((first[0].width - plot.width / 2.0 / 1.5).abs() < 1e-9);
        assert!((second[0].x - first[0].x).abs() < 1e-9);
    }

    #[test]
    fn stacked_columns_pile_up_on_both_sides_of_zero() {
        let mut chart = column_chart();
        chart.stacking = Stacking::Standard;
        chart.bars = Bars::stacked();
        let data = data(&[&[Some(100.0), Some(-50.0)], &[Some(50.0), Some(-100.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        // Same x, 40% of the band wide
        assert_eq!(first[0].x, second[0].x);
        assert!((first[0].width - plot.width / 2.0 / 2.5).abs() < 1e-9);
        // The second sits on top of the first in the first category...
        assert!((second[0].y + second[0].height - first[0].y).abs() < 1e-9);
        // ...and hangs below it in the second, where both are negative
        assert!((second[1].y - (first[1].y + first[1].height)).abs() < 1e-9);
        // The scale runs -200 to 200: the zero line is in the middle
        let zero = plot.y + plot.height / 2.0;
        assert!((first[0].y + first[0].height - zero).abs() < 1e-9);
        assert!((first[1].y - zero).abs() < 1e-9);
    }

    #[test]
    fn percent_stacked_columns_fill_the_plot() {
        let mut chart = column_chart();
        chart.stacking = Stacking::Full;
        chart.bars = Bars::stacked();
        let data = data(&[&[Some(25.0), Some(10.0)], &[Some(75.0), Some(30.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let first = bars(&scene, 0);
        let second = bars(&scene, 1);
        assert!((first[0].height - plot.height * 0.25).abs() < 1e-9);
        assert!((second[0].height - plot.height * 0.75).abs() < 1e-9);
        assert!((first[1].height - plot.height * 0.25).abs() < 1e-9);
        assert!((second[0].y - plot.y).abs() < 1e-9);
    }

    #[test]
    fn turned_labels_stand_on_end_and_turn_their_box() {
        let mut chart = column_chart();
        let mut up = labels(DataLabelPosition::OutsideEnd);
        up.direction = TextDirection::Rotate270;
        up.border = Line {
            style: LineStyle::Thin,
            color: Color::None,
        };
        let mut down = labels(DataLabelPosition::Center);
        down.direction = TextDirection::Rotate90;
        chart.data_labels = vec![Some(up), Some(down)];
        let data = data(&[&[Some(100.0)], &[Some(80.0)]]);
        let scene = scene_of(&chart, &data);
        let first_bars = bars(&scene, 0);
        let first = label_elements(&scene, 0);
        assert_eq!(first.len(), 2);
        // The box is taller than wide, centered on the column, and ends the
        // padding above it; the padding along the text is now vertical
        let Shape::Rect {
            x,
            y,
            width,
            height,
            ..
        } = first[0]
        else {
            panic!("the box comes first");
        };
        assert!(height > width);
        let extent = Inter.measure("100", 12.0, false);
        assert!((height - extent.width - 2.0 * LABEL_BOX_PADDING_X).abs() < 1e-9);
        assert!((width - extent.height() - 2.0 * LABEL_BOX_PADDING_Y).abs() < 1e-9);
        let center = first_bars[0].x + first_bars[0].width / 2.0;
        assert!((x + width / 2.0 - center).abs() < 1e-9);
        assert!(
            (y + height - LABEL_BOX_PADDING_X - (first_bars[0].y - LABEL_PADDING)).abs() < 1e-9
        );
        // Reading up: the anchor is in the middle of the box, a little right
        // of the column's center
        let (_, text_x, text_y, _) = text_at(first[1]);
        let Shape::Text { rotation, .. } = first[1] else {
            panic!("then the text");
        };
        assert_eq!(*rotation, -90.0);
        let lift = (extent.ascent - extent.descent) / 2.0;
        assert!((text_x - (center + lift)).abs() < 1e-9);
        assert!((text_y - (y + height / 2.0)).abs() < 1e-9);
        // Reading down, in the middle of its column, a little left of center
        let second_bars = bars(&scene, 1);
        let second = label_elements(&scene, 1);
        assert_eq!(second.len(), 1);
        let (_, text_x, text_y, _) = text_at(second[0]);
        let Shape::Text { rotation, .. } = second[0] else {
            panic!("no box, just the text");
        };
        assert_eq!(*rotation, 90.0);
        let extent = Inter.measure("80", 12.0, false);
        let lift = (extent.ascent - extent.descent) / 2.0;
        let center = second_bars[0].x + second_bars[0].width / 2.0;
        assert!((text_x - (center - lift)).abs() < 1e-9);
        assert!((text_y - (second_bars[0].y + second_bars[0].height / 2.0)).abs() < 1e-9);
    }

    fn labels(position: DataLabelPosition) -> DataLabels {
        DataLabels {
            position,
            direction: TextDirection::Horizontal,
            number_format: None,
            font: crate::test_utils::font(9),
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
        }
    }

    fn label_elements(scene: &Scene, series: usize) -> Vec<&Shape> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == Role::DataLabel(series))
            .map(|e| &e.shape)
            .collect()
    }

    fn text_at(shape: &Shape) -> (String, f64, f64, f64) {
        match shape {
            Shape::Text {
                text, x, y, font, ..
            } => (text.clone(), *x, *y, font.size),
            _ => panic!("expected a text"),
        }
    }

    #[test]
    fn labels_sit_at_the_four_positions_of_a_column() {
        let mut chart = column_chart();
        // A positive and a negative column, labelled outside their ends
        chart.data_labels = vec![Some(labels(DataLabelPosition::OutsideEnd))];
        let data = data(&[&[Some(100.0), Some(-50.0)]]);
        let scene = scene_of(&chart, &data);
        let bars = bars(&scene, 0);
        let texts = label_elements(&scene, 0);
        assert_eq!(texts.len(), 2);
        let (text, x, y, size) = text_at(texts[0]);
        assert_eq!(text, "100");
        assert_eq!(size, 12.0);
        assert!((x - (bars[0].x + bars[0].width / 2.0)).abs() < 1e-9);
        // Above the positive column: the whole text is over its top
        let extent = Inter.measure("100", 12.0, false);
        assert!((y + extent.descent + LABEL_PADDING - bars[0].y).abs() < 1e-9);
        // Below the negative one
        let (text, _, y, _) = text_at(texts[1]);
        assert_eq!(text, "-50");
        assert!((y - extent.ascent - LABEL_PADDING - (bars[1].y + bars[1].height)).abs() < 1e-9);

        // Inside end, inside base and center, on the positive column
        for (position, expected) in [
            (
                DataLabelPosition::InsideEnd,
                bars[0].y + LABEL_PADDING + extent.ascent,
            ),
            (
                DataLabelPosition::InsideBase,
                bars[0].y + bars[0].height - LABEL_PADDING - extent.descent,
            ),
            (
                DataLabelPosition::Center,
                bars[0].y + bars[0].height / 2.0 + (extent.ascent - extent.descent) / 2.0,
            ),
        ] {
            chart.data_labels = vec![Some(labels(position))];
            let scene = scene_of(&chart, &data);
            let (_, _, y, _) = text_at(label_elements(&scene, 0)[0]);
            assert!((y - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn stacked_columns_have_no_outside_end_and_show_the_raw_value() {
        let mut chart = column_chart();
        chart.stacking = Stacking::Full;
        chart.bars = Bars::stacked();
        chart.data_labels = vec![None, Some(labels(DataLabelPosition::OutsideEnd))];
        let data = data(&[&[Some(25.0)], &[Some(75.0)]]);
        let scene = scene_of(&chart, &data);
        assert!(label_elements(&scene, 0).is_empty());
        let bar = bars(&scene, 1)[0];
        let (text, _, y, _) = text_at(label_elements(&scene, 1)[0]);
        // The value, not the percentage, and centered as outside end is not allowed
        assert_eq!(text, "75");
        let extent = Inter.measure("75", 12.0, false);
        let center = bar.y + bar.height / 2.0 + (extent.ascent - extent.descent) / 2.0;
        assert!((y - center).abs() < 1e-9);
    }

    #[test]
    fn labels_can_have_a_format_a_fill_and_a_border() {
        let mut chart = column_chart();
        let mut styled = labels(DataLabelPosition::Center);
        styled.number_format = Some("#,##0.00".to_string());
        styled.fill = Color::Rgb("#FFFF00".to_string());
        styled.border.style = LineStyle::Thin;
        chart.data_labels = vec![Some(styled)];
        let data = data(&[&[Some(1234.5)]]);
        let scene = scene_of(&chart, &data);
        let elements = label_elements(&scene, 0);
        assert_eq!(elements.len(), 2);
        let (text, x, y, _) = text_at(elements[1]);
        assert_eq!(text, "1,234.50");
        let extent = Inter.measure("1,234.50", 12.0, false);
        match elements[0] {
            Shape::Rect {
                x: bx,
                y: by,
                width,
                height,
                fill,
                stroke,
            } => {
                assert_eq!(fill.as_deref(), Some("#FFFF00"));
                assert!(stroke.is_some());
                assert!((bx - (x - extent.width / 2.0 - LABEL_BOX_PADDING_X)).abs() < 1e-9);
                assert!((by - (y - extent.ascent - LABEL_BOX_PADDING_Y)).abs() < 1e-9);
                assert!((width - (extent.width + 2.0 * LABEL_BOX_PADDING_X)).abs() < 1e-9);
                assert!((height - (extent.height() + 2.0 * LABEL_BOX_PADDING_Y)).abs() < 1e-9);
            }
            _ => panic!("expected the label box"),
        }
        // Labels come after every column
        let last_column = scene
            .elements
            .iter()
            .rposition(|e| matches!(e.role, Role::Series(_)))
            .unwrap();
        let first_label = scene
            .elements
            .iter()
            .position(|e| matches!(e.role, Role::DataLabel(_)))
            .unwrap();
        assert!(first_label > last_column);
    }

    #[test]
    fn tick_marks_sit_on_the_axis_lines_and_push_the_labels_out() {
        let mut chart = column_chart();
        chart.value_axis.line = Line {
            style: LineStyle::Thin,
            color: Color::None,
        };
        let data = data(&[&[Some(200.0), Some(50.0)]]);
        let plain = scene_of(&chart, &data);
        let plain_plot = plot_rect(&plain).unwrap();
        let axis_lines = |scene: &Scene, role: Role| -> Vec<(f64, f64, f64, f64)> {
            scene
                .elements
                .iter()
                .filter(|e| e.role == role)
                .filter_map(|e| match &e.shape {
                    Shape::Line { x1, y1, x2, y2, .. } => Some((*x1, *y1, *x2, *y2)),
                    _ => None,
                })
                .collect()
        };
        // No ticks by default: only the axis lines
        assert_eq!(axis_lines(&plain, Role::ValueAxis).len(), 1);
        assert_eq!(axis_lines(&plain, Role::CategoryAxis).len(), 1);

        chart.value_axis.gridlines.major_ticks = TickMark::Cross;
        chart.value_axis.gridlines.minor_ticks = TickMark::Outside;
        chart.category_axis.gridlines.major_ticks = TickMark::Inside;
        chart.category_axis.gridlines.minor_ticks = TickMark::Cross;
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        // Outside ticks on the value axis move the plot right by their reach
        assert!((plot.x - plain_plot.x - MAJOR_TICK_LENGTH).abs() < 1e-9);
        // and on the category axis move the plot's bottom up
        assert!((plain_plot.bottom() - plot.bottom() - MINOR_TICK_LENGTH).abs() < 1e-9);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        // Value axis: 6 major ticks across the line and, outside, 4 minor
        // ones between each pair of them
        let value: Vec<_> = axis_lines(&scene, Role::ValueAxis)
            .into_iter()
            .filter(|(_, y1, _, y2)| y1 == y2)
            .collect();
        let major: Vec<_> = value
            .iter()
            .filter(|(x1, _, x2, _)| {
                near(*x1, plot.x - MAJOR_TICK_LENGTH) && near(*x2, plot.x + MAJOR_TICK_LENGTH)
            })
            .collect();
        let minor: Vec<_> = value
            .iter()
            .filter(|(x1, _, x2, _)| near(*x1, plot.x - MINOR_TICK_LENGTH) && near(*x2, plot.x))
            .collect();
        assert_eq!(major.len(), 6);
        assert_eq!(minor.len(), 20);
        assert!(near(major[0].1, plot.bottom()));
        // Category axis: 3 major ticks inside at the band boundaries, 2
        // minor ones across at the centers
        let category: Vec<_> = axis_lines(&scene, Role::CategoryAxis)
            .into_iter()
            .filter(|(x1, _, x2, _)| x1 == x2)
            .collect();
        let bottom = plot.bottom();
        let major: Vec<_> = category
            .iter()
            .filter(|(_, y1, _, y2)| near(*y1, bottom - MAJOR_TICK_LENGTH) && near(*y2, bottom))
            .collect();
        let minor: Vec<_> = category
            .iter()
            .filter(|(_, y1, _, y2)| {
                near(*y1, bottom - MINOR_TICK_LENGTH) && near(*y2, bottom + MINOR_TICK_LENGTH)
            })
            .collect();
        assert_eq!(major.len(), 3);
        assert_eq!(minor.len(), 2);
        assert!(near(major[0].0, plot.x));
        assert!(near(minor[0].0, plot.x + plot.width / 4.0));

        // Without a value axis line there are no value ticks either
        chart.value_axis.line.style = LineStyle::None;
        let scene = scene_of(&chart, &data);
        assert_eq!(axis_lines(&scene, Role::ValueAxis).len(), 0);
        assert!(near(plot_rect(&scene).unwrap().x, plain_plot.x));
    }

    #[test]
    fn gridlines_sit_on_the_ticks_behind_the_columns() {
        let chart = column_chart();
        let data = data(&[&[Some(200.0), Some(50.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let gridlines: Vec<f64> = scene
            .elements
            .iter()
            .filter(|e| e.role == Role::Gridlines)
            .map(|e| match &e.shape {
                Shape::Line { y1, .. } => *y1,
                _ => panic!("gridlines are lines"),
            })
            .collect();
        // 0, 50, ..., 250: six ticks
        assert_eq!(gridlines.len(), 6);
        assert_eq!(gridlines[0], plot.bottom());
        assert!((gridlines[5] - plot.y).abs() < 1e-9);
        // Order: chart area, gridlines, columns, then the rest
        assert_eq!(scene.elements[0].role, Role::ChartArea);
        assert_eq!(scene.elements[1].role, Role::Gridlines);
        assert_eq!(scene.elements[7].role, Role::Series(0));
        assert_eq!(scene.elements[8].role, Role::Series(0));
        assert_eq!(scene.elements[9].role, Role::Title);

        let mut no_gridlines = column_chart();
        no_gridlines.value_axis.gridlines.major.style = LineStyle::None;
        let scene = scene_of(&no_gridlines, &data);
        assert!(!scene.elements.iter().any(|e| e.role == Role::Gridlines));
    }

    fn gridline_shapes(scene: &Scene) -> Vec<(f64, f64, f64, f64, String)> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == Role::Gridlines)
            .map(|e| match &e.shape {
                Shape::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke,
                } => (*x1, *y1, *x2, *y2, stroke.color.clone()),
                _ => panic!("gridlines are lines"),
            })
            .collect()
    }

    #[test]
    fn minor_gridlines_split_every_major_step_in_five_and_go_under_the_major_ones() {
        let mut chart = column_chart();
        chart.value_axis.gridlines.minor.style = LineStyle::Automatic;
        let data = data(&[&[Some(200.0), Some(50.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let lines = gridline_shapes(&scene);
        // 0..250 by 50: 5 intervals, 4 minor lines each, then 6 major ones
        assert_eq!(lines.len(), 20 + 6);
        let minor = &lines[..20];
        let major = &lines[20..];
        assert!(minor.iter().all(|l| l.4 == "#F2F2F2"));
        assert!(major.iter().all(|l| l.4 == "#D9D9D9"));
        // The first minor line is one fifth of a step above the bottom
        let step = plot.height / 5.0;
        assert!((minor[0].1 - (plot.bottom() - step / 5.0)).abs() < 1e-9);
        // and none of them sits on a major tick
        for (_, y, _, _, _) in minor {
            assert!(major.iter().all(|m| (m.1 - y).abs() > 1e-6));
        }
    }

    #[test]
    fn category_gridlines_stand_on_the_boundaries_and_minor_ones_on_the_centers() {
        let mut chart = column_chart();
        chart.value_axis.gridlines.major.style = LineStyle::None;
        chart.category_axis.gridlines.major = Line {
            style: LineStyle::Thin,
            color: Color::Rgb("#FF0000".to_string()),
        };
        chart.category_axis.gridlines.minor.style = LineStyle::Dashed;
        // Three categories
        let data = data(&[&[Some(1.0), Some(2.0), Some(3.0)]]);
        let scene = scene_of(&chart, &data);
        let plot = plot_rect(&scene).unwrap();
        let lines = gridline_shapes(&scene);
        assert_eq!(lines.len(), 3 + 4);
        let band = plot.width / 3.0;
        // Minor first, through the middle of each band, top to bottom
        for (index, line) in lines[..3].iter().enumerate() {
            assert!((line.0 - (plot.x + (index as f64 + 0.5) * band)).abs() < 1e-9);
            assert_eq!(line.0, line.2);
            assert_eq!(line.1, plot.y);
            assert_eq!(line.3, plot.bottom());
            assert_eq!(line.4, "#D9D9D9");
        }
        // Then the major ones on every boundary, the plot edges included
        for (index, line) in lines[3..].iter().enumerate() {
            assert!((line.0 - (plot.x + index as f64 * band)).abs() < 1e-9);
            assert_eq!(line.4, "#FF0000");
        }
    }
}
