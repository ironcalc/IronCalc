use crate::{
    colors::hex_with_tint_to_rgb,
    types::{Border, BorderItem, BorderStyle, Fill, Font, FontScheme, Style},
};

const ACCENT1: &str = "#4472C4";
const ACCENT2: &str = "#ED7D31";
const ACCENT3: &str = "#A5A5A5";
const ACCENT4: &str = "#FFC000";
const ACCENT5: &str = "#5B9BD5";
const ACCENT6: &str = "#70AD47";
const DK2: &str = "#44546A";

fn solid_fill(c: &str) -> Fill {
    Fill {
        color: Some(c.to_string()),
    }
}

fn thin_box_border(color: &str) -> Border {
    let item = Some(BorderItem {
        style: BorderStyle::Thin,
        color: Some(color.to_string()),
    });
    Border {
        left: item.clone(),
        right: item.clone(),
        top: item.clone(),
        bottom: item,
        ..Default::default()
    }
}

fn double_box_border(color: &str) -> Border {
    let item = Some(BorderItem {
        style: BorderStyle::Double,
        color: Some(color.to_string()),
    });
    Border {
        left: item.clone(),
        right: item.clone(),
        top: item.clone(),
        bottom: item,
        ..Default::default()
    }
}

fn thick_bottom_border(color: &str) -> Border {
    Border {
        bottom: Some(BorderItem {
            style: BorderStyle::Thick,
            color: Some(color.to_string()),
        }),
        ..Default::default()
    }
}

fn thin_top_double_bottom_border(color: &str) -> Border {
    Border {
        top: Some(BorderItem {
            style: BorderStyle::Thin,
            color: Some(color.to_string()),
        }),
        bottom: Some(BorderItem {
            style: BorderStyle::Double,
            color: Some(color.to_string()),
        }),
        ..Default::default()
    }
}

