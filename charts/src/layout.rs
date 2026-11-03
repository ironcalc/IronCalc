//! Places the parts of a chart: the frame around the plot.
//!
//! Space is carved from the chart area in Excel's order: the title off the
//! top, the legend off its side, the axis titles, then the tick labels of
//! both axes. What remains is the plot area. Everything is measured with a
//! [`TextMeasurer`] so the margins fit the actual labels.
//!
//! Lengths are pixels at 96 dpi, so a 9pt label is 12px tall.

use ironcalc_base::charts::{
    Alignment, ChartData, ColumnChart, Fill, Font, Gridlines, Legend, Line, LineStyle, Position,
    Stacking, TickMark, Title,
};
use ironcalc_base::locale::Locale;
use ironcalc_base::types::{Color, Theme};

use crate::scale::{tick_labels, value_axis_format, value_scale, Scale};
use crate::scene::{Anchor, Element, ResolvedFont, Role, Scene, Shape, Stroke};
use crate::text::{TextExtent, TextMeasurer};

/// Pixels per point.
const PIXELS_PER_POINT: f64 = 4.0 / 3.0;
/// Space between the chart border and its content.
const PADDING: f64 = 8.0;
/// Space between blocks: title and plot, legend and plot, ...
const GAP: f64 = 6.0;
/// Space between tick labels and the plot area.
const TICK_LABEL_GAP: f64 = 4.0;
/// The length of the tick marks on the axis lines, in pixels.
pub(crate) const MAJOR_TICK_LENGTH: f64 = 5.0;
pub(crate) const MINOR_TICK_LENGTH: f64 = 3.0;
/// Side of the colored square of a legend entry, in ems of the legend font:
/// Excel's key is about 7px for 9pt text.
const SWATCH_EM: f64 = 0.6;
/// Space between a swatch and its text, in ems.
const SWATCH_GAP_EM: f64 = 0.4;
/// Space between legend entries, in ems.
const ENTRY_GAP_EM: f64 = 0.8;
/// Padding inside the legend box when the legend has none of its own.
const LEGEND_PADDING: f64 = 4.0;
/// Padding between a title and its box, when the title has a fill or a border.
const TITLE_BOX_PADDING_X: f64 = 4.0;
const TITLE_BOX_PADDING_Y: f64 = 2.0;
/// Widths of the line styles, in pixels.
const THIN_LINE: f64 = 1.0;
const THICK_LINE: f64 = 2.0;
const DASH: [f64; 2] = [4.0, 2.0];
/// The most intervals a value axis gets, however tall the plot.
const MAX_TICKS: usize = 10;
/// The fewest.
const MIN_TICKS: usize = 2;
/// Rotated category labels never take more than this share of the height
/// left for the plot.
const MAX_CATEGORY_LABEL_SHARE: f64 = 0.5;

/// Everything the layout needs besides the chart.
pub struct Context<'a> {
    pub width: f64,
    pub height: f64,
    pub theme: &'a Theme,
    pub locale: &'a Locale,
    pub measurer: &'a dyn TextMeasurer,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    fn deflate(&self, amount: f64) -> Rect {
        Rect {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - 2.0 * amount).max(0.0),
            height: (self.height - 2.0 * amount).max(0.0),
        }
    }

    /// Takes `amount` off the top.
    fn cut_top(&mut self, amount: f64) {
        let amount = amount.min(self.height);
        self.y += amount;
        self.height -= amount;
    }

    fn cut_bottom(&mut self, amount: f64) {
        self.height -= amount.min(self.height);
    }

    fn cut_left(&mut self, amount: f64) {
        let amount = amount.min(self.width);
        self.x += amount;
        self.width -= amount;
    }

    fn cut_right(&mut self, amount: f64) {
        self.width -= amount.min(self.width);
    }
}

/// The resolved frame of a chart: where everything is, before any data is
/// drawn in the plot area.
pub struct Frame {
    pub plot: Rect,
    pub scale: Scale,
    /// How many categories share the plot width.
    pub categories: usize,
}

struct Layout<'a> {
    chart: &'a ColumnChart,
    data: &'a ChartData,
    context: &'a Context<'a>,
    elements: Vec<Element>,
}

/// A line of a title, measured in its font.
struct TitleLine {
    text: String,
    font: ResolvedFont,
    extent: TextExtent,
}

/// How far the tick marks of an axis reach out of the plot, away from it.
pub(crate) fn outside_tick_reach(gridlines: &Gridlines) -> f64 {
    let reach = |tick: &TickMark, length: f64| match tick {
        TickMark::Outside | TickMark::Cross => length,
        TickMark::None | TickMark::Inside => 0.0,
    };
    f64::max(
        reach(&gridlines.major_ticks, MAJOR_TICK_LENGTH),
        reach(&gridlines.minor_ticks, MINOR_TICK_LENGTH),
    )
}

/// The width of the widest line and the height of all of them stacked.
fn block_size(lines: &[TitleLine]) -> (f64, f64) {
    lines.iter().fold((0.0, 0.0), |(width, height), line| {
        (
            f64::max(width, line.extent.width),
            height + line.extent.height(),
        )
    })
}

/// What is painted behind a title that has a fill or a border.
struct TitleBox {
    fill: Option<String>,
    stroke: Option<Stroke>,
}

impl TitleBox {
    /// The room the box adds around the text on each side, (x, y); none
    /// without a box.
    fn padding(boxed: &Option<TitleBox>) -> (f64, f64) {
        match boxed {
            Some(_) => (TITLE_BOX_PADDING_X, TITLE_BOX_PADDING_Y),
            None => (0.0, 0.0),
        }
    }
}

/// Excel's color for text that has none: dark 1 at 65%.
fn text_color(color: &Color, theme: &Theme) -> String {
    match color {
        Color::None => theme.resolve(1, 0.35),
        color => color.to_rgb(theme),
    }
}

/// A font with its size in pixels and its color resolved; no family is Inter.
pub(crate) fn resolve_font(font: &Font, theme: &Theme) -> ResolvedFont {
    ResolvedFont {
        family: font.family.clone().unwrap_or_else(|| "Inter".to_string()),
        size: f64::from(font.size) * PIXELS_PER_POINT,
        bold: font.bold,
        italic: font.italic,
        color: text_color(&font.color, theme),
    }
}

