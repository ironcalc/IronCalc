#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// "If there are fewer than three data points, or the sample standard deviation is zero,
// SKEW.P returns the #DIV/0! error value."
// Expected values captured from Excel for Mac 16.114.1 (en-US).
#[test]
fn fn_skew_p_needs_three_points() {
    let mut model = new_empty_model();
    model._set("A1", "=SKEW.P(1,2)");
    model._set("A2", "=SKEW.P(3,3,3)");
    model._set("A3", "=SKEW.P(1,2,3,4,5)");
    model._set("A4", "=SKEW.P(1,1,1,2,10)");
    model._set("A5", "=SKEW.P(1,9,9,9,10)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"0");

    let a4: f64 = model._get_text("A4").parse().unwrap();
    let a5: f64 = model._get_text("A5").parse().unwrap();
    assert!((a4 - 1.4565472846013436).abs() < 1e-9);
    assert!((a5 - -1.447247320791964).abs() < 1e-9);
}
