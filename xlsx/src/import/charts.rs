//! Reads a chart part (`xl/charts/chartN.xml`) into a [`Chart`].
//!
//! Only what the model holds is read. A chart part whose shape we do not
//! understand (several chart groups, a chart kind other than a column chart,
//! series given as literal lists) yields `None` and the drawing that holds it
//! becomes `DrawingContent::Unsupported`.

use ironcalc_base::charts::{
    Alignment, Bars, CategoryAxis, Chart, ColumnChart, DataLabelPosition, DataLabels, DataRange,
    DataSource, EmptyCells, Fill, Font, Gridlines, Legend, Line, LineStyle, Position, Series,
    Stacking, Subtitle, TextDirection, TickMark, Title, ValueAxis, ValueFormat,
};
use ironcalc_base::types::Color;

use super::xml::XmlNode;

const DEFAULT_TITLE_FONT_SIZE: u32 = 14;
const DEFAULT_LABEL_FONT_SIZE: u32 = 9;
/// Excel's sentinel for "rotate the text as needed".
const AUTOMATIC_ROTATION: i32 = -60_000_000;

/// Thickest line, in EMU, that we still call thin (3/4 pt).
const THIN_LINE_MAX_WIDTH: i64 = 12700;

/// The index of a DrawingML scheme color in the SpreadsheetML theme palette,
/// the one `Color::Theme` uses. `bg1`/`tx1` are aliases of `lt1`/`dk1`.
fn scheme_color_index(name: &str) -> Option<i32> {
    Some(match name {
        "lt1" | "bg1" => 0,
        "dk1" | "tx1" => 1,
        "lt2" | "bg2" => 2,
        "dk2" | "tx2" => 3,
        "accent1" => 4,
        "accent2" => 5,
        "accent3" => 6,
        "accent4" => 7,
        "accent5" => 8,
        "accent6" => 9,
        "hlink" => 10,
        "folHlink" => 11,
        _ => return None,
    })
}

fn val_attribute(node: &XmlNode) -> Option<&str> {
    node.attribute("val")
}

fn child_val(node: &XmlNode, name: &str) -> Option<String> {
    node.child(name).and_then(val_attribute).map(str::to_string)
}

/// Reads a DrawingML color (`a:srgbClr`, `a:schemeClr`, `a:sysClr`).
/// The luminance modifiers Excel writes on scheme colors become a tint:
/// `lumMod`/`lumOff` lighten, a lone `lumMod` darkens.
fn parse_color_node(node: &XmlNode) -> Color {
    match node.tag_name() {
        "srgbClr" => match val_attribute(node) {
            Some(hex) => Color::Rgb(format!("#{}", hex.to_ascii_uppercase())),
            None => Color::None,
        },
        "sysClr" => match node.attribute("lastClr") {
            Some(hex) => Color::Rgb(format!("#{}", hex.to_ascii_uppercase())),
            None => Color::None,
        },
        "schemeClr" => {
            let Some(index) = val_attribute(node).and_then(scheme_color_index) else {
                return Color::None;
            };
            let lum_mod = child_val(node, "lumMod").and_then(|v| v.parse::<f64>().ok());
            let lum_off = child_val(node, "lumOff").and_then(|v| v.parse::<f64>().ok());
            let tint = match (lum_mod, lum_off) {
                (_, Some(off)) => off / 100_000.0,
                (Some(m), None) => m / 100_000.0 - 1.0,
                (None, None) => 0.0,
            };
            Color::Theme(index, tint)
        }
        _ => Color::None,
    }
}

/// The color of a `solidFill` directly under `node`, or `Color::None` if there
/// is no solid fill (`noFill`, a gradient, a pattern or nothing at all).
fn parse_solid_fill(node: &XmlNode) -> Color {
    node.child("solidFill")
        .and_then(|fill| fill.children().next())
        .map(parse_solid_color_or_none)
        .unwrap_or(Color::None)
}

fn parse_solid_color_or_none(node: &XmlNode) -> Color {
    parse_color_node(node)
}

fn no_line() -> Line {
    Line {
        style: LineStyle::None,
        color: Color::None,
    }
}

fn automatic_line() -> Line {
    Line {
        style: LineStyle::Automatic,
        color: Color::None,
    }
}

/// The outline of a shape: the `a:ln` child of a `spPr`. A missing `spPr`,
/// a missing `a:ln` or a line without a fill of its own is automatic, an
/// explicit `noFill` is no line at all.
fn parse_line(sp_pr: Option<&XmlNode>) -> Line {
    let Some(ln) = sp_pr.and_then(|n| n.child("ln")) else {
        return automatic_line();
    };
    if ln.child("noFill").is_some() {
        return no_line();
    }
    if ln.child("solidFill").is_none() {
        return automatic_line();
    }
    let color = parse_solid_fill(ln);
    let dashed = ln
        .child("prstDash")
        .and_then(val_attribute)
        .is_some_and(|dash| dash != "solid");
    let width = ln
        .attribute("w")
        .and_then(|w| w.parse::<i64>().ok())
        .unwrap_or(0);
    let style = if dashed {
        LineStyle::Dashed
    } else if width > THIN_LINE_MAX_WIDTH {
        LineStyle::Thick
    } else {
        LineStyle::Thin
    };
    Line { style, color }
}

/// The fill of a shape. A missing `spPr` or a fill we cannot represent (a
/// gradient, a pattern, a picture) is automatic.
fn parse_fill(sp_pr: Option<&XmlNode>) -> Fill {
    let Some(sp_pr) = sp_pr else {
        return Fill::Automatic;
    };
    if sp_pr.child("noFill").is_some() {
        return Fill::None;
    }
    match parse_solid_fill(sp_pr) {
        Color::None => Fill::Automatic,
        color => Fill::Solid(color),
    }
}

/// The default run properties of the first paragraph under `node`, which is
/// a `c:txPr` or an `a:rich` element.
fn default_run_properties(node: &XmlNode) -> Option<&XmlNode> {
    node.child("p")?.child("pPr")?.child("defRPr")
}

/// The font described by an `a:defRPr`, with `default_size` (points) when the
/// element is missing or has no size.
fn parse_font(def_rpr: Option<&XmlNode>, default_size: u32) -> Font {
    let Some(def_rpr) = def_rpr else {
        return Font {
            size: default_size,
            family: None,
            bold: false,
            italic: false,
            color: Color::None,
        };
    };
    // sz is in hundredths of a point
    let size = def_rpr
        .attribute("sz")
        .and_then(|sz| sz.parse::<u32>().ok())
        .map(|sz| sz / 100)
        .unwrap_or(default_size);
    let flag = |name: &str| {
        def_rpr
            .attribute(name)
            .is_some_and(|v| v == "1" || v == "true")
    };
    // Theme fonts (`+mn-lt`, `+mj-lt`) are inherited: the chart decides.
    let family = def_rpr
        .child("latin")
        .and_then(|latin| latin.attribute("typeface"))
        .filter(|typeface| !typeface.is_empty() && !typeface.starts_with('+'))
        .map(str::to_string);
    Font {
        size,
        family,
        bold: flag("b"),
        italic: flag("i"),
        color: parse_solid_fill(def_rpr),
    }
}