/// The fill of the chart area when automatic: light 1, white in most themes.
fn automatic_fill(theme: &Theme) -> String {
    theme.resolve(0, 0.0)
}

/// Excel's automatic line: thin, dark 1 at 15%.
pub(crate) fn automatic_stroke(theme: &Theme) -> Stroke {
    Stroke {
        color: theme.resolve(1, 0.85),
        width: THIN_LINE,
        dash: vec![],
    }
}

/// Excel's automatic minor gridline: thin, dark 1 at 5%.
pub(crate) fn automatic_minor_stroke(theme: &Theme) -> Stroke {
    Stroke {
        color: theme.resolve(1, 0.95),
        width: THIN_LINE,
        dash: vec![],
    }
}

/// The stroke of a line, `None` when there is nothing to draw. `automatic`
/// is what `LineStyle::Automatic` means for this particular line.
pub(crate) fn stroke(line: &Line, automatic: Option<Stroke>, theme: &Theme) -> Option<Stroke> {
    let color = match &line.color {
        Color::None => theme.resolve(1, 0.85),
        color => color.to_rgb(theme),
    };
    match line.style {
        LineStyle::Automatic => automatic,
        LineStyle::None => None,
        LineStyle::Thin => Some(Stroke {
            color,
            width: THIN_LINE,
            dash: vec![],
        }),
        LineStyle::Thick => Some(Stroke {
            color,
            width: THICK_LINE,
            dash: vec![],
        }),
        LineStyle::Dashed => Some(Stroke {
            color,
            width: THIN_LINE,
            dash: DASH.to_vec(),
        }),
    }
}

/// The color of the i-th series: its own, or the theme accents in turn,
/// darker on every lap.
pub(crate) fn series_color(chart: &ColumnChart, index: usize, theme: &Theme) -> String {
    match chart.colors.get(index) {
        Some(color) if color.is_some() => color.to_rgb(theme),
        _ => {
            let lap = (index / 6) as f64;
            theme.resolve(4 + (index % 6) as i32, -0.25 * lap.min(3.0))
        }
    }
}

/// The name shown for a series: Excel's "Series N" when it has none.
fn series_label(data: &ChartData, index: usize) -> String {
    match data.series.get(index).map(|s| s.name.as_str()) {
        Some("") | None => format!("Series{}", index + 1),
        Some(name) => name.to_string(),
    }
}

impl<'a> Layout<'a> {
    fn theme(&self) -> &Theme {
        self.context.theme
    }

    fn resolve_font(&self, font: &Font) -> ResolvedFont {
        resolve_font(font, self.theme())
    }

    fn measure(&self, text: &str, font: &ResolvedFont) -> TextExtent {
        self.context.measurer.measure(text, font.size, font.bold)
    }

    fn line_height(&self, font: &ResolvedFont) -> f64 {
        self.measure("", font).height()
    }

    fn push(&mut self, role: Role, shape: Shape) {
        self.elements.push(Element { role, shape });
    }

    fn push_rect(&mut self, role: Role, rect: Rect, fill: Option<String>, stroke: Option<Stroke>) {
        self.push(
            role,
            Shape::Rect {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                fill,
                stroke,
            },
        );
    }

    /// Horizontal text with its anchor at (`x`, `y`) on the baseline.
    fn push_text(
        &mut self,
        role: Role,
        x: f64,
        y: f64,
        text: &str,
        font: &ResolvedFont,
        anchor: Anchor,
    ) {
        self.push(
            role,
            Shape::Text {
                x,
                y,
                text: text.to_string(),
                font: font.clone(),
                anchor,
                rotation: 0.0,
            },
        );
    }

    /// The chart area: the whole picture, with the drawing's fill and border.
    fn chart_area(&mut self, fill: &Fill, border: &Line) -> Rect {
        let area = Rect {
            x: 0.0,
            y: 0.0,
            width: self.context.width,
            height: self.context.height,
        };
        let fill = match fill {
            Fill::Automatic => Some(automatic_fill(self.theme())),
            Fill::None => None,
            Fill::Solid(color) => Some(color.to_rgb(self.theme())),
        };
        let stroke = stroke(border, Some(automatic_stroke(self.theme())), self.theme());
        self.push_rect(Role::ChartArea, area, fill, stroke);
        area.deflate(PADDING)
    }

    /// The box behind a title: `None` when it has neither fill nor border,
    /// else what to paint and the padding it adds around the text.
    fn title_box(&self, title: &Title) -> Option<TitleBox> {
        let fill = match &title.fill {
            Color::None => None,
            color => Some(color.to_rgb(self.theme())),
        };
        // Excel's automatic title has no border
        let stroke = stroke(&title.border, None, self.theme());
        if fill.is_none() && stroke.is_none() {
            return None;
        }
        Some(TitleBox { fill, stroke })
    }

    fn title_line(&self, text: &str, font: &Font) -> TitleLine {
        let font = self.resolve_font(font);
        let extent = self.measure(text, &font);
        TitleLine {
            text: text.to_string(),
            font,
            extent,
        }
    }

    /// The lines of a title: its text and, under it, the subtitle if any.
    fn title_lines(&self, title: &Title) -> Vec<TitleLine> {
        let mut lines = vec![self.title_line(&title.text, &title.font)];
        if let Some(subtitle) = title.subtitle.as_ref().filter(|s| !s.text.is_empty()) {
            lines.push(self.title_line(&subtitle.text, &subtitle.font));
        }
        lines
    }

