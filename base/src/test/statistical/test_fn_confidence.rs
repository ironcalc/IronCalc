#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_confidence_norm_smoke() {
    let mut model = new_empty_model();

    model._set("A1", "=CONFIDENCE.NORM(0.05, 2.5, 50)");

    // Some edge/error cases
    model._set("A2", "=CONFIDENCE.NORM(0, 2.5, 50)"); // alpha <= 0  -> #NUM!
    model._set("A3", "=CONFIDENCE.NORM(1, 2.5, 50)"); // alpha >= 1  -> #NUM!
    model._set("A4", "=CONFIDENCE.NORM(0.05, -1, 50)"); // std_dev <=0 -> #NUM!
    model._set("A5", "=CONFIDENCE.NORM(0.05, 2.5, 1)");
    model._set("A6", "=CONFIDENCE.NORM(0.05, 2.5, 0.99)"); // size < 1 -> #NUM!

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.692951912");

    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"4.899909961");
    assert_eq!(model._get_text("A6"), *"#NUM!");
}

#[test]
fn test_fn_confidence_t_smoke() {
    let mut model = new_empty_model();

    model._set("A1", "=CONFIDENCE.T(0.05, 50000, 100)");

    // Some edge/error cases
    model._set("A2", "=CONFIDENCE.T(0, 50000, 100)"); // alpha <= 0 -> #NUM!
    model._set("A3", "=CONFIDENCE.T(1, 50000, 100)"); // alpha >= 1 -> #NUM!
    model._set("A4", "=CONFIDENCE.T(0.05, -1, 100)");
    model._set("A5", "=CONFIDENCE.T(0.05, 50000, 1)");
    model._set("A6", "=CONFIDENCE.T(0.05, 50000, 1.7)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"9921.08475793");

    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#NUM!");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}
