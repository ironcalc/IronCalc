// https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_cfRule_topic_ID0EFKO4.html

use std::collections::HashMap;

use ironcalc_base::cf_types::{
    icon_set_icons, CfRule, Cfvo, ColorScaleThreshold, ConditionalFormatting, Icon, IconThreshold,
    PeriodType, TextOperator, ValueOperator,
};
use roxmltree::Node;

use crate::error::XlsxError;

use super::{
    theme::Theme,
    util::{get_attribute, get_color},
};

fn parse_cfvo(node: Node) -> Result<Cfvo, XlsxError> {
    let val = node.attribute("val").unwrap_or("0");
    match node.attribute("type").unwrap_or("num") {
        "min" => Ok(Cfvo::Min),
        "max" => Ok(Cfvo::Max),
        "num" => Ok(Cfvo::Number(val.parse::<f64>().unwrap_or(0.0))),
        "percent" => Ok(Cfvo::Percent(val.parse::<f64>().unwrap_or(0.0))),
        "percentile" => Ok(Cfvo::Percentile(val.parse::<f64>().unwrap_or(0.0))),
        "formula" => Ok(Cfvo::Formula(val.to_string())),
        // autoMin/autoMax are Excel 2010+ extensions; treat as Min/Max
        "autoMin" => Ok(Cfvo::Min),
        "autoMax" => Ok(Cfvo::Max),
        other => Err(XlsxError::Xml(format!("Unknown cfvo type: {other}"))),
    }
}

/// Parses an x14-namespace cfvo node where autoMin/autoMax map to `None`
/// (automatic axis) and the value lives in a child `<xm:f>` element.
fn parse_cfvo_x14(node: Node) -> Option<Cfvo> {
    let cfvo_type = node.attribute("type").unwrap_or("num");
    // autoMin / autoMax → None signals "use automatic axis"
    if cfvo_type == "autoMin" || cfvo_type == "autoMax" {
        return None;
    }
    let val = node
        .children()
        .find(|n| n.has_tag_name("f"))
        .and_then(|n| n.text())
        .unwrap_or("0");
    match cfvo_type {
        "min" => Some(Cfvo::Min),
        "max" => Some(Cfvo::Max),
        "num" => Some(Cfvo::Number(val.parse::<f64>().unwrap_or(0.0))),
        "percent" => Some(Cfvo::Percent(val.parse::<f64>().unwrap_or(0.0))),
        "percentile" => Some(Cfvo::Percentile(val.parse::<f64>().unwrap_or(0.0))),
        "formula" => Some(Cfvo::Formula(val.to_string())),
        _ => None,
    }
}

/// Extended data bar properties from the x14:dataBar section.
struct DataBarExt {
    /// `None` = automatic axis (autoMin/autoMax).
    min: Option<Cfvo>,
    max: Option<Cfvo>,
    negative_color: String,
    is_gradient: bool,
}

/// Pre-parses the worksheet-level `<extLst>` for x14 extended data bar rules.
/// Returns a map keyed by the x14:id GUID (e.g. "{7896903A-...}").
fn parse_x14_data_bars(ws: Node, theme: &Theme) -> HashMap<String, DataBarExt> {
    let mut map = HashMap::new();
    for ext_lst in ws.children().filter(|n| n.has_tag_name("extLst")) {
        for ext in ext_lst.children().filter(|n| n.has_tag_name("ext")) {
            for cfs in ext
                .children()
                .filter(|n| n.has_tag_name("conditionalFormattings"))
            {
                for cf in cfs
                    .children()
                    .filter(|n| n.has_tag_name("conditionalFormatting"))
                {
                    for rule in cf.children().filter(|n| n.has_tag_name("cfRule")) {
                        if rule.attribute("type") != Some("dataBar") {
                            continue;
                        }
                        let id = match rule.attribute("id") {
                            Some(id) => id.to_string(),
                            None => continue,
                        };
                        let db_node = match rule.children().find(|n| n.has_tag_name("dataBar")) {
                            Some(n) => n,
                            None => continue,
                        };
                        let is_gradient = db_node.attribute("gradient") != Some("0");
                        let mut cfvo_iter = db_node.children().filter(|n| n.has_tag_name("cfvo"));
                        let min = cfvo_iter.next().map(parse_cfvo_x14).unwrap_or(None);
                        let max = cfvo_iter.next().map(parse_cfvo_x14).unwrap_or(None);
                        let negative_color = db_node
                            .children()
                            .find(|n| n.has_tag_name("negativeFillColor"))
                            .and_then(|n| get_color(n, theme).ok().flatten())
                            .unwrap_or_else(|| "#FF0000".to_string());
                        map.insert(
                            id,
                            DataBarExt {
                                min,
                                max,
                                negative_color,
                                is_gradient,
                            },
                        );
                    }
                }
            }
        }
    }
    map
}

