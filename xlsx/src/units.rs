//! Conversions between the units used in xlsx files and the pixels IronCalc
//! uses internally.
//!
//! IronCalc works in (CSS) pixels: 96 pixels per inch. In an xlsx file:
//!
//! * Font sizes and row heights are in points: 72 points per inch, so one
//!   point is 4/3 of a pixel and `36pt` is `48px`.
//! * Column widths are in "characters": multiples of the maximum digit width
//!   of the default font of the workbook, the font of the `Normal` style
//!   (ECMA-376 Part 1, 18.3.1.13). See [`max_digit_width`].

use ironcalc_base::types::Styles;

/// The width of a digit as a fraction of the font size, for the fonts that
/// are a common default font of a workbook. Names are in lower case.
///
/// Excel measures the font on the screen and the result depends on the
/// platform. We can not measure fonts, these are the advance widths of the
/// digits in the font files.
const DIGIT_WIDTHS: [(&str, f64); 16] = [
    ("calibri", 0.507),
    ("calibri light", 0.507),
    ("carlito", 0.507),
    ("aptos narrow", 0.5),
    ("aptos", 0.56),
    ("arial", 0.556),
    ("helvetica", 0.556),
    ("helvetica neue", 0.556),
    ("liberation sans", 0.556),
    ("times new roman", 0.5),
    ("cambria", 0.554),
    ("tahoma", 0.546),
    ("segoe ui", 0.539),
    ("verdana", 0.636),
    ("courier new", 0.6),
    ("inter", 0.62),
];

/// The width of a digit, as a fraction of the font size, of any other font
const DEFAULT_DIGIT_WIDTH: f64 = 0.5;

/// Maximum digit width in pixels of a font: a whole number of pixels, 7 for
/// Calibri 11pt, 8 for Aptos Narrow 12pt. `font_size` is in pixels.
fn font_max_digit_width(font_name: &str, font_size: f64) -> f64 {
    let name = font_name.trim().to_lowercase();
    let digit_width = DIGIT_WIDTHS
        .iter()
        .find(|(font, _)| *font == name)
        .map(|(_, width)| *width)
        .unwrap_or(DEFAULT_DIGIT_WIDTH);
    (digit_width * font_size).round().max(1.0)
}

/// Maximum digit width in pixels of the default font of a workbook: the unit
/// of the widths of its columns.
///
/// The default font is the font of the `Normal` cell style.
pub(crate) fn max_digit_width(styles: &Styles) -> f64 {
    let xf_id = styles
        .cell_styles
        .iter()
        .find(|style| style.builtin_id == 0)
        .map(|style| style.xf_id)
        .unwrap_or(0);
    let font = styles
        .cell_style_xfs
        .get(xf_id as usize)
        .and_then(|xf| styles.fonts.get(xf.font_id as usize))
        .or(styles.fonts.first());
    match font {
        Some(font) => font_max_digit_width(&font.name, font.sz),
        None => 7.0,
    }
}

/// Removes the noise of the unit conversion so that, for instance, `11pt`
/// is written as `11` and not as `10.999999999999998`
fn round(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

/// Converts points (font sizes, row heights) to pixels
pub(crate) fn points_to_pixels(points: f64) -> f64 {
    // 96 / 72
    points * 4.0 / 3.0
}

/// Converts pixels to points (font sizes, row heights)
pub(crate) fn pixels_to_points(pixels: f64) -> f64 {
    round(pixels * 3.0 / 4.0)
}

/// Converts the `width` of a column in an xlsx file to pixels.
///
/// Excel stores the width truncated to 1/256th of a character, so a column
/// of 64 pixels is stored as `9.140625` and not as `64/7`. Widths that are
/// closer than that to a whole number of pixels are snapped to it.
pub(crate) fn column_width_to_pixels(width: f64, max_digit_width: f64) -> f64 {
    let pixels = width * max_digit_width;
    let nearest = pixels.round();
    if (pixels - nearest).abs() <= max_digit_width / 256.0 {
        nearest
    } else {
        pixels
    }
}

/// The default width in pixels of the columns of a sheet where `characters`
/// digits of the default font fit (`baseColWidth`).
///
/// There are 5 pixels of padding and Excel rounds the result up to a
/// multiple of 8 pixels: 8 characters are 64 pixels.
pub(crate) fn base_column_width_to_pixels(characters: f64, max_digit_width: f64) -> f64 {
    ((characters * max_digit_width + 5.0) / 8.0).ceil() * 8.0
}

/// Converts pixels to the `width` of a column in an xlsx file
pub(crate) fn pixels_to_column_width(pixels: f64, max_digit_width: f64) -> f64 {
    round(pixels / max_digit_width)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn points() {
        assert_eq!(points_to_pixels(36.0), 48.0);
        assert_eq!(points_to_pixels(15.0), 20.0);
        assert_eq!(pixels_to_points(48.0), 36.0);
        assert_eq!(pixels_to_points(25.0), 18.75);
        // The round trip does not lose precision
        for points in [8.0, 9.0, 10.0, 10.5, 11.0, 12.0, 14.0, 14.5, 16.0, 72.0] {
            assert_eq!(pixels_to_points(points_to_pixels(points)), points);
        }
    }

    #[test]
    fn column_widths() {
        // Excel's default column is 64 pixels
        assert_eq!(base_column_width_to_pixels(8.0, 7.0), 64.0);
        assert_eq!(base_column_width_to_pixels(10.0, 7.0), 80.0);
        assert_eq!(base_column_width_to_pixels(10.0, 8.0), 88.0);
        assert_eq!(column_width_to_pixels(9.140625, 7.0), 64.0);
        assert_eq!(column_width_to_pixels(10.0, 7.0), 70.0);
        assert_eq!(column_width_to_pixels(10.0, 8.0), 80.0);
        // widths that are not a whole number of pixels are kept
        assert_eq!(column_width_to_pixels(38.26953125, 7.0), 267.88671875);
        for max_digit_width in [6.0, 7.0, 8.0] {
            for pixels in [1.0, 20.0, 64.0, 90.0, 90.5, 123.0, 1000.0] {
                let width = pixels_to_column_width(pixels, max_digit_width);
                assert!((column_width_to_pixels(width, max_digit_width) - pixels).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn digit_widths() {
        let px = points_to_pixels;
        assert_eq!(font_max_digit_width("Calibri", px(11.0)), 7.0);
        assert_eq!(font_max_digit_width("Arial", px(10.0)), 7.0);
        assert_eq!(font_max_digit_width("Aptos Narrow", px(11.0)), 7.0);
        assert_eq!(font_max_digit_width("Aptos Narrow", px(12.0)), 8.0);
        assert_eq!(font_max_digit_width("Times New Roman", px(12.0)), 8.0);
        assert_eq!(font_max_digit_width("Courier New", px(10.0)), 8.0);
        // The default font of IronCalc
        assert_eq!(font_max_digit_width("Inter", 12.0), 7.0);
        assert_eq!(max_digit_width(&Styles::default()), 7.0);
        assert_eq!(font_max_digit_width("An unknown font", px(11.0)), 7.0);
    }
}
