#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::export::save_to_xlsx;
use ironcalc::import::load_from_xlsx;
use std::fs;

#[test]
// A <font> with no <color> child is semantically equivalent to <color auto="1"/> in OOXML —
// both mean "use the automatic/default color." The importer must surface this as
// Font.color = None so callers can distinguish it from an explicit <color rgb="FF000000"/>.
//
// PRODUCT_SUM.xlsx contains three fonts: two with <color theme="1"/> (sz=11) and one with
// no <color> child (sz=8, the only font with that size).
fn test_font_with_no_color_element_is_none() {
    let model = load_from_xlsx("tests/calc_tests/PRODUCT_SUM.xlsx", "en", "UTC", "en").unwrap();
    let fonts = &model.workbook.styles.fonts;

    let no_color_font = fonts.iter().find(|f| f.sz == 8).unwrap();
    assert_eq!(no_color_font.color, None);

    let themed_fonts: Vec<_> = fonts.iter().filter(|f| f.sz == 11).collect();
    assert!(!themed_fonts.is_empty());
    for font in themed_fonts {
        assert!(font.color.is_some());
    }

    // Round-trip: the no-<color> font must not gain an explicit color on export.
    let temp_file_name = "temp_file_test_font_no_color.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    let reloaded = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    fs::remove_file(temp_file_name).unwrap();
    let reloaded_no_color = reloaded
        .workbook
        .styles
        .fonts
        .iter()
        .find(|f| f.sz == 8)
        .unwrap();
    assert_eq!(reloaded_no_color.color, None);
}
