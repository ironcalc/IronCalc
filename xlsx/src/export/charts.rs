//! Writes a [`Chart`] as a chart part (`xl/charts/chartN.xml`).
//!
//! Next to every reference goes the cache Excel keeps of its values
//! (`strCache`/`numCache`), computed from the model. When a reference cannot
//! be resolved (a sheet we do not have) the caches are left out and Excel
//! computes them when the file is opened.

use ironcalc_base::charts::{
    expand_data_range, Alignment, Chart, ChartData, ColumnChart, DataLabelPosition, DataLabels,
    DataSource, EmptyCells, Fill, Font, Gridlines, Legend, Line, LineStyle, Position, Series,
    SeriesData, Stacking, TextDirection, TickMark, Title, ValueFormat,
};
use ironcalc_base::drawing_types::{Anchor, Drawing};
use ironcalc_base::types::Color;
use ironcalc_base::Model;

use super::escape::escape_xml;
use super::xml_constants::XML_DECLARATION;

const CATEGORY_AXIS_ID: u32 = 1;
const VALUE_AXIS_ID: u32 = 2;
/// Excel's sentinel for "rotate the text as needed".
const AUTOMATIC_ROTATION: i32 = -60_000_000;

/// Line widths in EMU
const THIN_LINE_WIDTH: u32 = 9525;
const THICK_LINE_WIDTH: u32 = 25400;

/// The name of a theme slot in DrawingML, inverse of the importer's table.
fn scheme_color_name(index: i32) -> &'static str {
    match index {
        0 => "lt1",
        1 => "dk1",
        2 => "lt2",
        3 => "dk2",
        4 => "accent1",
        5 => "accent2",
        6 => "accent3",
        7 => "accent4",
        8 => "accent5",
        9 => "accent6",
        10 => "hlink",
        11 => "folHlink",
        _ => "dk1",
    }
}

/// A DrawingML color element, or an empty string for `Color::None`.
fn color_xml(color: &Color) -> String {
    match color {
        Color::Rgb(hex) => format!(
            "<a:srgbClr val=\"{}\"/>",
            hex.trim_start_matches('#').to_ascii_uppercase()
        ),
        Color::Theme(index, tint) => {
            let name = scheme_color_name(*index);
            let modifiers = if *tint > 0.0 {
                format!(
                    "<a:lumMod val=\"{}\"/><a:lumOff val=\"{}\"/>",
                    ((1.0 - tint) * 100_000.0).round() as i64,
                    (tint * 100_000.0).round() as i64
                )
            } else if *tint < 0.0 {
                format!(
                    "<a:lumMod val=\"{}\"/>",
                    ((1.0 + tint) * 100_000.0).round() as i64
                )
            } else {
                "".to_string()
            };
            if modifiers.is_empty() {
                format!("<a:schemeClr val=\"{name}\"/>")
            } else {
                format!("<a:schemeClr val=\"{name}\">{modifiers}</a:schemeClr>")
            }
        }
        Color::None => "".to_string(),
    }
}

/// A solid fill, or no fill for `Color::None`.
fn color_fill_xml(color: &Color) -> String {
    match color {
        Color::None => "<a:noFill/>".to_string(),
        _ => format!("<a:solidFill>{}</a:solidFill>", color_xml(color)),
    }
}

/// Excel's default chart style, written for everything that is automatic:
/// a white area, a thin line in text color at 15%.
const AUTOMATIC_FILL: Color = Color::Theme(0, 0.0);
const AUTOMATIC_LINE: Line = Line {
    style: LineStyle::Thin,
    color: Color::Theme(1, 0.85),
};
const NO_LINE: Line = Line {
    style: LineStyle::None,
    color: Color::None,
};
/// Excel's default minor gridline: thin, text color at 5%.
const AUTOMATIC_MINOR_LINE: Line = Line {
    style: LineStyle::Thin,
    color: Color::Theme(1, 0.95),
};

fn fill_xml(fill: &Fill) -> String {
    match fill {
        Fill::Automatic => color_fill_xml(&AUTOMATIC_FILL),
        Fill::None => "<a:noFill/>".to_string(),
        Fill::Solid(color) => color_fill_xml(color),
    }
}

/// An `a:ln`. An automatic line is written as `automatic`, which is what
/// Excel's default style gives that element.
fn line_xml(line: &Line, automatic: &Line) -> String {
    let (width, dash) = match line.style {
        LineStyle::Automatic => return line_xml(automatic, &NO_LINE),
        LineStyle::None => return "<a:ln><a:noFill/></a:ln>".to_string(),
        LineStyle::Thin => (THIN_LINE_WIDTH, ""),
        LineStyle::Thick => (THICK_LINE_WIDTH, ""),
        LineStyle::Dashed => (THIN_LINE_WIDTH, "<a:prstDash val=\"dash\"/>"),
    };
    format!(
        "<a:ln w=\"{width}\">{}{dash}</a:ln>",
        color_fill_xml(&line.color)
    )
}

