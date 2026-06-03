use super::escape::escape_xml;
use ironcalc_base::cf_types::{
    icon_set_icons, CfRule, Cfvo, ConditionalFormatting, Icon, IconThreshold, PeriodType,
    TextOperator, ValueOperator,
};

fn cfvo_xml(cfvo: &Cfvo) -> String {
    match cfvo {
        Cfvo::Min => r#"<cfvo type="min"/>"#.to_string(),
        Cfvo::Max => r#"<cfvo type="max"/>"#.to_string(),
        Cfvo::Number(n) => format!(r#"<cfvo type="num" val="{n}"/>"#),
        Cfvo::Percent(p) => format!(r#"<cfvo type="percent" val="{p}"/>"#),
        Cfvo::Percentile(p) => format!(r#"<cfvo type="percentile" val="{p}"/>"#),
        Cfvo::Formula(f) => format!(r#"<cfvo type="formula" val="{}"/>"#, escape_xml(f)),
    }
}

fn cfvo_with_gte_xml(cfvo: &Cfvo, is_strict: bool) -> String {
    let gte = if is_strict { "" } else { r#" gte="0""# };
    match cfvo {
        Cfvo::Min => format!(r#"<cfvo type="min"{gte}/>"#),
        Cfvo::Max => format!(r#"<cfvo type="max"{gte}/>"#),
        Cfvo::Number(n) => format!(r#"<cfvo type="num" val="{n}"{gte}/>"#),
        Cfvo::Percent(p) => format!(r#"<cfvo type="percent" val="{p}"{gte}/>"#),
        Cfvo::Percentile(p) => format!(r#"<cfvo type="percentile" val="{p}"{gte}/>"#),
        Cfvo::Formula(f) => format!(r#"<cfvo type="formula" val="{}"{gte}/>"#, escape_xml(f)),
    }
}

fn cfvo_x14_xml(cfvo: &Cfvo) -> String {
    match cfvo {
        Cfvo::Min => r#"<x14:cfvo type="min"/>"#.to_string(),
        Cfvo::Max => r#"<x14:cfvo type="max"/>"#.to_string(),
        Cfvo::Number(n) => format!(r#"<x14:cfvo type="num"><xm:f>{n}</xm:f></x14:cfvo>"#),
        Cfvo::Percent(p) => format!(r#"<x14:cfvo type="percent"><xm:f>{p}</xm:f></x14:cfvo>"#),
        Cfvo::Percentile(p) => {
            format!(r#"<x14:cfvo type="percentile"><xm:f>{p}</xm:f></x14:cfvo>"#)
        }
        Cfvo::Formula(f) => format!(
            r#"<x14:cfvo type="formula"><xm:f>{}</xm:f></x14:cfvo>"#,
            escape_xml(f)
        ),
    }
}

fn color_rgb_xml(color: &str) -> String {
    format!(r#"<color rgb="FF{}"/>"#, color.trim_start_matches('#'))
}

fn value_operator_to_str(op: &ValueOperator) -> &'static str {
    match op {
        ValueOperator::Equal => "equal",
        ValueOperator::GreaterThan => "greaterThan",
        ValueOperator::GreaterThanOrEqual => "greaterThanOrEqual",
        ValueOperator::LessThan => "lessThan",
        ValueOperator::LessThanOrEqual => "lessThanOrEqual",
        ValueOperator::NotEqual => "notEqual",
        ValueOperator::Between => "between",
        ValueOperator::NotBetween => "notBetween",
    }
}

fn text_operator_to_type(op: &TextOperator) -> &'static str {
    match op {
        TextOperator::Contains | TextOperator::Equals => "containsText",
        TextOperator::DoesNotContain => "notContainsText",
        TextOperator::BeginsWith => "beginsWith",
        TextOperator::EndsWith => "endsWith",
    }
}

fn period_type_to_str(p: &PeriodType) -> Option<&'static str> {
    match p {
        PeriodType::Yesterday => Some("yesterday"),
        PeriodType::Today => Some("today"),
        PeriodType::Tomorrow => Some("tomorrow"),
        PeriodType::Last7Days => Some("last7Days"),
        PeriodType::Next7Days => Some("next7Days"),
        PeriodType::LastWeek => Some("lastWeek"),
        PeriodType::ThisWeek => Some("thisWeek"),
        PeriodType::NextWeek => Some("nextWeek"),
        PeriodType::LastMonth => Some("lastMonth"),
        PeriodType::ThisMonth => Some("thisMonth"),
        PeriodType::NextMonth => Some("nextMonth"),
        PeriodType::LastYear => Some("lastYear"),
        PeriodType::ThisYear => Some("thisYear"),
        PeriodType::NextYear => Some("nextYear"),
        PeriodType::Between | PeriodType::NotBetween => None,
    }
}

fn anchor_cell_from_sqref(sqref: &str) -> &str {
    sqref
        .split_whitespace()
        .next()
        .unwrap_or("A1")
        .split(':')
        .next()
        .unwrap_or("A1")
}

fn time_period_formula_str(anchor: &str, period: &PeriodType) -> String {
    match period {
        PeriodType::Yesterday => format!("FLOOR({anchor},1)=TODAY()-1"),
        PeriodType::Today => format!("FLOOR({anchor},1)=TODAY()"),
        PeriodType::Tomorrow => format!("FLOOR({anchor},1)=TODAY()+1"),
        PeriodType::Last7Days => {
            format!("AND(TODAY()-7<FLOOR({anchor},1),FLOOR({anchor},1)<=TODAY())")
        }
        PeriodType::Next7Days => {
            format!("AND(FLOOR({anchor},1)>=TODAY(),FLOOR({anchor},1)<=TODAY()+6)")
        }
        PeriodType::LastWeek => format!(
            "AND(FLOOR({anchor},1)>=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7)-7,\
             FLOOR({anchor},1)<=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7)-1)"
        ),
        PeriodType::ThisWeek => format!(
            "AND(FLOOR({anchor},1)>=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7),\
             FLOOR({anchor},1)<=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7)+6)"
        ),
        PeriodType::NextWeek => format!(
            "AND(FLOOR({anchor},1)>=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7)+7,\
             FLOOR({anchor},1)<=TODAY()-MOD(WEEKDAY(TODAY(),2)+6,7)+13)"
        ),
        PeriodType::LastMonth => format!(
            "AND(MONTH({anchor})=IF(MONTH(TODAY())=1,12,MONTH(TODAY())-1),\
             YEAR({anchor})=IF(MONTH(TODAY())=1,YEAR(TODAY())-1,YEAR(TODAY())))"
        ),
        PeriodType::ThisMonth => {
            format!("AND(MONTH({anchor})=MONTH(TODAY()),YEAR({anchor})=YEAR(TODAY()))")
        }
        PeriodType::NextMonth => format!(
            "AND(MONTH({anchor})=IF(MONTH(TODAY())=12,1,MONTH(TODAY())+1),\
             YEAR({anchor})=IF(MONTH(TODAY())=12,YEAR(TODAY())+1,YEAR(TODAY())))"
        ),
        PeriodType::LastYear => format!("YEAR({anchor})=YEAR(TODAY())-1"),
        PeriodType::ThisYear => format!("YEAR({anchor})=YEAR(TODAY())"),
        PeriodType::NextYear => format!("YEAR({anchor})=YEAR(TODAY())+1"),
        PeriodType::Between | PeriodType::NotBetween => String::new(),
    }
}

fn text_formula_str(anchor: &str, op: &TextOperator, value: &str) -> String {
    match op {
        TextOperator::Contains | TextOperator::Equals => {
            format!(r#"NOT(ISERROR(SEARCH("{value}",{anchor})))"#)
        }
        TextOperator::DoesNotContain => format!(r#"ISERROR(SEARCH("{value}",{anchor}))"#),
        TextOperator::BeginsWith => format!(r#"LEFT({anchor},LEN("{value}"))="{value}""#),
        TextOperator::EndsWith => format!(r#"RIGHT({anchor},LEN("{value}"))="{value}""#),
    }
}

const KNOWN_ICON_SETS: &[&str] = &[
    "3Arrows",
    "3ArrowsGray",
    "4Arrows",
    "4ArrowsGray",
    "5Arrows",
    "5ArrowsGray",
    "3Triangles",
    "3TrafficLights1",
    "4TrafficLights",
    "3Signs",
    "4RedToBlack",
    "3Symbols",
    "3Symbols2",
    "3Flags",
];

fn icon_set_name_from_thresholds(thresholds: &[IconThreshold]) -> Option<&'static str> {
    let pairs: Vec<(Icon, String)> = thresholds
        .iter()
        .map(|t| (t.icon.clone(), t.color.clone()))
        .collect();
    for &name in KNOWN_ICON_SETS {
        if let Some(set_pairs) = icon_set_icons(name) {
            if set_pairs == pairs {
                return Some(name);
            }
        }
    }
    None
}

fn icon_rating_set_name(icon: &Icon, count: usize) -> Option<&'static str> {
    match (icon, count) {
        (Icon::Star, 3) => Some("3Stars"),
        (Icon::Circle, 5) => Some("5Quarters"),
        (Icon::FlatRectangle, 3) => Some("3Rating"),
        (Icon::FlatRectangle, 4) => Some("4Rating"),
        (Icon::FlatRectangle, 5) => Some("5Rating"),
        _ => None,
    }
}

/// Maps an IronCalc (Icon, color) pair to an Excel (iconSet, iconId) reference.
/// First tries for an exact color match; falls back to icon-type only.
fn icon_to_excel_ref(icon: &Icon, color: &str) -> (&'static str, u32) {
    // Try standard sets with exact (icon, color) match.
    for &name in KNOWN_ICON_SETS {
        if let Some(icons) = icon_set_icons(name) {
            for (id, (set_icon, set_color)) in icons.iter().enumerate() {
                if set_icon == icon && set_color.as_str() == color {
                    return (name, id as u32);
                }
            }
        }
    }
    // Try x14 rating sets with exact color.
    const RATING_SETS: &[(&str, &[(Icon, &str)])] = &[
        (
            "3Stars",
            &[
                (Icon::Star, "#808080"),
                (Icon::Star, "#FFD700"),
                (Icon::Star, "#FFD700"),
            ],
        ),
        (
            "5Quarters",
            &[
                (Icon::Circle, "#808080"),
                (Icon::Circle, "#ffeb84"),
                (Icon::Circle, "#ffeb84"),
                (Icon::Circle, "#ffeb84"),
                (Icon::Circle, "#FFD700"),
            ],
        ),
        (
            "3Rating",
            &[
                (Icon::FlatRectangle, "#808080"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#4472C4"),
            ],
        ),
        (
            "4Rating",
            &[
                (Icon::FlatRectangle, "#808080"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#4472C4"),
            ],
        ),
        (
            "5Rating",
            &[
                (Icon::FlatRectangle, "#808080"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#ffeb84"),
                (Icon::FlatRectangle, "#4472C4"),
            ],
        ),
    ];
    for &(set_name, icons) in RATING_SETS {
        for (id, (set_icon, set_color)) in icons.iter().enumerate() {
            if set_icon == icon && *set_color == color {
                return (set_name, id as u32);
            }
        }
    }
    // Fallback by icon type, ignoring color.
    match icon {
        Icon::Star => ("3Stars", 2),
        Icon::ArrowUp => ("3Arrows", 2),
        Icon::ArrowRight => ("3Arrows", 1),
        Icon::ArrowDown => ("3Arrows", 0),
        Icon::ArrowAngleUp => ("4Arrows", 2),
        Icon::ArrowAngleDown => ("4Arrows", 1),
        Icon::Circle => ("3TrafficLights1", 2),
        Icon::TriangleUp | Icon::TriangleUpFilled => ("3Triangles", 2),
        Icon::TriangleDown | Icon::TriangleDownFilled => ("3Triangles", 0),
        Icon::FlatRectangle | Icon::Rhombus => ("3Triangles", 1),
        Icon::Flag => ("3Flags", 2),
        Icon::Check | Icon::ThumbsUp => ("3Symbols", 2),
        Icon::Cross | Icon::ThumbsDown => ("3Symbols", 0),
        Icon::Exclamation => ("3Symbols", 1),
        Icon::Heart => ("3TrafficLights1", 2),
    }
}

/// Builds the x14:conditionalFormatting block for a custom icon set (IconSet or IconRating).
/// This goes into the worksheet extLst when there is no matching named Excel icon set.
fn build_x14_custom_icon_xml(
    range: &str,
    excel_priority: u32,
    cfvos_with_strict: &[(Cfvo, bool)],
    icons_colors: &[(Icon, String)],
    show_value: bool,
) -> String {
    let show_val = if !show_value { r#" showValue="0""# } else { "" };
    let cfvo_xml: String = cfvos_with_strict
        .iter()
        .map(|(cfvo, is_strict)| {
            let gte = if *is_strict {
                r#" gte="1""#
            } else {
                r#" gte="0""#
            };
            match cfvo {
                Cfvo::Min => format!(r#"<x14:cfvo type="min"{gte}/>"#),
                Cfvo::Max => format!(r#"<x14:cfvo type="max"{gte}/>"#),
                Cfvo::Number(n) => {
                    format!(r#"<x14:cfvo type="num"{gte}><xm:f>{n}</xm:f></x14:cfvo>"#)
                }
                Cfvo::Percent(p) => {
                    format!(r#"<x14:cfvo type="percent"{gte}><xm:f>{p}</xm:f></x14:cfvo>"#)
                }
                Cfvo::Percentile(p) => {
                    format!(r#"<x14:cfvo type="percentile"{gte}><xm:f>{p}</xm:f></x14:cfvo>"#)
                }
                Cfvo::Formula(f) => format!(
                    r#"<x14:cfvo type="formula"{gte}><xm:f>{}</xm:f></x14:cfvo>"#,
                    escape_xml(f)
                ),
            }
        })
        .collect();
    let icon_refs: String = icons_colors
        .iter()
        .map(|(icon, color)| {
            let (set, id) = icon_to_excel_ref(icon, color);
            format!(r#"<x14:cfIcon iconSet="{set}" iconId="{id}"/>"#)
        })
        .collect();
    format!(
        r#"<x14:conditionalFormatting xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main"><x14:cfRule type="iconSet" priority="{excel_priority}"><x14:iconSet custom="1"{show_val}>{cfvo_xml}{icon_refs}</x14:iconSet></x14:cfRule><xm:sqref>{range}</xm:sqref></x14:conditionalFormatting>"#
    )
}

fn make_guid(idx: usize) -> String {
    format!("{{{:08X}-0000-0000-0000-000000000000}}", idx + 1)
}

fn stop_if_true_attr(stop: bool) -> &'static str {
    if stop {
        r#" stopIfTrue="1""#
    } else {
        ""
    }
}

fn build_cf_rule_xml(
    cf: &ConditionalFormatting,
    excel_priority: u32,
    x14_parts: &mut Vec<String>,
    databar_guid_idx: &mut usize,
) -> Option<String> {
    // remove $ from range for XML output
    let range = &cf.range.replace('$', "");
    let rule = match &cf.cf_rule {
        CfRule::CellIs {
            operator,
            formula,
            formula2,
            dxf_id,
            stop_if_true,
        } => {
            let op = value_operator_to_str(operator);
            let sif = stop_if_true_attr(*stop_if_true);
            let f1 = format!("<formula>{}</formula>", escape_xml(formula));
            let f2 = formula2
                .as_deref()
                .map(|f| format!("<formula>{}</formula>", escape_xml(f)))
                .unwrap_or_default();
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="cellIs" dxfId="{dxf_id}" priority="{excel_priority}"{sif} operator="{op}">{f1}{f2}</cfRule></conditionalFormatting>"#
            )
        }
        CfRule::Formula {
            formula,
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let body = formula.trim().strip_prefix('=').unwrap_or(formula.trim());
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="expression" dxfId="{dxf_id}" priority="{excel_priority}"{sif}><formula>{}</formula></cfRule></conditionalFormatting>"#,
                escape_xml(body)
            )
        }
        CfRule::DuplicateValues {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="duplicateValues" dxfId="{dxf_id}" priority="{excel_priority}"{sif}/></conditionalFormatting>"#
            )
        }
        CfRule::UniqueValues {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="uniqueValues" dxfId="{dxf_id}" priority="{excel_priority}"{sif}/></conditionalFormatting>"#
            )
        }
        CfRule::AboveAverage {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="aboveAverage" dxfId="{dxf_id}" priority="{excel_priority}"{sif}/></conditionalFormatting>"#
            )
        }
        CfRule::BelowAverage {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="aboveAverage" dxfId="{dxf_id}" priority="{excel_priority}" aboveAverage="0"{sif}/></conditionalFormatting>"#
            )
        }
        CfRule::Top10 {
            rank,
            percent,
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let pct = if *percent { r#" percent="1""# } else { "" };
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="top10" dxfId="{dxf_id}" priority="{excel_priority}"{sif}{pct} rank="{rank}"/></conditionalFormatting>"#
            )
        }
        CfRule::Bottom10 {
            rank,
            percent,
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let pct = if *percent { r#" percent="1""# } else { "" };
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="top10" dxfId="{dxf_id}" priority="{excel_priority}"{sif}{pct} rank="{rank}" bottom="1"/></conditionalFormatting>"#
            )
        }
        CfRule::Text {
            operator,
            value,
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let type_str = text_operator_to_type(operator);
            let anchor = anchor_cell_from_sqref(range);
            let formula = text_formula_str(anchor, operator, value);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="{type_str}" dxfId="{dxf_id}" priority="{excel_priority}"{sif} text="{}"><formula>{}</formula></cfRule></conditionalFormatting>"#,
                escape_xml(value),
                escape_xml(&formula),
            )
        }
        CfRule::TimePeriod {
            time_period,
            dxf_id,
            stop_if_true,
            ..
        } => {
            let period_str = period_type_to_str(time_period)?;
            let sif = stop_if_true_attr(*stop_if_true);
            let anchor = anchor_cell_from_sqref(range);
            let formula = time_period_formula_str(anchor, time_period);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="timePeriod" dxfId="{dxf_id}" priority="{excel_priority}"{sif} timePeriod="{period_str}"><formula>{}</formula></cfRule></conditionalFormatting>"#,
                escape_xml(&formula),
            )
        }
        CfRule::Blanks {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let anchor = anchor_cell_from_sqref(range);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="containsBlanks" dxfId="{dxf_id}" priority="{excel_priority}"{sif}><formula>LEN(TRIM({anchor}))=0</formula></cfRule></conditionalFormatting>"#
            )
        }
        CfRule::NotBlanks {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let anchor = anchor_cell_from_sqref(range);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="notContainsBlanks" dxfId="{dxf_id}" priority="{excel_priority}"{sif}><formula>LEN(TRIM({anchor}))&gt;0</formula></cfRule></conditionalFormatting>"#
            )
        }
        CfRule::Errors {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let anchor = anchor_cell_from_sqref(range);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="containsErrors" dxfId="{dxf_id}" priority="{excel_priority}"{sif}><formula>ISERROR({anchor})</formula></cfRule></conditionalFormatting>"#
            )
        }
        CfRule::NoErrors {
            dxf_id,
            stop_if_true,
        } => {
            let sif = stop_if_true_attr(*stop_if_true);
            let anchor = anchor_cell_from_sqref(range);
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="notContainsErrors" dxfId="{dxf_id}" priority="{excel_priority}"{sif}><formula>NOT(ISERROR({anchor}))</formula></cfRule></conditionalFormatting>"#
            )
        }
        CfRule::ColorScale { thresholds } => {
            let cfvos: String = thresholds.iter().map(|t| cfvo_xml(&t.cfvo)).collect();
            let colors: String = thresholds.iter().map(|t| color_rgb_xml(&t.color)).collect();
            format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="colorScale" priority="{excel_priority}"><colorScale>{cfvos}{colors}</colorScale></cfRule></conditionalFormatting>"#
            )
        }
        CfRule::DataBar {
            min,
            max,
            positive_color,
            negative_color,
            is_gradient,
            show_value,
        } => {
            let guid = make_guid(*databar_guid_idx);
            *databar_guid_idx += 1;

            let min_cfvo = min
                .as_ref()
                .map(cfvo_xml)
                .unwrap_or_else(|| r#"<cfvo type="min"/>"#.to_string());
            let max_cfvo = max
                .as_ref()
                .map(cfvo_xml)
                .unwrap_or_else(|| r#"<cfvo type="max"/>"#.to_string());
            let pos_color = color_rgb_xml(positive_color);
            let show_val = if !show_value { r#" showValue="0""# } else { "" };

            let ext_ref = format!(
                r#"<extLst><ext uri="{{B025F937-C7B1-47D3-B67F-A62EFF666E3E}}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:id>{guid}</x14:id></ext></extLst>"#
            );
            let main = format!(
                r#"<conditionalFormatting sqref="{range}"><cfRule type="dataBar" priority="{excel_priority}"><dataBar{show_val}>{min_cfvo}{max_cfvo}{pos_color}</dataBar>{ext_ref}</cfRule></conditionalFormatting>"#
            );

            let gradient_attr = if !is_gradient { r#" gradient="0""# } else { "" };
            let x14_min = min
                .as_ref()
                .map(cfvo_x14_xml)
                .unwrap_or_else(|| r#"<x14:cfvo type="autoMin"/>"#.to_string());
            let x14_max = max
                .as_ref()
                .map(cfvo_x14_xml)
                .unwrap_or_else(|| r#"<x14:cfvo type="autoMax"/>"#.to_string());
            let neg_color_xml = format!(
                r#"<x14:negativeFillColor rgb="FF{}"/>"#,
                negative_color.trim_start_matches('#')
            );
            let x14_entry = format!(
                r#"<x14:conditionalFormatting xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main"><x14:cfRule type="dataBar" id="{guid}"><x14:dataBar minLength="0" maxLength="100"{gradient_attr}>{x14_min}{x14_max}{neg_color_xml}<x14:axisColor rgb="FF000000"/></x14:dataBar></x14:cfRule><xm:sqref>{range}</xm:sqref></x14:conditionalFormatting>"#
            );
            x14_parts.push(x14_entry);
            main
        }
        CfRule::IconSet {
            thresholds,
            show_value,
        } => {
            if let Some(set_name) = icon_set_name_from_thresholds(thresholds) {
                // Known named set → standard conditionalFormatting element.
                let show_val = if !show_value { r#" showValue="0""# } else { "" };
                let cfvos: String = thresholds
                    .iter()
                    .map(|t| cfvo_with_gte_xml(&t.cfvo, t.is_strict))
                    .collect();
                format!(
                    r#"<conditionalFormatting sqref="{range}"><cfRule type="iconSet" priority="{excel_priority}"><iconSet iconSet="{set_name}"{show_val}>{cfvos}</iconSet></cfRule></conditionalFormatting>"#
                )
            } else {
                // Custom / unrecognized icon set → x14 custom block only.
                let cfvos_with_strict: Vec<(Cfvo, bool)> = thresholds
                    .iter()
                    .map(|t| (t.cfvo.clone(), t.is_strict))
                    .collect();
                let icons_colors: Vec<(Icon, String)> = thresholds
                    .iter()
                    .map(|t| (t.icon.clone(), t.color.clone()))
                    .collect();
                x14_parts.push(build_x14_custom_icon_xml(
                    range,
                    excel_priority,
                    &cfvos_with_strict,
                    &icons_colors,
                    *show_value,
                ));
                return None;
            }
        }
        CfRule::IconRating {
            icon,
            color,
            thresholds,
            show_value,
        } => {
            if let Some(set_name) = icon_rating_set_name(icon, thresholds.len()) {
                // Known rating set → standard conditionalFormatting element.
                let show_val = if !show_value { r#" showValue="0""# } else { "" };
                let cfvos: String = thresholds
                    .iter()
                    .map(|(cfvo, is_strict)| cfvo_with_gte_xml(cfvo, *is_strict))
                    .collect();
                format!(
                    r#"<conditionalFormatting sqref="{range}"><cfRule type="iconSet" priority="{excel_priority}"><iconSet iconSet="{set_name}"{show_val}>{cfvos}</iconSet></cfRule></conditionalFormatting>"#
                )
            } else {
                // Unrecognized rating → emit as x14 custom with repeated icon references.
                let cfvos_with_strict: Vec<(Cfvo, bool)> =
                    thresholds.iter().map(|(c, s)| (c.clone(), *s)).collect();
                let icons_colors: Vec<(Icon, String)> = thresholds
                    .iter()
                    .map(|_| (icon.clone(), color.clone()))
                    .collect();
                x14_parts.push(build_x14_custom_icon_xml(
                    range,
                    excel_priority,
                    &cfvos_with_strict,
                    &icons_colors,
                    *show_value,
                ));
                return None;
            }
        }
    };
    Some(rule)
}

/// Returns `(cf_sections_xml, ext_lst_xml)`.
/// `cf_sections_xml` contains all the `<conditionalFormatting>` blocks.
/// `ext_lst_xml` contains the `<extLst>` block for x14 DataBar extensions (may be empty).
pub(crate) fn get_conditional_formatting_xml(cfs: &[ConditionalFormatting]) -> (String, String) {
    if cfs.is_empty() {
        return (String::new(), String::new());
    }
    let max_p = cfs.iter().map(|cf| cf.priority).max().unwrap_or(1);
    let mut main_parts: Vec<String> = Vec::new();
    let mut x14_parts: Vec<String> = Vec::new();
    let mut databar_guid_idx: usize = 0;

    for cf in cfs {
        let excel_priority = max_p + 1 - cf.priority;
        if let Some(xml) =
            build_cf_rule_xml(cf, excel_priority, &mut x14_parts, &mut databar_guid_idx)
        {
            main_parts.push(xml);
        }
    }

    let main_xml = main_parts.join("");
    let ext_lst_xml = if x14_parts.is_empty() {
        String::new()
    } else {
        let inner = x14_parts.join("");
        format!(
            r#"<extLst><ext uri="{{78C0D931-6437-407d-A8EE-F0AAD7539E65}}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:conditionalFormattings>{inner}</x14:conditionalFormattings></ext></extLst>"#
        )
    };
    (main_xml, ext_lst_xml)
}
