use std::io::Read;

use roxmltree::Node;

use crate::error::XlsxError;

use super::colors::hex_with_tint_to_rgb;

const PALETTE_LEN: usize = 12;

const SCHEME_TAGS: [&str; PALETTE_LEN] = [
    "dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
    "hlink", "folHlink",
];

const DEFAULT_PALETTE: [&str; PALETTE_LEN] = [
    "#000000", // dk1
    "#FFFFFF", // lt1
    "#44546A", // dk2
    "#E7E6E6", // lt2
    "#4472C4", // accent1
    "#ED7D31", // accent2
    "#A5A5A5", // accent3
    "#FFC000", // accent4
    "#5B9BD5", // accent5
    "#70AD47", // accent6
    "#0563C1", // hlink
    "#954F72", // folHlink
];

/// Per-workbook OOXML theme color palette.
///
/// Colors are stored in `xl/theme/theme1.xml` declaration order:
/// `dk1, lt1, dk2, lt2, accent1..accent6, hlink, folHlink`.
#[derive(Clone, Debug)]
pub(crate) struct Theme {
    palette: [String; PALETTE_LEN],
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            palette: DEFAULT_PALETTE.map(String::from),
        }
    }
}

impl Theme {
    /// Resolves a `theme="N"` attribute (and optional `tint`) to an `#RRGGBB`
    /// string. The first four indices apply the OOXML dk/lt swap; the rest are
    /// straight-through.
    pub(crate) fn resolve(&self, theme_index: i32, tint: f64) -> String {
        let position = match theme_index {
            // theme=0 → lt1, theme=1 → dk1, theme=2 → lt2, theme=3 → dk2.
            0 => 1,
            1 => 0,
            2 => 3,
            3 => 2,
            n if (4..PALETTE_LEN as i32).contains(&n) => n as usize,
            _ => return hex_with_tint_to_rgb(&self.palette[0], tint),
        };
        hex_with_tint_to_rgb(&self.palette[position], tint)
    }

    /// Reads `xl/theme/theme1.xml` from the archive and returns the resulting
    /// palette. Falls back to [`Theme::default`] (and logs to stderr) when the
    /// part is missing or cannot be parsed.
    pub(crate) fn load<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> Theme {
        match try_load(archive) {
            Ok(theme) => theme,
            Err(e) => {
                eprintln!(
                    "IronCalc: falling back to default theme palette (could not read xl/theme/theme1.xml: {e})"
                );
                Theme::default()
            }
        }
    }
}

fn try_load<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> Result<Theme, XlsxError> {
    let mut file = archive.by_name("xl/theme/theme1.xml")?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;

    let scheme = doc
        .descendants()
        .find(|n| n.has_tag_name("clrScheme"))
        .ok_or_else(|| XlsxError::Xml("Missing clrScheme in theme1.xml".to_string()))?;

    let mut palette: [String; PALETTE_LEN] = DEFAULT_PALETTE.map(String::from);
    for (i, tag) in SCHEME_TAGS.iter().enumerate() {
        if let Some(slot) = scheme.children().find(|n| n.has_tag_name(*tag)) {
            if let Some(hex) = read_color(&slot) {
                palette[i] = hex;
            }
        }
    }

    Ok(Theme { palette })
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
                // sysClr carries a logical reference (e.g. "windowText") plus
                // the resolved value in `lastClr`. Use that — only fall back
                // to `val` if the resolver is absent.
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
    // OOXML stores hex without leading '#'. Some files wrap in alpha (8 hex
    // chars) — strip the leading two if present.
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
        // Same expectations as the legacy hardcoded `get_themed_color` table:
        // theme indices map directly to entries in [white, black, lightgray,
        // darkgray, accent1..accent6, hlink, folHlink].
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
    fn resolve_swaps_dk_lt_pairs_for_custom_theme() {
        // Custom palette where dk1/lt1 and dk2/lt2 are distinguishable, so a
        // missed swap would be visible.
        let theme = Theme {
            palette: [
                "#111111".into(), // dk1   (XML 0)
                "#EEEEEE".into(), // lt1   (XML 1)
                "#222222".into(), // dk2   (XML 2)
                "#DDDDDD".into(), // lt2   (XML 3)
                "#18A303".into(), // accent1
                "#0369A3".into(), // accent2
                "#A33E03".into(), // accent3
                "#8E03A3".into(), // accent4
                "#C99C00".into(), // accent5
                "#C9211E".into(), // accent6
                "#0000EE".into(), // hlink
                "#551A8B".into(), // folHlink
            ],
        };
        assert_eq!(theme.resolve(0, 0.0), "#EEEEEE"); // theme=0 → lt1
        assert_eq!(theme.resolve(1, 0.0), "#111111"); // theme=1 → dk1
        assert_eq!(theme.resolve(2, 0.0), "#DDDDDD"); // theme=2 → lt2
        assert_eq!(theme.resolve(3, 0.0), "#222222"); // theme=3 → dk2
        assert_eq!(theme.resolve(9, 0.0), "#C9211E"); // theme=9 → accent6
        assert_eq!(theme.resolve(10, 0.0), "#0000EE"); // theme=10 → hlink
        assert_eq!(theme.resolve(11, 0.0), "#551A8B"); // theme=11 → folHlink
    }

    #[test]
    fn resolve_applies_tint_via_existing_algorithm() {
        let theme = Theme::default();
        // Same expectations as the original `colors::tests::test_known_colors`.
        assert_eq!(theme.resolve(0, -0.05), "#F2F2F2");
        assert_eq!(theme.resolve(5, -0.25), "#C55911");
        assert_eq!(theme.resolve(4, 0.6), "#B5C8E8");
    }
}