/// The shape properties of an element that has a line but no fill of its
/// own: an axis, a gridline.
fn line_only_shape_xml(line: &Line, automatic: &Line) -> String {
    format!("<c:spPr><a:noFill/>{}</c:spPr>", line_xml(line, automatic))
}

/// An `a:defRPr` (the default of a paragraph) or an `a:rPr` (one run) with
/// `font`.
fn font_properties_xml(tag: &str, font: &Font) -> String {
    let fill = match &font.color {
        Color::None => "".to_string(),
        color => format!("<a:solidFill>{}</a:solidFill>", color_xml(color)),
    };
    // A missing family is the theme's minor font, like Excel's defaults.
    let typeface = match &font.family {
        Some(family) => escape_xml(family).to_string(),
        None => "+mn-lt".to_string(),
    };
    let lang = if tag == "rPr" { " lang=\"en-US\"" } else { "" };
    format!(
        "<a:{tag}{lang} sz=\"{}\" b=\"{}\" i=\"{}\">{fill}<a:latin typeface=\"{typeface}\"/></a:{tag}>",
        font.size * 100,
        u8::from(font.bold),
        u8::from(font.italic)
    )
}

fn default_run_properties_xml(font: &Font) -> String {
    font_properties_xml("defRPr", font)
}

/// The text properties of an element that has no text of its own (an axis,
/// a legend, an automatic title).
fn text_properties_xml(font: &Font) -> String {
    text_properties_with_body_xml("<a:bodyPr/>", font)
}

fn text_properties_with_body_xml(body: &str, font: &Font) -> String {
    format!(
        "<c:txPr>{body}<a:lstStyle/><a:p><a:pPr>{}</a:pPr><a:endParaRPr lang=\"en-US\"/></a:p></c:txPr>",
        default_run_properties_xml(font)
    )
}

/// The text properties of data labels: the font and, when they are turned,
/// the quarter turn in 60000ths of a degree clockwise.
fn data_labels_text_properties_xml(font: &Font, direction: &TextDirection) -> String {
    let body = match direction {
        TextDirection::Horizontal => "<a:bodyPr/>",
        TextDirection::Rotate90 => "<a:bodyPr rot=\"5400000\" vert=\"horz\"/>",
        TextDirection::Rotate270 => "<a:bodyPr rot=\"-5400000\" vert=\"horz\"/>",
    };
    text_properties_with_body_xml(body, font)
}

/// The text properties of the category axis labels: the font and their angle.
/// Excel's degrees run counter-clockwise, OOXML's 60000ths of a degree clockwise.
fn category_labels_properties_xml(font: &Font, rotation: Option<i32>) -> String {
    let rot = match rotation {
        Some(degrees) => -degrees * 60000,
        None => AUTOMATIC_ROTATION,
    };
    format!(
        "<c:txPr><a:bodyPr rot=\"{rot}\" vert=\"horz\"/><a:lstStyle/><a:p><a:pPr>{}</a:pPr><a:endParaRPr lang=\"en-US\"/></a:p></c:txPr>",
        default_run_properties_xml(font)
    )
}

/// Excel's rotation of the value axis title, reading upwards, in 60000ths
/// of a degree clockwise.
const VALUE_AXIS_TITLE_ROTATION: i32 = -5400000;

/// A `c:title`. An empty text is Excel's automatic title and has no `c:tx`.
/// `rotation` is in 60000ths of a degree clockwise: 0 for a horizontal
/// title, `VALUE_AXIS_TITLE_ROTATION` for the one along the value axis.
/// `title` as a `c:title`; `layout` is its placement, empty for Excel's own.
fn title_xml(title: &Title, rotation: i32, layout: &str) -> String {
    let def_rpr = default_run_properties_xml(&title.font);
    let body = format!(
        "<a:bodyPr rot=\"{rotation}\" spcFirstLastPara=\"1\" vertOverflow=\"ellipsis\" vert=\"horz\" wrap=\"square\" anchor=\"ctr\" anchorCtr=\"1\"/>"
    );
    // Excel repeats the body and the font in a `c:txPr` next to the text
    let text_properties = format!(
        "<c:txPr>{body}<a:lstStyle/><a:p><a:pPr>{def_rpr}</a:pPr><a:endParaRPr lang=\"en-US\"/></a:p></c:txPr>"
    );
    // Excel's default title has no fill and no outline
    let shape = format!(
        "<c:spPr>{}{}</c:spPr>",
        color_fill_xml(&title.fill),
        line_xml(&title.border, &NO_LINE)
    );
    let subtitle = title.subtitle.as_ref().filter(|s| !s.text.is_empty());
    if title.text.is_empty() && subtitle.is_none() {
        return format!(
            "<c:title>{layout}<c:overlay val=\"0\"/>{shape}{text_properties}</c:title>"
        );
    }
    // The subtitle is a second line of the same paragraph, with its own font
    let subtitle = subtitle
        .map(|subtitle| {
            format!(
                "<a:br><a:rPr lang=\"en-US\"/></a:br><a:r>{}<a:t>{}</a:t></a:r>",
                font_properties_xml("rPr", &subtitle.font),
                escape_xml(&subtitle.text)
            )
        })
        .unwrap_or_default();
    format!(
        "<c:title><c:tx><c:rich>{body}<a:lstStyle/><a:p><a:pPr>{def_rpr}</a:pPr><a:r><a:rPr lang=\"en-US\"/><a:t>{}</a:t></a:r>{subtitle}</a:p></c:rich></c:tx>{layout}<c:overlay val=\"0\"/>{shape}{text_properties}</c:title>",
        escape_xml(&title.text)
    )
}