/// Returns the full list of Excel built-in named styles with their style definitions.
#[allow(clippy::vec_init_then_push)]
pub fn builtin_named_styles() -> Vec<(String, Style)> {
    let mut result = vec![];

    // Good, Bad, Neutral
    result.push((
        "Good".to_string(),
        Style {
            font: Font {
                color: Some("#006100".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#C6EFCE"),
            ..Default::default()
        },
    ));
    result.push((
        "Bad".to_string(),
        Style {
            font: Font {
                color: Some("#9C0006".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFC7CE"),
            ..Default::default()
        },
    ));
    result.push((
        "Neutral".to_string(),
        Style {
            font: Font {
                color: Some("#9C5700".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFEB9C"),
            ..Default::default()
        },
    ));

    // Normal (always in every model, included here for the panel)
    result.push(("Normal".to_string(), Style::default()));

    // Data and Model
    result.push((
        "Calculation".to_string(),
        Style {
            font: Font {
                b: true,
                color: Some("#FA7D00".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#F2F2F2"),
            border: thin_box_border("#7F7F7F"),
            ..Default::default()
        },
    ));
    result.push((
        "Check Cell".to_string(),
        Style {
            font: Font {
                b: true,
                color: Some("#FFFFFF".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#A5A5A5"),
            border: double_box_border("#3F3F3F"),
            ..Default::default()
        },
    ));
    result.push((
        "Explanatory Text".to_string(),
        Style {
            font: Font {
                i: true,
                color: Some("#7F7F7F".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    result.push((
        "Input".to_string(),
        Style {
            font: Font {
                color: Some("#3F3F76".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#FFCC99"),
            border: thin_box_border("#7F7F7F"),
            ..Default::default()
        },
    ));
    result.push((
        "Linked Cell".to_string(),
        Style {
            font: Font {
                color: Some("#FA7D00".to_string()),
                ..Default::default()
            },
            border: Border {
                bottom: Some(BorderItem {
                    style: BorderStyle::Double,
                    color: Some("#FF8001".to_string()),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    result.push((
        "Note".to_string(),
        Style {
            fill: solid_fill("#FFFFE1"),
            border: thin_box_border("#B2B2B2"),
            ..Default::default()
        },
    ));
    result.push((
        "Output".to_string(),
        Style {
            font: Font {
                b: true,
                color: Some("#3F3F3F".to_string()),
                ..Default::default()
            },
            fill: solid_fill("#F2F2F2"),
            border: thin_box_border("#3F3F3F"),
            ..Default::default()
        },
    ));
    result.push((
        "Warning Text".to_string(),
        Style {
            font: Font {
                color: Some("#FF0000".to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    // Titles and Headings
    result.push((
        "Title".to_string(),
        Style {
            font: Font {
                sz: 18,
                color: Some(DK2.to_string()),
                scheme: FontScheme::Major,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    result.push((
        "Heading 1".to_string(),
        Style {
            font: Font {
                b: true,
                sz: 15,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: thick_bottom_border(ACCENT1),
            ..Default::default()
        },
    ));
    let h2_border_color = hex_with_tint_to_rgb(ACCENT1, 0.5);
    result.push((
        "Heading 2".to_string(),
        Style {
            font: Font {
                b: true,
                sz: 13,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: thick_bottom_border(&h2_border_color),
            ..Default::default()
        },
    ));
    result.push((
        "Heading 3".to_string(),
        Style {
            font: Font {
                b: true,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            border: Border {
                bottom: Some(BorderItem {
                    style: BorderStyle::Thin,
                    color: Some(ACCENT1.to_string()),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    result.push((
        "Heading 4".to_string(),
        Style {
            font: Font {
                b: true,
                i: true,
                color: Some(DK2.to_string()),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    result.push((
        "Total".to_string(),
        Style {
            font: Font {
                b: true,
                ..Default::default()
            },
            border: thin_top_double_bottom_border(ACCENT1),
            ..Default::default()
        },
    ));

    // Themed Cell Styles: 20% / 40% / 60% tints and solid for each accent
    for (accent_name, accent_hex) in [
        ("Accent1", ACCENT1),
        ("Accent2", ACCENT2),
        ("Accent3", ACCENT3),
        ("Accent4", ACCENT4),
        ("Accent5", ACCENT5),
        ("Accent6", ACCENT6),
    ] {
        let c20 = hex_with_tint_to_rgb(accent_hex, 0.8);
        result.push((
            format!("20% - {accent_name}"),
            Style {
                fill: solid_fill(&c20),
                ..Default::default()
            },
        ));
        let c40 = hex_with_tint_to_rgb(accent_hex, 0.6);
        result.push((
            format!("40% - {accent_name}"),
            Style {
                fill: solid_fill(&c40),
                ..Default::default()
            },
        ));
        let c60 = hex_with_tint_to_rgb(accent_hex, 0.4);
        result.push((
            format!("60% - {accent_name}"),
            Style {
                fill: solid_fill(&c60),
                ..Default::default()
            },
        ));
        result.push((
            accent_name.to_string(),
            Style {
                fill: solid_fill(accent_hex),
                ..Default::default()
            },
        ));
    }

    // Number Format styles
    result.push((
        "Comma".to_string(),
        Style {
            num_fmt: "#,##0.00".to_string(),
            ..Default::default()
        },
    ));
    result.push((
        "Comma [0]".to_string(),
        Style {
            num_fmt: "#,##0".to_string(),
            ..Default::default()
        },
    ));
    result.push((
        "Currency".to_string(),
        Style {
            num_fmt: r#"_("$"* #,##0.00_);_("$"* \(#,##0.00\);_("$"* "-"??_);_(@_)"#.to_string(),
            ..Default::default()
        },
    ));
    result.push((
        "Currency [0]".to_string(),
        Style {
            num_fmt: r#"_("$"* #,##0_);_("$"* \(#,##0\);_("$"* "-"_);_(@_)"#.to_string(),
            ..Default::default()
        },
    ));
    result.push((
        "Percent".to_string(),
        Style {
            num_fmt: "0%".to_string(),
            ..Default::default()
        },
    ));

    result
}

/// Looks up a built-in named style by name (case-sensitive). Returns `None` if not found.
pub fn get_builtin_style(name: &str) -> Option<Style> {
    builtin_named_styles()
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, s)| s)
}
