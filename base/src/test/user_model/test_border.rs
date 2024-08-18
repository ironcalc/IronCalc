#![allow(clippy::unwrap_used)]

use crate::{
    expressions::{types::Area, utils::number_to_column},
    types::{Border, BorderItem, BorderStyle},
    BorderArea, UserModel,
};

#[test]
fn borders_all() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "All"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: Some(border_item.clone()),
                right: Some(border_item.clone()),
                top: Some(border_item.clone()),
                bottom: Some(border_item.clone()),
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }

    // Lets remove all of them:
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "None"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: None,
                top: None,
                bottom: None,
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
}

#[test]
fn borders_inner() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Inner"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    // The inner part all have borders
    for row in 6..8 {
        for column in 7..8 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: Some(border_item.clone()),
                right: Some(border_item.clone()),
                top: Some(border_item.clone()),
                bottom: Some(border_item.clone()),
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
    // F5 has border only left and bottom
    {
        // We check the border on F5
        let style = model.get_cell_style(0, 5, 6).unwrap();
        let border_item = BorderItem {
            style: BorderStyle::Thin,
            color: Some("#FF5566".to_string()),
        };
        // It should be right and bottom
        let expected_border = Border {
            diagonal_up: false,
            diagonal_down: false,
            left: None,
            right: Some(border_item.clone()),
            top: None,
            bottom: Some(border_item),
            diagonal: None,
        };
        assert_eq!(style.border, expected_border);
    }
    {
        // Then let's try the bottom-right border
        let style = model.get_cell_style(0, 8, 8).unwrap();
        let border_item = BorderItem {
            style: BorderStyle::Thin,
            color: Some("#FF5566".to_string()),
        };
        // It should be only left and top
        let expected_border = Border {
            diagonal_up: false,
            diagonal_down: false,
            left: Some(border_item.clone()),
            right: None,
            top: Some(border_item.clone()),
            bottom: None,
            diagonal: None,
        };
        assert_eq!(style.border, expected_border);
    }
}

#[test]
fn borders_outer() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Outer"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    {
        // We check the border on F5
        let style = model.get_cell_style(0, 5, 6).unwrap();
        let border_item = BorderItem {
            style: BorderStyle::Thin,
            color: Some("#FF5566".to_string()),
        };
        // It should be only left and top
        let expected_border = Border {
            diagonal_up: false,
            diagonal_down: false,
            left: Some(border_item.clone()),
            right: None,
            top: Some(border_item),
            bottom: None,
            diagonal: None,
        };
        assert_eq!(style.border, expected_border);
    }
    {
        // Then let's try the bottom-right border
        let style = model.get_cell_style(0, 8, 8).unwrap();
        let border_item = BorderItem {
            style: BorderStyle::Thin,
            color: Some("#FF5566".to_string()),
        };
        // It should be only left and top
        let expected_border = Border {
            diagonal_up: false,
            diagonal_down: false,
            left: None,
            right: Some(border_item.clone()),
            top: None,
            bottom: Some(border_item.clone()),
            diagonal: None,
        };
        assert_eq!(style.border, expected_border);
    }
}

#[test]
fn borders_top() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Top"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: None,
                top: Some(border_item.clone()),
                bottom: None,
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
}

#[test]
fn borders_right() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Right"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: Some(border_item.clone()),
                top: None,
                bottom: None,
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
}

#[test]
fn borders_bottom() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Bottom"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: None,
                right: None,
                top: None,
                bottom: Some(border_item.clone()),
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
}

#[test]
fn borders_left() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    // We set an outer border in cells F5:H9
    let range = &Area {
        sheet: 0,
        row: 5,
        column: 6,
        width: 3,
        height: 4,
    };
    assert_eq!(number_to_column(6).unwrap(), "F");
    assert_eq!(number_to_column(8).unwrap(), "H");
    // ATM we don't have a way to create the object from Rust, that's ok.
    let border_area: BorderArea = serde_json::from_str(
        r##"{
      "item": {
        "style": "thin",
        "color": "#FF5566"
      },
      "type": "Left"
    }"##,
    )
    .unwrap();
    model.set_area_with_border(range, &border_area).unwrap();
    for row in 5..9 {
        for column in 6..9 {
            let style = model.get_cell_style(0, row, column).unwrap();
            let border_item = BorderItem {
                style: BorderStyle::Thin,
                color: Some("#FF5566".to_string()),
            };
            let expected_border = Border {
                diagonal_up: false,
                diagonal_down: false,
                left: Some(border_item.clone()),
                right: None,
                top: None,
                bottom: None,
                diagonal: None,
            };
            assert_eq!(style.border, expected_border);
        }
    }
}