fn optional_title_xml(title: &Option<Title>, rotation: i32) -> String {
    title
        .as_ref()
        .map(|title| title_xml(title, rotation, ""))
        .unwrap_or_default()
}

const EMU_PER_PIXEL: f64 = 9525.0;
const PIXELS_PER_POINT: f64 = 4.0 / 3.0;
/// The average advance of a character, in ems, when estimating a title's width
const AVERAGE_ADVANCE: f64 = 0.5;
/// Excel's inset on each side of a title's text, in pixels (0.1 inch)
const TITLE_INSET: f64 = 9.6;
/// Where Excel puts the top of a title, in pixels from the chart's edge
const TITLE_TOP: f64 = 6.0;

/// The size in pixels of the frame `anchor` spans on `sheet`.
fn frame_size(anchor: &Anchor, sheet: u32, model: &Model) -> (f64, f64) {
    let width: f64 = (anchor.from.column..anchor.to.column)
        .map(|column| model.get_column_width(sheet, column).unwrap_or(0.0))
        .sum::<f64>()
        + (anchor.to.column_offset - anchor.from.column_offset) as f64 / EMU_PER_PIXEL;
    let height: f64 = (anchor.from.row..anchor.to.row)
        .map(|row| model.get_row_height(sheet, row).unwrap_or(0.0))
        .sum::<f64>()
        + (anchor.to.row_offset - anchor.from.row_offset) as f64 / EMU_PER_PIXEL;
    (width, height)
}

/// About how wide Excel draws `title`, in pixels: its widest line at an
/// average advance, plus the insets of the box.
fn estimated_title_width(title: &Title) -> f64 {
    let line = |text: &str, font: &Font| {
        text.chars().count() as f64 * f64::from(font.size) * PIXELS_PER_POINT * AVERAGE_ADVANCE
    };
    let mut width = line(&title.text, &title.font);
    if let Some(subtitle) = &title.subtitle {
        width = width.max(line(&subtitle.text, &subtitle.font));
    }
    width + 2.0 * TITLE_INSET
}

/// The placement of the chart title when it is not centered. Excel has no
/// alignment, only the position of the title's left edge as a fraction of
/// the chart, so a right-aligned title needs its width, which is estimated.
fn title_layout_xml(title: &Title, drawing: &Drawing, sheet: u32, model: &Model) -> String {
    let (width, height) = frame_size(&drawing.anchor, sheet, model);
    if width <= 0.0 || height <= 0.0 {
        return String::new();
    }
    let x = match title.alignment {
        Alignment::Middle => return String::new(),
        Alignment::Left => 0.0,
        Alignment::Right => (1.0 - estimated_title_width(title) / width).max(0.0),
    };
    let y = TITLE_TOP / height;
    format!(
        "<c:layout><c:manualLayout><c:xMode val=\"edge\"/><c:yMode val=\"edge\"/><c:x val=\"{x}\"/><c:y val=\"{y}\"/></c:manualLayout></c:layout>"
    )
}

fn legend_xml(legend: &Legend) -> String {
    let (position, layout) = match legend.position {
        Position::None => return "".to_string(),
        Position::Top => ("t", "".to_string()),
        Position::Right => ("r", "".to_string()),
        Position::Bottom => ("b", "".to_string()),
        Position::Left => ("l", "".to_string()),
        Position::Float(x, y) => (
            if legend.vertical { "r" } else { "b" },
            format!(
                "<c:layout><c:manualLayout><c:xMode val=\"edge\"/><c:yMode val=\"edge\"/><c:x val=\"{x}\"/><c:y val=\"{y}\"/></c:manualLayout></c:layout>"
            ),
        ),
    };
    let overlay = u8::from(legend.overlay);
    format!(
        "<c:legend><c:legendPos val=\"{position}\"/>{layout}<c:overlay val=\"{overlay}\"/><c:spPr>{}{}</c:spPr>{}</c:legend>",
        color_fill_xml(&legend.fill),
        // Excel's default legend has no outline
        line_xml(&legend.border, &NO_LINE),
        text_properties_xml(&legend.font)
    )
}

/// A `strCache`: the count of points and the text of the non-empty ones.
fn string_cache_xml(values: &[String]) -> String {
    let points: Vec<String> = values
        .iter()
        .enumerate()
        .filter(|(_, text)| !text.is_empty())
        .map(|(index, text)| {
            format!(
                "<c:pt idx=\"{index}\"><c:v>{}</c:v></c:pt>",
                escape_xml(text)
            )
        })
        .collect();
    format!(
        "<c:strCache><c:ptCount val=\"{}\"/>{}</c:strCache>",
        values.len(),
        points.join("")
    )
}