    /// A title across the top of `inner`, which shrinks by its height.
    fn title(&mut self, title: &Title, inner: &mut Rect, role: Role) {
        let lines = self.title_lines(title);
        let (width, height) = block_size(&lines);
        let boxed = self.title_box(title);
        let (pad_x, pad_y) = TitleBox::padding(&boxed);
        let (x, anchor) = match title.alignment {
            Alignment::Left => (inner.x + pad_x, Anchor::Start),
            Alignment::Middle => (inner.x + inner.width / 2.0, Anchor::Middle),
            Alignment::Right => (inner.right() - pad_x, Anchor::End),
        };
        if let Some(boxed) = boxed {
            let left = match anchor {
                Anchor::Start => x,
                Anchor::Middle => x - width / 2.0,
                Anchor::End => x - width,
            };
            let rect = Rect {
                x: left - pad_x,
                y: inner.y,
                width: width + 2.0 * pad_x,
                height: height + 2.0 * pad_y,
            };
            self.push_rect(role, rect, boxed.fill, boxed.stroke);
        }
        let mut top = inner.y + pad_y;
        for line in &lines {
            self.push_text(
                role,
                x,
                top + line.extent.ascent,
                &line.text,
                &line.font,
                anchor,
            );
            top += line.extent.height();
        }
        inner.cut_top(height + 2.0 * pad_y + GAP);
    }

    /// The legend, carved from the side it sits on unless it floats.
    fn legend(&mut self, legend: &Legend, inner: &mut Rect) {
        if legend.position == Position::None || self.data.series.is_empty() {
            return;
        }
        let font = self.resolve_font(&legend.font);
        let line_height = self.line_height(&font);
        let swatch_side = font.size * SWATCH_EM;
        let swatch_gap = font.size * SWATCH_GAP_EM;
        let entry_gap = font.size * ENTRY_GAP_EM;
        // Stacked series are listed top to bottom as they stack, so a
        // vertical legend shows the last series first, as in Excel
        let reversed = legend.vertical && self.chart.stacking != Stacking::None;
        let mut order: Vec<usize> = (0..self.data.series.len()).collect();
        if reversed {
            order.reverse();
        }
        let entries: Vec<(usize, String, TextExtent)> = order
            .into_iter()
            .map(|index| {
                let label = series_label(self.data, index);
                let extent = self.measure(&label, &font);
                (index, label, extent)
            })
            .collect();
        let entry_height = line_height.max(swatch_side);
        let (top, right, bottom, left) = match legend.padding {
            Some((top, right, bottom, left)) => (
                f64::from(top),
                f64::from(right),
                f64::from(bottom),
                f64::from(left),
            ),
            None => (
                LEGEND_PADDING,
                LEGEND_PADDING,
                LEGEND_PADDING,
                LEGEND_PADDING,
            ),
        };
        let entry_width = |extent: &TextExtent| swatch_side + swatch_gap + extent.width;
        let (content_width, content_height) = if legend.vertical {
            let widest = entries
                .iter()
                .map(|(_, _, extent)| entry_width(extent))
                .fold(0.0, f64::max);
            let count = entries.len() as f64;
            (widest, count * entry_height + (count - 1.0) * entry_gap)
        } else {
            let total: f64 = entries
                .iter()
                .map(|(_, _, extent)| entry_width(extent))
                .sum();
            let count = entries.len() as f64;
            (total + (count - 1.0) * entry_gap, entry_height)
        };
        let width = left + content_width + right;
        let height = top + content_height + bottom;

        // Where the box goes, and what it takes from the plot
        let (x, y) = match legend.position {
            Position::Bottom => {
                let y = inner.bottom() - height;
                inner.cut_bottom(height + GAP);
                (inner.x + (inner.width - width) / 2.0, y)
            }
            Position::Top => {
                let y = inner.y;
                inner.cut_top(height + GAP);
                (inner.x + (inner.width - width) / 2.0, y)
            }
            Position::Right => {
                let x = inner.right() - width;
                inner.cut_right(width + GAP);
                (x, inner.y + (inner.height - height) / 2.0)
            }
            Position::Left => {
                let x = inner.x;
                inner.cut_left(width + GAP);
                (x, inner.y + (inner.height - height) / 2.0)
            }
            Position::Float(fx, fy) => {
                let (x, y) = (fx * self.context.width, fy * self.context.height);
                if !legend.overlay {
                    self.make_room_for(
                        Rect {
                            x,
                            y,
                            width,
                            height,
                        },
                        inner,
                    );
                }
                (x, y)
            }
            Position::None => return,
        };
        let frame = Rect {
            x,
            y,
            width,
            height,
        };
        let fill = match &legend.fill {
            Color::None => None,
            color => Some(color.to_rgb(self.theme())),
        };
        // Excel's automatic legend has no border
        let border = stroke(&legend.border, None, self.theme());
        if fill.is_some() || border.is_some() {
            self.push_rect(Role::Legend, frame, fill, border);
        }

        let mut cursor_x = x + left;
        let mut cursor_y = y + top;
        for (index, label, extent) in &entries {
            let swatch = Rect {
                x: cursor_x,
                y: cursor_y + (entry_height - swatch_side) / 2.0,
                width: swatch_side,
                height: swatch_side,
            };
            let color = series_color(self.chart, *index, self.theme());
            self.push_rect(Role::Legend, swatch, Some(color), None);
            let baseline = cursor_y + (entry_height + extent.ascent - extent.descent) / 2.0;
            self.push_text(
                Role::Legend,
                cursor_x + swatch_side + swatch_gap,
                baseline,
                label,
                &font,
                Anchor::Start,
            );
            if legend.vertical {
                cursor_y += entry_height + entry_gap;
            } else {
                cursor_x += entry_width(extent) + entry_gap;
            }
        }
    }

    /// Keeps the plot out of a floating box that does not overlay it: the
    /// plot gives up the strip between the box and the chart edge the box is
    /// closest to, as Excel does for a legend dragged to a side.
    fn make_room_for(&self, boxed: Rect, inner: &mut Rect) {
        let (width, height) = (self.context.width, self.context.height);
        let center_x = (boxed.x + boxed.width / 2.0) / width;
        let center_y = (boxed.y + boxed.height / 2.0) / height;
        let distances = [
            (center_x, 0),
            (1.0 - center_x, 1),
            (center_y, 2),
            (1.0 - center_y, 3),
        ];
        let nearest = distances
            .iter()
            .fold(distances[0], |best, d| if d.0 < best.0 { *d } else { best })
            .1;
        match nearest {
            0 => inner.cut_left((boxed.right() + GAP - inner.x).max(0.0)),
            1 => inner.cut_right((inner.right() - boxed.x + GAP).max(0.0)),
            2 => inner.cut_top((boxed.bottom() + GAP - inner.y).max(0.0)),
            _ => inner.cut_bottom((inner.bottom() - boxed.y + GAP).max(0.0)),
        }
    }

