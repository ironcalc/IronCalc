#![allow(clippy::unwrap_used)]

use std::borrow::Cow;

use colors::{get_indexed_color, get_themed_color};
use quick_xml::events::BytesStart;
use roxmltree::{ExpandedName, Node};

use crate::error::XlsxError;

use super::colors;

pub(crate) fn get_required_attribute<'a>(
    tag: &'a BytesStart,
    attr_name: &str,
) -> Result<Cow<'a, str>, XlsxError> {
    tag.try_get_attribute(attr_name)
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to parse attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })?
        .ok_or_else(|| {
            XlsxError::Xml(format!(
                "Missing required \"{:?}\" XML attribute",
                attr_name
            ))
        })?
        .unescape_value()
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to decode and unescape attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })
}

pub(crate) fn get_optional_attribute<'a>(
    tag: &'a BytesStart,
    attr_name: &str,
) -> Result<Option<Cow<'a, str>>, XlsxError> {
    tag.try_get_attribute(attr_name)
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to parse attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })?
        .map(|a| a.unescape_value())
        .transpose()
        .map_err(|e| {
            XlsxError::Xml(format!(
                "Unable to decode and unescape attribute: \"{:?}\": {:?}",
                attr_name,
                e.to_string()
            ))
        })
}

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
        .ok_or_else(|| XlsxError::Xml(format!("Missing \"{:?}\" XML attribute", attr_name)))
}

pub(super) fn get_color(node: Node) -> Result<Option<String>, XlsxError> {
    // 18.3.1.15 color (Data Bar Color)
    if node.has_attribute("rgb") {
        let mut val = node.attribute("rgb").unwrap().to_string();
        // FIXME the two first values is normally the alpha.
        if val.len() == 8 {
            val = format!("#{}", &val[2..8]);
        }
        Ok(Some(val))
    } else if node.has_attribute("indexed") {
        let index = node.attribute("indexed").unwrap().parse::<i32>()?;
        let rgb = get_indexed_color(index);
        Ok(Some(rgb))
    // Color::Indexed(val)
    } else if node.has_attribute("theme") {
        let theme = node.attribute("theme").unwrap().parse::<i32>()?;
        let tint = match node.attribute("tint") {
            Some(t) => t.parse::<f64>().unwrap_or(0.0),
            None => 0.0,
        };
        let rgb = get_themed_color(theme, tint);
        Ok(Some(rgb))
    // Color::Theme { theme, tint }
    } else if node.has_attribute("auto") {
        // TODO: Is this correct?
        // A boolean value indicating the color is automatic and system color dependent.
        Ok(None)
    } else {
        println!("Unexpected color node {:?}", node);
        Ok(None)
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
