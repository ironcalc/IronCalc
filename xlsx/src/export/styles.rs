use ironcalc_base::types::{
    Alignment, BorderItem, HorizontalAlignment, Styles, VerticalAlignment, Workbook,
};

use super::{escape::escape_xml, xml_constants::XML_DECLARATION};

fn get_fonts_xml(styles: &Styles) -> String {
    let fonts = &styles.fonts;
    let mut fonts_str: Vec<String> = vec![];
    for font in fonts {
        let size = format!("<sz val=\"{}\"/>", font.sz);
        let color = if let Some(some_color) = &font.color {
            format!("<color rgb=\"FF{}\"/>", some_color.trim_start_matches('#'))
        } else {
            "".to_string()
        };
        let name = format!("<name val=\"{}\"/>", escape_xml(&font.name));
        let bold = if font.b { "<b/>" } else { "" };
        let italic = if font.i { "<i/>" } else { "" };
        let underline = if font.u { "<u/>" } else { "" };
        let strike = if font.strike { "<strike/>" } else { "" };
        let family = format!("<family val=\"{}\"/>", font.family);
        let scheme = format!("<scheme val=\"{}\"/>", font.scheme);
        fonts_str.push(format!(
            "<font>\
                {size}\
                {color}\
                {name}\
                {bold}\
                {italic}\
                {underline}\
                {strike}\
                {family}\
                {scheme}\
             </font>"
        ));
    }
    let font_count = fonts.len();
    format!(
        "<fonts count=\"{font_count}\">{}</fonts>",
        fonts_str.join("")
    )
}

fn get_color_xml(color: &Option<String>, name: &str) -> String {
    // We blindly append FF at the beginning of these RGB color to make it ARGB
    if let Some(some_color) = color {
        format!("<{name} rgb=\"FF{}\"/>", some_color.trim_start_matches('#'))
    } else {
        "".to_string()
    }
}

fn get_fills_xml(styles: &Styles) -> String {
    let fills = &styles.fills;
    let mut fills_str: Vec<String> = vec![];
    for fill in fills {
        let pattern_type = &fill.pattern_type;
        let fg_color = get_color_xml(&fill.fg_color, "fgColor");
        let bg_color = get_color_xml(&fill.bg_color, "bgColor");
        fills_str.push(format!(
            "<fill><patternFill patternType=\"{pattern_type}\">{fg_color}{bg_color}</patternFill></fill>"
        ));
    }
    let fill_count = fills.len();
    format!(
        "<fills count=\"{fill_count}\">{}</fills>",
        fills_str.join("")
    )
}

fn get_border_xml(border: &Option<BorderItem>, name: &str) -> String {
    if let Some(border_item) = border {
        let color = get_color_xml(&border_item.color, "color");
        return format!("<{name} style=\"{}\">{color}</{name}>", border_item.style);
    }
    format!("<{name}/>")
}

fn get_borders_xml(styles: &Styles) -> String {
    let borders = &styles.borders;
    let mut borders_str: Vec<String> = vec![];
    let border_count = borders.len();
    for border in borders {
        // TODO: diagonal_up/diagonal_down?
        let border_left = get_border_xml(&border.left, "left");
        let border_right = get_border_xml(&border.right, "right");
        let border_top = get_border_xml(&border.top, "top");
        let border_bottom = get_border_xml(&border.bottom, "bottom");
        let border_diagonal = get_border_xml(&border.diagonal, "diagonal");
        borders_str.push(format!(
            "<border>{border_left}{border_right}{border_top}{border_bottom}{border_diagonal}</border>"
        ));
    }
    format!(
        "<borders count=\"{border_count}\">{}</borders>",
        borders_str.join("")
    )
}

// <numFmts count="1">
//   <numFmt numFmtId="164" formatCode="##,#00;[Blue]\-\-#,##0"/>
// </numFmts>
fn get_cell_number_formats_xml(styles: &Styles) -> String {
    let num_fmts = &styles.num_fmts;
    let mut num_fmts_str: Vec<String> = vec![];
    let num_fmt_count = num_fmts.len();
    for num_fmt in num_fmts {
        let num_fmt_id = num_fmt.num_fmt_id;
        let format_code = &num_fmt.format_code;
        let format_code = escape_xml(format_code);
        num_fmts_str.push(format!(
            "<numFmt numFmtId=\"{num_fmt_id}\" formatCode=\"{format_code}\"/>"
        ));
    }
    if num_fmt_count == 0 {
        return "".to_string();
    }
    format!(
        "<numFmts count=\"{num_fmt_count}\">{}</numFmts>",
        num_fmts_str.join("")
    )
}

fn get_alignment(alignment: &Alignment) -> String {
    let wrap_text = if alignment.wrap_text {
        " wrapText=\"1\""
    } else {
        ""
    };
    let horizontal = if alignment.horizontal != HorizontalAlignment::default() {
        format!(" horizontal=\"{}\"", alignment.horizontal)
    } else {
        "".to_string()
    };
    let vertical = if alignment.vertical != VerticalAlignment::default() {
        format!(" vertical=\"{}\"", alignment.vertical)
    } else {
        "".to_string()
    };
    format!("<alignment{wrap_text}{horizontal}{vertical}/>")
}