/// The lines of a rich text: its paragraphs, split again at the `a:br`
/// breaks inside them, each with the properties of its first run.
fn rich_lines(rich: &XmlNode) -> Vec<(String, Option<&XmlNode>)> {
    let mut lines = Vec::new();
    for paragraph in rich.children().filter(|n| n.has_tag_name("p")) {
        let mut text = String::new();
        let mut first_run: Option<&XmlNode> = None;
        let mut fresh = true;
        for node in paragraph.children() {
            if node.has_tag_name("r") {
                if fresh {
                    first_run = node.child("rPr");
                    fresh = false;
                }
                if let Some(t) = node.child("t").and_then(|t| t.text()) {
                    text.push_str(t);
                }
            } else if node.has_tag_name("br") {
                lines.push((std::mem::take(&mut text), first_run.take()));
                fresh = true;
            }
        }
        lines.push((text, first_run));
    }
    lines
}

/// `font` with what an `a:rPr` overrides: the run's own size, style, color
/// and family, when it has them.
fn run_font(rpr: Option<&XmlNode>, font: Font) -> Font {
    let Some(rpr) = rpr else {
        return font;
    };
    let run = parse_font(Some(rpr), font.size);
    Font {
        size: run.size,
        family: run.family.or(font.family),
        bold: if rpr.attribute("b").is_some() {
            run.bold
        } else {
            font.bold
        },
        italic: if rpr.attribute("i").is_some() {
            run.italic
        } else {
            font.italic
        },
        color: match run.color {
            Color::None => font.color,
            color => color,
        },
    }
}

/// The text Excel shows for a chart title that has none of its own: the
/// name of the series when there is one, or "Chart Title".
fn automatic_chart_title(bar_chart: &XmlNode) -> String {
    let mut series = bar_chart.children().filter(|n| n.has_tag_name("ser"));
    let (Some(only), None) = (series.next(), series.next()) else {
        return "Chart Title".to_string();
    };
    // The cached text of the name, or the literal
    only.child("tx")
        .and_then(|tx| tx.descendants().find(|n| n.has_tag_name("v")))
        .and_then(|v| v.text())
        .filter(|text| !text.is_empty())
        .map_or_else(|| "Chart Title".to_string(), str::to_string)
}

/// Left of this fraction of the chart width a moved title is flush left.
const LEFT_TITLE_LIMIT: f64 = 0.15;

/// The alignment of a title from where Excel put it. Excel keeps no
/// alignment, only the left edge of a moved title as a fraction of the chart
/// width (`edge` mode) or as an offset from its default, centered place
/// (`factor` mode). A title at the left edge sits at 0; one at the right
/// edge has its left edge past the middle unless it is wider than half the
/// chart. A title that was not moved, or only nudged, is centered.
fn title_alignment(title: &XmlNode) -> Alignment {
    let Some(layout) = title
        .child("layout")
        .and_then(|layout| layout.child("manualLayout"))
    else {
        return Alignment::Middle;
    };
    let Some(x) = child_val(layout, "x").and_then(|v| v.parse::<f64>().ok()) else {
        return Alignment::Middle;
    };
    let factor = child_val(layout, "xMode").is_some_and(|mode| mode == "factor");
    if factor {
        if x < -LEFT_TITLE_LIMIT {
            Alignment::Left
        } else if x > LEFT_TITLE_LIMIT {
            Alignment::Right
        } else {
            Alignment::Middle
        }
    } else if x <= LEFT_TITLE_LIMIT {
        Alignment::Left
    } else if x >= 0.5 {
        Alignment::Right
    } else {
        Alignment::Middle
    }
}