/// A `numCache`: the format, the count of points and the ones with a value.
fn number_cache_xml(series: &SeriesData) -> String {
    // The model spells the general format in lowercase, Excel does not.
    let format_code = if series.number_format.eq_ignore_ascii_case("general") {
        "General"
    } else {
        &series.number_format
    };
    let points: Vec<String> = series
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            value.map(|value| format!("<c:pt idx=\"{index}\"><c:v>{value}</c:v></c:pt>"))
        })
        .collect();
    format!(
        "<c:numCache><c:formatCode>{}</c:formatCode><c:ptCount val=\"{}\"/>{}</c:numCache>",
        escape_xml(format_code),
        series.values.len(),
        points.join("")
    )
}

/// The `c:tx` of a series. A name in quotes is a literal, anything else a
/// reference; an empty name lets Excel name the series.
fn series_name_xml(name: &str, data: Option<&SeriesData>) -> String {
    if name.is_empty() {
        return "".to_string();
    }
    if name.len() >= 2 && name.starts_with('"') && name.ends_with('"') {
        let literal = name[1..name.len() - 1].replace("\"\"", "\"");
        return format!("<c:tx><c:v>{}</c:v></c:tx>", escape_xml(&literal));
    }
    let cache = match data {
        Some(data) => string_cache_xml(std::slice::from_ref(&data.name)),
        None => "".to_string(),
    };
    format!(
        "<c:tx><c:strRef><c:f>{}</c:f>{cache}</c:strRef></c:tx>",
        escape_xml(name)
    )
}

/// One `c:ser`. `data` holds the resolved values of the whole chart, if they
/// could be computed; the series then gets its caches.
/// A `c:dLbls` showing the value of every column of a series.
fn data_labels_xml(labels: &DataLabels, stacking: &Stacking) -> String {
    let position = match (&labels.position, stacking) {
        // Outside end does not exist for stacked columns
        (DataLabelPosition::OutsideEnd, Stacking::None) => "outEnd",
        (DataLabelPosition::OutsideEnd, _) | (DataLabelPosition::Center, _) => "ctr",
        (DataLabelPosition::InsideEnd, _) => "inEnd",
        (DataLabelPosition::InsideBase, _) => "inBase",
    };
    let number_format = match &labels.number_format {
        Some(code) => format!(
            "<c:numFmt formatCode=\"{}\" sourceLinked=\"0\"/>",
            escape_xml(code)
        ),
        None => "".to_string(),
    };
    format!(
        "<c:dLbls>{number_format}<c:spPr>{}{}</c:spPr>{}<c:dLblPos val=\"{position}\"/><c:showLegendKey val=\"0\"/><c:showVal val=\"1\"/><c:showCatName val=\"0\"/><c:showSerName val=\"0\"/><c:showPercent val=\"0\"/><c:showBubbleSize val=\"0\"/></c:dLbls>",
        color_fill_xml(&labels.fill),
        line_xml(&labels.border, &NO_LINE),
        data_labels_text_properties_xml(&labels.font, &labels.direction)
    )
}

fn series_xml(
    index: usize,
    series: &Series,
    categories: &str,
    color: Option<&Color>,
    labels: Option<&DataLabels>,
    stacking: &Stacking,
    data: Option<&ChartData>,
) -> String {
    let fill = match color {
        Some(Color::None) | None => "".to_string(),
        Some(color) => format!(
            "<c:spPr><a:solidFill>{}</a:solidFill></c:spPr>",
            color_xml(color)
        ),
    };
    let series_data = data.and_then(|data| data.series.get(index));
    let category_cache = match data {
        Some(data) => string_cache_xml(&data.categories),
        None => "".to_string(),
    };
    let value_cache = match series_data {
        Some(series_data) => number_cache_xml(series_data),
        None => "".to_string(),
    };
    let data_labels = match labels {
        Some(labels) => data_labels_xml(labels, stacking),
        None => "".to_string(),
    };
    format!(
        "<c:ser><c:idx val=\"{index}\"/><c:order val=\"{index}\"/>{}{fill}<c:invertIfNegative val=\"0\"/>{data_labels}<c:cat><c:strRef><c:f>{}</c:f>{category_cache}</c:strRef></c:cat><c:val><c:numRef><c:f>{}</c:f>{value_cache}</c:numRef></c:val></c:ser>",
        series_name_xml(&series.name, series_data),
        escape_xml(categories),
        escape_xml(&series.values)
    )
}

/// The `c:majorGridlines` and `c:minorGridlines` of an axis; nothing for
/// lines that are not drawn.
fn gridlines_xml(gridlines: &Gridlines) -> String {
    let element = |name: &str, line: &Line, automatic: &Line| match line.style {
        LineStyle::None => "".to_string(),
        _ => format!(
            "<c:{name}>{}</c:{name}>",
            line_only_shape_xml(line, automatic)
        ),
    };
    format!(
        "{}{}",
        element("majorGridlines", &gridlines.major, &AUTOMATIC_LINE),
        element("minorGridlines", &gridlines.minor, &AUTOMATIC_MINOR_LINE)
    )
}

