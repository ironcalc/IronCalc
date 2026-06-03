use ironcalc_base::types::{
    Alignment, Border, BorderItem, Fill, HorizontalAlignment, VerticalAlignment,
};

pub(crate) fn get_color_xml(color: &Option<String>, name: &str) -> String {
    // We blindly append FF at the beginning of these RGB color to make it ARGB
    if let Some(some_color) = color {
        format!("<{name} rgb=\"FF{}\"/>", some_color.trim_start_matches('#'))
    } else {
        "".to_string()
    }
}

pub(crate) fn get_alignment(alignment: &Alignment) -> String {
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

fn get_border_xml_inner(border: &Option<BorderItem>, name: &str) -> String {
    if let Some(border_item) = border {
        let color = get_color_xml(&border_item.color, "color");
        return format!("<{name} style=\"{}\">{color}</{name}>", border_item.style);
    }
    format!("<{name}/>")
}

pub(crate) fn get_border_xml(border: &Border) -> String {
    let left = get_border_xml_inner(&border.left, "left");
    let right = get_border_xml_inner(&border.right, "right");
    let top = get_border_xml_inner(&border.top, "top");
    let bottom = get_border_xml_inner(&border.bottom, "bottom");
    let diagonal = get_border_xml_inner(&border.diagonal, "diagonal");
    format!("<border>{left}{right}{top}{bottom}{diagonal}</border>")
}

pub(crate) fn get_fill_xml(fill: &Fill) -> String {
    let pattern_type = &fill.pattern_type;
    let fg_color = get_color_xml(&fill.fg_color, "fgColor");
    let bg_color = get_color_xml(&fill.bg_color, "bgColor");
    format!(
        "<fill><patternFill patternType=\"{pattern_type}\">{fg_color}{bg_color}</patternFill></fill>"
    )
}