/// Returns the (Icon, color) pair for an Excel (iconSet, iconId) reference.
/// iconId is 0-based from the lowest bucket to the highest.
/// Unrecognized sets (e.g. 5Signal) fall back to Circle.
fn icon_from_excel_id(icon_set: &str, icon_id: u32) -> (Icon, String) {
    let s = |c: &'static str| c.to_string();
    // Standard sets are already in icon_set_icons, indexed from lowest to highest.
    if let Some(icons) = icon_set_icons(icon_set) {
        if let Some((icon, color)) = icons.get(icon_id as usize) {
            return (icon.clone(), color.clone());
        }
    }
    // Rating/extension sets not covered by icon_set_icons.
    match icon_set {
        "3Stars" => match icon_id {
            0 => (Icon::Star, s("#808080")),
            _ => (Icon::Star, s("#FFD700")),
        },
        "3Rating" => match icon_id {
            0 => (Icon::FlatRectangle, s("#808080")),
            1 => (Icon::FlatRectangle, s("#ffeb84")),
            _ => (Icon::FlatRectangle, s("#4472C4")),
        },
        "4Rating" => match icon_id {
            0 => (Icon::FlatRectangle, s("#808080")),
            1 | 2 => (Icon::FlatRectangle, s("#ffeb84")),
            _ => (Icon::FlatRectangle, s("#4472C4")),
        },
        "5Rating" => match icon_id {
            0 => (Icon::FlatRectangle, s("#808080")),
            1..=3 => (Icon::FlatRectangle, s("#ffeb84")),
            _ => (Icon::FlatRectangle, s("#4472C4")),
        },
        "5Quarters" => match icon_id {
            0 => (Icon::Circle, s("#808080")),
            1..=3 => (Icon::Circle, s("#ffeb84")),
            _ => (Icon::Circle, s("#FFD700")),
        },
        "4Boxes" => match icon_id {
            0 => (Icon::FlatRectangle, s("#808080")),
            1 | 2 => (Icon::FlatRectangle, s("#ffeb84")),
            _ => (Icon::FlatRectangle, s("#4472C4")),
        },
        "5Boxes" => match icon_id {
            0 => (Icon::FlatRectangle, s("#808080")),
            1..=3 => (Icon::FlatRectangle, s("#ffeb84")),
            _ => (Icon::FlatRectangle, s("#4472C4")),
        },
        // 5Signal and any unknown set → substitute with Circle
        _ => match icon_id {
            0 => (Icon::Circle, s("#808080")),
            _ => (Icon::Circle, s("#63be7b")),
        },
    }
}

/// Returns the representative (Icon, color) used when displaying a rating type
/// as repeated icons (e.g. ★★☆ for 2 out of 3 stars).
fn rating_icon_color(icon_set_attr: &str) -> (Icon, String) {
    let s = |c: &'static str| c.to_string();
    match icon_set_attr {
        "3Stars" => (Icon::Star, s("#FFD700")),
        "5Quarters" => (Icon::Circle, s("#FFD700")),
        _ => (Icon::FlatRectangle, s("#4472C4")), // 3Rating, 4Rating, 5Rating, 5Boxes, 4Boxes
    }
}

