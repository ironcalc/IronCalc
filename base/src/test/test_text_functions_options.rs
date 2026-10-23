#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn text_functions_left_out_arguments_and_lists() {
    let mut model = new_empty_model();
    model._set("A1", "=TEXTBEFORE(\"a-b\",\"x\",,,,\"none\")");
    model._set("A2", "=TEXTAFTER(\"A-b\",\"a\",,1)");
    model._set("A3", "=VALUETOTEXT(\"a\",1)");
    model._set("A4", "=CONCAT({1,2;3,4})");
    model._set("A5", "=TEXTJOIN(\",\",TRUE,{1,\"\",3})");
    model._set("A6", "=TEXTJOIN(\",\",FALSE,{1,\"\",3})");
    model.evaluate();

    // an omitted instance_num (the commas left empty) is the first one
    assert_eq!(model._get_text("A1"), "none");
    assert_eq!(model._get_text("A2"), "-b");
    // format 1 puts text in quotes, as a formula would show it
    assert_eq!(model._get_text("A3"), "\"a\"");
    // CONCAT and TEXTJOIN take a literal array, not just ranges
    assert_eq!(model._get_text("A4"), "1234");
    assert_eq!(model._get_text("A5"), "1,3");
    assert_eq!(model._get_text("A6"), "1,,3");
}