/// The `c:majorTickMark` and `c:minorTickMark` of an axis.
fn tick_marks_xml(gridlines: &Gridlines) -> String {
    let name = |tick: &TickMark| match tick {
        TickMark::None => "none",
        TickMark::Inside => "in",
        TickMark::Outside => "out",
        TickMark::Cross => "cross",
    };
    format!(
        "<c:majorTickMark val=\"{}\"/><c:minorTickMark val=\"{}\"/>",
        name(&gridlines.major_ticks),
        name(&gridlines.minor_ticks)
    )
}

fn category_axis_xml(chart: &ColumnChart) -> String {
    format!(
        "<c:catAx><c:axId val=\"{CATEGORY_AXIS_ID}\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"b\"/>{}{}<c:numFmt formatCode=\"General\" sourceLinked=\"1\"/>{}<c:tickLblPos val=\"nextTo\"/>{}{}<c:crossAx val=\"{VALUE_AXIS_ID}\"/><c:crosses val=\"autoZero\"/><c:auto val=\"1\"/><c:lblAlgn val=\"ctr\"/><c:lblOffset val=\"100\"/><c:noMultiLvlLbl val=\"0\"/></c:catAx>",
        gridlines_xml(&chart.category_axis.gridlines),
        optional_title_xml(&chart.category_axis.title, 0),
        tick_marks_xml(&chart.category_axis.gridlines),
        line_only_shape_xml(&chart.category_axis.line, &AUTOMATIC_LINE),
        category_labels_properties_xml(
            &chart.category_axis.labels,
            chart.category_axis.rotation
        )
    )
}

fn number_format_xml(format: &ValueFormat) -> String {
    match &format.number_format {
        Some(code) => format!(
            "<c:numFmt formatCode=\"{}\" sourceLinked=\"0\"/>",
            escape_xml(code)
        ),
        None => "<c:numFmt formatCode=\"General\" sourceLinked=\"1\"/>".to_string(),
    }
}

fn value_axis_xml(chart: &ColumnChart) -> String {
    let gridlines = gridlines_xml(&chart.value_axis.gridlines);
    format!(
        "<c:valAx><c:axId val=\"{VALUE_AXIS_ID}\"/><c:scaling><c:orientation val=\"minMax\"/></c:scaling><c:delete val=\"0\"/><c:axPos val=\"l\"/>{gridlines}{}{}{}<c:tickLblPos val=\"nextTo\"/>{}{}<c:crossAx val=\"{CATEGORY_AXIS_ID}\"/><c:crosses val=\"autoZero\"/><c:crossBetween val=\"between\"/></c:valAx>",
        optional_title_xml(&chart.value_axis.title, VALUE_AXIS_TITLE_ROTATION),
        number_format_xml(&chart.value_axis.labels),
        tick_marks_xml(&chart.value_axis.gridlines),
        // Excel's default value axis has no line
        line_only_shape_xml(&chart.value_axis.line, &NO_LINE),
        text_properties_xml(&chart.value_axis.labels.font)
    )
}

/// The settings for hidden and empty cells, at the end of `c:chart`.
fn data_settings_xml(data: &DataSource) -> String {
    let blanks = match data.empty_cells {
        EmptyCells::Gaps => "gap",
        EmptyCells::Zero => "zero",
    };
    // `dispNaAsBlank` is a Microsoft extension; Excel writes it only when set.
    let na = if data.na_as_empty {
        "<c:extLst><c:ext uri=\"{56B9EC1D-385E-4148-901F-78D8002777C0}\" xmlns:c16r3=\"http://schemas.microsoft.com/office/drawing/2017/03/chart\"><c16r3:dataDisplayOptions16><c16r3:dispNaAsBlank val=\"1\"/></c16r3:dataDisplayOptions16></c:ext></c:extLst>"
    } else {
        ""
    };
    format!(
        "<c:plotVisOnly val=\"{}\"/><c:dispBlanksAs val=\"{blanks}\"/>{na}",
        u8::from(!data.show_hidden_data)
    )
}

