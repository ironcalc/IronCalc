use std::{collections::HashMap, io::Read};

use ironcalc_base::types::{
    Alignment, Border, BorderItem, BorderStyle, CellStyleXfs, CellStyles, CellXfs, Fill, Font,
    FontScheme, HorizontalAlignment, NumFmt, Styles, VerticalAlignment,
};
use roxmltree::Node;

use crate::error::XlsxError;

use super::util::{get_attribute, get_bool, get_bool_false, get_color, get_number};

fn get_border(node: Node, name: &str) -> Result<Option<BorderItem>, XlsxError> {
    let style;
    let color;
    let border_nodes = node
        .children()
        .filter(|n| n.has_tag_name(name))
        .collect::<Vec<Node>>();
    if border_nodes.len() == 1 {
        let border = border_nodes[0];
        style = match border.attribute("style") {
            Some("thin") => BorderStyle::Thin,
            Some("medium") => BorderStyle::Medium,
            Some("thick") => BorderStyle::Thick,
            Some("double") => BorderStyle::Double,
            Some("slantdashdot") => BorderStyle::SlantDashDot,
            Some("mediumdashed") => BorderStyle::MediumDashed,
            Some("mediumdashdot") => BorderStyle::MediumDashDot,
            Some("mediumdashdotdot") => BorderStyle::MediumDashDotDot,
            // TODO: Should we fail in this case or set the border to None?
            Some(_) => BorderStyle::Thin,
            None => {
                return Ok(None);
            }
        };

        let color_node = border
            .children()
            .filter(|n| n.has_tag_name("color"))
            .collect::<Vec<Node>>();
        if color_node.len() == 1 {
            color = get_color(color_node[0])?;
        } else {
            color = None;
        }
    } else {
        return Ok(None);
    }
    Ok(Some(BorderItem { style, color }))
}

