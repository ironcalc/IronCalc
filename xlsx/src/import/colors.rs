#![allow(clippy::unwrap_used)]

use core::cmp::max;
use core::cmp::min;

// https://gist.github.com/emanuel-sanabria-developer/5793377
// https://github.com/ClosedXML/ClosedXML/wiki/Excel-Indexed-Colors

// Warning: Excel uses a weird normalization for HSL colors (0, 255)
// We use a more standard one but our HSL numbers will not coincide with Excel's

pub(crate) fn hex_to_rgb(h: &str) -> [i32; 3] {
    let r = i32::from_str_radix(&h[1..3], 16).unwrap();
    let g = i32::from_str_radix(&h[3..5], 16).unwrap();
    let b = i32::from_str_radix(&h[5..7], 16).unwrap();
    [r, g, b]
}

pub(crate) fn rgb_to_hex(rgb: [i32; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

pub(crate) fn rgb_to_hsl(rgb: [i32; 3]) -> [i32; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let red = r as f64 / 255.0;
    let green = g as f64 / 255.0;
    let blue = b as f64 / 255.0;
    let max_color = max(max(r, g), b);
    let min_color = min(min(r, g), b);
    let chroma = (max_color - min_color) as f64 / 255.0;
    if chroma == 0.0 {
        return [0, 0, (red * 100.0).round() as i32];
    }

    let hue;
    let luminosity = (max_color + min_color) as f64 / (255.0 * 2.0);
    let saturation = if luminosity > 0.5 {
        0.5 * chroma / (1.0 - luminosity)
    } else {
        0.5 * chroma / luminosity
    };
    if max_color == r {
        if green >= blue {
            hue = 60.0 * (green - blue) / chroma;
        } else {
            hue = ((green - blue) / chroma + 6.0) * 60.0;
        }
    } else if max_color == g {
        hue = ((blue - red) / chroma + 2.0) * 60.0;
    } else {
        hue = ((red - green) / chroma + 4.0) * 60.0;
    }
    let hue = hue.round() as i32;
    let saturation = (saturation * 100.0).round() as i32;
    let luminosity = (luminosity * 100.0).round() as i32;
    [hue, saturation, luminosity]
}

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut c = t;
    if c < 0.0 {
        c += 1.0;
    }
    if c > 1.0 {
        c -= 1.0;
    }
    if c < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    };
    if c < 0.5 {
        return q;
    };
    if c < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    };
    p
}

pub(crate) fn hsl_to_rgb(hsl: [i32; 3]) -> [i32; 3] {
    let hue = (hsl[0] as f64) / 360.0;
    let saturation = (hsl[1] as f64) / 100.0;
    let luminosity = (hsl[2] as f64) / 100.0;
    let red;
    let green;
    let blue;

    if saturation == 0.0 {
        // achromatic
        red = luminosity * 255.0;
        green = luminosity * 255.0;
        blue = luminosity * 255.0;
    } else {
        let q = if luminosity < 0.5 {
            luminosity * (1.0 + saturation)
        } else {
            luminosity + saturation - luminosity * saturation
        };
        let p = 2.0 * luminosity - q;
        red = 255.0 * hue_to_rgb(p, q, hue + 1.0 / 3.0);
        green = 255.0 * hue_to_rgb(p, q, hue);
        blue = 255.0 * hue_to_rgb(p, q, hue - 1.0 / 3.0);
    }
    [
        red.round() as i32,
        green.round() as i32,
        blue.round() as i32,
    ]
}

/* 18.8.3 bgColor tint algorithm */
fn hex_with_tint_to_rgb(hex: &str, tint: f64) -> String {
    if tint == 0.0 {
        return hex.to_string();
    }
    let mut hsl = rgb_to_hsl(hex_to_rgb(hex));
    let l = hsl[2] as f64;
    if tint < 0.0 {
        // Lum’ = Lum * (1.0 + tint)
        hsl[2] = (l * (1.0 + tint)).round() as i32;
    } else {
        // HLSMAX here would be 100, for Excel 255
        // Lum‘ = Lum * (1.0-tint) + (HLSMAX – HLSMAX * (1.0-tint))
        hsl[2] = (l + (100.0 - l) * tint).round() as i32;
    };
    rgb_to_hex(hsl_to_rgb(hsl))
}

