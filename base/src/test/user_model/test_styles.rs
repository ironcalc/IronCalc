#![allow(clippy::unwrap_used)]

use crate::{
    expressions::types::Area,
    types::{Alignment, BorderItem, BorderStyle, HorizontalAlignment, VerticalAlignment},
    UserModel,
};

#[test]
fn basic_fonts() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(!style.font.i);
    assert!(!style.font.b);
    assert!(!style.font.u);
    assert!(!style.font.strike);
    assert_eq!(style.font.color, Some("#000000".to_owned()));

    // bold
    model.update_range_style(&range, "font.b", "true").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.b);

    // italics
    model.update_range_style(&range, "font.i", "true").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.i);

    // underline
    model.update_range_style(&range, "font.u", "true").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.u);

    // strike
    model
        .update_range_style(&range, "font.strike", "true")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.strike);

    // color
    model
        .update_range_style(&range, "font.color", "#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.font.color, Some("#F1F1F1".to_owned()));

    while model.can_undo() {
        model.undo().unwrap();
    }

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(!style.font.i);
    assert!(!style.font.b);
    assert!(!style.font.u);
    assert!(!style.font.strike);
    assert_eq!(style.font.color, Some("#000000".to_owned()));

    while model.can_redo() {
        model.redo().unwrap();
    }

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.i);
    assert!(style.font.b);
    assert!(style.font.u);
    assert!(style.font.strike);
    assert_eq!(style.font.color, Some("#F1F1F1".to_owned()));

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    let style = model2.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.i);
    assert!(style.font.b);
    assert!(style.font.u);
    assert!(style.font.strike);
    assert_eq!(style.font.color, Some("#F1F1F1".to_owned()));
}

#[test]
fn font_errors() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    assert_eq!(
        model.update_range_style(&range, "font.b", "True"),
        Err("Invalid value for boolean: 'True'.".to_string())
    );
    assert_eq!(
        model.update_range_style(&range, "font.i", "FALSE"),
        Err("Invalid value for boolean: 'FALSE'.".to_string())
    );
    assert_eq!(
        model.update_range_style(&range, "font.bold", "true"),
        Err("Invalid style path: 'font.bold'.".to_string())
    );
    assert_eq!(
        model.update_range_style(&range, "font.strike", ""),
        Err("Invalid value for boolean: ''.".to_string())
    );
    // There is no cast for booleans
    assert_eq!(
        model.update_range_style(&range, "font.b", "1"),
        Err("Invalid value for boolean: '1'.".to_string())
    );
    // colors don't work by name
    assert_eq!(
        model.update_range_style(&range, "font.color", "blue"),
        Err("Invalid color: 'blue'.".to_string())
    );
    // No short form
    assert_eq!(
        model.update_range_style(&range, "font.color", "#FFF"),
        Err("Invalid color: '#FFF'.".to_string())
    );
}

#[test]
fn basic_fill() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.fill.bg_color, None);
    assert_eq!(style.fill.fg_color, None);

    // bg_color
    model
        .update_range_style(&range, "fill.bg_color", "#F2F2F2")
        .unwrap();
    model
        .update_range_style(&range, "fill.fg_color", "#F3F4F5")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#F2F2F2".to_owned()));
    assert_eq!(style.fill.fg_color, Some("#F3F4F5".to_owned()));

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    let style = model2.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.fill.bg_color, Some("#F2F2F2".to_owned()));
    assert_eq!(style.fill.fg_color, Some("#F3F4F5".to_owned()));
}

#[test]
fn fill_errors() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    assert_eq!(
        model.update_range_style(&range, "fill.bg_color", "#FFF"),
        Err("Invalid color: '#FFF'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "fill.fg_color", "#FFF"),
        Err("Invalid color: '#FFF'.".to_string())
    );
}

