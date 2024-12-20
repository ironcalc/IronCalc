use crate::types::BorderItem;

fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;
            // Expand single hex digits to full bytes
            Some((r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

fn compute_luminance(r: u8, g: u8, b: u8) -> f64 {
    // Normalize RGB values to [0, 1]
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;
    // Calculate luminance using the Rec. 601 formula
    0.299 * r + 0.587 * g + 0.114 * b
}

fn is_max_color(a: &str, b: &str) -> bool {
    let (ar, ag, ab) = match parse_color(a) {
        Some(rgb) => rgb,
        None => return false, // Invalid color format for 'a'
    };

    let (br, bg, bb) = match parse_color(b) {
        Some(rgb) => rgb,
        None => return false, // Invalid color format for 'b'
    };

    let luminance_a = compute_luminance(ar, ag, ab);
    let luminance_b = compute_luminance(br, bg, bb);

    // 'b' is heavier if its luminance is less than 'a's luminance
    luminance_b < luminance_a
}

/// Is border b "heavier" than a?
pub(crate) fn is_max_border(a: Option<&BorderItem>, b: Option<&BorderItem>) -> bool {
    match (a, b) {
        (_, None) => false,
        (None, Some(_)) => true,
        (Some(item_a), Some(item_b)) => {
            if item_a.style < item_b.style {
                return true;
            } else if item_a.style > item_b.style {
                return false;
            }
            match (&item_a.color, &item_b.color) {
                (_, None) => false,
                (None, Some(_)) => true,
                (Some(color_a), Some(color_b)) => is_max_color(color_a, color_b),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BorderStyle;

    #[test]
    fn compare_borders() {
        let b = BorderItem {
            style: BorderStyle::Thin,
            color: Some("#FFF".to_string()),
        };
        // Some border *always* beats no border
        assert!(is_max_border(None, Some(&b)));

        // No border is beaten by some border
        assert!(!is_max_border(Some(&b), None));
    }

    #[test]
    fn basic_colors() {
        // Black vs White
        assert!(is_max_color("#FFFFFF", "#000000"));
        assert!(!is_max_color("#000000", "#FFFFFF"));

        // Red vs Dark Red
        assert!(is_max_color("#FF0000", "#800000"));
        assert!(!is_max_color("#800000", "#FF0000"));

        // Green vs Dark Green
        assert!(is_max_color("#00FF00", "#008000"));
        assert!(!is_max_color("#008000", "#00FF00"));

        // Blue vs Dark Blue
        assert!(is_max_color("#0000FF", "#000080"));
        assert!(!is_max_color("#000080", "#0000FF"));
    }

    #[test]
    fn same_color() {
        // Comparing the same color should return false
        assert!(!is_max_color("#123456", "#123456"));
    }

    #[test]
    fn edge_cases() {
        // Colors with minimal luminance difference
        assert!(!is_max_color("#000000", "#010101"));
        assert!(!is_max_color("#FEFEFE", "#FFFFFF"));
        assert!(!is_max_color("#7F7F7F", "#808080"));
    }

    #[test]
    fn luminance_ordering() {
        // Colors with known luminance differences
        assert!(is_max_color("#CCCCCC", "#333333")); // Light gray vs Day
        assert!(is_max_color("#FFFF00", "#808000")); // Yellow ve
        assert!(is_max_color("#FF00FF", "#800080")); // Magenta vle
    }

    #[test]
    fn borderline_cases() {
        // Testing colors with equal luminance
        assert!(!is_max_color("#777777", "#777777"));

        // Testing black against near-black
        assert!(!is_max_color("#000000", "#010000"));
    }
}