/// Parses x14 extended icon-set rules from the worksheet's `<extLst>`.
/// Returns the resulting ConditionalFormatting entries (range + priority embedded).
fn parse_x14_icon_sets(ws: Node) -> Vec<ConditionalFormatting> {
    let mut result = Vec::new();
    for ext_lst in ws.children().filter(|n| n.has_tag_name("extLst")) {
        for ext in ext_lst.children().filter(|n| n.has_tag_name("ext")) {
            for cfs in ext
                .children()
                .filter(|n| n.has_tag_name("conditionalFormattings"))
            {
                for cf in cfs
                    .children()
                    .filter(|n| n.has_tag_name("conditionalFormatting"))
                {
                    // Range lives in <xm:sqref> (local name "sqref")
                    let range = match cf.children().find(|n| n.has_tag_name("sqref")) {
                        Some(n) => match n.text() {
                            Some(t) => t.to_string(),
                            None => continue,
                        },
                        None => continue,
                    };

                    for rule in cf.children().filter(|n| n.has_tag_name("cfRule")) {
                        if rule.attribute("type") != Some("iconSet") {
                            continue;
                        }
                        let priority = rule
                            .attribute("priority")
                            .unwrap_or("0")
                            .parse::<u32>()
                            .unwrap_or(0);
                        let is_node = match rule.children().find(|n| n.has_tag_name("iconSet")) {
                            Some(n) => n,
                            None => continue,
                        };

                        let icon_set_attr =
                            is_node.attribute("iconSet").unwrap_or("3TrafficLights1");
                        let is_custom = is_node.attribute("custom") == Some("1");
                        let show_value = is_node.attribute("showValue") != Some("0");

                        // cfvo values live in <xm:f> children.
                        // is_strict=true → >= (gte="1", default); is_strict=false → > (gte="0")
                        let cfvo_with_strict: Vec<(Cfvo, bool)> = is_node
                            .children()
                            .filter(|n| n.has_tag_name("cfvo"))
                            .filter_map(|n| {
                                let cfvo = parse_cfvo_x14(n)?;
                                let is_strict = n.attribute("gte") != Some("0");
                                Some((cfvo, is_strict))
                            })
                            .collect();

                        if cfvo_with_strict.is_empty() {
                            continue;
                        }

                        let cf_rule = if is_custom {
                            // Explicit per-icon overrides via <x14:cfIcon iconSet="..." iconId="..."/>
                            let icon_list: Vec<(Icon, String)> = is_node
                                .children()
                                .filter(|n| n.has_tag_name("cfIcon"))
                                .map(|n| {
                                    let set = n.attribute("iconSet").unwrap_or("3TrafficLights1");
                                    let id = n
                                        .attribute("iconId")
                                        .and_then(|s| s.parse::<u32>().ok())
                                        .unwrap_or(0);
                                    icon_from_excel_id(set, id)
                                })
                                .collect();

                            if icon_list.is_empty() {
                                continue;
                            }

                            let thresholds: Vec<IconThreshold> = icon_list
                                .into_iter()
                                .zip(cfvo_with_strict)
                                .map(|((icon, color), (cfvo, is_strict))| IconThreshold {
                                    icon,
                                    cfvo,
                                    color,
                                    is_strict,
                                })
                                .collect();

                            CfRule::IconSet {
                                thresholds,
                                show_value,
                            }
                        } else if rating_count(icon_set_attr).is_some() {
                            let (icon, color) = rating_icon_color(icon_set_attr);
                            CfRule::IconRating {
                                icon,
                                color,
                                thresholds: cfvo_with_strict,
                                show_value,
                            }
                        } else {
                            let icon_colors = match icon_set_icons(icon_set_attr) {
                                Some(v) => v,
                                None => continue,
                            };
                            let thresholds: Vec<IconThreshold> = icon_colors
                                .into_iter()
                                .zip(cfvo_with_strict)
                                .map(|((icon, color), (cfvo, is_strict))| IconThreshold {
                                    icon,
                                    cfvo,
                                    color,
                                    is_strict,
                                })
                                .collect();
                            CfRule::IconSet {
                                thresholds,
                                show_value,
                            }
                        };

                        result.push(ConditionalFormatting {
                            range: range.clone(),
                            cf_rule,
                            priority,
                        });
                    }
                }
            }
        }
    }
    result
}

