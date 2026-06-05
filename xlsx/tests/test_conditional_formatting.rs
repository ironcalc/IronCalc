#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::expect_used)]

use std::fs;

use ironcalc::{export::save_to_xlsx, import::load_from_xlsx};
use ironcalc_base::{cf_types::Icon, types::Color, Model, UserModel};

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
            assert_eq!(style.font.color, Color::Rgb("#9C0006".to_string()));
            assert_eq!(style.fill.color, Color::Rgb("#FFC7CE".to_string()));
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
            assert_eq!(icon.color, "#ffeb84".to_string());
        }
    }
    {
        // D16 is a flag
        let (sheet, column) = (1, 4);
        let row = 16;
        let extended_style = model
            .get_extended_style_for_cell(sheet, row, column)
            .unwrap();
        let icon = extended_style.icon.unwrap();
        assert_eq!(icon.icon, Icon::Flag);
        assert_eq!(icon.color, "#f8696b".to_string());
    }
}

fn test_cf_lists(model: Model) {
    let model = UserModel::from_model(model);
    {
        // Sheet1
        let list = model.get_conditional_formatting_list(0).unwrap();
        assert_eq!(list.len(), 18);
    }
    {
        // IconSets
        let list = model.get_conditional_formatting_list(1).unwrap();
        assert_eq!(list.len(), 20);
    }
    {
        // Text
        let list = model.get_conditional_formatting_list(2).unwrap();
        assert_eq!(list.len(), 3);
    }
    {
        // TimePeriod
        let list = model.get_conditional_formatting_list(3).unwrap();
        assert_eq!(list.len(), 4);
    }
    {
        // ColorScales3
        let list = model.get_conditional_formatting_list(4).unwrap();
        assert_eq!(list.len(), 3);
    }
    {
        // ColorScales2
        let list = model.get_conditional_formatting_list(5).unwrap();
        assert_eq!(list.len(), 2);
    }
    {
        // stop-if-true
        let list = model.get_conditional_formatting_list(6).unwrap();
        assert_eq!(list.len(), 4);
    }
}

#[test]
fn test_conditional_formatting_lists() {
    let model = load_from_xlsx(
        "tests/conditional_formatting/cf_tests.xlsx",
        "en",
        "UTC",
        "en",
    )
    .unwrap();
    // save and load back to check that lists are preserved
    let temp_file_name = "tests/conditional_formatting/cf_tests_round_trip.xlsx";
    save_to_xlsx(&model, temp_file_name).unwrap();
    test_cf_lists(model);

    let imported = load_from_xlsx(temp_file_name, "en", "UTC", "en").unwrap();
    test_cf_lists(imported);
    fs::remove_file(temp_file_name).unwrap();
}