#[test]
fn basic_format() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.num_fmt, "general");

    model
        .update_range_style(&range, "num_fmt", "$#,##0.0000")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.num_fmt, "$#,##0.0000");

    model.undo().unwrap();

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.num_fmt, "general");

    model.redo().unwrap();

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.num_fmt, "$#,##0.0000");

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    let style = model2.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.num_fmt, "$#,##0.0000");
}

#[test]
fn basic_borders() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    model
        .update_range_style(&range, "border.left", "thin,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::Thin,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.left", "thin,")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::Thin,
            color: None,
        })
    );

    model
        .update_range_style(&range, "border.right", "dotted,#F1F1F2")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.right,
        Some(BorderItem {
            style: BorderStyle::Dotted,
            color: Some("#F1F1F2".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.top", "double,#F1F1F3")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.top,
        Some(BorderItem {
            style: BorderStyle::Double,
            color: Some("#F1F1F3".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.bottom", "medium,#F1F1F4")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.bottom,
        Some(BorderItem {
            style: BorderStyle::Medium,
            color: Some("#F1F1F4".to_owned()),
        })
    );

    while model.can_undo() {
        model.undo().unwrap();
    }

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.border.left, None);
    assert_eq!(style.border.top, None);
    assert_eq!(style.border.right, None);
    assert_eq!(style.border.bottom, None);

    while model.can_redo() {
        model.redo().unwrap();
    }

    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::Thin,
            color: None,
        })
    );
    assert_eq!(
        style.border.right,
        Some(BorderItem {
            style: BorderStyle::Dotted,
            color: Some("#F1F1F2".to_owned()),
        })
    );
    assert_eq!(
        style.border.top,
        Some(BorderItem {
            style: BorderStyle::Double,
            color: Some("#F1F1F3".to_owned()),
        })
    );
    assert_eq!(
        style.border.bottom,
        Some(BorderItem {
            style: BorderStyle::Medium,
            color: Some("#F1F1F4".to_owned()),
        })
    );

    let send_queue = model.flush_send_queue();

    let mut model2 = UserModel::new_empty("model", "en", "UTC").unwrap();
    model2.apply_external_diffs(&send_queue).unwrap();

    let style = model2.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::Thin,
            color: None,
        })
    );
    assert_eq!(
        style.border.right,
        Some(BorderItem {
            style: BorderStyle::Dotted,
            color: Some("#F1F1F2".to_owned()),
        })
    );
    assert_eq!(
        style.border.top,
        Some(BorderItem {
            style: BorderStyle::Double,
            color: Some("#F1F1F3".to_owned()),
        })
    );
    assert_eq!(
        style.border.bottom,
        Some(BorderItem {
            style: BorderStyle::Medium,
            color: Some("#F1F1F4".to_owned()),
        })
    );
}

#[test]
fn basic_alignment() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(alignment, None);

    model
        .update_range_style(&range, "alignment.horizontal", "center")
        .unwrap();
    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::Center,
            vertical: VerticalAlignment::Bottom,
            wrap_text: false
        })
    );

    model
        .update_range_style(&range, "alignment.horizontal", "centerContinuous")
        .unwrap();
    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::CenterContinuous,
            vertical: VerticalAlignment::Bottom,
            wrap_text: false
        })
    );

    let range = Area {
        sheet: 0,
        row: 2,
        column: 2,
        width: 1,
        height: 1,
    };

    model
        .update_range_style(&range, "alignment.vertical", "distributed")
        .unwrap();
    let alignment = model.get_cell_style(0, 2, 2).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::General,
            vertical: VerticalAlignment::Distributed,
            wrap_text: false
        })
    );

    model
        .update_range_style(&range, "alignment.vertical", "justify")
        .unwrap();
    let alignment = model.get_cell_style(0, 2, 2).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::General,
            vertical: VerticalAlignment::Justify,
            wrap_text: false
        })
    );

    model.update_range_style(&range, "alignment", "").unwrap();
    let alignment = model.get_cell_style(0, 2, 2).unwrap().alignment;
    assert_eq!(alignment, None);

    model.undo().unwrap();

    let alignment = model.get_cell_style(0, 2, 2).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::General,
            vertical: VerticalAlignment::Justify,
            wrap_text: false
        })
    );
}