fn parse_operator(s: &str) -> Result<ValueOperator, XlsxError> {
    match s {
        "equal" => Ok(ValueOperator::Equal),
        "greaterThan" => Ok(ValueOperator::GreaterThan),
        "greaterThanOrEqual" => Ok(ValueOperator::GreaterThanOrEqual),
        "lessThan" => Ok(ValueOperator::LessThan),
        "lessThanOrEqual" => Ok(ValueOperator::LessThanOrEqual),
        "notEqual" => Ok(ValueOperator::NotEqual),
        "between" => Ok(ValueOperator::Between),
        "notBetween" => Ok(ValueOperator::NotBetween),
        other => Err(XlsxError::Xml(format!("Unknown cellIs operator: {other}"))),
    }
}

fn parse_text_operator(s: &str) -> Option<TextOperator> {
    match s {
        "containsText" => Some(TextOperator::Contains),
        "notContainsText" => Some(TextOperator::DoesNotContain),
        "beginsWith" => Some(TextOperator::BeginsWith),
        "endsWith" => Some(TextOperator::EndsWith),
        _ => None,
    }
}

fn parse_period_type(s: &str) -> Option<PeriodType> {
    match s {
        "yesterday" => Some(PeriodType::Yesterday),
        "today" => Some(PeriodType::Today),
        "tomorrow" => Some(PeriodType::Tomorrow),
        "last7Days" => Some(PeriodType::Last7Days),
        "next7Days" => Some(PeriodType::Next7Days),
        "lastWeek" => Some(PeriodType::LastWeek),
        "thisWeek" => Some(PeriodType::ThisWeek),
        "nextWeek" => Some(PeriodType::NextWeek),
        "lastMonth" => Some(PeriodType::LastMonth),
        "thisMonth" => Some(PeriodType::ThisMonth),
        "nextMonth" => Some(PeriodType::NextMonth),
        "lastYear" => Some(PeriodType::LastYear),
        "thisYear" => Some(PeriodType::ThisYear),
        "nextYear" => Some(PeriodType::NextYear),
        _ => None,
    }
}

fn rating_count(icon_set_attr: &str) -> Option<u32> {
    match icon_set_attr {
        "3Stars" | "3Rating" => Some(3),
        "4Rating" | "4Boxes" => Some(4),
        "5Quarters" | "5Boxes" | "5Rating" => Some(5),
        _ => None,
    }
}