/// A `c:title` element, of the chart or of an axis. A title without `tx` is
/// Excel's automatic title; it gets the text Excel would show, `automatic`.
fn parse_title(title: &XmlNode, automatic: &str) -> Title {
    let rich = title.child("tx").and_then(|tx| tx.child("rich"));
    let def_rpr = rich
        .and_then(default_run_properties)
        .or_else(|| title.child("txPr").and_then(default_run_properties));
    let base = parse_font(def_rpr, DEFAULT_TITLE_FONT_SIZE);
    // The first line is the title, the lines under it the subtitle; each
    // takes the font of its first run over the paragraph's default.
    let (text, font, subtitle) = match title.child("tx") {
        Some(tx) => match tx.child("rich") {
            Some(rich) => {
                let mut lines = rich_lines(rich).into_iter();
                let (text, rpr) = lines.next().unwrap_or_default();
                let font = run_font(rpr, base.clone());
                let rest: Vec<(String, Option<&XmlNode>)> =
                    lines.filter(|(text, _)| !text.trim().is_empty()).collect();
                let subtitle = rest.first().map(|(_, rpr)| Subtitle {
                    text: rest
                        .iter()
                        .map(|(text, _)| text.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                    font: run_font(*rpr, base.clone()),
                });
                (text, font, subtitle)
            }
            // A title linked to a cell: keep what Excel cached for it.
            None => (
                tx.descendants()
                    .find(|n| n.has_tag_name("v"))
                    .and_then(|v| v.text())
                    .unwrap_or("")
                    .to_string(),
                base,
                None,
            ),
        },
        None => (automatic.to_string(), base, None),
    };
    let sp_pr = title.child("spPr");
    Title {
        text,
        font,
        alignment: title_alignment(title),
        fill: sp_pr.map(parse_solid_fill).unwrap_or(Color::None),
        border: parse_line(sp_pr),
        subtitle,
    }
}

fn parse_legend(legend: Option<&XmlNode>) -> Legend {
    let Some(legend) = legend else {
        return Legend {
            position: Position::None,
            vertical: false,
            padding: None,
            fill: Color::None,
            border: no_line(),
            font: parse_font(None, DEFAULT_LABEL_FONT_SIZE),
            overlay: false,
        };
    };
    let legend_pos = child_val(legend, "legendPos").unwrap_or_else(|| "r".to_string());
    let vertical = matches!(legend_pos.as_str(), "l" | "r" | "tr");
    let manual_layout = legend
        .child("layout")
        .and_then(|layout| layout.child("manualLayout"));
    let position = match manual_layout {
        Some(layout) => {
            let x = child_val(layout, "x")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let y = child_val(layout, "y")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            Position::Float(x, y)
        }
        None => match legend_pos.as_str() {
            "t" => Position::Top,
            "b" => Position::Bottom,
            "l" => Position::Left,
            _ => Position::Right,
        },
    };
    let sp_pr = legend.child("spPr");
    let def_rpr = legend.child("txPr").and_then(default_run_properties);
    Legend {
        position,
        vertical,
        padding: None,
        fill: sp_pr.map(parse_solid_fill).unwrap_or(Color::None),
        border: parse_line(sp_pr),
        font: parse_font(def_rpr, DEFAULT_LABEL_FONT_SIZE),
        overlay: child_val(legend, "overlay").is_some_and(|v| v == "1" || v == "true"),
    }
}

fn axis_labels_font(axis: Option<&XmlNode>) -> Font {
    let def_rpr = axis
        .and_then(|axis| axis.child("txPr"))
        .and_then(default_run_properties);
    parse_font(def_rpr, DEFAULT_LABEL_FONT_SIZE)
}

/// The angle of the axis labels in Excel's degrees. OOXML stores 60000ths of
/// a degree clockwise; -60000000 and a missing value are Excel's automatic
/// choice.
fn axis_labels_rotation(axis: Option<&XmlNode>) -> Option<i32> {
    let rot = axis
        .and_then(|axis| axis.child("txPr"))
        .and_then(|tx_pr| tx_pr.child("bodyPr"))
        .and_then(|body| body.attribute("rot"))
        .and_then(|rot| rot.parse::<i32>().ok())?;
    if rot == AUTOMATIC_ROTATION {
        return None;
    }
    Some((-rot / 60000).clamp(-90, 90))
}

fn axis_title(axis: Option<&XmlNode>) -> Option<Title> {
    axis.and_then(|axis| axis.child("title"))
        .map(|title| parse_title(title, "Axis Title"))
}

/// The gridlines and tick marks of an axis. A missing gridlines element is
/// no gridlines, one without a style is automatic; a missing tick mark
/// element is none, as Excel writes for its new charts.
fn parse_gridlines(axis: Option<&XmlNode>) -> Gridlines {
    let line = |name: &str| match axis.and_then(|axis| axis.child(name)) {
        Some(gridlines) => parse_line(gridlines.child("spPr")),
        None => no_line(),
    };
    let tick = |name: &str| match axis.and_then(|axis| child_val(axis, name)).as_deref() {
        Some("in") => TickMark::Inside,
        Some("out") => TickMark::Outside,
        Some("cross") => TickMark::Cross,
        _ => TickMark::None,
    };
    Gridlines {
        major: line("majorGridlines"),
        minor: line("minorGridlines"),
        major_ticks: tick("majorTickMark"),
        minor_ticks: tick("minorTickMark"),
    }
}

fn parse_category_axis(axis: Option<&XmlNode>) -> CategoryAxis {
    CategoryAxis {
        title: axis_title(axis),
        labels: axis_labels_font(axis),
        rotation: axis_labels_rotation(axis),
        line: parse_line(axis.and_then(|axis| axis.child("spPr"))),
        gridlines: parse_gridlines(axis),
    }
}

fn parse_value_axis(axis: Option<&XmlNode>) -> ValueAxis {
    // A format linked to the source cells is the default and is stored as None.
    let number_format = axis
        .and_then(|axis| axis.child("numFmt"))
        .filter(|fmt| fmt.attribute("sourceLinked") != Some("1"))
        .and_then(|fmt| fmt.attribute("formatCode"))
        .map(str::to_string);
    ValueAxis {
        title: axis_title(axis),
        labels: ValueFormat {
            number_format,
            font: axis_labels_font(axis),
        },
        line: parse_line(axis.and_then(|axis| axis.child("spPr"))),
        gridlines: parse_gridlines(axis),
    }
}

/// The formula of a `c:strRef`, `c:numRef` or `c:multiLvlStrRef` under
/// `node`, or `None` if the data is a literal list.
fn reference_formula(node: &XmlNode) -> Option<String> {
    node.children()
        .find(|n| matches!(n.tag_name(), "strRef" | "numRef" | "multiLvlStrRef"))
        .and_then(|reference| reference.child("f"))
        .and_then(|f| f.text())
        .map(str::to_string)
}

/// The name of a series: a reference as is, a literal quoted as a formula
/// string, or empty when Excel names it automatically.
fn series_name(ser: &XmlNode) -> String {
    let Some(tx) = ser.child("tx") else {
        return "".to_string();
    };
    if let Some(reference) = reference_formula(tx) {
        return reference;
    }
    match tx.child("v").and_then(|v| v.text()) {
        Some(literal) => format!("\"{}\"", literal.replace('"', "\"\"")),
        None => "".to_string(),
    }
}

/// The data of a bar chart group: the shared categories and one series per
/// `c:ser`, plus the fill color of each series. `None` if a series has no
/// reference for its categories or values.
/// The data labels described by a `c:dLbls`, `None` when they do not show
/// the value. A missing `dLblPos` is Excel's default: the center of a
/// stacked column, the top of a clustered one.
fn parse_data_labels(labels: Option<&XmlNode>, stacking: &Stacking) -> Option<DataLabels> {
    let labels = labels?;
    if !child_val(labels, "showVal").is_some_and(|v| v == "1" || v == "true") {
        return None;
    }
    let position = match child_val(labels, "dLblPos").as_deref() {
        Some("ctr") => DataLabelPosition::Center,
        Some("inEnd") => DataLabelPosition::InsideEnd,
        Some("inBase") => DataLabelPosition::InsideBase,
        Some("outEnd") => DataLabelPosition::OutsideEnd,
        _ if *stacking == Stacking::None => DataLabelPosition::OutsideEnd,
        _ => DataLabelPosition::Center,
    };
    let number_format = labels.child("numFmt").and_then(|fmt| {
        let linked = fmt
            .attribute("sourceLinked")
            .is_some_and(|v| v == "1" || v == "true");
        if linked {
            None
        } else {
            fmt.attribute("formatCode").map(str::to_string)
        }
    });
    let sp_pr = labels.child("spPr");
    // The text direction: a quarter turn either way, in 60000ths of a degree
    // clockwise, like the category labels
    let direction = match labels
        .child("txPr")
        .and_then(|txpr| txpr.child("bodyPr"))
        .and_then(|body| body.attribute("rot"))
        .and_then(|rot| rot.parse::<i64>().ok())
    {
        Some(5_400_000) => TextDirection::Rotate90,
        Some(-5_400_000) => TextDirection::Rotate270,
        _ => TextDirection::Horizontal,
    };
    Some(DataLabels {
        position,
        direction,
        number_format,
        font: parse_font(
            labels.child("txPr").and_then(default_run_properties),
            DEFAULT_LABEL_FONT_SIZE,
        ),
        fill: sp_pr.map(parse_solid_fill).unwrap_or(Color::None),
        border: parse_line(sp_pr),
    })
}

/// The series of a bar chart group: the shared categories, one series per
/// `c:ser`, the fill color of each and its data labels. Labels set on the
/// group apply to the series that have none of their own.
struct SeriesGroup {
    categories: String,
    series: Vec<Series>,
    colors: Vec<Color>,
    data_labels: Vec<Option<DataLabels>>,
}

fn parse_series(bar_chart: &XmlNode) -> Option<SeriesGroup> {
    let stacking = parse_stacking(bar_chart);
    let group_labels = parse_data_labels(bar_chart.child("dLbls"), &stacking);
    let mut categories = None;
    let mut series = Vec::new();
    let mut colors = Vec::new();
    let mut data_labels = Vec::new();
    for ser in bar_chart.children().filter(|n| n.has_tag_name("ser")) {
        let values = reference_formula(ser.child("val")?)?;
        let cat = reference_formula(ser.child("cat")?)?;
        if categories.is_none() {
            categories = Some(cat);
        }
        series.push(Series {
            name: series_name(ser),
            values,
        });
        colors.push(
            ser.child("spPr")
                .map(parse_solid_fill)
                .unwrap_or(Color::None),
        );
        data_labels.push(match ser.child("dLbls") {
            Some(labels) => parse_data_labels(Some(labels), &stacking),
            None => group_labels.clone(),
        });
    }
    Some(SeriesGroup {
        categories: categories?,
        series,
        colors,
        data_labels,
    })
}

/// The spacing of the columns. The schema's defaults when an element is
/// missing: a gap of 150% and no overlap. Stacked columns always overlap
/// fully.
fn parse_bars(bar_chart: &XmlNode, stacking: &Stacking) -> Bars {
    let number = |name: &str, default: i64| {
        child_val(bar_chart, name)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(default)
    };
    Bars {
        gap: number("gapWidth", 150).clamp(0, 500) as u32,
        overlap: if *stacking == Stacking::None {
            number("overlap", 0).clamp(-100, 100) as i32
        } else {
            100
        },
    }
}

fn parse_stacking(bar_chart: &XmlNode) -> Stacking {
    match child_val(bar_chart, "grouping").as_deref() {
        Some("stacked") => Stacking::Standard,
        Some("percentStacked") => Stacking::Full,
        _ => Stacking::None,
    }
}

/// A chart part: the chart and the fill and border of its area, which the
/// model keeps on the drawing.
pub(super) struct ChartPart {
    pub chart: Chart,
    pub fill: Fill,
    pub border: Line,
}

/// Reads a `c:chartSpace` document. `None` if it is not a chart we support.
pub(super) fn load_chart(chart_space: &XmlNode) -> Option<ChartPart> {
    let chart = chart_space.child("chart")?;
    let plot_area = chart.child("plotArea")?;
    let mut groups = plot_area
        .children()
        .filter(|n| n.tag_name().ends_with("Chart"));
    let bar_chart = groups.next()?;
    if groups.next().is_some() {
        // A combo chart
        return None;
    }
    if !bar_chart.has_tag_name("barChart")
        || child_val(bar_chart, "barDir").as_deref() != Some("col")
    {
        return None;
    }
    let SeriesGroup {
        categories,
        series,
        colors,
        data_labels,
    } = parse_series(bar_chart)?;

    let title = chart
        .child("title")
        .map(|title| parse_title(title, &automatic_chart_title(bar_chart)));
    let legend = parse_legend(chart.child("legend"));
    let category_axis = parse_category_axis(plot_area.child("catAx"));
    let value_axis = parse_value_axis(plot_area.child("valAx"));

    // The schema defaults: blanks plotted as zero, hidden cells not plotted.
    let empty_cells = match child_val(chart, "dispBlanksAs").as_deref() {
        Some("gap") | Some("span") => EmptyCells::Gaps,
        Some("zero") => EmptyCells::Zero,
        _ => EmptyCells::Zero,
    };
    let show_hidden_data =
        child_val(chart, "plotVisOnly").is_some_and(|v| v == "0" || v == "false");
    let na_as_empty = chart
        .descendants()
        .find(|n| n.has_tag_name("dispNaAsBlank"))
        .and_then(val_attribute)
        .is_some_and(|v| v == "1" || v == "true");

    let area = chart_space.child("spPr");
    let stacking = parse_stacking(bar_chart);
    let bars = parse_bars(bar_chart, &stacking);
    Some(ChartPart {
        chart: Chart::ColumnChart(ColumnChart {
            title,
            legend,
            stacking,
            bars,
            category_axis,
            value_axis,
            data: DataSource {
                range: DataRange::Manual { categories, series },
                empty_cells,
                na_as_empty,
                show_hidden_data,
            },
            colors,
            data_labels,
        }),
        fill: parse_fill(area),
        border: parse_line(area),
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn chart_space(inner: &str) -> String {
        format!(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{inner}</c:chartSpace>"#
        )
    }

    const SERIES: &str = r#"<c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:strRef><c:f>Sheet1!$E$4</c:f></c:strRef></c:tx><c:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></c:spPr><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$E$5:$E$9</c:f></c:numRef></c:val></c:ser>"#;

    fn column_chart(part: ChartPart) -> ColumnChart {
        match part.chart {
            Chart::ColumnChart(c) => c,
        }
    }

    #[test]
    fn reads_a_clustered_column_chart() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:title><c:tx><c:rich><a:p><a:pPr><a:defRPr sz="1400" b="1"><a:solidFill><a:schemeClr val="tx1"><a:lumMod val="65000"/><a:lumOff val="35000"/></a:schemeClr></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:pPr><a:r><a:t>Column</a:t></a:r><a:r><a:t> Chart</a:t></a:r></a:p></c:rich></c:tx></c:title><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="clustered"/>{SERIES}<c:ser><c:idx val="1"/><c:tx><c:v>Literal "name"</c:v></c:tx><c:spPr><a:solidFill><a:srgbClr val="ff0000"/></a:solidFill></c:spPr><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$F$5:$F$9</c:f></c:numRef></c:val></c:ser></c:barChart><c:catAx><c:txPr><a:p><a:pPr><a:defRPr sz="900"/></a:pPr></a:p></c:txPr></c:catAx><c:valAx><c:numFmt formatCode="0.00" sourceLinked="0"/></c:valAx></c:plotArea><c:legend><c:legendPos val="b"/><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr></c:legend><c:plotVisOnly val="1"/><c:dispBlanksAs val="gap"/></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());

        let title = chart.title.unwrap();
        assert_eq!(title.text, "Column Chart");
        assert_eq!(title.font.size, 14);
        assert!(title.font.bold);
        assert_eq!(title.font.family, None);
        assert_eq!(title.font.color, Color::Theme(1, 0.35));

        assert_eq!(chart.stacking, Stacking::None);
        assert_eq!(chart.legend.position, Position::Bottom);
        assert!(!chart.legend.vertical);
        assert!(!chart.legend.overlay);
        assert_eq!(chart.category_axis.labels.size, 9);
        assert_eq!(chart.category_axis.rotation, None);
        assert_eq!(chart.category_axis.line, automatic_line());
        assert_eq!(chart.value_axis.line, automatic_line());
        assert_eq!(chart.value_axis.gridlines.major, no_line());
        assert_eq!(chart.value_axis.gridlines.minor, no_line());
        assert_eq!(chart.category_axis.gridlines.major, no_line());
        assert_eq!(chart.legend.font.size, 9);
        assert_eq!(
            chart.value_axis.labels.number_format,
            Some("0.00".to_string())
        );
        assert_eq!(
            chart.colors,
            vec![Color::Theme(4, 0.0), Color::Rgb("#FF0000".to_string())]
        );
        match chart.data.range {
            DataRange::Manual { categories, series } => {
                assert_eq!(categories, "Sheet1!$D$5:$D$9");
                assert_eq!(series.len(), 2);
                assert_eq!(series[0].name, "Sheet1!$E$4");
                assert_eq!(series[0].values, "Sheet1!$E$5:$E$9");
                assert_eq!(series[1].name, "\"Literal \"\"name\"\"\"");
            }
            DataRange::Range { .. } => panic!("expected a manual range"),
        }
        assert_eq!(chart.data.empty_cells, EmptyCells::Gaps);
        assert!(!chart.data.na_as_empty);
        assert!(!chart.data.show_hidden_data);
    }

    #[test]
    fn reads_the_hidden_and_empty_cell_settings_and_a_floating_legend() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:title><c:overlay val="0"/></c:title><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="percentStacked"/>{SERIES}</c:barChart></c:plotArea><c:legend><c:legendPos val="b"/><c:layout><c:manualLayout><c:x val="0.25"/><c:y val="0.5"/></c:manualLayout></c:layout><c:overlay val="1"/><c:spPr><a:solidFill><a:schemeClr val="accent6"><a:lumMod val="20000"/><a:lumOff val="80000"/></a:schemeClr></a:solidFill><a:ln w="19050"><a:solidFill><a:srgbClr val="000000"/></a:solidFill></a:ln></c:spPr></c:legend><c:plotVisOnly val="0"/><c:dispBlanksAs val="zero"/><c:extLst><c:ext uri="x" xmlns:c16r3="http://schemas.microsoft.com/office/drawing/2017/03/chart"><c16r3:dataDisplayOptions16><c16r3:dispNaAsBlank val="1"/></c16r3:dataDisplayOptions16></c:ext></c:extLst></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());

        // A title element without text is the automatic title: with a single
        // series whose name is not cached, Excel's placeholder
        assert_eq!(chart.title.unwrap().text, "Chart Title");
        assert_eq!(chart.stacking, Stacking::Full);
        assert_eq!(chart.legend.position, Position::Float(0.25, 0.5));
        assert!(chart.legend.overlay);
        assert_eq!(chart.legend.fill, Color::Theme(9, 0.8));
        assert_eq!(chart.legend.border.style, LineStyle::Thick);
        assert_eq!(chart.legend.border.color, Color::Rgb("#000000".to_string()));
        assert_eq!(chart.data.empty_cells, EmptyCells::Zero);
        assert!(chart.data.na_as_empty);
        assert!(chart.data.show_hidden_data);
        assert_eq!(chart.value_axis.labels.number_format, None);
    }

    #[test]
    fn reads_axis_lines_gridlines_and_the_legend_font() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:spPr><a:noFill/><a:ln w="9525"><a:solidFill><a:schemeClr val="tx1"><a:lumMod val="15000"/><a:lumOff val="85000"/></a:schemeClr></a:solidFill></a:ln></c:spPr></c:catAx><c:valAx><c:majorGridlines><c:spPr><a:ln w="9525"><a:solidFill><a:schemeClr val="accent4"><a:lumMod val="60000"/><a:lumOff val="40000"/></a:schemeClr></a:solidFill><a:prstDash val="sysDash"/></a:ln></c:spPr></c:majorGridlines><c:spPr><a:noFill/><a:ln><a:solidFill><a:srgbClr val="C00000"/></a:solidFill></a:ln></c:spPr></c:valAx></c:plotArea><c:legend><c:legendPos val="b"/><c:txPr><a:p><a:pPr><a:defRPr sz="2000"><a:solidFill><a:schemeClr val="accent2"><a:lumMod val="50000"/></a:schemeClr></a:solidFill></a:defRPr></a:pPr></a:p></c:txPr></c:legend></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());

        assert_eq!(
            chart.category_axis.line,
            Line {
                style: LineStyle::Thin,
                color: Color::Theme(1, 0.85)
            }
        );
        assert_eq!(
            chart.value_axis.line,
            Line {
                style: LineStyle::Thin,
                color: Color::Rgb("#C00000".to_string())
            }
        );
        assert_eq!(
            chart.value_axis.gridlines.major,
            Line {
                style: LineStyle::Dashed,
                color: Color::Theme(7, 0.4)
            }
        );
        assert_eq!(chart.value_axis.gridlines.minor, no_line());
        assert_eq!(chart.legend.font.size, 20);
        assert_eq!(chart.legend.font.color, Color::Theme(5, -0.5));
    }

    #[test]
    fn a_gridlines_element_without_style_is_automatic() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:valAx><c:majorGridlines/></c:valAx></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        assert_eq!(chart.value_axis.gridlines.major, automatic_line());
        assert_eq!(chart.value_axis.gridlines.minor, no_line());
    }

    #[test]
    fn reads_major_and_minor_gridlines_of_both_axes() {
        // Chart 16 of AssortedColumns.xlsx: everything in Excel's default style
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:majorGridlines><c:spPr><a:ln w="9525" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="tx1"><a:lumMod val="15000"/><a:lumOff val="85000"/></a:schemeClr></a:solidFill><a:round/></a:ln></c:spPr></c:majorGridlines><c:minorGridlines><c:spPr><a:ln w="9525" cap="flat" cmpd="sng" algn="ctr"><a:solidFill><a:schemeClr val="tx1"><a:lumMod val="5000"/><a:lumOff val="95000"/></a:schemeClr></a:solidFill><a:round/></a:ln></c:spPr></c:minorGridlines></c:catAx><c:valAx><c:majorGridlines><c:spPr><a:ln w="9525"><a:solidFill><a:schemeClr val="accent2"/></a:solidFill></a:ln></c:spPr></c:majorGridlines><c:minorGridlines/></c:valAx></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        let thin = |color: Color| Line {
            style: LineStyle::Thin,
            color,
        };
        assert_eq!(
            chart.category_axis.gridlines,
            Gridlines {
                major: thin(Color::Theme(1, 0.85)),
                minor: thin(Color::Theme(1, 0.95)),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            }
        );
        assert_eq!(
            chart.value_axis.gridlines,
            Gridlines {
                major: thin(Color::Theme(5, 0.0)),
                minor: automatic_line(),
                major_ticks: TickMark::None,
                minor_ticks: TickMark::None,
            }
        );
    }

    #[test]
    fn reads_the_tick_marks_of_both_axes() {
        // Chart 19 of AssortedColumns.xlsx
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:majorTickMark val="in"/><c:minorTickMark val="cross"/></c:catAx><c:valAx><c:majorTickMark val="cross"/><c:minorTickMark val="out"/></c:valAx></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        assert_eq!(chart.category_axis.gridlines.major_ticks, TickMark::Inside);
        assert_eq!(chart.category_axis.gridlines.minor_ticks, TickMark::Cross);
        assert_eq!(chart.value_axis.gridlines.major_ticks, TickMark::Cross);
        assert_eq!(chart.value_axis.gridlines.minor_ticks, TickMark::Outside);
        // Missing elements, or "none"
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:majorTickMark val="none"/></c:catAx></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        assert_eq!(chart.category_axis.gridlines.major_ticks, TickMark::None);
        assert_eq!(chart.category_axis.gridlines.minor_ticks, TickMark::None);
        assert_eq!(chart.value_axis.gridlines.major_ticks, TickMark::None);
    }

    #[test]
    fn reads_the_angle_of_the_category_labels() {
        let with_rotation = |rot: &str| {
            let chart = format!(
                r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:txPr><a:bodyPr rot="{rot}" vert="horz"/><a:p><a:pPr><a:defRPr sz="900"/></a:pPr></a:p></c:txPr></c:catAx></c:plotArea></c:chart>"#
            );
            let doc = XmlNode::parse_str(&chart_space(&chart)).unwrap();
            column_chart(load_chart(&doc).unwrap())
                .category_axis
                .rotation
        };
        // Excel's custom angle of 45 reads bottom-left to top-right
        assert_eq!(with_rotation("-2700000"), Some(45));
        // and -60 top-left to bottom-right
        assert_eq!(with_rotation("3600000"), Some(-60));
        // Excel's sentinel for the automatic angle
        assert_eq!(with_rotation("-60000000"), None);
        assert_eq!(with_rotation("0"), Some(0));
    }

    #[test]
    fn reads_the_chart_area_fill_and_border() {
        let chart = r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart></c:plotArea></c:chart>"#.replace("{SERIES}", SERIES);
        // No shape properties at all: automatic
        let doc = XmlNode::parse_str(&chart_space(&chart)).unwrap();
        let part = load_chart(&doc).unwrap();
        assert_eq!(part.fill, Fill::Automatic);
        assert_eq!(part.border, automatic_line());

        // Chart 9: a light orange fill and a red dashed border
        let doc = XmlNode::parse_str(&chart_space(&format!(
            r#"{chart}<c:spPr><a:solidFill><a:schemeClr val="accent2"><a:lumMod val="40000"/><a:lumOff val="60000"/></a:schemeClr></a:solidFill><a:ln w="9525"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:prstDash val="lgDash"/></a:ln></c:spPr>"#
        )))
        .unwrap();
        let part = load_chart(&doc).unwrap();
        assert_eq!(part.fill, Fill::Solid(Color::Theme(5, 0.6)));
        assert_eq!(
            part.border,
            Line {
                style: LineStyle::Dashed,
                color: Color::Rgb("#FF0000".to_string())
            }
        );

        // Chart 10: no fill, the border left alone
        let doc = XmlNode::parse_str(&chart_space(&format!(
            r#"{chart}<c:spPr><a:noFill/><a:ln w="9525"/></c:spPr>"#
        )))
        .unwrap();
        let part = load_chart(&doc).unwrap();
        assert_eq!(part.fill, Fill::None);
        assert_eq!(part.border, automatic_line());
    }

    #[test]
    fn no_title_element_is_none() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:autoTitleDeleted val="1"/><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        assert!(chart.title.is_none());
        assert_eq!(chart.legend.position, Position::None);
    }

    #[test]
    fn an_automatic_title_gets_the_text_excel_shows() {
        let named = r#"<c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:strRef><c:f>Sheet1!$E$4</c:f><c:strCache><c:ptCount val="1"/><c:pt idx="0"><c:v>September</c:v></c:pt></c:strCache></c:strRef></c:tx><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$E$5:$E$9</c:f></c:numRef></c:val></c:ser>"#;
        let literal = r#"<c:ser><c:idx val="1"/><c:order val="1"/><c:tx><c:v>Literal</c:v></c:tx><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$F$5:$F$9</c:f></c:numRef></c:val></c:ser>"#;
        let title_of = |series: &str| {
            let xml = chart_space(&format!(
                r#"<c:chart><c:title><c:overlay val="0"/></c:title><c:plotArea><c:barChart><c:barDir val="col"/>{series}</c:barChart><c:valAx><c:title><c:overlay val="0"/></c:title></c:valAx></c:plotArea></c:chart>"#
            ));
            let doc = XmlNode::parse_str(&xml).unwrap();
            let chart = column_chart(load_chart(&doc).unwrap());
            assert_eq!(chart.value_axis.title.unwrap().text, "Axis Title");
            chart.title.unwrap().text
        };
        // One series: its name
        assert_eq!(title_of(named), "September");
        assert_eq!(title_of(literal), "Literal");
        // Several: the placeholder
        assert_eq!(title_of(&format!("{named}{literal}")), "Chart Title");
    }

    #[test]
    fn reads_the_fill_and_border_of_the_titles() {
        let xml = chart_space(
            r#"<c:chart><c:title><c:tx><c:rich><a:p><a:r><a:t>Sales</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/><c:spPr><a:solidFill><a:schemeClr val="accent1"><a:lumMod val="20000"/><a:lumOff val="80000"/></a:schemeClr></a:solidFill><a:ln w="9525"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></c:spPr></c:title><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:title><c:tx><c:rich><a:p><a:r><a:t>City</a:t></a:r></a:p></c:rich></c:tx><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr></c:title></c:catAx><c:valAx><c:title><c:tx><c:rich><a:p><a:r><a:t>People</a:t></a:r></a:p></c:rich></c:tx></c:title></c:valAx></c:plotArea></c:chart>"#
                .replace("{SERIES}", SERIES)
                .as_str(),
        );
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        let title = chart.title.unwrap();
        assert_eq!(title.fill, Color::Theme(4, 0.8));
        assert_eq!(
            title.border,
            Line {
                style: LineStyle::Thin,
                color: Color::Rgb("#FF0000".to_string()),
            }
        );
        // An explicit noFill is none, a missing spPr is automatic (no border)
        let category = chart.category_axis.title.unwrap();
        assert_eq!(category.fill, Color::None);
        assert_eq!(category.border.style, LineStyle::None);
        let value = chart.value_axis.title.unwrap();
        assert_eq!(value.fill, Color::None);
        assert_eq!(value.border.style, LineStyle::Automatic);
    }

    #[test]
    fn reads_the_alignment_of_a_moved_title() {
        let alignment_of = |layout: &str| {
            let xml = chart_space(
                r#"<c:chart><c:title>{LAYOUT}<c:overlay val="0"/></c:title><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart></c:plotArea></c:chart>"#
                    .replace("{SERIES}", SERIES)
                    .replace("{LAYOUT}", layout)
                    .as_str(),
            );
            let doc = XmlNode::parse_str(&xml).unwrap();
            column_chart(load_chart(&doc).unwrap())
                .title
                .unwrap()
                .alignment
        };
        let edge = |x: &str| {
            format!(
                r#"<c:layout><c:manualLayout><c:xMode val="edge"/><c:yMode val="edge"/><c:x val="{x}"/><c:y val="2.3148148148148147E-2"/></c:manualLayout></c:layout>"#
            )
        };
        // Not moved: centered
        assert_eq!(alignment_of(""), Alignment::Middle);
        assert_eq!(alignment_of("<c:layout/>"), Alignment::Middle);
        // Chart 18 of AssortedColumns.xlsx: dragged to the right edge
        assert_eq!(alignment_of(&edge("0.86459532268228989")), Alignment::Right);
        assert_eq!(alignment_of(&edge("0.5")), Alignment::Right);
        assert_eq!(alignment_of(&edge("0")), Alignment::Left);
        assert_eq!(alignment_of(&edge("0.12")), Alignment::Left);
        // Nudged, still about the middle
        assert_eq!(alignment_of(&edge("0.35")), Alignment::Middle);
        // Factor mode: an offset from the centered place
        let factor = |x: &str| {
            format!(
                r#"<c:layout><c:manualLayout><c:xMode val="factor"/><c:x val="{x}"/></c:manualLayout></c:layout>"#
            )
        };
        assert_eq!(alignment_of(&factor("-0.3")), Alignment::Left);
        assert_eq!(alignment_of(&factor("0.3")), Alignment::Right);
        assert_eq!(alignment_of(&factor("0.05")), Alignment::Middle);
    }

    #[test]
    fn reads_a_multiline_title_as_title_and_subtitle() {
        // Chart 17 of AssortedColumns.xlsx: a 28pt red first line, a break,
        // a 10pt second line; the paragraph's default is 14pt gray
        let xml = chart_space(
            r#"<c:chart><c:title><c:tx><c:rich><a:bodyPr rot="0"/><a:p><a:pPr><a:defRPr sz="1400" b="0" i="0"><a:solidFill><a:schemeClr val="tx1"><a:lumMod val="65000"/><a:lumOff val="35000"/></a:schemeClr></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:pPr><a:r><a:rPr lang="en-US" sz="2800"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:rPr><a:t>The</a:t></a:r><a:r><a:rPr lang="en-US" sz="2800" baseline="0"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:rPr><a:t> main title is here</a:t></a:r><a:br><a:rPr lang="en-US" baseline="0"/></a:br><a:r><a:rPr lang="en-US" sz="1000" i="1" baseline="0"/><a:t>and another line</a:t></a:r></a:p></c:rich></c:tx></c:title><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:catAx><c:title><c:tx><c:rich><a:p><a:r><a:t>City</a:t></a:r></a:p><a:p><a:r><a:t>in</a:t></a:r></a:p><a:p><a:r><a:t>2024</a:t></a:r></a:p></c:rich></c:tx></c:title></c:catAx><c:valAx><c:title><c:tx><c:rich><a:p><a:r><a:t>People</a:t></a:r></a:p><a:p><a:endParaRPr lang="en-US"/></a:p></c:rich></c:tx></c:title></c:valAx></c:plotArea></c:chart>"#
                .replace("{SERIES}", SERIES)
                .as_str(),
        );
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        let title = chart.title.unwrap();
        assert_eq!(title.text, "The main title is here");
        assert_eq!(title.font.size, 28);
        assert_eq!(title.font.color, Color::Rgb("#FF0000".to_string()));
        assert!(!title.font.bold);
        let subtitle = title.subtitle.unwrap();
        assert_eq!(subtitle.text, "and another line");
        // The run's size and style over the paragraph's color
        assert_eq!(subtitle.font.size, 10);
        assert!(subtitle.font.italic);
        assert_eq!(subtitle.font.color, Color::Theme(1, 0.35));
        // Further paragraphs join the subtitle; an empty one is nothing
        let category = chart.category_axis.title.unwrap();
        assert_eq!(category.text, "City");
        assert_eq!(category.subtitle.unwrap().text, "in 2024");
        let value = chart.value_axis.title.unwrap();
        assert_eq!(value.text, "People");
        assert_eq!(value.subtitle, None);
    }

    #[test]
    fn reads_the_gap_and_overlap_of_clustered_columns() {
        let bars_of = |grouping: &str, spacing: &str| {
            let xml = chart_space(
                r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="{GROUPING}"/>{SERIES}{SPACING}</c:barChart></c:plotArea></c:chart>"#
                    .replace("{SERIES}", SERIES)
                    .replace("{GROUPING}", grouping)
                    .replace("{SPACING}", spacing)
                    .as_str(),
            );
            let doc = XmlNode::parse_str(&xml).unwrap();
            column_chart(load_chart(&doc).unwrap()).bars
        };
        // Chart 17 of AssortedColumns.xlsx
        assert_eq!(
            bars_of(
                "clustered",
                r#"<c:gapWidth val="100"/><c:overlap val="-50"/>"#
            ),
            Bars {
                gap: 100,
                overlap: -50
            }
        );
        // The schema's defaults for missing elements
        assert_eq!(
            bars_of("clustered", r#"<c:gapWidth val="219"/>"#),
            Bars {
                gap: 219,
                overlap: 0
            }
        );
        assert_eq!(
            bars_of("clustered", ""),
            Bars {
                gap: 150,
                overlap: 0
            }
        );
        // Stacked columns keep their gap and always overlap fully
        assert_eq!(
            bars_of("stacked", r#"<c:gapWidth val="80"/><c:overlap val="100"/>"#),
            Bars {
                gap: 80,
                overlap: 100
            }
        );
        assert_eq!(
            bars_of("percentStacked", r#"<c:overlap val="-20"/>"#),
            Bars {
                gap: 150,
                overlap: 100
            }
        );
    }

    #[test]
    fn reads_the_direction_of_the_data_labels() {
        // Chart 18 of AssortedColumns.xlsx: one series per text direction
        let series = |index: usize, rot: &str| {
            format!(
                r#"<c:ser><c:idx val="{index}"/><c:order val="{index}"/><c:dLbls><c:txPr><a:bodyPr rot="{rot}" vert="horz"/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="900"/></a:pPr></a:p></c:txPr><c:dLblPos val="ctr"/><c:showVal val="1"/></c:dLbls><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$E$5:$E$9</c:f></c:numRef></c:val></c:ser>"#
            )
        };
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{}{}{}{SERIES}</c:barChart></c:plotArea></c:chart>"#,
            series(0, "0"),
            series(1, "-5400000"),
            series(2, "5400000"),
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        let directions: Vec<Option<TextDirection>> = chart
            .data_labels
            .iter()
            .map(|labels| labels.as_ref().map(|labels| labels.direction.clone()))
            .collect();
        assert_eq!(
            directions,
            vec![
                Some(TextDirection::Horizontal),
                Some(TextDirection::Rotate270),
                Some(TextDirection::Rotate90),
                // The last series has no labels at all
                None,
            ]
        );
    }

    #[test]
    fn reads_the_data_labels_of_each_series() {
        let labelled = r##"<c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:strRef><c:f>Sheet1!$E$4</c:f></c:strRef></c:tx><c:invertIfNegative val="0"/><c:dLbls><c:numFmt formatCode="#,##0.00" sourceLinked="0"/><c:spPr><a:solidFill><a:schemeClr val="accent1"><a:lumMod val="20000"/><a:lumOff val="80000"/></a:schemeClr></a:solidFill><a:ln><a:noFill/></a:ln></c:spPr><c:txPr><a:bodyPr/><a:lstStyle/><a:p><a:pPr><a:defRPr sz="900" b="1" i="0"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:pPr><a:endParaRPr lang="en-US"/></a:p></c:txPr><c:dLblPos val="inEnd"/><c:showLegendKey val="0"/><c:showVal val="1"/><c:showCatName val="0"/><c:showSerName val="0"/><c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$E$5:$E$9</c:f></c:numRef></c:val></c:ser>"##;
        let hidden = r#"<c:ser><c:idx val="1"/><c:order val="1"/><c:dLbls><c:showLegendKey val="0"/><c:showVal val="0"/><c:showCatName val="0"/><c:showSerName val="0"/><c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$F$5:$F$9</c:f></c:numRef></c:val></c:ser>"#;
        let default_position = r#"<c:ser><c:idx val="2"/><c:order val="2"/><c:dLbls><c:showLegendKey val="0"/><c:showVal val="1"/><c:showCatName val="0"/><c:showSerName val="0"/><c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls><c:cat><c:strRef><c:f>Sheet1!$D$5:$D$9</c:f></c:strRef></c:cat><c:val><c:numRef><c:f>Sheet1!$G$5:$G$9</c:f></c:numRef></c:val></c:ser>"#;
        let chart_of = |grouping: &str| {
            let xml = chart_space(&format!(
                r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/><c:grouping val="{grouping}"/>{labelled}{hidden}{default_position}{SERIES}<c:dLbls><c:showLegendKey val="0"/><c:showVal val="0"/><c:showCatName val="0"/><c:showSerName val="0"/><c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls></c:barChart></c:plotArea></c:chart>"#
            ));
            let doc = XmlNode::parse_str(&xml).unwrap();
            column_chart(load_chart(&doc).unwrap())
        };

        let chart = chart_of("clustered");
        assert_eq!(chart.data_labels.len(), 4);
        let labels = chart.data_labels[0].clone().unwrap();
        assert_eq!(labels.position, DataLabelPosition::InsideEnd);
        assert_eq!(labels.number_format, Some("#,##0.00".to_string()));
        assert_eq!(labels.font.size, 9);
        assert!(labels.font.bold);
        assert_eq!(labels.font.color, Color::Rgb("#FF0000".to_string()));
        assert_eq!(labels.fill, Color::Theme(4, 0.8));
        assert_eq!(labels.border.style, LineStyle::None);
        // Labels that do not show the value are no labels
        assert!(chart.data_labels[1].is_none());
        // No position: outside end for clustered columns, automatic styling
        let defaults = chart.data_labels[2].clone().unwrap();
        assert_eq!(defaults.position, DataLabelPosition::OutsideEnd);
        assert_eq!(defaults.number_format, None);
        assert_eq!(defaults.fill, Color::None);
        assert_eq!(defaults.border.style, LineStyle::Automatic);
        // The group's labels, which show nothing, give the last series none
        assert!(chart.data_labels[3].is_none());

        // The same without a position is the center of a stacked column
        let stacked = chart_of("stacked");
        assert_eq!(
            stacked.data_labels[2].clone().unwrap().position,
            DataLabelPosition::Center
        );
    }

    #[test]
    fn group_labels_apply_to_the_series_without_their_own() {
        let xml = chart_space(&format!(
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}<c:dLbls><c:dLblPos val="inBase"/><c:showLegendKey val="0"/><c:showVal val="1"/><c:showCatName val="0"/><c:showSerName val="0"/><c:showPercent val="0"/><c:showBubbleSize val="0"/></c:dLbls></c:barChart></c:plotArea></c:chart>"#
        ));
        let doc = XmlNode::parse_str(&xml).unwrap();
        let chart = column_chart(load_chart(&doc).unwrap());
        assert_eq!(
            chart.data_labels[0].clone().unwrap().position,
            DataLabelPosition::InsideBase
        );
    }

    #[test]
    fn other_charts_are_not_supported() {
        for inner in [
            // a horizontal bar chart
            format!(r#"<c:chart><c:plotArea><c:barChart><c:barDir val="bar"/>{SERIES}</c:barChart></c:plotArea></c:chart>"#),
            // a line chart
            format!(r#"<c:chart><c:plotArea><c:lineChart>{SERIES}</c:lineChart></c:plotArea></c:chart>"#),
            // a combo chart
            format!(r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/>{SERIES}</c:barChart><c:lineChart>{SERIES}</c:lineChart></c:plotArea></c:chart>"#),
            // literal values
            r#"<c:chart><c:plotArea><c:barChart><c:barDir val="col"/><c:ser><c:cat><c:strLit><c:ptCount val="1"/></c:strLit></c:cat><c:val><c:numLit><c:ptCount val="1"/></c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart>"#.to_string(),
        ] {
            let doc = XmlNode::parse_str(&chart_space(&inner)).unwrap();
            assert!(load_chart(&doc).is_none(), "{inner}");
        }
    }
}