    /// The title of the value axis, reading upwards along the left edge; the
    /// subtitle reads up beside it, on its right.
    fn value_axis_title(&mut self, title: &Title, inner: &mut Rect) {
        let lines = self.title_lines(title);
        // Along the text and across it, before the turn
        let (length, thickness) = block_size(&lines);
        let boxed = self.title_box(title);
        let (pad_x, pad_y) = TitleBox::padding(&boxed);
        let middle = inner.y + inner.height / 2.0;
        if let Some(boxed) = boxed {
            // The box turns with the text: its padding along the text is
            // vertical here
            let rect = Rect {
                x: inner.x,
                y: middle - length / 2.0 - pad_x,
                width: thickness + 2.0 * pad_y,
                height: length + 2.0 * pad_x,
            };
            self.push_rect(Role::ValueAxis, rect, boxed.fill, boxed.stroke);
        }
        // Rotated -90 around the anchor: the baseline runs up from (x, y)
        let mut left = inner.x + pad_y;
        for line in lines {
            self.push(
                Role::ValueAxis,
                Shape::Text {
                    x: left + line.extent.ascent,
                    y: middle,
                    text: line.text,
                    font: line.font,
                    anchor: Anchor::Middle,
                    rotation: -90.0,
                },
            );
            left += line.extent.height();
        }
        inner.cut_left(thickness + 2.0 * pad_y + GAP);
    }

    /// The title of the category axis, along the bottom edge.
    fn category_axis_title(&mut self, title: &Title, inner: &mut Rect) {
        let lines = self.title_lines(title);
        let (width, height) = block_size(&lines);
        let boxed = self.title_box(title);
        let (pad_x, pad_y) = TitleBox::padding(&boxed);
        let middle = inner.x + inner.width / 2.0;
        if let Some(boxed) = boxed {
            let rect = Rect {
                x: middle - width / 2.0 - pad_x,
                y: inner.bottom() - height - 2.0 * pad_y,
                width: width + 2.0 * pad_x,
                height: height + 2.0 * pad_y,
            };
            self.push_rect(Role::CategoryAxis, rect, boxed.fill, boxed.stroke);
        }
        // Stacked up from the bottom, the last line first
        let mut bottom = inner.bottom() - pad_y;
        for line in lines.iter().rev() {
            self.push_text(
                Role::CategoryAxis,
                middle,
                bottom - line.extent.descent,
                &line.text,
                &line.font,
                Anchor::Middle,
            );
            bottom -= line.extent.height();
        }
        inner.cut_bottom(height + 2.0 * pad_y + GAP);
    }

    /// How many intervals fit on the value axis: one every two label heights.
    fn max_ticks(&self, plot_height: f64, label_height: f64) -> usize {
        ((plot_height / (2.0 * label_height)).floor() as usize).clamp(MIN_TICKS, MAX_TICKS)
    }

    fn frame(&mut self, fill: &Fill, border: &Line) -> Frame {
        let chart = self.chart;
        let mut inner = self.chart_area(fill, border);
        // A title without any text shows nothing and takes no room
        let shown = |title: &'a Option<Title>| {
            title.as_ref().filter(|t| {
                !t.text.is_empty() || t.subtitle.as_ref().is_some_and(|s| !s.text.is_empty())
            })
        };
        if let Some(title) = shown(&chart.title) {
            self.title(title, &mut inner, Role::Title);
        }
        self.legend(&chart.legend, &mut inner);
        if let Some(title) = shown(&chart.value_axis.title) {
            self.value_axis_title(title, &mut inner);
        }
        if let Some(title) = shown(&chart.category_axis.title) {
            self.category_axis_title(title, &mut inner);
        }

        // Tick labels: the value axis needs the plot height to pick its
        // ticks, which the category labels at the bottom reduce first.
        let categories = self
            .data
            .categories
            .len()
            .max(
                self.data
                    .series
                    .iter()
                    .map(|s| s.values.len())
                    .max()
                    .unwrap_or(0),
            )
            .max(1);
        let category_labels: Vec<String> = (0..categories)
            .map(|index| match self.data.categories.get(index) {
                Some(text) if !self.data.categories.is_empty() => text.clone(),
                // Without categories Excel numbers them
                _ => (index + 1).to_string(),
            })
            .collect();
        let category_font = self.resolve_font(&chart.category_axis.labels);
        let category_extent = self.measure("", &category_font);
        // Excel's angle runs counter-clockwise, the scene's clockwise. The
        // labels hang below the axis in the box of their rotated extent.
        let rotation = -f64::from(chart.category_axis.rotation.unwrap_or(0));
        let (sin, cos) = rotation.to_radians().sin_cos();
        let labels_height = if rotation == 0.0 {
            category_extent.height()
        } else {
            let widest = category_labels
                .iter()
                .map(|label| self.measure(label, &category_font).width)
                .fold(0.0, f64::max);
            let height = widest * sin.abs() + category_extent.height() * cos;
            height.min(inner.height * MAX_CATEGORY_LABEL_SHARE)
        };
        // Tick marks that point away from the plot push the labels out
        let theme = self.context.theme;
        let category_stroke = stroke(
            &chart.category_axis.line,
            Some(automatic_stroke(theme)),
            theme,
        );
        let value_stroke = stroke(&chart.value_axis.line, None, theme);
        let category_reach = if category_stroke.is_some() {
            outside_tick_reach(&chart.category_axis.gridlines)
        } else {
            0.0
        };
        let value_reach = if value_stroke.is_some() {
            outside_tick_reach(&chart.value_axis.gridlines)
        } else {
            0.0
        };
        inner.cut_bottom(labels_height + TICK_LABEL_GAP + category_reach);

