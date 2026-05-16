#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;
use ironcalc_base::cf_types::Icon;

#[test]
fn test_cf_file() {
    // Text color is #9c0006
    // Background color is #ffc7ce
    let model = load_from_xlsx(
        "tests/conditional_formatting/cf_tests.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    {
        // Greater than 2 (data is 1,2,3,4,5,6)
        // A4-A7 should have the same conditional formatting
        for row in 4..=7 {
            let (sheet, column) = (0, 1);
            let extended_style = model
                .get_extended_style_for_cell(sheet, row, column)
                .unwrap();
            let style = extended_style.style;
            assert_eq!(style.font.color, Some("#9C0006".to_string()));
            assert_eq!(style.fill.bg_color, Some("#FFC7CE".to_string()));
        }
    }

    {
        // Icon sets A17, A22
        let (sheet, column) = (0, 1);
        {
            let row = 17;
            let extended_style = model
                .get_extended_style_for_cell(sheet, row, column)
                .unwrap();
            let icon = extended_style.icon.unwrap();
            assert_eq!(icon.icon, Icon::ArrowDown);
            assert_eq!(icon.color, "#e43400".to_string());
        }
        {
            let row = 19;
            let extended_style = model
                .get_extended_style_for_cell(sheet, row, column)
                .unwrap();
            let icon = extended_style.icon.unwrap();
            assert_eq!(icon.icon, Icon::ArrowAngleDown);
            assert_eq!(icon.color, "#e43400".to_string());
        }
    }
}
