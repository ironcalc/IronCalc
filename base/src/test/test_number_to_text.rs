#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn numbers_become_text_with_15_significant_digits() {
    let mut model = new_empty_model();
    model._set("A1", "=\"a\"&0.1+0.2");
    model._set("A2", "=\"\"&1E+20");
    model._set("A3", "=CONCATENATE(1/3)");
    model._set("A4", "=VALUETOTEXT(0.000000007123456)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "a0.3");
    assert_eq!(model._get_text("A2"), "100000000000000000000");
    assert_eq!(model._get_text("A3"), "0.333333333333333");
    // No scientific notation, matching Excel's own text conversion
    assert_eq!(model._get_text("A4"), "0.000000007123456");
}