fn column_chart_xml(chart: &ColumnChart, drawing: &Drawing, sheet: u32, model: &Model) -> String {
    // Stacked columns always overlap fully
    let gap_width = chart.bars.gap.min(500);
    let (grouping, overlap) = match chart.stacking {
        Stacking::None => ("clustered", chart.bars.overlap.clamp(-100, 100)),
        Stacking::Standard => ("stacked", 100),
        Stacking::Full => ("percentStacked", 100),
    };
    let (categories, series) = expand_data_range(&chart.data.range);
    let data = model.get_chart_data(&chart.data).ok();
    let series_xml: Vec<String> = series
        .iter()
        .enumerate()
        .map(|(index, series)| {
            series_xml(
                index,
                series,
                &categories,
                chart.colors.get(index),
                chart.data_labels.get(index).and_then(Option::as_ref),
                &chart.stacking,
                data.as_ref(),
            )
        })
        .collect();
    format!(
        "<c:chart>{}<c:autoTitleDeleted val=\"{}\"/><c:plotArea><c:layout/><c:barChart><c:barDir val=\"col\"/><c:grouping val=\"{grouping}\"/><c:varyColors val=\"0\"/>{}<c:gapWidth val=\"{gap_width}\"/><c:overlap val=\"{overlap}\"/><c:axId val=\"{CATEGORY_AXIS_ID}\"/><c:axId val=\"{VALUE_AXIS_ID}\"/></c:barChart>{}{}<c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr></c:plotArea>{}{}</c:chart>",
        chart
            .title
            .as_ref()
            .map(|title| title_xml(title, 0, &title_layout_xml(title, drawing, sheet, model)))
            .unwrap_or_default(),
        u8::from(chart.title.is_none()),
        series_xml.join(""),
        category_axis_xml(chart),
        value_axis_xml(chart),
        legend_xml(&chart.legend),
        data_settings_xml(&chart.data)
    )
}

