#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn degrees_and_radians_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=DEGREES()");
    model._set("A2", "=DEGREES(PI())");
    model._set("A3", "=DEGREES(1,2)");

    model._set("B1", "=RADIANS()");
    model._set("B2", "=RADIANS(180)");
    model._set("B3", "=RADIANS(1,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"180");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"3.141592654");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn degrees_and_radians_typical_angles() {
    let mut model = new_empty_model();

    // RADIANS for common degree values
    model._set("A1", "=RADIANS(90)");
    model._set("A2", "=RADIANS(270)");
    model._set("A3", "=RADIANS(360)");

    // DEGREES for common radian values
    model._set("B1", "=DEGREES(PI()/2)");
    model._set("B2", "=DEGREES(3*PI()/2)");
    model._set("B3", "=DEGREES(2*PI())");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1.570796327");
    assert_eq!(model._get_text("A2"), *"4.71238898");
    assert_eq!(model._get_text("A3"), *"6.283185307");

    assert_eq!(model._get_text("B1"), *"90");
    assert_eq!(model._get_text("B2"), *"270");
    assert_eq!(model._get_text("B3"), *"360");
}

#[test]
fn degrees_radians_round_trip_precision() {
    let mut model = new_empty_model();

    model._set("C1", "=DEGREES(RADIANS(123.456))");
    model._set("C2", "=RADIANS(DEGREES(PI()/4))");

    model.evaluate();

    // Round-trip check within general-format precision (string equality is enough)
    assert_eq!(model._get_text("C1"), *"123.456");
    assert_eq!(model._get_text("C2"), *"0.785398163");
}

#[test]
fn test_fn_pi_value() {
    let mut model = new_empty_model();
    model._set("D1", "=PI()");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"3.141592654");
}

#[test]
fn degrees_radians_negative_and_zero() {
    let mut model = new_empty_model();

    // Zero angle
    model._set("A1", "=RADIANS(0)");
    model._set("B1", "=DEGREES(0)");

    // Negative angles
    model._set("A2", "=RADIANS(-45)"); // -pi/4
    model._set("B2", "=DEGREES(-PI()/2)"); // -90

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("B1"), *"0");

    assert_eq!(model._get_text("A2"), *"-0.785398163");
    assert_eq!(model._get_text("B2"), *"-90");
}
