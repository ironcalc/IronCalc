#![allow(clippy::unwrap_used)]

use crate::expressions::types::Area;
use crate::test::util::new_empty_model;
use crate::types::{Color, Link};
use crate::UserModel;

fn example_link() -> Link {
    Link::External {
        target: "https://www.ironcalc.com/".to_string(),
        tooltip: None,
    }
}

#[test]
fn add_update_delete() {
    let mut model = UserModel::from_model(new_empty_model());

    // no link in an empty cell
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));

    // add
    model.set_cell_link(0, 2, 2, example_link(), None).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    assert_eq!(model.get_links(0).unwrap().len(), 1);

    // update
    let updated = Link::Internal {
        location: "Sheet1!A30".to_string(),
        tooltip: Some("An internal link".to_string()),
    };
    model.set_cell_link(0, 2, 2, updated.clone(), None).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated)));
    assert_eq!(model.get_links(0).unwrap().len(), 1);

    // delete
    model.delete_cell_link(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert!(model.get_links(0).unwrap().is_empty());
}

#[test]
fn invalid_references() {
    let mut model = UserModel::from_model(new_empty_model());
    assert!(model.set_cell_link(1, 1, 1, example_link(), None).is_err());
    assert!(model.set_cell_link(0, 0, 1, example_link(), None).is_err());
    assert!(model
        .set_cell_link(0, 1, 20_000, example_link(), None)
        .is_err());
    assert!(model.get_cell_link(0, -1, 1).is_err());
}

#[test]
fn creating_a_link_sets_label_and_style() {
    let mut model = UserModel::from_model(new_empty_model());

    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    // the label is the cell content
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    // a new link gets the link style: underline + theme hyperlink color
    let style = model.get_cell_style(0, 2, 2).unwrap();
    assert!(style.font.u);
    assert_eq!(style.font.color, Color::Theme(10, 0.0));
}

#[test]
fn creating_a_link_is_a_single_undo_step() {
    let mut model = UserModel::from_model(new_empty_model());

    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    // one undo reverts the link, the content and the style together
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert_eq!(model.get_formatted_cell_value(0, 2, 2), Ok("".to_string()));
    let style = model.get_cell_style(0, 2, 2).unwrap();
    assert!(!style.font.u);
    assert_eq!(style.font.color, Color::None);
    assert!(!model.can_undo());

    // one redo restores everything
    model.redo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    let style = model.get_cell_style(0, 2, 2).unwrap();
    assert!(style.font.u);
    assert_eq!(style.font.color, Color::Theme(10, 0.0));
}

#[test]
fn updating_a_link_keeps_content_and_style() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    // the user customizes the style
    model
        .update_range_style(
            &crate::expressions::types::Area {
                sheet: 0,
                row: 2,
                column: 2,
                width: 1,
                height: 1,
            },
            "font.color",
            "#FF0000",
        )
        .unwrap();

    // updating the link of a cell that already has one touches neither the
    // content nor the style
    let updated = Link::Internal {
        location: "Sheet1!A30".to_string(),
        tooltip: None,
    };
    model.set_cell_link(0, 2, 2, updated.clone(), None).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated.clone())));
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    let style = model.get_cell_style(0, 2, 2).unwrap();
    assert_eq!(style.font.color, Color::Rgb("#FF0000".to_string()));

    // and so does deleting it
    model.delete_cell_link(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    let style = model.get_cell_style(0, 2, 2).unwrap();
    assert_eq!(style.font.color, Color::Rgb("#FF0000".to_string()));
    assert!(style.font.u);
}

#[test]
fn undo_redo() {
    let mut model = UserModel::from_model(new_empty_model());

    model.set_cell_link(0, 2, 2, example_link(), None).unwrap();
    let updated = Link::Internal {
        location: "Sheet1!A30".to_string(),
        tooltip: None,
    };
    model.set_cell_link(0, 2, 2, updated.clone(), None).unwrap();

    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));

    model.redo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    model.redo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated.clone())));

    // deleting is undoable too
    model.delete_cell_link(0, 2, 2).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(updated)));
}

#[test]
fn deleting_cell_contents_removes_the_link() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    // clearing the content with an empty input removes the link...
    model.set_user_input(0, 2, 2, "").unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    // ...and undo restores both the content and the link
    model.undo().unwrap();
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));

    // deleting a range removes the links in it
    let area = Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 5,
        height: 5,
    };
    model.range_clear_contents(&area).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    assert_eq!(
        model.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );

    // clear all (contents and formatting) removes the links too
    model.range_clear_all(&area).unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
}

#[test]
fn deleting_a_range_keeps_links_outside_of_it() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();
    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 5,
            column: 5,
            width: 2,
            height: 2,
        })
        .unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
}

