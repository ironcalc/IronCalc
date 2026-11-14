#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;
#[test]
fn test_fn_t_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("A4", "5");
    model._set("A5", "6");
    model._set("A6", "10");
    model._set("A7", "3");
    model._set("A8", "2");
    model._set("A9", "4");
    model._set("A10", "7");

    model._set("B2", "6");
    model._set("B3", "19");
    model._set("B4", "3");
    model._set("B5", "2");
    model._set("B6", "13");
    model._set("B7", "4");
    model._set("B8", "5");
    model._set("B9", "17");
    model._set("B10", "3");

    model._set("C1", "=T.TEST(A2:A10, B2:B10, 1, 1)");
    model._set("C2", "=T.TEST(A2:A10, B2:B10, 1, 2)");
    model._set("C3", "=T.TEST(A2:A10, B2:B10, 1, 3)");
    model._set("C4", "=T.TEST(A2:A10, B2:B10, 2, 1)");
    model._set("C5", "=T.TEST(A2:A10, B2:B10, 2, 2)");
    model._set("C6", "=T.TEST(A2:A10, B2:B10, 2, 3)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0.103836888");
    assert_eq!(model._get_text("C2"), *"0.100244599");
    assert_eq!(model._get_text("C3"), *"0.105360319");
    assert_eq!(model._get_text("C4"), *"0.207673777");
    assert_eq!(model._get_text("C5"), *"0.200489197");
    assert_eq!(model._get_text("C6"), *"0.210720639");
}

// Whole columns are read only as far as the used area: the cells past it are
// empty, and T.TEST skips empty cells, in pairs for the paired test.
#[test]
fn test_fn_t_test_whole_columns() {
    let mut model = new_empty_model();
    for (row, (x, y)) in [(3, 4), (5, 9), (2, 2), (8, 11), (6, 7)].iter().enumerate() {
        model._set(&format!("D{}", row + 1), &x.to_string());
        model._set(&format!("E{}", row + 1), &y.to_string());
    }
    model._set("E7", "text");
    for test_type in 1..=3 {
        model._set(
            &format!("A{test_type}"),
            &format!("=T.TEST(D:D,E:E,2,{test_type})=T.TEST(D1:D5,E1:E5,2,{test_type})"),
        );
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), "TRUE");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "TRUE");
}
