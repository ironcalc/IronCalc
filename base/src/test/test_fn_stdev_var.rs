#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_stdev_var_no_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=STDEVA()");
    model._set("A2", "=STDEVPA()");
    model._set("A3", "=VARA()");
    model._set("A4", "=VARPA()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
}

#[test]
fn test_fn_stdev_var_single_value() {
    let mut model = new_empty_model();
    model._set("B1", "5");

    // Sample functions (STDEVA, VARA) should error with single value
    model._set("A1", "=STDEVA(B1)");
    model._set("A2", "=VARA(B1)");

    // Population functions (STDEVPA, VARPA) should work with single value
    model._set("A3", "=STDEVPA(B1)");
    model._set("A4", "=VARPA(B1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"0"); // Single value has zero deviation
    assert_eq!(model._get_text("A4"), *"0"); // Single value has zero variance
}

#[test]
fn test_fn_stdev_var_identical_values() {
    let mut model = new_empty_model();
    model._set("B1", "3");
    model._set("B2", "3");
    model._set("B3", "3");
    model._set("B4", "3");

    model._set("A1", "=STDEVA(B1:B4)");
    model._set("A2", "=STDEVPA(B1:B4)");
    model._set("A3", "=VARA(B1:B4)");
    model._set("A4", "=VARPA(B1:B4)");

    model.evaluate();

    // All identical values should have zero variance and standard deviation
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"0");
}

#[test]
fn test_fn_stdev_var_negative_values() {
    let mut model = new_empty_model();
    model._set("B1", "-2");
    model._set("B2", "-1");
    model._set("B3", "0");
    model._set("B4", "1");
    model._set("B5", "2");

    model._set("A1", "=STDEVA(B1:B5)");
    model._set("A2", "=STDEVPA(B1:B5)");
    model._set("A3", "=VARA(B1:B5)");
    model._set("A4", "=VARPA(B1:B5)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.58113883");
    assert_eq!(model._get_text("A2"), *"1.414213562");
    assert_eq!(model._get_text("A3"), *"2.5");
    assert_eq!(model._get_text("A4"), *"2");
}

#[test]
fn test_fn_stdev_var_data_types() {
    let mut model = new_empty_model();
    model._set("B1", "10"); // Number
    model._set("B2", "20"); // Number
    model._set("B3", "true"); // Boolean TRUE -> 1
    model._set("B4", "false"); // Boolean FALSE -> 0
    model._set("B5", "'Hello"); // Text -> 0
    model._set("B6", "'123"); // Text number -> 0

    model._set("A1", "=STDEVA(B1:B7)");
    model._set("A2", "=STDEVPA(B1:B7)");
    model._set("A3", "=VARA(B1:B7)");
    model._set("A4", "=VARPA(B1:B7)");

    model.evaluate();
    assert_eq!(model._get_text("A1"), *"8.256310718");
    assert_eq!(model._get_text("A2"), *"7.536946036");
    assert_eq!(model._get_text("A3"), *"68.166666667");
    assert_eq!(model._get_text("A4"), *"56.805555556");
}

#[test]
fn test_fn_stdev_var_mixed_arguments() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "4");
    model._set("B3", "7");

    // Test with mixed range and direct arguments
    model._set("A1", "=STDEVA(B1:B2, B3, 10)");
    model._set("A2", "=STDEVPA(B1:B2, B3, 10)");
    model._set("A3", "=VARA(B1:B2, B3, 10)");
    model._set("A4", "=VARPA(B1:B2, B3, 10)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3.872983346");
    assert_eq!(model._get_text("A2"), *"3.354101966");
    assert_eq!(model._get_text("A3"), *"15");
    assert_eq!(model._get_text("A4"), *"11.25");
}

#[test]
fn test_fn_stdev_var_error_propagation() {
    let mut model = new_empty_model();
    model._set("B1", "1");
    model._set("B2", "=1/0"); // #DIV/0! error
    model._set("B3", "3");

    model._set("A1", "=STDEVA(B1:B3)");
    model._set("A2", "=STDEVPA(B1:B3)");
    model._set("A3", "=VARA(B1:B3)");
    model._set("A4", "=VARPA(B1:B3)");

    model.evaluate();

    // All should propagate the #DIV/0! error
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#DIV/0!");
}

#[test]
fn test_fn_stdev_var_empty_range() {
    let mut model = new_empty_model();
    // B1:B3 contains only empty cells and text (treated as 0 but empty cells ignored)
    model._set("B2", "'text"); // Text -> 0, but this is the only value

    model._set("A1", "=STDEVA(B1:B3)");
    model._set("A2", "=STDEVPA(B1:B3)");
    model._set("A3", "=VARA(B1:B3)");
    model._set("A4", "=VARPA(B1:B3)");

    model.evaluate();

    // Only one value (0 from text), so sample functions error, population functions return 0
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"0");
}

#[test]
fn test_fn_stdev_var_large_dataset() {
    let mut model = new_empty_model();

    // Create a larger dataset with known statistical properties
    for i in 1..=10 {
        model._set(&format!("B{i}"), &format!("{i}"));
    }

    model._set("A1", "=STDEVA(B1:B10)");
    model._set("A2", "=STDEVPA(B1:B10)");
    model._set("A3", "=VARA(B1:B10)");
    model._set("A4", "=VARPA(B1:B10)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"3.027650354");
    assert_eq!(model._get_text("A2"), *"2.872281323");
    assert_eq!(model._get_text("A3"), *"9.166666667");
    assert_eq!(model._get_text("A4"), *"8.25");
}

#[test]
fn test_fn_stdev_var_boolean_only() {
    let mut model = new_empty_model();
    model._set("B1", "true"); // 1
    model._set("B2", "false"); // 0
    model._set("B3", "true"); // 1
    model._set("B4", "false"); // 0

    model._set("A1", "=STDEVA(B1:B4)");
    model._set("A2", "=STDEVPA(B1:B4)");
    model._set("A3", "=VARA(B1:B4)");
    model._set("A4", "=VARPA(B1:B4)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.577350269");
    assert_eq!(model._get_text("A2"), *"0.5");
    assert_eq!(model._get_text("A3"), *"0.333333333");
    assert_eq!(model._get_text("A4"), *"0.25");
}

#[test]
fn test_fn_stdev_var_precision() {
    let mut model = new_empty_model();
    model._set("B1", "1.5");
    model._set("B2", "2.5");

    model._set("A1", "=STDEVA(B1:B2)");
    model._set("A2", "=STDEVPA(B1:B2)");
    model._set("A3", "=VARA(B1:B2)");
    model._set("A4", "=VARPA(B1:B2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.707106781");
    assert_eq!(model._get_text("A2"), *"0.5");
    assert_eq!(model._get_text("A3"), *"0.5");
    assert_eq!(model._get_text("A4"), *"0.25");
}

#[test]
fn test_fn_stdev_var_direct_argument_error_propagation() {
    let mut model = new_empty_model();

    // Test that specific errors in direct arguments are properly propagated
    // This is different from the range error test - this tests direct error arguments
    // Bug fix: Previously converted specific errors to generic #ERROR!
    model._set("A1", "=STDEVA(1, 1/0, 3)"); // #DIV/0! in direct argument
    model._set("A2", "=VARA(2, VALUE(\"text\"), 4)"); // #VALUE! in direct argument

    model.evaluate();

    // Should propagate specific errors, not generic #ERROR!
    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}