pub(super) fn load_conditional_formatting(
    ws: Node,
    theme: &Theme,
) -> Result<Vec<ConditionalFormatting>, XlsxError> {
    let x14_data_bars = parse_x14_data_bars(ws, theme);
    let mut result = Vec::new();

    for cf in ws
        .children()
        .filter(|n| n.has_tag_name("conditionalFormatting"))
    {
        let range = get_attribute(&cf, "sqref")?.to_string();

        for cf_rule in cf.children().filter(|n| n.has_tag_name("cfRule")) {
            let priority = cf_rule
                .attribute("priority")
                .unwrap_or("0")
                .parse::<u32>()
                .unwrap_or(0);
            let dxf_id = cf_rule
                .attribute("dxfId")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            let stop_if_true = cf_rule.attribute("stopIfTrue") == Some("1");

            let cf_type = match cf_rule.attribute("type") {
                Some(t) => t,
                None => continue,
            };

            let rule = match cf_type {
                "colorScale" => {
                    let cs_nodes: Vec<Node> = cf_rule
                        .children()
                        .filter(|n| n.has_tag_name("colorScale"))
                        .collect();
                    if cs_nodes.is_empty() {
                        continue;
                    }
                    let mut cfvo_list: Vec<Cfvo> = Vec::new();
                    let mut color_list: Vec<String> = Vec::new();
                    for child in cs_nodes[0].children() {
                        match child.tag_name().name() {
                            "cfvo" => cfvo_list.push(parse_cfvo(child)?),
                            "color" => {
                                if let Some(c) = get_color(child, theme)? {
                                    color_list.push(c);
                                }
                            }
                            _ => {}
                        }
                    }
                    let thresholds = cfvo_list
                        .into_iter()
                        .zip(color_list)
                        .map(|(cfvo, color)| ColorScaleThreshold { cfvo, color })
                        .collect();
                    CfRule::ColorScale { thresholds }
                }
                "cellIs" => {
                    let operator = parse_operator(cf_rule.attribute("operator").unwrap_or(""))?;
                    let formulas: Vec<String> = cf_rule
                        .children()
                        .filter(|n| n.has_tag_name("formula"))
                        .filter_map(|n| n.text().map(|s| s.to_string()))
                        .collect();
                    let formula = formulas.first().cloned().unwrap_or_default();
                    let formula2 = formulas.get(1).cloned();
                    CfRule::CellIs {
                        operator,
                        formula,
                        formula2,
                        dxf_id,
                        stop_if_true,
                    }
                }
                "duplicateValues" => CfRule::DuplicateValues {
                    dxf_id,
                    stop_if_true,
                },
                "expression" => {
                    let formula = cf_rule
                        .children()
                        .find(|n| n.has_tag_name("formula"))
                        .and_then(|n| n.text())
                        .unwrap_or("")
                        .to_string();
                    CfRule::Formula {
                        formula: format!("={}", formula),
                        dxf_id,
                        stop_if_true,
                    }
                }
                "aboveAverage" => {
                    if cf_rule.attribute("aboveAverage") == Some("0") {
                        CfRule::BelowAverage {
                            dxf_id,
                            stop_if_true,
                        }
                    } else {
                        CfRule::AboveAverage {
                            dxf_id,
                            stop_if_true,
                        }
                    }
                }
                "top10" => {
                    let rank = cf_rule
                        .attribute("rank")
                        .unwrap_or("10")
                        .parse::<u32>()
                        .unwrap_or(10);
                    let percent = cf_rule.attribute("percent") == Some("1");
                    if cf_rule.attribute("bottom") == Some("1") {
                        CfRule::Bottom10 {
                            rank,
                            percent,
                            dxf_id,
                            stop_if_true,
                        }
                    } else {
                        CfRule::Top10 {
                            rank,
                            percent,
                            dxf_id,
                            stop_if_true,
                        }
                    }
                }
                "dataBar" => {
                    let db_nodes: Vec<Node> = cf_rule
                        .children()
                        .filter(|n| n.has_tag_name("dataBar"))
                        .collect();
                    if db_nodes.is_empty() {
                        continue;
                    }
                    let db = db_nodes[0];
                    let show_value = db.attribute("showValue") != Some("0");
                    let mut simple_cfvos: Vec<Cfvo> = Vec::new();
                    let mut positive_color = String::new();
                    for child in db.children() {
                        match child.tag_name().name() {
                            "cfvo" => simple_cfvos.push(parse_cfvo(child)?),
                            "color" => {
                                if let Some(c) = get_color(child, theme)? {
                                    positive_color = c;
                                }
                            }
                            _ => {}
                        }
                    }
                    // Prefer extended x14 info (negative color, gradient, auto axes).
                    // Fall back to simple cfvo and sensible defaults when absent.
                    let x14_id = cf_rule
                        .children()
                        .find(|n| n.has_tag_name("extLst"))
                        .and_then(|el| el.children().find(|n| n.has_tag_name("ext")))
                        .and_then(|ex| ex.children().find(|n| n.has_tag_name("id")))
                        .and_then(|id| id.text())
                        .map(|s| s.to_string());
                    let (min, max, negative_color, is_gradient) =
                        match x14_id.as_deref().and_then(|id| x14_data_bars.get(id)) {
                            Some(ext) => (
                                ext.min.clone(),
                                ext.max.clone(),
                                ext.negative_color.clone(),
                                ext.is_gradient,
                            ),
                            None => (
                                simple_cfvos.first().cloned(),
                                simple_cfvos.get(1).cloned(),
                                "#FF0000".to_string(),
                                true,
                            ),
                        };
                    CfRule::DataBar {
                        min,
                        max,
                        positive_color,
                        negative_color,
                        is_gradient,
                        show_value,
                    }
                }
                "iconSet" => {
                    let is_nodes: Vec<Node> = cf_rule
                        .children()
                        .filter(|n| n.has_tag_name("iconSet"))
                        .collect();
                    if is_nodes.is_empty() {
                        continue;
                    }
                    let is_node = is_nodes[0];
                    // Missing iconSet attribute defaults to 3TrafficLights1 per OOXML spec.
                    let icon_set_attr = is_node.attribute("iconSet").unwrap_or("3TrafficLights1");
                    let show_value = is_node.attribute("showValue") != Some("0");

                    // Collect cfvo nodes.
                    // is_strict=true → >= (gte="1", default); is_strict=false → > (gte="0")
                    let cfvo_nodes: Vec<Node> = is_node
                        .children()
                        .filter(|n| n.has_tag_name("cfvo"))
                        .collect();
                    let cfvo_vec: Vec<Cfvo> = cfvo_nodes
                        .iter()
                        .map(|n| parse_cfvo(*n))
                        .collect::<Result<Vec<_>, _>>()?;
                    let is_strict_vec: Vec<bool> = cfvo_nodes
                        .iter()
                        .map(|n| n.attribute("gte") != Some("0"))
                        .collect();

                    if rating_count(icon_set_attr).is_some() {
                        let (icon, color) = rating_icon_color(icon_set_attr);
                        let thresholds = cfvo_vec.into_iter().zip(is_strict_vec).collect();
                        CfRule::IconRating {
                            icon,
                            color,
                            thresholds,
                            show_value,
                        }
                    } else {
                        let icon_colors = match icon_set_icons(icon_set_attr) {
                            Some(v) => v,
                            None => continue,
                        };
                        let thresholds = icon_colors
                            .into_iter()
                            .zip(cfvo_vec.into_iter().zip(is_strict_vec))
                            .map(|((icon, color), (cfvo, is_strict))| IconThreshold {
                                icon,
                                cfvo,
                                color,
                                is_strict,
                            })
                            .collect();
                        CfRule::IconSet {
                            thresholds,
                            show_value,
                        }
                    }
                }
                "containsBlanks" => CfRule::Blanks {
                    dxf_id,
                    stop_if_true,
                },
                "notContainsBlanks" => CfRule::NotBlanks {
                    dxf_id,
                    stop_if_true,
                },
                "containsErrors" => CfRule::Errors {
                    dxf_id,
                    stop_if_true,
                },
                "notContainsErrors" => CfRule::NoErrors {
                    dxf_id,
                    stop_if_true,
                },
                "containsText" | "notContainsText" | "beginsWith" | "endsWith" => {
                    let raw_type = cf_rule.attribute("type").unwrap_or("");
                    let operator = match parse_text_operator(raw_type) {
                        Some(op) => op,
                        None => continue,
                    };
                    let value = cf_rule.attribute("text").unwrap_or("").to_string();
                    CfRule::Text {
                        operator,
                        value,
                        dxf_id,
                        stop_if_true,
                    }
                }
                "uniqueValues" => CfRule::UniqueValues {
                    dxf_id,
                    stop_if_true,
                },
                "timePeriod" => {
                    let period =
                        match parse_period_type(cf_rule.attribute("timePeriod").unwrap_or("")) {
                            Some(p) => p,
                            None => continue,
                        };
                    CfRule::TimePeriod {
                        time_period: period,
                        date1: None,
                        date2: None,
                        dxf_id,
                        stop_if_true,
                    }
                }
                // Skip unknown rule types silently
                _ => continue,
            };

            result.push(ConditionalFormatting {
                range: range.clone(),
                cf_rule: rule,
                priority,
            });
        }
    }

    result.extend(parse_x14_icon_sets(ws));

    // Excel: priority=1 is the most important rule (lowest number wins).
    // IronCalc: the highest priority number wins (new rules get max+1 and override old ones).
    // Reverse all priorities so that the Excel winner (1) maps to the IronCalc winner (max).
    if !result.is_empty() {
        let max_p = result.iter().map(|cf| cf.priority).max().unwrap_or(0);
        for cf in &mut result {
            cf.priority = max_p + 1 - cf.priority;
        }
    }

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn parse_ws(xml: &str) -> roxmltree::Document<'_> {
        roxmltree::Document::parse(xml).expect("invalid test XML")
    }

    fn dummy_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_priority_reversal_three_rules() {
        let xml = r#"<worksheet>
            <conditionalFormatting sqref="A1:A5">
                <cfRule type="duplicateValues" priority="1" dxfId="0"/>
                <cfRule type="uniqueValues"    priority="2" dxfId="1"/>
                <cfRule type="aboveAverage"    priority="3"/>
            </conditionalFormatting>
        </worksheet>"#;
        let doc = parse_ws(xml);
        let ws = doc.root_element();
        let rules = load_conditional_formatting(ws, &dummy_theme()).unwrap();
        assert_eq!(rules.len(), 3);
        // Excel priority=1 (most important) must map to the highest IronCalc number (3).
        let p: Vec<u32> = rules.iter().map(|r| r.priority).collect();
        assert!(p.contains(&3)); // was Excel priority=1
        assert!(p.contains(&2)); // was Excel priority=2
        assert!(p.contains(&1)); // was Excel priority=3
                                 // The rule that was priority=1 in Excel should now have priority=3.
        let dup = rules
            .iter()
            .find(|r| matches!(r.cf_rule, CfRule::DuplicateValues { .. }))
            .unwrap();
        assert_eq!(dup.priority, 3);
        let unique = rules
            .iter()
            .find(|r| matches!(r.cf_rule, CfRule::UniqueValues { .. }))
            .unwrap();
        assert_eq!(unique.priority, 2);
        let above = rules
            .iter()
            .find(|r| matches!(r.cf_rule, CfRule::AboveAverage { .. }))
            .unwrap();
        assert_eq!(above.priority, 1);
    }

    #[test]
    fn test_priority_reversal_single_rule() {
        let xml = r#"<worksheet>
            <conditionalFormatting sqref="B1:B10">
                <cfRule type="uniqueValues" priority="5" dxfId="0"/>
            </conditionalFormatting>
        </worksheet>"#;
        let doc = parse_ws(xml);
        let ws = doc.root_element();
        let rules = load_conditional_formatting(ws, &dummy_theme()).unwrap();
        assert_eq!(rules.len(), 1);
        // Single rule: max+1-priority = 5+1-5 = 1
        assert_eq!(rules[0].priority, 1);
    }

    #[test]
    fn test_priority_reversal_preserves_relative_order() {
        // After reversal the highest-Excel-priority rule must have the highest IronCalc priority.
        let xml = r#"<worksheet>
            <conditionalFormatting sqref="C1:C3">
                <cfRule type="aboveAverage" priority="1"/>
                <cfRule type="duplicateValues" priority="10" dxfId="0"/>
            </conditionalFormatting>
        </worksheet>"#;
        let doc = parse_ws(xml);
        let ws = doc.root_element();
        let rules = load_conditional_formatting(ws, &dummy_theme()).unwrap();
        assert_eq!(rules.len(), 2);
        let above = rules
            .iter()
            .find(|r| matches!(r.cf_rule, CfRule::AboveAverage { .. }))
            .unwrap();
        let dup = rules
            .iter()
            .find(|r| matches!(r.cf_rule, CfRule::DuplicateValues { .. }))
            .unwrap();
        // Excel priority=1 should be highest in IronCalc.
        assert!(above.priority > dup.priority);
    }
}
