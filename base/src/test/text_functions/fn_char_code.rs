#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── CHAR ──────────────────────────────────────────────────────────────────────

#[test]
fn test_char_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=CHAR(65)");
    model._set("A2", "=CHAR(97)");
    model._set("A3", "=CHAR(32)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "A");
    assert_eq!(model._get_text("A2"), "a");
    assert_eq!(model._get_text("A3"), " ");
}

#[test]
fn test_char_win1252() {
    let mut model = new_empty_model();
    // Code point 128 in Windows-1252 is €
    model._set("A1", "=CHAR(128)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "€");
}

#[test]
fn test_char_error() {
    let mut model = new_empty_model();
    model._set("A1", "=CHAR(0)");
    model._set("A2", "=CHAR(300)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
}

#[test]
fn test_char_inline_array_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=CHAR({45,67,89})");
    model._set("A2", "=CHAR({65;66})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "-");
    assert_eq!(model._get_text("B1"), "C");
    assert_eq!(model._get_text("C1"), "Y");
    assert_eq!(model._get_text("A2"), "A");
    assert_eq!(model._get_text("A3"), "B");
}

#[test]
fn test_char_range_spills() {
    let mut model = new_empty_model();
    for (i, code) in [72, 101, 108, 108, 111].iter().enumerate() {
        model._set(&format!("A{}", i + 1), &code.to_string());
    }
    model._set("C1", "=CHAR(A1:A5)");
    // Consumed element-wise inside another function and by an operator
    model._set("D1", "=SUM(LEN(CHAR(A1:A5)))");
    model._set("E1", "=CHAR(A1:A5)&\"!\"");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "H");
    assert_eq!(model._get_text("C2"), "e");
    assert_eq!(model._get_text("C3"), "l");
    assert_eq!(model._get_text("C4"), "l");
    assert_eq!(model._get_text("C5"), "o");
    assert_eq!(model._get_text("D1"), "5");
    assert_eq!(model._get_text("E1"), "H!");
    assert_eq!(model._get_text("E5"), "o!");
}

#[test]
fn test_char_array_element_errors() {
    let mut model = new_empty_model();
    // Invalid codes and non-numeric text error per element; others still work.
    model._set("A1", "=CHAR({65,0,300,\"B\",\"66\",TRUE})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "A");
    assert_eq!(model._get_text("B1"), "#VALUE!");
    assert_eq!(model._get_text("C1"), "#VALUE!");
    assert_eq!(model._get_text("D1"), "#VALUE!");
    assert_eq!(model._get_text("E1"), "B");
    // TRUE coerces to 1, which is a valid (control) character
    assert_eq!(model._get_text("F1"), "\u{1}");
}

#[test]
fn test_char_range_with_empty_and_error_cells() {
    let mut model = new_empty_model();
    model._set("A1", "65");
    model._set("A3", "=1/0");
    model._set("C1", "=CHAR(A1:A3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "A");
    // Empty cell is 0 → invalid code
    assert_eq!(model._get_text("C2"), "#VALUE!");
    assert_eq!(model._get_text("C3"), "#DIV/0!");
}

// ── CODE ──────────────────────────────────────────────────────────────────────

#[test]
fn test_code_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=CODE(\"A\")");
    model._set("A2", "=CODE(\"ABC\")");
    model._set("A3", "=CODE(\" \")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "65");
    assert_eq!(model._get_text("A2"), "65"); // only first char
    assert_eq!(model._get_text("A3"), "32");
}

#[test]
fn test_code_empty() {
    let mut model = new_empty_model();
    model._set("A1", "=CODE(\"\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

// ── UNICHAR ───────────────────────────────────────────────────────────────────

#[test]
fn test_unichar_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICHAR(65)");
    model._set("A2", "=UNICHAR(9786)"); // ☺ U+263A
    model.evaluate();
    assert_eq!(model._get_text("A1"), "A");
    assert_eq!(model._get_text("A2"), "☺");
}

#[test]
fn test_unichar_error() {
    let mut model = new_empty_model();
    model._set("A1", "=UNICHAR(0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
}
