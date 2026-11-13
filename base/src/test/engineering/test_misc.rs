use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_getstep() {
    let mut model = new_empty_model();
    model._set("A1", "=GESTEP(7, 4.6)");
    model._set("A2", "=GESTEP(45, 45)");
    model._set("A3", "=GESTEP(-7, -6)");
    model._set("A4", "=GESTEP(0.1)");
    model._set("A5", "=GESTEP(-0.1)");

    model._set("B1", "=GESTEP()");
    model._set("B2", "=GESTEP(1,2,3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"1");
    assert_eq!(model._get_text("A5"), *"0");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn fn_delta() {
    let mut model = new_empty_model();
    model._set("A1", "=DELTA(7, 7)");
    model._set("A2", "=DELTA(-7, -7)");
    model._set("A3", "=DELTA(-7, 7)");
    model._set("A4", "=DELTA(5, 0.5)");
    model._set("A5", "=DELTA(-0, 0)");

    model._set("B1", "=DELTA()");
    model._set("B2", "=DELTA(1,2,3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"1");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
}

#[test]
fn fn_delta_misc() {
    let mut model = new_empty_model();
    model._set("A1", "=3+1e-16");
    model._set("A2", "=3");
    model._set("A3", "=3+1e-15");
    model._set("B1", "=DELTA(A1, A2)");
    model._set("B2", "=DELTA(A1, A3)");

    model._set("B1", "1");
    model._set("B2", "0");

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"1");
}

#[test]
fn fn_erf_erfc_full_precision() {
    // Values cached by Excel (AppVersion 16.0300, workbook saved on 12 March 2023) in
    // xlsx/tests/calc_tests/ENGINEERING/ERFs.xlsx. The calc tests compare to 5e-8, which is why
    // statrs' erf (0.8427007929427149 for ERF(1)) passed them.
    let cases = [
        ("=ERF(1)", 0.8427007929497149),
        ("=ERF.PRECISE(1)", 0.8427007929497149),
        ("=ERFC(1)", 0.15729920705028513),
        ("=ERFC.PRECISE(1)", 0.15729920705028513),
    ];
    let mut model = new_empty_model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        let value = model.get_cell_value_by_ref(&format!("Sheet1!A{}", index + 1));
        assert_eq!(value, Ok(CellValue::Number(*expected)), "{formula}");
    }
}