pub(super) fn load_styles<R: Read + std::io::Seek>(
    archive: &mut zip::read::ZipArchive<R>,
) -> Result<Styles, XlsxError> {
    let mut file = archive.by_name("xl/styles.xml")?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let doc = roxmltree::Document::parse(&text)?;
    let style_sheet = doc
        .root()
        .first_child()
        .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))?;

    let mut num_fmts = Vec::new();
    let num_fmts_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("numFmts"))
        .collect::<Vec<Node>>();
    if num_fmts_nodes.len() == 1 {
        for num_fmt in num_fmts_nodes[0].children() {
            let num_fmt_id = get_number(num_fmt, "numFmtId");
            let format_code = num_fmt.attribute("formatCode").unwrap_or("").to_string();
            num_fmts.push(NumFmt {
                num_fmt_id,
                format_code,
            });
        }
    }

    let mut fonts = Vec::new();
    let font_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("fonts"))
        .collect::<Vec<Node>>()[0];
    for font in font_nodes.children() {
        let mut sz = 11;
        let mut name = "Calibri".to_string();
        // NOTE: In Excel you can have simple underline or double underline
        // In IronCalc convert double underline to simple
        // This in excel is u with a value of "double"
        let mut u = false;
        let mut b = false;
        let mut i = false;
        let mut strike = false;
        let mut color = Some("FFFFFF00".to_string());
        let mut family = 2;
        let mut scheme = FontScheme::default();
        for feature in font.children() {
            match feature.tag_name().name() {
                "sz" => {
                    sz = feature
                        .attribute("val")
                        .unwrap_or("11")
                        .parse::<i32>()
                        .unwrap_or(11);
                }
                "color" => {
                    color = get_color(feature)?;
                }
                "u" => {
                    u = true;
                }
                "b" => {
                    b = true;
                }
                "i" => {
                    i = true;
                }
                "strike" => {
                    strike = true;
                }
                "name" => name = feature.attribute("val").unwrap_or("Calibri").to_string(),
                // If there is a theme the font scheme and family overrides other properties like the name
                "family" => {
                    family = feature
                        .attribute("val")
                        .unwrap_or("2")
                        .parse::<i32>()
                        .unwrap_or(2);
                }
                "scheme" => {
                    scheme = match feature.attribute("val") {
                        None => FontScheme::default(),
                        Some("minor") => FontScheme::Minor,
                        Some("major") => FontScheme::Major,
                        Some("none") => FontScheme::None,
                        // TODO: Should we fail?
                        Some(_) => FontScheme::default(),
                    }
                }
                "charset" => {}
                _ => {
                    println!("Unexpected feature {:?}", feature);
                }
            }
        }
        fonts.push(Font {
            strike,
            u,
            b,
            i,
            sz,
            color,
            name,
            family,
            scheme,
        });
    }

    let mut fills = Vec::new();
    let fill_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("fills"))
        .collect::<Vec<Node>>()[0];
    for fill in fill_nodes.children() {
        let pattern_fill = fill
            .children()
            .filter(|n| n.has_tag_name("patternFill"))
            .collect::<Vec<Node>>();
        if pattern_fill.len() != 1 {
            // safety belt
            // Some fills do not have a patternFill, but they have gradientFill
            fills.push(Fill {
                pattern_type: "solid".to_string(),
                fg_color: None,
                bg_color: None,
            });
            continue;
        }
        let pattern_fill = pattern_fill[0];

        let pattern_type = pattern_fill
            .attribute("patternType")
            .unwrap_or("none")
            .to_string();
        let mut fg_color = None;
        let mut bg_color = None;
        for feature in pattern_fill.children() {
            match feature.tag_name().name() {
                "fgColor" => {
                    fg_color = get_color(feature)?;
                }
                "bgColor" => {
                    bg_color = get_color(feature)?;
                }
                _ => {
                    println!("Unexpected pattern");
                    dbg!(feature);
                }
            }
        }
        fills.push(Fill {
            pattern_type,
            fg_color,
            bg_color,
        })
    }

    let mut borders = Vec::new();
    let border_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("borders"))
        .collect::<Vec<Node>>()[0];
    for border in border_nodes.children() {
        let diagonal_up = get_bool_false(border, "diagonal_up");
        let diagonal_down = get_bool_false(border, "diagonal_down");
        let left = get_border(border, "left")?;
        let right = get_border(border, "right")?;
        let top = get_border(border, "top")?;
        let bottom = get_border(border, "bottom")?;
        let diagonal = get_border(border, "diagonal")?;
        borders.push(Border {
            diagonal_up,
            diagonal_down,
            left,
            right,
            top,
            bottom,
            diagonal,
        });
    }

    let mut cell_style_xfs = Vec::new();
    let cell_style_xfs_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("cellStyleXfs"))
        .collect::<Vec<Node>>()[0];
    for xfs in cell_style_xfs_nodes.children() {
        let num_fmt_id = get_number(xfs, "numFmtId");
        let font_id = get_number(xfs, "fontId");
        let fill_id = get_number(xfs, "fillId");
        let border_id = get_number(xfs, "borderId");
        let apply_number_format = get_bool(xfs, "applyNumberFormat");
        let apply_border = get_bool(xfs, "applyBorder");
        let apply_alignment = get_bool(xfs, "applyAlignment");
        let apply_protection = get_bool(xfs, "applyProtection");
        let apply_font = get_bool(xfs, "applyFont");
        let apply_fill = get_bool(xfs, "applyFill");

        cell_style_xfs.push(CellStyleXfs {
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
            apply_number_format,
            apply_border,
            apply_alignment,
            apply_protection,
            apply_font,
            apply_fill,
        });
    }

    let mut cell_styles = Vec::new();
    let mut style_names = HashMap::new();
    let cell_style_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("cellStyles"))
        .collect::<Vec<Node>>()[0];
    for cell_style in cell_style_nodes.children() {
        let name = get_attribute(&cell_style, "name")?.to_string();
        let xf_id = get_number(cell_style, "xfId");
        let builtin_id = get_number(cell_style, "builtinId");
        style_names.insert(xf_id, name.clone());
        cell_styles.push(CellStyles {
            name,
            xf_id,
            builtin_id,
        })
    }

    let mut cell_xfs = Vec::new();
    let cell_xfs_nodes = style_sheet
        .children()
        .filter(|n| n.has_tag_name("cellXfs"))
        .collect::<Vec<Node>>()[0];
    for xfs in cell_xfs_nodes.children() {
        let xf_id = get_attribute(&xfs, "xfId")?.parse::<i32>()?;
        let num_fmt_id = get_number(xfs, "numFmtId");
        let font_id = get_number(xfs, "fontId");
        let fill_id = get_number(xfs, "fillId");
        let border_id = get_number(xfs, "borderId");
        let apply_number_format = get_bool_false(xfs, "applyNumberFormat");
        let apply_border = get_bool_false(xfs, "applyBorder");
        let apply_alignment = get_bool_false(xfs, "applyAlignment");
        let apply_protection = get_bool_false(xfs, "applyProtection");
        let apply_font = get_bool_false(xfs, "applyFont");
        let apply_fill = get_bool_false(xfs, "applyFill");
        let quote_prefix = get_bool_false(xfs, "quotePrefix");

        // TODO: Pivot Tables
        // let pivotButton = get_bool(xfs, "pivotButton");

        let alignment_nodes = xfs
            .children()
            .filter(|n| n.has_tag_name("alignment"))
            .collect::<Vec<Node>>();
        let alignment = if alignment_nodes.len() == 1 {
            let alignment_node = alignment_nodes[0];
            let wrap_text = get_bool_false(alignment_node, "wrapText");

            let horizontal = match alignment_node.attribute("horizontal") {
                Some("center") => HorizontalAlignment::Center,
                Some("centerContinuous") => HorizontalAlignment::CenterContinuous,
                Some("distributed") => HorizontalAlignment::Distributed,
                Some("fill") => HorizontalAlignment::Fill,
                Some("general") => HorizontalAlignment::General,
                Some("justify") => HorizontalAlignment::Justify,
                Some("left") => HorizontalAlignment::Left,
                Some("right") => HorizontalAlignment::Right,
                // TODO: Should we fail in this case or set the alignment to default?
                Some(_) => HorizontalAlignment::default(),
                None => HorizontalAlignment::default(),
            };

            let vertical = match alignment_node.attribute("vertical") {
                Some("bottom") => VerticalAlignment::Bottom,
                Some("center") => VerticalAlignment::Center,
                Some("distributed") => VerticalAlignment::Distributed,
                Some("justify") => VerticalAlignment::Justify,
                Some("top") => VerticalAlignment::Top,
                // TODO: Should we fail in this case or set the alignment to default?
                Some(_) => VerticalAlignment::default(),
                None => VerticalAlignment::default(),
            };

            Some(Alignment {
                horizontal,
                vertical,
                wrap_text,
            })
        } else {
            None
        };

        cell_xfs.push(CellXfs {
            xf_id,
            num_fmt_id,
            font_id,
            fill_id,
            border_id,
            apply_number_format,
            apply_border,
            apply_alignment,
            apply_protection,
            apply_font,
            apply_fill,
            quote_prefix,
            alignment,
        });
    }

    // TODO
    // let mut dxfs = Vec::new();
    // let mut tableStyles = Vec::new();
    // let mut colors = Vec::new();
    // <colors>
    //     <mruColors>
    //         <color rgb="FFB1BB4D"/>
    //         <color rgb="FFFF99CC"/>
    //         <color rgb="FF6C56DC"/>
    //         <color rgb="FFFF66CC"/>
    //     </mruColors>
    // </colors>

    Ok(Styles {
        num_fmts,
        fonts,
        fills,
        borders,
        cell_style_xfs,
        cell_xfs,
        cell_styles,
    })
}
