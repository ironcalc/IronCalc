//! Writes a [`Scene`] as SVG. Nothing is computed here: every number comes
//! from the scene, rounded to a hundredth of a pixel.

use crate::scene::{Anchor, Role, Scene, Shape, Stroke};

fn number(value: f64) -> String {
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-0" {
        "0".to_string()
    } else {
        text.to_string()
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A class name per role, so a reader (or a stylesheet) can tell the parts
/// apart.
fn class(role: &Role) -> String {
    match role {
        Role::ChartArea => "chart-area".to_string(),
        Role::Title => "title".to_string(),
        Role::Legend => "legend".to_string(),
        Role::PlotArea => "plot-area".to_string(),
        Role::Gridlines => "gridlines".to_string(),
        Role::CategoryAxis => "category-axis".to_string(),
        Role::ValueAxis => "value-axis".to_string(),
        Role::Series(index) => format!("series-{index}"),
        Role::DataLabel(index) => format!("data-label-{index}"),
    }
}

fn stroke_attributes(stroke: Option<&Stroke>) -> String {
    match stroke {
        None => " stroke=\"none\"".to_string(),
        Some(stroke) => {
            let dash = if stroke.dash.is_empty() {
                "".to_string()
            } else {
                let lengths: Vec<String> = stroke.dash.iter().map(|d| number(*d)).collect();
                format!(" stroke-dasharray=\"{}\"", lengths.join(" "))
            };
            format!(
                " stroke=\"{}\" stroke-width=\"{}\"{dash}",
                escape(&stroke.color),
                number(stroke.width)
            )
        }
    }
}

pub fn to_svg(scene: &Scene) -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n",
        w = number(scene.width),
        h = number(scene.height)
    );
    for element in &scene.elements {
        let class = class(&element.role);
        match &element.shape {
            Shape::Rect {
                x,
                y,
                width,
                height,
                fill,
                stroke,
            } => {
                let fill = match fill {
                    Some(color) => escape(color),
                    None => "none".to_string(),
                };
                out.push_str(&format!(
                    "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{fill}\"{}/>\n",
                    number(*x),
                    number(*y),
                    number(*width),
                    number(*height),
                    stroke_attributes(stroke.as_ref())
                ));
            }
            Shape::Line {
                x1,
                y1,
                x2,
                y2,
                stroke,
            } => {
                out.push_str(&format!(
                    "<line class=\"{class}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>\n",
                    number(*x1),
                    number(*y1),
                    number(*x2),
                    number(*y2),
                    stroke_attributes(Some(stroke))
                ));
            }
            Shape::Text {
                x,
                y,
                text,
                font,
                anchor,
                rotation,
            } => {
                let anchor = match anchor {
                    Anchor::Start => "start",
                    Anchor::Middle => "middle",
                    Anchor::End => "end",
                };
                let weight = if font.bold {
                    " font-weight=\"600\""
                } else {
                    ""
                };
                let style = if font.italic {
                    " font-style=\"italic\""
                } else {
                    ""
                };
                let transform = if *rotation == 0.0 {
                    "".to_string()
                } else {
                    format!(
                        " transform=\"rotate({} {} {})\"",
                        number(*rotation),
                        number(*x),
                        number(*y)
                    )
                };
                out.push_str(&format!(
                    "<text class=\"{class}\" x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{weight}{style} fill=\"{}\" text-anchor=\"{anchor}\"{transform}>{}</text>\n",
                    number(*x),
                    number(*y),
                    escape(&font.family),
                    number(font.size),
                    escape(&font.color),
                    escape(text)
                ));
            }
        }
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Element, ResolvedFont};

    #[test]
    fn numbers_are_short() {
        assert_eq!(number(203.33333333), "203.33");
        assert_eq!(number(12.0), "12");
        assert_eq!(number(12.5), "12.5");
        assert_eq!(number(-0.001), "0");
    }

    #[test]
    fn writes_every_shape() {
        let scene = Scene {
            width: 100.0,
            height: 50.0,
            elements: vec![
                Element {
                    role: Role::ChartArea,
                    shape: Shape::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 50.0,
                        fill: Some("#FFFFFF".to_string()),
                        stroke: Some(Stroke {
                            color: "#D9D9D9".to_string(),
                            width: 1.0,
                            dash: vec![4.0, 2.0],
                        }),
                    },
                },
                Element {
                    role: Role::Gridlines,
                    shape: Shape::Line {
                        x1: 10.0,
                        y1: 20.0,
                        x2: 90.0,
                        y2: 20.0,
                        stroke: Stroke {
                            color: "#D9D9D9".to_string(),
                            width: 1.0,
                            dash: vec![],
                        },
                    },
                },
                Element {
                    role: Role::ValueAxis,
                    shape: Shape::Text {
                        x: 10.0,
                        y: 25.0,
                        text: "A & B".to_string(),
                        font: ResolvedFont {
                            family: "Inter".to_string(),
                            size: 12.0,
                            bold: true,
                            italic: false,
                            color: "#595959".to_string(),
                        },
                        anchor: Anchor::Middle,
                        rotation: -90.0,
                    },
                },
            ],
        };
        assert_eq!(
            to_svg(&scene),
            concat!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"50\" viewBox=\"0 0 100 50\">\n",
                "<rect class=\"chart-area\" x=\"0\" y=\"0\" width=\"100\" height=\"50\" fill=\"#FFFFFF\" stroke=\"#D9D9D9\" stroke-width=\"1\" stroke-dasharray=\"4 2\"/>\n",
                "<line class=\"gridlines\" x1=\"10\" y1=\"20\" x2=\"90\" y2=\"20\" stroke=\"#D9D9D9\" stroke-width=\"1\"/>\n",
                "<text class=\"value-axis\" x=\"10\" y=\"25\" font-family=\"Inter\" font-size=\"12\" font-weight=\"600\" fill=\"#595959\" text-anchor=\"middle\" transform=\"rotate(-90 10 25)\">A &amp; B</text>\n",
                "</svg>\n"
            )
        );
    }
}