        let value_font = self.resolve_font(&chart.value_axis.labels.font);
        let value_extent = self.measure("", &value_font);
        let max_ticks = self.max_ticks(inner.height, value_extent.height());
        let scale = value_scale(chart, self.data, max_ticks);
        let format = value_axis_format(chart, self.data);
        let labels = tick_labels(&scale, &format, self.context.locale);
        let widest = labels
            .iter()
            .map(|label| self.measure(label, &value_font).width)
            .fold(0.0, f64::max);
        inner.cut_left(widest + TICK_LABEL_GAP + value_reach);
        let plot = inner;

        // Now that the plot is known, place the labels around it
        for (tick, label) in scale.ticks().iter().zip(&labels) {
            let y = plot.bottom() - scale.fraction(*tick) * plot.height;
            self.push_text(
                Role::ValueAxis,
                plot.x - TICK_LABEL_GAP - value_reach,
                y + (value_extent.ascent - value_extent.descent) / 2.0,
                label,
                &value_font,
                Anchor::End,
            );
        }
        // Horizontal labels are centered under their band. Rotated ones hang
        // from the axis by the end nearest to it: the end of the text when it
        // reads upwards, the start when it reads downwards. The middle of
        // that edge sits under the band's center, the top at the gap.
        let band = plot.width / categories as f64;
        let anchor = if rotation == 0.0 {
            Anchor::Middle
        } else if rotation < 0.0 {
            Anchor::End
        } else {
            Anchor::Start
        };
        // The unit vector from the baseline towards the ascenders
        let (up_x, up_y) = (sin, -cos);
        let half_height = category_extent.height() / 2.0;
        let baseline_offset = (category_extent.ascent - category_extent.descent) / 2.0;
        for (index, label) in category_labels.iter().enumerate() {
            let middle_x = plot.x + (index as f64 + 0.5) * band;
            let middle_y = plot.bottom() + TICK_LABEL_GAP + category_reach + half_height * cos;
            self.push(
                Role::CategoryAxis,
                Shape::Text {
                    x: middle_x - baseline_offset * up_x,
                    y: middle_y - baseline_offset * up_y,
                    text: label.clone(),
                    font: category_font.clone(),
                    anchor,
                    rotation,
                },
            );
        }

        // The axis lines: the category axis sits where zero is
        if let Some(stroke) = category_stroke {
            let y = plot.bottom() - scale.baseline_fraction() * plot.height;
            self.push(
                Role::CategoryAxis,
                Shape::Line {
                    x1: plot.x,
                    y1: y,
                    x2: plot.right(),
                    y2: y,
                    stroke,
                },
            );
        }
        // Excel's automatic value axis has no line
        if let Some(stroke) = value_stroke {
            self.push(
                Role::ValueAxis,
                Shape::Line {
                    x1: plot.x,
                    y1: plot.y,
                    x2: plot.x,
                    y2: plot.bottom(),
                    stroke,
                },
            );
        }
        self.push_rect(Role::PlotArea, plot, None, None);
        Frame {
            plot,
            scale,
            categories,
        }
    }
}