#[test]
fn alignment_errors() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    assert_eq!(
        model.update_range_style(&range, "alignment", "some"),
        Err("Alignment must be empty, but found: 'some'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "alignment.vertical", "justified"),
        Err("Invalid value for vertical alignment: 'justified'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "alignment.horizontal", "unjustified"),
        Err("Invalid value for horizontal alignment: 'unjustified'.".to_string())
    );

    model
        .update_range_style(&range, "alignment.vertical", "justify")
        .unwrap();

    // Also fail if there is an alignment

    assert_eq!(
        model.update_range_style(&range, "alignment", "some"),
        Err("Alignment must be empty, but found: 'some'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "alignment.vertical", "justified"),
        Err("Invalid value for vertical alignment: 'justified'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "alignment.horizontal", "unjustified"),
        Err("Invalid value for horizontal alignment: 'unjustified'.".to_string())
    );
}

#[test]
fn basic_wrap_text() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    assert_eq!(
        model.update_range_style(&range, "alignment.wrap_text", "T"),
        Err("Invalid value for boolean: 'T'.".to_string())
    );
    model
        .update_range_style(&range, "alignment.wrap_text", "true")
        .unwrap();
    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::General,
            vertical: VerticalAlignment::Bottom,
            wrap_text: true
        })
    );
    model.undo().unwrap();
    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(alignment, None);

    model.redo().unwrap();

    let alignment = model.get_cell_style(0, 1, 1).unwrap().alignment;
    assert_eq!(
        alignment,
        Some(Alignment {
            horizontal: HorizontalAlignment::General,
            vertical: VerticalAlignment::Bottom,
            wrap_text: true
        })
    );

    assert_eq!(
        model.update_range_style(&range, "alignment.wrap_text", "True"),
        Err("Invalid value for boolean: 'True'.".to_string())
    );
}

#[test]
fn more_basic_borders() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    model
        .update_range_style(&range, "border.left", "thick,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::Thick,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.left", "slantDashDot,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::SlantDashDot,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.left", "mediumDashDot,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::MediumDashDot,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.left", "mediumDashDotDot,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::MediumDashDotDot,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model
        .update_range_style(&range, "border.left", "mediumDashed,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::MediumDashed,
            color: Some("#F1F1F1".to_owned()),
        })
    );
}

#[test]
fn border_errors() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    assert_eq!(
        model.update_range_style(&range, "border.lef", "thick,#F1F1F1"),
        Err("Invalid style path: 'border.lef'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "border.left", "thic,#F1F1F1"),
        Err("Invalid border style: 'thic'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "border.left", "thick,#F1F1F"),
        Err("Invalid color: '#F1F1F'.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "border.left", " "),
        Err("Invalid border value: ' '.".to_string())
    );

    assert_eq!(
        model.update_range_style(&range, "border.left", "thick,#F1F1F1,thin"),
        Err("Invalid border value: 'thick,#F1F1F1,thin'.".to_string())
    );
}

#[test]
fn empty_removes_border() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    model
        .update_range_style(&range, "border.left", "mediumDashDotDot,#F1F1F1")
        .unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(
        style.border.left,
        Some(BorderItem {
            style: BorderStyle::MediumDashDotDot,
            color: Some("#F1F1F1".to_owned()),
        })
    );

    model.update_range_style(&range, "border.left", "").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert_eq!(style.border.left, None);
}

#[test]
fn false_removes_value() {
    let mut model = UserModel::new_empty("model", "en", "UTC").unwrap();
    let range = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };

    // bold
    model.update_range_style(&range, "font.b", "true").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(style.font.b);

    model.update_range_style(&range, "font.b", "false").unwrap();
    let style = model.get_cell_style(0, 1, 1).unwrap();
    assert!(!style.font.b);
}
