use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_convert() {
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1, "lbm", "kg")"#);
    model._set("A2", r#"=CONVERT(68, "F", "C")"#);
    model._set("A3", r#"=CONVERT(2.5, "ft", "sec")"#);
    model._set("A4", r#"=CONVERT(CONVERT(100,"ft","m"),"ft","m""#);

    model._set("B1", "6");

    model._set("A5", r#"=CONVERT(B1,"C","F")"#);
    model._set("A6", r#"=CONVERT(B1,"tsp","tbs")"#);
    model._set("A7", r#"=CONVERT(B1,"gal","l")"#);
    model._set("A8", r#"=CONVERT(B1,"mi","km")"#);
    model._set("A9", r#"=CONVERT(B1,"km","mi")"#);
    model._set("A10", r#"=CONVERT(B1,"in","ft")"#);
    model._set("A11", r#"=CONVERT(B1,"cm","in")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.45359237");
    // Microsoft's example: =CONVERT(68, "F", "C") "Converts 68 degrees Fahrenheit to Celsius." (20)
    // https://support.microsoft.com/en-us/office/convert-function-d785bef1-808e-4aac-bdcd-666c810f9af2
    assert_eq!(model._get_text("A2"), "20");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "9.290304");

    assert_eq!(model._get_text("A5"), "42.8");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "22.712470704"); //22.71741274");
    assert_eq!(model._get_text("A8"), "9.656064");
    assert_eq!(model._get_text("A9"), "3.728227153");
    assert_eq!(model._get_text("A10"), "0.5");
    assert_eq!(model._get_text("A11"), "2.362204724");
}

#[test]
fn fn_convert_temperature_offsets() {
    // Values captured from Excel for Mac 16.114.1, en-US locale, 1900 date system, on
    // 25 October 2026 (published in the Formulon project's oracle goldens, Apache-2.0).
    let cases = [
        (r#"=CONVERT(273.15,"K","C")"#, 0.0),
        (r#"=CONVERT(0.00075,"F","C")"#, -17.777361111111112),
        (r#"=CONVERT(0.00075,"C","C")"#, 0.00075),
        (r#"=CONVERT(1,"kK","C")"#, 726.85),
        (r#"=CONVERT(1,"C","kK")"#, 0.27415),
        (r#"=CONVERT(1,"kK","F")"#, 1340.3300000000002),
        // Microsoft's CONVERT page: "Converts 68 degrees Fahrenheit to Celsius." (20)
        (r#"=CONVERT(68,"F","C")"#, 20.0),
        // Microsoft's CONVERT page: "Convert 6 degrees Celsius to Fahrenheit (42.8)"
        (r#"=CONVERT(6,"C","F")"#, 42.8),
        (r#"=CONVERT(100,"C","F")"#, 212.0),
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