/// Lays out the frame of `chart` in a picture of the context's size: chart
/// area, title, legend, axes and their labels. `fill` and `border` are those
/// of the drawing that holds the chart.
pub fn layout_frame(
    chart: &ColumnChart,
    data: &ChartData,
    fill: &Fill,
    border: &Line,
    context: &Context,
) -> (Scene, Frame) {
    let mut layout = Layout {
        chart,
        data,
        context,
        elements: Vec::new(),
    };
    let frame = layout.frame(fill, border);
    let scene = Scene {
        width: context.width,
        height: context.height,
        elements: layout.elements,
    };
    (scene, frame)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::test_utils::{column_chart, title};
    use crate::text::Inter;
    use ironcalc_base::charts::{Bars, SeriesData};
    use ironcalc_base::locale::get_locale;

    fn data() -> ChartData {
        ChartData {
            categories: vec!["London", "Madrid", "Rome", "Paris"]
                .into_iter()
                .map(String::from)
                .collect(),
            series: vec![
                SeriesData {
                    name: "September".to_string(),
                    values: vec![Some(123.0), Some(23.0), Some(234.0), Some(156.0)],
                    number_format: "General".to_string(),
                },
                SeriesData {
                    name: "".to_string(),
                    values: vec![Some(167.0), Some(456.0), Some(150.0), Some(300.0)],
                    number_format: "General".to_string(),
                },
            ],
        }
    }

    fn frame_of(chart: &ColumnChart, data: &ChartData) -> (Scene, Frame) {
        let theme = Theme::default();
        let context = Context {
            width: 480.0,
            height: 288.0,
            theme: &theme,
            locale: get_locale("en").unwrap(),
            measurer: &Inter,
        };
        layout_frame(
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

    fn texts(scene: &Scene, role: Role) -> Vec<(&str, f64, f64, Anchor)> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == role)
            .filter_map(|e| match &e.shape {
                Shape::Text {
                    x, y, text, anchor, ..
                } => Some((text.as_str(), *x, *y, *anchor)),
                _ => None,
            })
            .collect()
    }

    fn rects(scene: &Scene, role: Role) -> Vec<Rect> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == role)
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
    fn the_title_is_centered_at_the_top_and_the_legend_sits_at_the_bottom() {
        let chart = column_chart();
        let (scene, frame) = frame_of(&chart, &data());
        assert_eq!(scene.width, 480.0);
        let area = rects(&scene, Role::ChartArea)[0];
        assert_eq!(area.width, 480.0);

        let title = texts(&scene, Role::Title);
        assert_eq!(title.len(), 1);
        assert_eq!(title[0].0, "Population");
        assert_eq!(title[0].1, 240.0);
        assert!(matches!(title[0].3, Anchor::Middle));
        // The baseline is one ascent below the padding: 14pt is 18.67px
        let ascent = Inter.measure("", 14.0 * PIXELS_PER_POINT, false).ascent;
        assert!((title[0].2 - (PADDING + ascent)).abs() < 1e-9);

        let legend = texts(&scene, Role::Legend);
        assert_eq!(legend.len(), 2);
        assert_eq!(legend[0].0, "September");
        assert_eq!(legend[1].0, "Series2");
        // Both on one line, the second to the right of the first
        assert_eq!(legend[0].2, legend[1].2);
        assert!(legend[1].1 > legend[0].1);
        let swatches = rects(&scene, Role::Legend);
        assert_eq!(swatches.len(), 2);
        assert!(legend[0].2 > frame.plot.bottom());

        // The plot is inside, below the title and above the legend
        assert!(frame.plot.y > title[0].2);
        assert!(frame.plot.bottom() < swatches[0].y);
        assert!(frame.plot.x > PADDING && frame.plot.right() <= 480.0 - PADDING);
        assert_eq!(frame.categories, 4);
    }

    #[test]
    fn tick_labels_are_right_aligned_left_of_the_plot_and_categories_centered_below() {
        let chart = column_chart();
        let (scene, frame) = frame_of(&chart, &data());
        assert_eq!(
            frame.scale,
            Scale {
                min: 0.0,
                max: 500.0,
                step: 100.0
            }
        );
        let values = texts(&scene, Role::ValueAxis);
        assert_eq!(values.len(), 6);
        assert_eq!(values[0].0, "0");
        assert_eq!(values[5].0, "500");
        for (_, x, _, anchor) in &values {
            assert_eq!(*x, frame.plot.x - TICK_LABEL_GAP);
            assert!(matches!(anchor, Anchor::End));
        }
        // The widest label sets the margin (Inter's digits are not all the same width)
        let widest = ["0", "100", "200", "300", "400", "500"]
            .iter()
            .map(|label| Inter.measure(label, 12.0, false).width)
            .fold(0.0, f64::max);
        assert!((frame.plot.x - (PADDING + widest + TICK_LABEL_GAP)).abs() < 1e-9);
        // "0" is at the bottom of the plot, "500" at the top
        assert!(values[0].2 > values[5].2);

        let categories = texts(&scene, Role::CategoryAxis);
        assert_eq!(categories.len(), 4);
        assert_eq!(categories[1].0, "Madrid");
        let band = frame.plot.width / 4.0;
        assert!((categories[1].1 - (frame.plot.x + 1.5 * band)).abs() < 1e-9);
        assert!(categories[1].2 > frame.plot.bottom());

        // The category axis line is at zero, the bottom of the plot
        let lines: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| matches!(e.shape, Shape::Line { .. }))
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].role, Role::CategoryAxis);
        if let Shape::Line { y1, x1, x2, .. } = &lines[0].shape {
            assert_eq!(*y1, frame.plot.bottom());
            assert_eq!((*x1, *x2), (frame.plot.x, frame.plot.right()));
        }
    }

    fn rotated_texts(scene: &Scene, role: Role) -> Vec<(f64, f64, Anchor, f64)> {
        scene
            .elements
            .iter()
            .filter(|e| e.role == role)
            .filter_map(|e| match &e.shape {
                Shape::Text {
                    x,
                    y,
                    anchor,
                    rotation,
                    ..
                } => Some((*x, *y, *anchor, *rotation)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn rotated_category_labels_hang_from_the_axis_and_make_room_below_it() {
        let data = data();
        let mut chart = column_chart();
        let (_, horizontal) = frame_of(&chart, &data);

        // Excel's 45 reads upwards: the end of the text is nearest the axis
        chart.category_axis.rotation = Some(45);
        let (scene, rising) = frame_of(&chart, &data);
        let labels = rotated_texts(&scene, Role::CategoryAxis);
        assert_eq!(labels.len(), 4);
        for (_, y, anchor, rotation) in &labels {
            assert_eq!(*rotation, -45.0);
            assert!(matches!(anchor, Anchor::End));
            assert!(*y > rising.plot.bottom());
        }
        // The widest label at 45 degrees takes more height than a line of
        // text, which the plot gives up
        let font = Inter.measure("London", 12.0, false);
        let widest = ["London", "Madrid", "Rome", "Paris"]
            .iter()
            .map(|label| Inter.measure(label, 12.0, false).width)
            .fold(0.0, f64::max);
        let rotated_height = widest * 0.5_f64.sqrt() + font.height() * 0.5_f64.sqrt();
        let difference = horizontal.plot.height - rising.plot.height;
        assert!((difference - (rotated_height - font.height())).abs() < 1e-9);
        // The text end sits under the center of its band, shifted by half the
        // text height along the slanted edge
        let band = rising.plot.width / 4.0;
        let baseline_offset = (font.ascent - font.descent) / 2.0;
        let expected_x =
            rising.plot.x + 0.5 * band - baseline_offset * (-45.0_f64).to_radians().sin();
        assert!((labels[0].0 - expected_x).abs() < 1e-9);

        // Excel's -60 reads downwards: the start of the text is nearest the axis
        chart.category_axis.rotation = Some(-60);
        let (scene, falling) = frame_of(&chart, &data);
        for (_, _, anchor, rotation) in rotated_texts(&scene, Role::CategoryAxis) {
            assert_eq!(rotation, 60.0);
            assert!(matches!(anchor, Anchor::Start));
        }
        assert!(falling.plot.height < rising.plot.height);

        // Vertical labels take exactly the width of the widest
        chart.category_axis.rotation = Some(90);
        let (_, vertical) = frame_of(&chart, &data);
        let difference = horizontal.plot.height - vertical.plot.height;
        assert!((difference - (widest - font.height())).abs() < 1e-9);
    }

    #[test]
    fn a_legend_on_the_right_stacks_entries_and_narrows_the_plot() {
        let mut chart = column_chart();
        chart.legend.position = Position::Right;
        chart.legend.vertical = true;
        let (scene, frame) = frame_of(&chart, &data());
        let legend = texts(&scene, Role::Legend);
        assert_eq!(legend[0].1, legend[1].1);
        assert!(legend[1].2 > legend[0].2);
        assert!(legend[0].1 > frame.plot.right());

        let (_, bottom_frame) = frame_of(&column_chart(), &data());
        assert!(frame.plot.width < bottom_frame.plot.width);
        assert!(frame.plot.height > bottom_frame.plot.height);
    }

    #[test]
    fn a_vertical_legend_of_a_stacked_chart_lists_the_top_series_first() {
        let mut chart = column_chart();
        chart.legend.position = Position::Right;
        chart.legend.vertical = true;
        chart.stacking = Stacking::Standard;
        chart.bars = Bars::stacked();
        let (scene, _) = frame_of(&chart, &data());
        let legend = texts(&scene, Role::Legend);
        assert_eq!(legend[0].0, "Series2");
        assert_eq!(legend[1].0, "September");
        // The swatch colors follow the names
        let theme = Theme::default();
        let swatches: Vec<_> = scene
            .elements
            .iter()
            .filter(|e| e.role == Role::Legend)
            .filter_map(|e| match &e.shape {
                Shape::Rect { fill, .. } => fill.clone(),
                _ => None,
            })
            .collect();
        assert_eq!(swatches, vec![theme.accent2.clone(), theme.accent1.clone()]);

        // Clustered, or horizontal, keeps the natural order
        chart.stacking = Stacking::None;
        let (scene, _) = frame_of(&chart, &data());
        assert_eq!(texts(&scene, Role::Legend)[0].0, "September");
        chart.stacking = Stacking::Standard;
        chart.bars = Bars::stacked();
        chart.legend.position = Position::Bottom;
        chart.legend.vertical = false;
        let (scene, _) = frame_of(&chart, &data());
        assert_eq!(texts(&scene, Role::Legend)[0].0, "September");
    }

    #[test]
    fn the_legend_key_scales_with_the_font() {
        let mut chart = column_chart();
        let (scene, _) = frame_of(&chart, &data());
        let small = rects(&scene, Role::Legend)[0];
        assert!((small.width - 12.0 * SWATCH_EM).abs() < 1e-9);
        chart.legend.font.size = 18;
        let (scene, _) = frame_of(&chart, &data());
        let large = rects(&scene, Role::Legend)[0];
        assert!((large.width - 24.0 * SWATCH_EM).abs() < 1e-9);
        assert_eq!(large.width, large.height);
    }

    #[test]
    fn a_floating_legend_that_overlays_takes_no_room() {
        let mut chart = column_chart();
        chart.legend.position = Position::Float(0.5, 0.25);
        chart.legend.overlay = true;
        let (scene, frame) = frame_of(&chart, &data());
        let mut no_legend = column_chart();
        no_legend.legend.position = Position::None;
        let (_, plain) = frame_of(&no_legend, &data());
        assert_eq!(frame.plot, plain.plot);
        let swatches = rects(&scene, Role::Legend);
        assert_eq!(swatches[0].x, 240.0 + LEGEND_PADDING);
    }

    #[test]
    fn a_floating_legend_without_overlay_pushes_the_plot_away() {
        let mut chart = column_chart();
        chart.legend.position = Position::Float(0.8, 0.4);
        chart.legend.vertical = true;
        let (scene, frame) = frame_of(&chart, &data());
        let swatches = rects(&scene, Role::Legend);
        // The legend is nearest the right edge: the plot ends left of it
        assert!((frame.plot.right() - (384.0 - GAP)).abs() < 1e-9);
        assert_eq!(swatches[0].x, 384.0 + LEGEND_PADDING);

        // Nearest the bottom: the plot, and the category labels under it,
        // end above the legend
        chart.legend.position = Position::Float(0.3, 0.9);
        let (_, frame) = frame_of(&chart, &data());
        assert!(frame.plot.bottom() < 0.9 * 288.0 - GAP);
        chart.legend.overlay = true;
        let (_, overlaid) = frame_of(&chart, &data());
        assert!(overlaid.plot.bottom() > 0.9 * 288.0);
    }

    #[test]
    fn axis_titles_take_their_side() {
        let mut chart = column_chart();
        chart.title = None;
        chart.value_axis.title = Some(title("People"));
        chart.category_axis.title = Some(title("City"));
        let (scene, frame) = frame_of(&chart, &data());
        assert!(texts(&scene, Role::Title).is_empty());
        let value_texts = texts(&scene, Role::ValueAxis);
        assert_eq!(value_texts[0].0, "People");
        assert!(value_texts[0].1 < frame.plot.x);
        let category_texts = texts(&scene, Role::CategoryAxis);
        assert_eq!(category_texts[0].0, "City");
        assert!(category_texts[0].2 > category_texts[1].2);
        assert!(category_texts[0].2 < 288.0 - PADDING);
        // No title: the plot starts right under the padding
        assert_eq!(frame.plot.y, PADDING);
    }

    #[test]
    fn a_subtitle_stacks_under_the_title_and_widens_its_box() {
        use ironcalc_base::charts::Subtitle;
        let mut chart = column_chart();
        let plain = frame_of(&chart, &data()).1;
        let mut titled = title("Population");
        titled.border = Line {
            style: LineStyle::Thin,
            color: Color::None,
        };
        titled.subtitle = Some(Subtitle {
            text: "by city, in thousands of people".to_string(),
            font: Font {
                size: 10,
                family: None,
                bold: false,
                italic: true,
                color: Color::None,
            },
        });
        chart.title = Some(titled.clone());
        chart.value_axis.title = Some(titled.clone());
        chart.category_axis.title = Some(titled);
        let (scene, frame) = frame_of(&chart, &data());

        let title_height = Inter.measure("", 14.0 * PIXELS_PER_POINT, false).height();
        let subtitle_height = Inter.measure("", 10.0 * PIXELS_PER_POINT, false).height();
        let texts_of = |role| texts(&scene, role);
        // Chart title: the subtitle's baseline is one title line lower
        let top = texts_of(Role::Title);
        assert_eq!(top.len(), 2);
        assert_eq!(top[1].0, "by city, in thousands of people");
        let ascent = |size: f64| Inter.measure("", size * PIXELS_PER_POINT, false).ascent;
        let expected = top[0].2 - ascent(14.0) + title_height + ascent(10.0);
        assert!((top[1].2 - expected).abs() < 1e-9);
        // The box is as wide as the longer line and as tall as both
        let boxed = rects(&scene, Role::Title)[0];
        let longest = Inter
            .measure(
                "by city, in thousands of people",
                10.0 * PIXELS_PER_POINT,
                false,
            )
            .width;
        assert!((boxed.width - longest - 2.0 * TITLE_BOX_PADDING_X).abs() < 1e-9);
        assert!(
            (boxed.height - title_height - subtitle_height - 2.0 * TITLE_BOX_PADDING_Y).abs()
                < 1e-9
        );
        // The plot moves down by the subtitle and the box's padding
        assert!(
            (frame.plot.y - plain.plot.y - subtitle_height - 2.0 * TITLE_BOX_PADDING_Y).abs()
                < 1e-9
        );
        // The category axis title is laid out from the bottom up, before
        // the tick labels: the subtitle is the lowest text of the axis
        let bottom = texts_of(Role::CategoryAxis);
        assert_eq!(bottom[0].0, "by city, in thousands of people");
        assert_eq!(bottom[1].0, "Population");
        assert!(bottom[0].2 > bottom[1].2);
        // The value axis subtitle reads up to the right of the title
        let turned: Vec<(&str, f64)> = scene
            .elements
            .iter()
            .filter(|e| e.role == Role::ValueAxis)
            .filter_map(|e| match &e.shape {
                Shape::Text {
                    x, text, rotation, ..
                } if *rotation == -90.0 => Some((text.as_str(), *x)),
                _ => None,
            })
            .collect();
        assert_eq!(turned.len(), 2);
        assert_eq!(turned[1].0, "by city, in thousands of people");
        assert!(turned[1].1 > turned[0].1);
    }

    #[test]
    fn a_title_with_a_fill_or_border_gets_a_box_that_takes_its_padding() {
        let mut chart = column_chart();
        let plain = frame_of(&chart, &data()).1;
        let mut styled = title("Population");
        styled.fill = Color::Rgb("#FFFF00".to_string());
        styled.border = Line {
            style: LineStyle::Thin,
            color: Color::Rgb("#FF0000".to_string()),
        };
        chart.title = Some(styled.clone());
        let (scene, frame) = frame_of(&chart, &data());
        let boxes = rects(&scene, Role::Title);
        assert_eq!(boxes.len(), 1);
        let text = texts(&scene, Role::Title)[0];
        // The box starts under the padding and surrounds the text
        assert!((boxes[0].y - PADDING).abs() < 1e-9);
        let ascent = Inter.measure("", 14.0 * PIXELS_PER_POINT, false).ascent;
        assert!((text.2 - (PADDING + TITLE_BOX_PADDING_Y + ascent)).abs() < 1e-9);
        assert!(boxes[0].x < text.1 && text.1 < boxes[0].right());
        assert!((boxes[0].x + boxes[0].width / 2.0 - text.1).abs() < 1e-9);
        // and the plot moves down by the vertical padding
        assert!((frame.plot.y - plain.plot.y - 2.0 * TITLE_BOX_PADDING_Y).abs() < 1e-9);
        let painted = scene
            .elements
            .iter()
            .find(|e| e.role == Role::Title)
            .unwrap();
        match &painted.shape {
            Shape::Rect { fill, stroke, .. } => {
                assert_eq!(fill.as_deref(), Some("#FFFF00"));
                assert_eq!(stroke.as_ref().unwrap().color, "#FF0000");
            }
            _ => panic!("the box is painted before the text"),
        }

        // A border alone is enough for a box; axis titles get theirs too
        chart.title = None;
        let mut bordered = title("People");
        bordered.border = Line {
            style: LineStyle::Dashed,
            color: Color::None,
        };
        chart.value_axis.title = Some(bordered.clone());
        chart.category_axis.title = Some(bordered);
        let (scene, _) = frame_of(&chart, &data());
        let value_box = rects(&scene, Role::ValueAxis)[0];
        let category_box = rects(&scene, Role::CategoryAxis)[0];
        // The value axis box stands on its side: taller than wide
        assert!(value_box.height > value_box.width);
        assert!((value_box.x - PADDING).abs() < 1e-9);
        assert!(category_box.width > category_box.height);
        // It sits around its text, above the legend
        let (_, x, baseline, _) = texts(&scene, Role::CategoryAxis)
            .into_iter()
            .find(|t| t.0 == "People")
            .unwrap();
        assert!(category_box.y < baseline && baseline < category_box.bottom());
        assert!((category_box.x + category_box.width / 2.0 - x).abs() < 1e-9);
        let legend_top = rects(&scene, Role::Legend)[0].y;
        assert!(category_box.bottom() < legend_top);
    }

    #[test]
    fn an_empty_title_takes_no_room() {
        let mut chart = column_chart();
        chart.title = Some(title(""));
        chart.value_axis.title = Some(title(""));
        let (scene, frame) = frame_of(&chart, &data());
        assert!(texts(&scene, Role::Title).is_empty());
        chart.title = None;
        chart.value_axis.title = None;
        let (_, plain) = frame_of(&chart, &data());
        assert_eq!(frame.plot, plain.plot);
    }

    #[test]
    fn categories_are_numbered_when_there_are_none() {
        let mut data = data();
        data.categories.clear();
        let (scene, frame) = frame_of(&column_chart(), &data);
        assert_eq!(frame.categories, 4);
        let labels: Vec<&str> = texts(&scene, Role::CategoryAxis)
            .iter()
            .map(|t| t.0)
            .collect();
        assert_eq!(labels, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn series_colors_cycle_through_the_accents() {
        let theme = Theme::default();
        let mut chart = column_chart();
        assert_eq!(series_color(&chart, 0, &theme), theme.accent1);
        assert_eq!(series_color(&chart, 5, &theme), theme.accent6);
        assert_ne!(series_color(&chart, 6, &theme), theme.accent1);
        chart.colors = vec![Color::Rgb("#FF0000".to_string())];
        assert_eq!(series_color(&chart, 0, &theme), "#FF0000");
        assert_eq!(series_color(&chart, 1, &theme), theme.accent2);
    }
}