pub fn get_themed_color(theme: i32, tint: f64) -> String {
    let color_theme = [
        "#FFFFFF", "#000000", // "window",
        "#E7E6E6", "#44546A", "#4472C4", "#ED7D31", "#A5A5A5", "#FFC000", "#5B9BD5", "#70AD47",
        "#0563C1", "#954F72",
    ];
    hex_with_tint_to_rgb(color_theme[theme as usize], tint)
}

pub fn get_indexed_color(index: i32) -> String {
    let color_list = [
        "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
        "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
        "#800000", "#008000", "#000080", "#808000", "#800080", "#008080", "#C0C0C0", "#808080",
        "#9999FF", "#993366", "#FFFFCC", "#CCFFFF", "#660066", "#FF8080", "#0066CC", "#CCCCFF",
        "#000080", "#FF00FF", "#FFFF00", "#00FFFF", "#800080", "#800000", "#008080", "#0000FF",
        "#00CCFF", "#CCFFFF", "#CCFFCC", "#FFFF99", "#99CCFF", "#FF99CC", "#CC99FF", "#FFCC99",
        "#3366FF", "#33CCCC", "#99CC00", "#FFCC00", "#FF9900", "#FF6600", "#666699", "#969696",
        "#003366", "#339966", "#003300", "#333300", "#993300", "#993366", "#333399",
        "#333333",
        // 64, Transparent)
    ];
    if index > 63 {
        return color_list[0].to_string();
    }
    color_list[index as usize].to_string()
}

#[cfg(test)]
mod tests {
    use crate::import::colors::*;

    #[test]
    fn test_known_colors() {
        let color1 = get_themed_color(0, -0.05);
        assert_eq!(color1, "#F2F2F2");

        let color2 = get_themed_color(5, -0.25);
        // Excel returns "#C65911" (rounding error)
        assert_eq!(color2, "#C55911");

        let color3 = get_themed_color(4, 0.6);
        // Excel returns "#b4c6e7" (rounding error)
        assert_eq!(color3, "#B5C8E8");
    }

    #[test]
    fn test_rgb_hex() {
        struct ColorTest {
            hex: String,
            rgb: [i32; 3],
            hsl: [i32; 3],
        }
        let color_tests = [
            ColorTest {
                hex: "#FFFFFF".to_string(),
                rgb: [255, 255, 255],
                hsl: [0, 0, 100],
            },
            ColorTest {
                hex: "#000000".to_string(),
                rgb: [0, 0, 0],
                hsl: [0, 0, 0],
            },
            ColorTest {
                hex: "#44546A".to_string(),
                rgb: [68, 84, 106],
                hsl: [215, 22, 34],
            },
            ColorTest {
                hex: "#E7E6E6".to_string(),
                rgb: [231, 230, 230],
                hsl: [0, 2, 90],
            },
            ColorTest {
                hex: "#4472C4".to_string(),
                rgb: [68, 114, 196],
                hsl: [218, 52, 52],
            },
            ColorTest {
                hex: "#ED7D31".to_string(),
                rgb: [237, 125, 49],
                hsl: [24, 84, 56],
            },
            ColorTest {
                hex: "#A5A5A5".to_string(),
                rgb: [165, 165, 165],
                hsl: [0, 0, 65],
            },
            ColorTest {
                hex: "#FFC000".to_string(),
                rgb: [255, 192, 0],
                hsl: [45, 100, 50],
            },
            ColorTest {
                hex: "#5B9BD5".to_string(),
                rgb: [91, 155, 213],
                hsl: [209, 59, 60],
            },
            ColorTest {
                hex: "#70AD47".to_string(),
                rgb: [112, 173, 71],
                hsl: [96, 42, 48],
            },
            ColorTest {
                hex: "#0563C1".to_string(),
                rgb: [5, 99, 193],
                hsl: [210, 95, 39],
            },
            ColorTest {
                hex: "#954F72".to_string(),
                rgb: [149, 79, 114],
                hsl: [330, 31, 45],
            },
        ];
        for color in color_tests.iter() {
            let rgb = color.rgb;
            let hsl = color.hsl;
            assert_eq!(rgb, hex_to_rgb(&color.hex));
            assert_eq!(hsl, rgb_to_hsl(rgb));
            assert_eq!(rgb_to_hex(rgb), color.hex);
            // The round trip has rounding errors
            // FIXME: We could also hardcode the hsl21 in the testcase
            let rgb2 = hsl_to_rgb(hsl);
            let diff =
                (rgb2[0] - rgb[0]).abs() + (rgb2[1] - rgb[1]).abs() + (rgb2[2] - rgb[2]).abs();
            assert!(diff < 4);
        }
    }
}
