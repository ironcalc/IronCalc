#![allow(clippy::unwrap_used)]

use ironcalc_base::colors::get_indexed_color;
use roxmltree::{ExpandedName, Node};

use crate::error::XlsxError;

use ironcalc_base::types::{Color, Theme};

pub(crate) fn get_number(node: Node, s: &str) -> i32 {
    node.attribute(s).unwrap_or("0").parse::<i32>().unwrap_or(0)
}

#[inline]
pub(super) fn get_attribute<'a, 'n, 'm, N>(
    node: &'a Node,
    attr_name: N,
) -> Result<&'a str, XlsxError>
where
    N: Into<ExpandedName<'n, 'm>>,
{
    let attr_name = attr_name.into();
    node.attribute(attr_name)
        .ok_or_else(|| XlsxError::Xml(format!("Missing \"{attr_name:?}\" XML attribute")))
}

pub(super) fn get_value_or_default(node: &Node, tag_name: &str, default: &str) -> String {
    let application_nodes = node
        .children()
        .filter(|n| n.has_tag_name(tag_name))
        .collect::<Vec<Node>>();
    if application_nodes.len() == 1 {
        application_nodes[0].text().unwrap_or(default).to_string()
    } else {
        default.to_string()
    }
}

pub(super) fn get_color(node: Node, _theme: &Theme) -> Result<Color, XlsxError> {
    // 18.3.1.15 color (Data Bar Color)
    if node.has_attribute("rgb") {
        let raw = node.attribute("rgb").unwrap();
        // Strip leading alpha byte from ARGB (e.g. "FF4472C4" → "#4472C4")
        let hex = if raw.len() == 8 {
            format!("#{}", raw[2..].to_ascii_uppercase())
        } else {
            format!("#{}", raw.to_ascii_uppercase())
        };
        Ok(Color::Rgb(hex))
    } else if node.has_attribute("indexed") {
        let index = node.attribute("indexed").unwrap().parse::<i32>()?;
        if index == 64 {
            // 64 is "transparent" in OOXML, but we don't have a good way to represent that, so we'll just ignore it
            return Ok(Color::None);
        }
        let rgb = get_indexed_color(index);
        Ok(Color::Rgb(rgb))
    } else if node.has_attribute("theme") {
        let theme_index = node.attribute("theme").unwrap().parse::<i32>()?;
        let tint = match node.attribute("tint") {
            Some(t) => t.parse::<f64>().unwrap_or(0.0),
            None => 0.0,
        };
        Ok(Color::Theme(theme_index, tint))
    } else if node.has_attribute("auto") {
        Ok(Color::None)
    } else {
        println!("Unexpected color node {node:?}");
        Ok(Color::None)
    }
}

pub(super) fn get_bool(node: Node, s: &str) -> bool {
    // defaults to true
    !matches!(node.attribute(s), Some("0"))
}

pub(super) fn get_bool_false(node: Node, s: &str) -> bool {
    // defaults to false
    matches!(node.attribute(s), Some("1"))
}