/// The chart part for `chart`. The fill and border of the chart area are
/// those of the drawing that holds it; the cached values come from `model`.
pub(crate) fn get_chart_xml(chart: &Chart, drawing: &Drawing, sheet: u32, model: &Model) -> String {
    let chart_xml = match chart {
        Chart::ColumnChart(column_chart) => column_chart_xml(column_chart, drawing, sheet, model),
    };
    let area = format!(
        "<c:spPr>{}{}</c:spPr>",
        fill_xml(&drawing.fill),
        line_xml(&drawing.border, &AUTOMATIC_LINE)
    );
    format!(
        "{XML_DECLARATION}\n<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><c:date1904 val=\"0\"/><c:lang val=\"en-US\"/><c:roundedCorners val=\"0\"/>{chart_xml}{area}</c:chartSpace>"
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ironcalc_base::charts::{DataRange, Orientation, Subtitle};
    use ironcalc_base::drawing_types::{AnchorBehavior, AnchorPoint, DrawingContent};

    #[test]
    fn theme_colors_become_scheme_colors_with_luminance() {
        assert_eq!(
            color_xml(&Color::Theme(4, 0.0)),
            "<a:schemeClr val=\"accent1\"/>"
        );
        assert_eq!(
            color_xml(&Color::Theme(1, 0.35)),
            "<a:schemeClr val=\"dk1\"><a:lumMod val=\"65000\"/><a:lumOff val=\"35000\"/></a:schemeClr>"
        );
        assert_eq!(
            color_xml(&Color::Theme(9, -0.25)),
            "<a:schemeClr val=\"accent6\"><a:lumMod val=\"75000\"/></a:schemeClr>"
        );
        assert_eq!(
            color_xml(&Color::Rgb("#ff0000".to_string())),
            "<a:srgbClr val=\"FF0000\"/>"
        );
    }

    #[test]
    fn automatic_lines_and_fills_are_written_as_the_default_style() {
        let automatic = Line {
            style: LineStyle::Automatic,
            color: Color::Rgb("#123456".to_string()),
        };
        assert_eq!(
            line_xml(&automatic, &AUTOMATIC_LINE),
            "<a:ln w=\"9525\"><a:solidFill><a:schemeClr val=\"dk1\"><a:lumMod val=\"15000\"/><a:lumOff val=\"85000\"/></a:schemeClr></a:solidFill></a:ln>"
        );
        assert_eq!(line_xml(&automatic, &NO_LINE), "<a:ln><a:noFill/></a:ln>");
        assert_eq!(
            fill_xml(&Fill::Automatic),
            "<a:solidFill><a:schemeClr val=\"lt1\"/></a:solidFill>"
        );
        assert_eq!(fill_xml(&Fill::None), "<a:noFill/>");
    }

    #[test]
    fn titles_carry_their_fill_and_border() {
        let font = Font {
            size: 14,
            family: None,
            bold: false,
            italic: false,
            color: Color::None,
        };
        let plain = Title {
            text: "Sales".to_string(),
            font: font.clone(),
            alignment: Alignment::Middle,
            subtitle: None,
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
        };
        // The default title: no fill and no outline, horizontal
        let xml = title_xml(&plain, 0, "");
        assert!(xml.contains("<c:overlay val=\"0\"/><c:spPr><a:noFill/><a:ln><a:noFill/></a:ln></c:spPr><c:txPr><a:bodyPr rot=\"0\""));
        assert!(xml.starts_with("<c:title><c:tx><c:rich><a:bodyPr rot=\"0\""));
        // The value axis title reads upwards, in the text and in its properties
        let upwards = title_xml(&plain, VALUE_AXIS_TITLE_ROTATION, "");
        assert_eq!(upwards.matches("rot=\"-5400000\"").count(), 2);
        let styled = Title {
            text: "".to_string(),
            font,
            alignment: Alignment::Middle,
            subtitle: None,
            fill: Color::Theme(4, 0.8),
            border: Line {
                style: LineStyle::Dashed,
                color: Color::Rgb("#FF0000".to_string()),
            },
        };
        let xml = title_xml(&styled, VALUE_AXIS_TITLE_ROTATION, "");
        assert!(xml.starts_with(
            "<c:title><c:overlay val=\"0\"/><c:spPr><a:solidFill><a:schemeClr val=\"accent1\"><a:lumMod val=\"20000\"/><a:lumOff val=\"80000\"/></a:schemeClr></a:solidFill><a:ln w=\"9525\"><a:solidFill><a:srgbClr val=\"FF0000\"/></a:solidFill><a:prstDash val=\"dash\"/></a:ln></c:spPr><c:txPr><a:bodyPr rot=\"-5400000\""
        ));
        assert!(xml.contains(&default_run_properties_xml(&styled.font)));
    }

    #[test]
    fn a_subtitle_is_a_second_line_of_the_title() {
        let title = Title {
            text: "Sales".to_string(),
            font: Font {
                size: 14,
                family: None,
                bold: false,
                italic: false,
                color: Color::None,
            },
            alignment: Alignment::Middle,
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
            subtitle: Some(Subtitle {
                text: "in <2024>".to_string(),
                font: Font {
                    size: 10,
                    family: None,
                    bold: false,
                    italic: true,
                    color: Color::Rgb("#FF0000".to_string()),
                },
            }),
        };
        let xml = title_xml(&title, 0, "");
        assert!(xml.contains(
            "<a:r><a:rPr lang=\"en-US\"/><a:t>Sales</a:t></a:r><a:br><a:rPr lang=\"en-US\"/></a:br><a:r><a:rPr lang=\"en-US\" sz=\"1000\" b=\"0\" i=\"1\"><a:solidFill><a:srgbClr val=\"FF0000\"/></a:solidFill><a:latin typeface=\"+mn-lt\"/></a:rPr><a:t>in &lt;2024&gt;</a:t></a:r></a:p>"
        ));
        // One paragraph: the second line is a break, not a paragraph
        assert_eq!(xml.matches("<a:p>").count(), 2);
    }

    #[test]
    fn an_aligned_title_is_placed_by_its_left_edge() {
        let model = Model::new_empty("model", "en", "UTC", "en").unwrap();
        // Six default columns wide, offsets apart: the frame's size in pixels
        let anchor = Anchor {
            from: AnchorPoint {
                row: 2,
                column: 2,
                row_offset: 0,
                column_offset: 9525 * 10,
            },
            to: AnchorPoint {
                row: 12,
                column: 8,
                row_offset: 9525 * 5,
                column_offset: 0,
            },
            behavior: AnchorBehavior::MoveAndResize,
        };
        let drawing = Drawing {
            id: 1,
            name: "Chart 1".to_string(),
            anchor,
            fill: Fill::Automatic,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
            content: DrawingContent::Unsupported,
        };
        let column = model.get_column_width(0, 2).unwrap();
        let row = model.get_row_height(0, 2).unwrap();
        let (width, height) = frame_size(&drawing.anchor, 0, &model);
        assert!((width - (6.0 * column - 10.0)).abs() < 1e-9);
        assert!((height - (10.0 * row + 5.0)).abs() < 1e-9);

        let mut title = Title {
            text: "Sales".to_string(),
            font: Font {
                size: 14,
                family: None,
                bold: false,
                italic: false,
                color: Color::None,
            },
            alignment: Alignment::Middle,
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
            subtitle: None,
        };
        // Centered: Excel's own placement
        assert_eq!(title_layout_xml(&title, &drawing, 0, &model), "");
        title.alignment = Alignment::Left;
        let left = title_layout_xml(&title, &drawing, 0, &model);
        assert!(left.starts_with(
            "<c:layout><c:manualLayout><c:xMode val=\"edge\"/><c:yMode val=\"edge\"/><c:x val=\"0\"/><c:y val=\""
        ));
        // Right: the left edge leaves room for the estimated width
        title.alignment = Alignment::Right;
        let right = title_layout_xml(&title, &drawing, 0, &model);
        let x: f64 = right
            .split("<c:x val=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .and_then(|x| x.parse().ok())
            .unwrap();
        let estimated = 5.0 * 14.0 * PIXELS_PER_POINT * AVERAGE_ADVANCE + 2.0 * TITLE_INSET;
        assert!((x - (1.0 - estimated / width)).abs() < 1e-9);
        // The layout goes between the text and the overlay
        let xml = title_xml(&title, 0, &right);
        assert!(xml.contains(&format!("</c:tx>{right}<c:overlay val=\"0\"/>")));
    }

    #[test]
    fn turned_data_labels_carry_their_rotation() {
        let mut labels = DataLabels {
            position: DataLabelPosition::Center,
            direction: TextDirection::Horizontal,
            number_format: None,
            font: Font {
                size: 9,
                family: None,
                bold: false,
                italic: false,
                color: Color::None,
            },
            fill: Color::None,
            border: Line {
                style: LineStyle::Automatic,
                color: Color::None,
            },
        };
        let xml = data_labels_xml(&labels, &Stacking::None);
        assert!(xml.contains("<c:txPr><a:bodyPr/>"));
        labels.direction = TextDirection::Rotate90;
        let xml = data_labels_xml(&labels, &Stacking::None);
        assert!(xml.contains("<c:txPr><a:bodyPr rot=\"5400000\" vert=\"horz\"/>"));
        labels.direction = TextDirection::Rotate270;
        let xml = data_labels_xml(&labels, &Stacking::None);
        assert!(xml.contains("<c:txPr><a:bodyPr rot=\"-5400000\" vert=\"horz\"/>"));
    }

    #[test]
    fn tick_marks_are_written_with_excels_names() {
        let mut gridlines = Gridlines {
            major: Line {
                style: LineStyle::None,
                color: Color::None,
            },
            minor: Line {
                style: LineStyle::None,
                color: Color::None,
            },
            major_ticks: TickMark::Inside,
            minor_ticks: TickMark::Cross,
        };
        assert_eq!(
            tick_marks_xml(&gridlines),
            "<c:majorTickMark val=\"in\"/><c:minorTickMark val=\"cross\"/>"
        );
        gridlines.major_ticks = TickMark::Outside;
        gridlines.minor_ticks = TickMark::None;
        assert_eq!(
            tick_marks_xml(&gridlines),
            "<c:majorTickMark val=\"out\"/><c:minorTickMark val=\"none\"/>"
        );
    }

    #[test]
    fn literal_series_names_are_values() {
        assert_eq!(
            series_name_xml("\"Say \"\"hi\"\"\"", None),
            "<c:tx><c:v>Say &quot;hi&quot;</c:v></c:tx>"
        );
        assert_eq!(
            series_name_xml("Sheet1!$C$1", None),
            "<c:tx><c:strRef><c:f>Sheet1!$C$1</c:f></c:strRef></c:tx>"
        );
        assert_eq!(series_name_xml("", None), "");
    }

    #[test]
    fn references_are_written_with_their_caches() {
        let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
        for (row, column, value) in [
            (1, 3, "Sales & more"),
            (2, 2, "Q1"),
            (2, 3, "10.5"),
            (3, 2, "Q2"),
            (4, 2, "Q3"),
            (4, 3, "text"),
            (5, 2, "Q4"),
            (5, 3, "=NA()"),
        ] {
            model
                .set_user_input(0, row, column, value.to_string())
                .unwrap();
        }
        model.evaluate();
        let source = DataSource {
            range: DataRange::Range {
                range: "Sheet1!$B$1:$C$5".to_string(),
                orientation: Orientation::Columns,
            },
            empty_cells: EmptyCells::Gaps,
            na_as_empty: false,
            show_hidden_data: false,
        };
        let data = model.get_chart_data(&source).unwrap();
        let (categories, series) = expand_data_range(&source.range);
        assert_eq!(
            series_xml(
                0,
                &series[0],
                &categories,
                None,
                None,
                &Stacking::None,
                Some(&data)
            ),
            concat!(
                "<c:ser><c:idx val=\"0\"/><c:order val=\"0\"/>",
                "<c:tx><c:strRef><c:f>Sheet1!$C$1</c:f><c:strCache><c:ptCount val=\"1\"/><c:pt idx=\"0\"><c:v>Sales &amp; more</c:v></c:pt></c:strCache></c:strRef></c:tx>",
                "<c:invertIfNegative val=\"0\"/>",
                "<c:cat><c:strRef><c:f>Sheet1!$B$2:$B$5</c:f><c:strCache><c:ptCount val=\"4\"/><c:pt idx=\"0\"><c:v>Q1</c:v></c:pt><c:pt idx=\"1\"><c:v>Q2</c:v></c:pt><c:pt idx=\"2\"><c:v>Q3</c:v></c:pt><c:pt idx=\"3\"><c:v>Q4</c:v></c:pt></c:strCache></c:strRef></c:cat>",
                "<c:val><c:numRef><c:f>Sheet1!$C$2:$C$5</c:f><c:numCache><c:formatCode>General</c:formatCode><c:ptCount val=\"4\"/><c:pt idx=\"0\"><c:v>10.5</c:v></c:pt><c:pt idx=\"2\"><c:v>0</c:v></c:pt></c:numCache></c:numRef></c:val>",
                "</c:ser>"
            )
        );
        // No caches when the values cannot be resolved
        assert_eq!(
            series_xml(
                0,
                &series[0],
                &categories,
                None,
                None,
                &Stacking::None,
                None
            ),
            concat!(
                "<c:ser><c:idx val=\"0\"/><c:order val=\"0\"/>",
                "<c:tx><c:strRef><c:f>Sheet1!$C$1</c:f></c:strRef></c:tx>",
                "<c:invertIfNegative val=\"0\"/>",
                "<c:cat><c:strRef><c:f>Sheet1!$B$2:$B$5</c:f></c:strRef></c:cat>",
                "<c:val><c:numRef><c:f>Sheet1!$C$2:$C$5</c:f></c:numRef></c:val>",
                "</c:ser>"
            )
        );
    }
}
