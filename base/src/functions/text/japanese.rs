// ASC, JIS and PHONETIC: functions for Japanese text

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

// Half-width katakana (U+FF61..U+FF9F) and the full-width character of each one, in order
const HALF_WIDTH_KANA: &str = "｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ";
const FULL_WIDTH_KANA: &str =
    "。「」、・ヲァィゥェォャュョッーアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワン゛゜";

// Voiced katakana are two half-width characters: ガ = ｶ + ﾞ, パ = ﾊ + ﾟ
const VOICED: &str = "ガギグゲゴザジズゼゾダヂヅデドバビブベボヴ";
const VOICED_BASE: &str = "カキクケコサシスセソタチツテトハヒフヘホウ";
const SEMI_VOICED: &str = "パピプペポ";
const SEMI_VOICED_BASE: &str = "ハヒフヘホ";
const DAKUTEN: char = 'ﾞ';
const HANDAKUTEN: char = 'ﾟ';

fn position(table: &str, c: char) -> Option<usize> {
    table.chars().position(|x| x == c)
}

fn nth(table: &str, index: usize) -> Option<char> {
    table.chars().nth(index)
}

fn half_kana_to_full(c: char) -> Option<char> {
    position(HALF_WIDTH_KANA, c).and_then(|i| nth(FULL_WIDTH_KANA, i))
}

fn full_kana_to_half(c: char) -> Option<char> {
    position(FULL_WIDTH_KANA, c).and_then(|i| nth(HALF_WIDTH_KANA, i))
}

/// ASC: full-width (zenkaku) characters to half-width (hankaku).
/// ASCII letters, digits and symbols (U+FF01..U+FF5E), the ideographic space (U+3000) and
/// katakana. Hiragana and kanji have no half-width form and are kept.
pub(crate) fn asc_convert(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        let cp = c as u32;
        if cp == 0x3000 {
            result.push(' ');
        } else if (0xFF01..=0xFF5E).contains(&cp) {
            result.push(char::from_u32(cp - 0xFF01 + 0x0021).unwrap_or(c));
        } else if let Some(h) = full_kana_to_half(c) {
            result.push(h);
        } else if let Some(i) = position(VOICED, c) {
            result.extend(nth(VOICED_BASE, i).and_then(full_kana_to_half));
            result.push(DAKUTEN);
        } else if let Some(i) = position(SEMI_VOICED, c) {
            result.extend(nth(SEMI_VOICED_BASE, i).and_then(full_kana_to_half));
            result.push(HANDAKUTEN);
        } else {
            result.push(c);
        }
    }
    result
}

/// JIS: half-width (hankaku) characters to full-width (zenkaku), the reverse of ASC.
/// A half-width katakana followed by ﾞ or ﾟ becomes a single voiced katakana (ｶﾞ => ガ).
pub(crate) fn jis_convert(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len() * 3);
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        let cp = c as u32;
        index += 1;
        if c == ' ' {
            result.push('\u{3000}');
        } else if (0x21..=0x7E).contains(&cp) {
            result.push(char::from_u32(cp - 0x21 + 0xFF01).unwrap_or(c));
        } else if let Some(full) = half_kana_to_full(c) {
            let mark = chars.get(index).copied();
            let voiced = match mark {
                Some(DAKUTEN) => position(VOICED_BASE, full).and_then(|i| nth(VOICED, i)),
                Some(HANDAKUTEN) => {
                    position(SEMI_VOICED_BASE, full).and_then(|i| nth(SEMI_VOICED, i))
                }
                _ => None,
            };
            if let Some(v) = voiced {
                result.push(v);
                index += 1;
            } else {
                result.push(full);
            }
        } else {
            result.push(c);
        }
    }
    result
}

impl<'a> Model<'a> {
    /// ASC(text) — Converts full-width characters to half-width ones
    pub(crate) fn fn_asc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, asc_convert)
    }

    /// JIS(text) — Converts half-width characters to full-width ones
    pub(crate) fn fn_jis(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_text_unary(args, cell, jis_convert)
    }

    /// PHONETIC(reference) — Returns the furigana (reading) of the text in the reference.
    /// IronCalc does not keep furigana (they are not imported from xlsx files), and like Excel
    /// for a cell without furigana it returns the text itself. With a range, the texts of the
    /// cells are joined, row by row. Numbers, booleans and formula results are skipped, as in
    /// Excel.
    pub(crate) fn fn_phonetic(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let range = match self.get_reference(&args[0], cell) {
            Ok(range) => range,
            Err(error) => return error,
        };
        let (left, right) = (range.left, range.right);
        if left.sheet != right.sheet {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            );
        }
        // Only look at the used part of the sheet (PHONETIC(A:A) is fine)
        let dimension = match self.workbook.worksheet(left.sheet) {
            Ok(worksheet) => worksheet.dimension(),
            Err(message) => return CalcResult::new_error(Error::ERROR, cell, message),
        };
        let row_end = right.row.min(dimension.max_row);
        let column_end = right.column.min(dimension.max_column);
        let mut result = String::new();
        for row in left.row..=row_end {
            for column in left.column..=column_end {
                if let Ok(Some(_)) = self.get_cell_formula(left.sheet, row, column) {
                    continue;
                }
                if let CalcResult::String(text) = self.evaluate_cell(CellReferenceIndex {
                    sheet: left.sheet,
                    row,
                    column,
                }) {
                    result.push_str(&text);
                }
            }
        }
        CalcResult::String(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kana_tables_have_the_same_length() {
        assert_eq!(HALF_WIDTH_KANA.chars().count(), 0xFF9F - 0xFF61 + 1);
        assert_eq!(
            HALF_WIDTH_KANA.chars().count(),
            FULL_WIDTH_KANA.chars().count()
        );
        assert_eq!(VOICED.chars().count(), VOICED_BASE.chars().count());
        assert_eq!(
            SEMI_VOICED.chars().count(),
            SEMI_VOICED_BASE.chars().count()
        );
        for (i, c) in HALF_WIDTH_KANA.chars().enumerate() {
            assert_eq!(c as u32, 0xFF61 + i as u32);
        }
    }
}
