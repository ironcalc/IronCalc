//! Conversions between the units used in xlsx files and the pixels IronCalc
//! uses internally.
//!
//! IronCalc works in (CSS) pixels: 96 pixels per inch. In an xlsx file:
//!
//! * Font sizes and row heights are in points: 72 points per inch, so one
//!   point is 4/3 of a pixel and `36pt` is `48px`.
//! * Column widths are in "characters": multiples of the maximum digit width
//!   of the default font of the workbook (ECMA-376 Part 1, 18.3.1.13). We do
//!   not measure fonts, we assume the 7 pixels of Calibri 11, that is also
//!   the value for most of the default fonts in use.

/// Maximum digit width, in pixels, of the default font
const MAX_DIGIT_WIDTH: f64 = 7.0;

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
pub(crate) fn column_width_to_pixels(width: f64) -> f64 {
    let pixels = width * MAX_DIGIT_WIDTH;
    let nearest = pixels.round();
    if (pixels - nearest).abs() <= MAX_DIGIT_WIDTH / 256.0 {
        nearest
    } else {
        pixels
    }
}

/// Converts pixels to the `width` of a column in an xlsx file
pub(crate) fn pixels_to_column_width(pixels: f64) -> f64 {
    round(pixels / MAX_DIGIT_WIDTH)
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
        assert_eq!(column_width_to_pixels(9.140625), 64.0);
        assert_eq!(column_width_to_pixels(10.0), 70.0);
        // widths that are not a whole number of pixels are kept
        assert_eq!(column_width_to_pixels(38.26953125), 267.88671875);
        assert_eq!(column_width_to_pixels(10.5), 73.5);
        for pixels in [1.0, 20.0, 64.0, 90.0, 90.5, 123.0, 1000.0] {
            let width = pixels_to_column_width(pixels);
            assert!((column_width_to_pixels(width) - pixels).abs() < 1e-5);
        }
    }
}