fn get_cell_style_xfs_xml(styles: &Styles) -> String {
    let cell_style_xfs = &styles.cell_style_xfs;
    let mut cell_style_str: Vec<String> = vec![];
    for cell_style_xf in cell_style_xfs {
        let border_id = cell_style_xf.border_id;
        let fill_id = cell_style_xf.fill_id;
        let font_id = cell_style_xf.font_id;
        let num_fmt_id = cell_style_xf.num_fmt_id;
        let apply_alignment_str = if cell_style_xf.apply_alignment {
            r#" applyAlignment="1""#
        } else {
            ""
        };
        let apply_font_str = if cell_style_xf.apply_font {
            r#" applyFont="1""#
        } else {
            ""
        };
        let apply_fill_str = if cell_style_xf.apply_fill {
            r#" applyFill="1""#
        } else {
            ""
        };
        cell_style_str.push(format!(
            "<xf \
              borderId=\"{border_id}\" \
              fillId=\"{fill_id}\" \
              fontId=\"{font_id}\" \
              numFmtId=\"{num_fmt_id}\"\
              {apply_alignment_str}\
              {apply_font_str}\
              {apply_fill_str}/>"
        ));
    }
    let style_count = cell_style_xfs.len();
    format!(
        "<cellStyleXfs count=\"{style_count}\">{}</cellStyleXfs>",
        cell_style_str.join("")
    )
}

fn get_cell_xfs_xml(styles: &Styles) -> String {
    let cell_xfs = &styles.cell_xfs;
    let mut cell_xfs_str: Vec<String> = vec![];
    for cell_xf in cell_xfs {
        let xf_id = cell_xf.xf_id;
        let border_id = cell_xf.border_id;
        let fill_id = cell_xf.fill_id;
        let font_id = cell_xf.font_id;
        let num_fmt_id = cell_xf.num_fmt_id;
        let quote_prefix_str = if cell_xf.quote_prefix {
            r#" quotePrefix="1""#
        } else {
            ""
        };
        let apply_alignment_str = if cell_xf.apply_alignment {
            r#" applyAlignment="1""#
        } else {
            ""
        };
        let apply_font_str = if cell_xf.apply_font {
            r#" applyFont="1""#
        } else {
            ""
        };
        let apply_fill_str = if cell_xf.apply_fill {
            r#" applyFill="1""#
        } else {
            ""
        };
        let properties = format!(
            "xfId=\"{xf_id}\" \
                borderId=\"{border_id}\" \
                fillId=\"{fill_id}\" \
                fontId=\"{font_id}\" \
                numFmtId=\"{num_fmt_id}\"\
                {quote_prefix_str}\
                {apply_alignment_str}\
                {apply_font_str}\
                {apply_fill_str}"
        );
        if let Some(alignment) = &cell_xf.alignment {
            let alignment = get_alignment(alignment);
            cell_xfs_str.push(format!("<xf {properties}>{alignment}</xf>"));
        } else {
            cell_xfs_str.push(format!("<xf {properties}/>"));
        }
    }
    let style_count = cell_xfs.len();
    format!(
        "<cellXfs count=\"{style_count}\">{}</cellXfs>",
        cell_xfs_str.join("")
    )
}

// <cellStyle xfId="0" name="Normal" builtinId="0"/>
fn get_cell_styles_xml(styles: &Styles) -> String {
    let cell_styles = &styles.cell_styles;
    let mut cell_styles_str: Vec<String> = vec![];
    for cell_style in cell_styles {
        let xf_id = cell_style.xf_id;
        let name = &cell_style.name;
        let name = escape_xml(name);
        let builtin_id = cell_style.builtin_id;
        cell_styles_str.push(format!(
            "<cellStyle xfId=\"{xf_id}\" name=\"{name}\" builtinId=\"{builtin_id}\"/>"
        ));
    }
    let style_count = cell_styles.len();
    format!(
        "<cellStyles count=\"{style_count}\">{}</cellStyles>",
        cell_styles_str.join("")
    )
}

pub(crate) fn get_styles_xml(model: &Workbook) -> String {
    let styles = &model.styles;
    let fonts = get_fonts_xml(styles);
    let fills = get_fills_xml(styles);
    let borders = get_borders_xml(styles);
    let number_formats = get_cell_number_formats_xml(styles);
    let cell_style_xfs = get_cell_style_xfs_xml(styles);
    let cell_xfs = get_cell_xfs_xml(styles);
    let cell_styles = get_cell_styles_xml(styles);

    format!(
        "{XML_DECLARATION}
<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
{number_formats}\
{fonts}\
{fills}\
{borders}\
{cell_style_xfs}\
{cell_xfs}\
{cell_styles}\
<dxfs count=\"0\"/>\
</styleSheet>"
    )
}
