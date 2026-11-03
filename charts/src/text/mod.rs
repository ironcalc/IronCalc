//! Text measurement without a browser or a font engine.
//!
//! The layout only ever asks two things of a font: how wide is this string,
//! and how tall is a line. [`TextMeasurer`] is that question; [`Inter`]
//! answers it from a table of advance widths generated from the Inter files
//! the webapp ships (`inter_v13`), which keeps the layout deterministic and
//! the same in wasm and native builds. Kerning is ignored, which makes a
//! line about one percent too wide.

mod inter_v13;

/// The box a string occupies, in pixels. `ascent` and `descent` are measured
/// from the baseline, both positive.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TextExtent {
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
}

impl TextExtent {
    pub fn height(&self) -> f64 {
        self.ascent + self.descent
    }
}

pub trait TextMeasurer {
    /// Measures `text` set at `size` pixels. Italic has the same widths as
    /// upright in Inter, so only the weight matters.
    fn measure(&self, text: &str, size: f64, bold: bool) -> TextExtent;
}

/// Inter, from its advance widths. The bold weight is the 600 the webapp
/// uses for bold text.
#[derive(Debug, Default, Clone, Copy)]
pub struct Inter;

/// The advance of a character in font units, the average for anything
/// outside ASCII and Latin-1.
fn advance(table: &[u16], character: char) -> f64 {
    let code_point = character as u32;
    let index = match code_point {
        0x20..=0x7E => code_point - 0x20,
        0xA0..=0xFF => code_point - 0xA0 + (0x7F - 0x20),
        _ => return inter_v13::AVERAGE_ADVANCE,
    };
    table
        .get(index as usize)
        .map_or(inter_v13::AVERAGE_ADVANCE, |advance| f64::from(*advance))
}

impl TextMeasurer for Inter {
    fn measure(&self, text: &str, size: f64, bold: bool) -> TextExtent {
        let table: &[u16] = if bold {
            &inter_v13::SEMIBOLD
        } else {
            &inter_v13::REGULAR
        };
        let units: f64 = text.chars().map(|c| advance(table, c)).sum();
        let scale = size / inter_v13::UNITS_PER_EM;
        TextExtent {
            width: units * scale,
            ascent: inter_v13::ASCENDER * scale,
            descent: -inter_v13::DESCENDER * scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_add_up_per_character() {
        let one_em = Inter.measure("0", 2816.0, false);
        assert_eq!(one_em.width, 1760.0);
        let two = Inter.measure("00", 2816.0, false);
        assert_eq!(two.width, 3520.0);
        assert_eq!(Inter.measure("", 12.0, false).width, 0.0);
    }

    #[test]
    fn bold_is_wider_and_narrow_letters_are_narrow() {
        let regular = Inter.measure("Hello", 12.0, false);
        let bold = Inter.measure("Hello", 12.0, true);
        assert!(bold.width > regular.width);
        assert!(Inter.measure("iii", 12.0, false).width < Inter.measure("WWW", 12.0, false).width);
    }

    #[test]
    fn latin_1_is_in_the_table_and_the_rest_is_average() {
        let e_acute = Inter.measure("é", 12.0, false).width;
        let e = Inter.measure("e", 12.0, false).width;
        assert!((e_acute - e).abs() < 0.01);
        let average = 1838.0 * 12.0 / 2816.0;
        assert!((Inter.measure("漢", 12.0, false).width - average).abs() < 1e-9);
    }

    #[test]
    fn line_height_is_inters() {
        let extent = Inter.measure("x", 16.0, false);
        // (2728 + 680) / 2816 em
        assert!((extent.height() - 16.0 * 3408.0 / 2816.0).abs() < 1e-9);
        assert!(extent.ascent > extent.descent);
    }
}