#[test]
fn copy_paste_copies_the_link() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    // copy B2, paste into D5
    model.set_selected_cell(2, 2).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 4).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 5, 4),
        Ok("IronCalc".to_string())
    );
    assert_eq!(model.get_cell_link(0, 5, 4), Ok(Some(example_link())));
    // the source keeps its link on a copy
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));

    // undo removes the pasted link
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 5, 4), Ok(None));
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
}

#[test]
fn cut_paste_moves_the_link() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    model.set_selected_cell(2, 2).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 4).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, true)
        .unwrap();

    assert_eq!(model.get_cell_link(0, 5, 4), Ok(Some(example_link())));
    // the source loses its link on a cut
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));

    // undo restores both cells
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 5, 4), Ok(None));
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(Some(example_link())));
}

#[test]
fn pasting_over_a_linked_cell_replaces_the_link() {
    let mut model = UserModel::from_model(new_empty_model());
    // B2 has no link, D5 has one
    model.set_user_input(0, 2, 2, "Hello").unwrap();
    model
        .set_cell_link(0, 5, 4, example_link(), Some("IronCalc"))
        .unwrap();

    // paste the linkless B2 over D5
    model.set_selected_cell(2, 2).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(5, 4).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();

    assert_eq!(
        model.get_formatted_cell_value(0, 5, 4),
        Ok("Hello".to_string())
    );
    assert_eq!(model.get_cell_link(0, 5, 4), Ok(None));

    // undo restores the old link
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 5, 4), Ok(Some(example_link())));
}

#[test]
fn autofill_copies_the_link() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 1, 1, example_link(), Some("IronCalc"))
        .unwrap();

    // pull A1 down to A4
    model
        .auto_fill_rows(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 1,
            },
            4,
        )
        .unwrap();
    for row in 1..=4 {
        assert_eq!(model.get_cell_link(0, row, 1), Ok(Some(example_link())));
    }

    // and A1:A4 to the right up to column C
    model
        .auto_fill_columns(
            &Area {
                sheet: 0,
                row: 1,
                column: 1,
                width: 1,
                height: 4,
            },
            3,
        )
        .unwrap();
    for row in 1..=4 {
        for column in 1..=3 {
            assert_eq!(
                model.get_cell_link(0, row, column),
                Ok(Some(example_link()))
            );
        }
    }

    // undo removes the filled links (last operation: the column fill)
    model.undo().unwrap();
    for row in 1..=4 {
        assert_eq!(model.get_cell_link(0, row, 2), Ok(None));
        assert_eq!(model.get_cell_link(0, row, 3), Ok(None));
        assert_eq!(model.get_cell_link(0, row, 1), Ok(Some(example_link())));
    }
}

#[test]
fn deleting_cell_contents_removes_the_link_on_peers() {
    let mut model = UserModel::from_model(new_empty_model());
    let mut peer = UserModel::from_model(new_empty_model());

    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();
    peer.apply_external_diffs(&model.flush_send_queue())
        .unwrap();
    assert_eq!(peer.get_cell_link(0, 2, 2), Ok(Some(example_link())));

    model
        .range_clear_contents(&Area {
            sheet: 0,
            row: 2,
            column: 2,
            width: 1,
            height: 1,
        })
        .unwrap();
    peer.apply_external_diffs(&model.flush_send_queue())
        .unwrap();
    assert_eq!(peer.get_cell_link(0, 2, 2), Ok(None));
}

#[test]
fn no_op_operations_do_not_pollute_history() {
    let mut model = UserModel::from_model(new_empty_model());

    // deleting a link that is not there is a no-op
    model.delete_cell_link(0, 5, 5).unwrap();
    assert!(!model.can_undo());

    // setting the same link twice records only one history entry
    model.set_cell_link(0, 2, 2, example_link(), None).unwrap();
    model.set_cell_link(0, 2, 2, example_link(), None).unwrap();
    model.undo().unwrap();
    assert_eq!(model.get_cell_link(0, 2, 2), Ok(None));
    assert!(!model.can_undo());
}

#[test]
fn diffs_are_sent_to_other_models() {
    let mut model = UserModel::from_model(new_empty_model());
    model
        .set_cell_link(0, 2, 2, example_link(), Some("IronCalc"))
        .unwrap();

    let send_queue = model.flush_send_queue();
    let mut model2 = UserModel::from_model(new_empty_model());
    model2.apply_external_diffs(&send_queue).unwrap();

    assert_eq!(model2.get_cell_link(0, 2, 2), Ok(Some(example_link())));
    assert_eq!(
        model2.get_formatted_cell_value(0, 2, 2),
        Ok("IronCalc".to_string())
    );
    let style = model2.get_cell_style(0, 2, 2).unwrap();
    assert!(style.font.u);
}
