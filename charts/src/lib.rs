//! IronCharts turns a chart and its data into a picture, almost.
//!
//! Everything that needs measuring or deciding happens here, in Rust: where
//! the title goes, how much room the legend takes, which ticks the value axis
//! shows, where every bar sits. The result is a [`scene::Scene`], a flat list
//! of rectangles, lines and text already placed in pixels. Drawing it is the
//! job of a renderer that never measures anything: an SVG writer, the canvas
//! in the webapp, a rasterizer.
//!
//! Text is measured with the metrics of Inter, the font the webapp uses, so
//! the layout matches what the browser draws without asking it. See [`text`].

pub mod layout;
pub mod render;
pub mod scale;
pub mod scene;
pub mod svg;
pub mod text;

pub use render::layout;
pub use svg::to_svg;

#[cfg(test)]
mod test_utils;
