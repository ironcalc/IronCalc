#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::Link;

fn external(target: &str) -> Link {
    Link::External {
        target: target.to_string(),
        tooltip: None,
    }
}

#[test]
fn without_friendly_name_displays_the_location() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPERLINK(\"https://www.ironcalc.com/\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "https://www.ironcalc.com/");
    assert_eq!(
        model.get_cell_link(0, 1, 1),
        Ok(Some(external("https://www.ironcalc.com/")))
    );
}

#[test]
fn friendly_name_is_displayed() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPERLINK(\"https://www.ironcalc.com/\", \"IronCalc\")");
    // the friendly name can be a number or come from a reference
    model._set("A2", "=HYPERLINK(\"https://www.ironcalc.com/\", 42)");
    model._set("A3", "=HYPERLINK(\"https://www.ironcalc.com/\", B3)");
    model._set("B3", "Click here");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "IronCalc");
    assert_eq!(model._get_text("A2"), "42");
    assert_eq!(model._get_text("A3"), "Click here");
    for row in 1..=3 {
        assert_eq!(
            model.get_cell_link(0, row, 1),
            Ok(Some(external("https://www.ironcalc.com/")))
        );
    }
}

#[test]
fn location_can_come_from_a_reference() {
    let mut model = new_empty_model();
    model._set("B1", "https://www.ironcalc.com/");
    model._set("A1", "=HYPERLINK(B1, \"IronCalc\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "IronCalc");
    assert_eq!(
        model.get_cell_link(0, 1, 1),
        Ok(Some(external("https://www.ironcalc.com/")))
    );
}

#[test]
fn a_hash_prefix_is_an_internal_link() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPERLINK(\"#Sheet1!A5\", \"Jump to A5\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "Jump to A5");
    assert_eq!(
        model.get_cell_link(0, 1, 1),
        Ok(Some(Link::Internal {
            location: "Sheet1!A5".to_string(),
            tooltip: None,
        }))
    );
}

#[test]
fn dynamic_links_are_rebuilt_on_every_evaluation() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "=HYPERLINK(\"https://www.ironcalc.com/\")".to_string())
        .unwrap();
    model.evaluate();
    assert_eq!(
        model.get_cell_link(0, 1, 1),
        Ok(Some(external("https://www.ironcalc.com/")))
    );

    // replacing the formula removes the dynamic link
    model.set_user_input(0, 1, 1, "Hello".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model.get_cell_link(0, 1, 1), Ok(None));
}

#[test]
fn dynamic_links_are_in_the_links_list() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPERLINK(\"https://www.ironcalc.com/\")");
    model.evaluate();
    // an explicit worksheet link on another cell
    model
        .set_cell_link(0, 2, 2, external("https://www.example.com"))
        .unwrap();

    let links = model.get_links_list(0).unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!((links[0].row, links[0].column), (1, 1));
    assert_eq!(links[0].link, external("https://www.ironcalc.com/"));
    assert_eq!((links[1].row, links[1].column), (2, 2));
    // but the raw worksheet links only contain the explicit one
    assert_eq!(model.get_links(0).unwrap().len(), 1);
}

#[test]
fn an_explicit_link_takes_precedence() {
    let mut model = new_empty_model();
    model._set("A1", "=HYPERLINK(\"https://www.ironcalc.com/\")");
    model.evaluate();
    model
        .set_cell_link(0, 1, 1, external("https://www.example.com"))
        .unwrap();

    assert_eq!(
        model.get_cell_link(0, 1, 1),
        Ok(Some(external("https://www.example.com")))
    );
    assert_eq!(model.get_links_list(0).unwrap().len(), 1);
}

#[test]
fn errors() {
    let mut model = new_empty_model();
    // wrong number of arguments
    model._set("A1", "=HYPERLINK()");
    model._set("A2", "=HYPERLINK(\"a\", \"b\", \"c\")");
    // an error in the location propagates and creates no link
    model._set("A3", "=HYPERLINK(1/0)");
    // an error in the friendly name propagates but the link is created
    model._set("A4", "=HYPERLINK(\"https://www.ironcalc.com/\", 1/0)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
    assert_eq!(model._get_text("A3"), "#DIV/0!");
    assert_eq!(model.get_cell_link(0, 3, 1), Ok(None));
    assert_eq!(model._get_text("A4"), "#DIV/0!");
    assert_eq!(
        model.get_cell_link(0, 4, 1),
        Ok(Some(external("https://www.ironcalc.com/")))
    );
}
