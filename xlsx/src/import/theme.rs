use std::io::Read;

use ironcalc_base::types::Theme;
use roxmltree::Node;

use crate::error::XlsxError;

/// Reads the theme part at `path`. Returns `Theme::default()` if the workbook
/// has no theme relationship or if parsing fails.
pub(crate) fn load<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    path: Option<&str>,
) -> Theme {
    let Some(path) = path else {
        return Theme::default();
    };
    match try_load(archive, path) {
        Ok(theme) => theme,
        Err(e) => {
            eprintln!(
                "IronCalc: falling back to default theme palette (could not read {path}: {e})"
            );
            Theme::default()
        }
    }
}

// XML tag name → setter closure, in OOXML declaration order.
type Setter = fn(&mut Theme, String);
const SLOTS: [(&str, Setter); 12] = [
    ("dk1", |t, v| t.dk1 = v),
    ("lt1", |t, v| t.lt1 = v),
    ("dk2", |t, v| t.dk2 = v),
    ("lt2", |t, v| t.lt2 = v),
    ("accent1", |t, v| t.accent1 = v),
    ("accent2", |t, v| t.accent2 = v),
    ("accent3", |t, v| t.accent3 = v),
    ("accent4", |t, v| t.accent4 = v),
    ("accent5", |t, v| t.accent5 = v),
    ("accent6", |t, v| t.accent6 = v),
    ("hlink", |t, v| t.hlink = v),
    ("folHlink", |t, v| t.fol_hlink = v),
];

fn try_load<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    path: &str,
) -> Result<Theme, XlsxError> {
    let mut file = archive.by_name(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;

    let scheme = doc
        .descendants()
        .find(|n| n.has_tag_name("clrScheme"))
        .ok_or_else(|| XlsxError::Xml(format!("Missing clrScheme in {path}")))?;

    let theme_name = doc
        .descendants()
        .find(|n| n.has_tag_name("theme"))
        .and_then(|n| n.attribute("name"))
        .unwrap_or("Office Theme")
        .to_string();
    // strips "Theme" suffix if present, to avoid redundant "Office Theme Theme" default name.
    let theme_name = theme_name
        .strip_suffix(" Theme")
        .unwrap_or(&theme_name)
        .to_string();
    let mut theme = Theme {
        name: theme_name,
        ..Default::default()
    };

    for (tag, set) in &SLOTS {
        if let Some(slot) = scheme.children().find(|n| n.has_tag_name(*tag)) {
            if let Some(hex) = read_color(&slot) {
                set(&mut theme, hex);
            }
        }
    }

    Ok(theme)
}

fn read_color(slot: &Node) -> Option<String> {
    for child in slot.children().filter(|n| n.is_element()) {
        match child.tag_name().name() {
            "srgbClr" => {
                if let Some(val) = child.attribute("val") {
                    return Some(format_hex(val));
                }
            }
            "sysClr" => {
                if let Some(val) = child
                    .attribute("lastClr")
                    .or_else(|| child.attribute("val"))
                {
                    return Some(format_hex(val));
                }
            }
            _ => {}
        }
    }
    None
}

fn format_hex(raw: &str) -> String {
    let trimmed = raw.trim_start_matches('#');
    let rgb = if trimmed.len() == 8 {
        &trimmed[2..]
    } else {
        trimmed
    };
    format!("#{}", rgb.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_default_palette_matches_legacy_array() {
        let theme = Theme::default();
        let cases = [
            (0, "#FFFFFF"),  // lt1
            (1, "#000000"),  // dk1
            (2, "#E7E6E6"),  // lt2
            (3, "#44546A"),  // dk2
            (4, "#4472C4"),  // accent1
            (5, "#ED7D31"),  // accent2
            (6, "#A5A5A5"),  // accent3
            (7, "#FFC000"),  // accent4
            (8, "#5B9BD5"),  // accent5
            (9, "#70AD47"),  // accent6
            (10, "#0563C1"), // hlink
            (11, "#954F72"), // folHlink
        ];
        for (index, expected) in cases {
            assert_eq!(theme.resolve(index, 0.0), expected, "theme={index}");
        }
    }

    #[test]
    fn resolve_applies_tint_via_existing_algorithm() {
        let theme = Theme::default();
        assert_eq!(theme.resolve(0, -0.05), "#F2F2F2");
        assert_eq!(theme.resolve(5, -0.25), "#C55911");
        assert_eq!(theme.resolve(4, 0.6), "#B5C8E8");
    }
}
