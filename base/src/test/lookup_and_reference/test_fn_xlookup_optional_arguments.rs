use crate::test::util::new_empty_model;

fn assert_xlookup(formula: &str, expected: &str) {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "5"),
        ("A2", "3"),
        ("A3", "8"),
        ("A4", "1"),
        ("B1", "b"),
        ("B2", "d"),
        ("B3", "a"),
        ("B4", "c"),
    ] {
        model._set(cell, value);
    }
    model._set("D1", formula);
    model.evaluate();
    assert_eq!(model._get_text("D1"), expected, "{formula}");
}

#[test]
fn test_xlookup_empty_optional_arguments_use_defaults() {
    for (formula, expected) in [
        ("=XLOOKUP(8,A1:A4,B1:B4)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,1)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,,)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,)", "a"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,,1)", "a"),
        ("=XLOOKUP(99,A1:A4,B1:B4,)", "#N/A"),
        ("=XLOOKUP(99,A1:A4,B1:B4,,,)", "#N/A"),
    ] {
        assert_xlookup(formula, expected);
    }
}

#[test]
fn test_xlookup_empty_search_mode_preserves_match_mode() {
    for (formula, expected) in [
        ("=XLOOKUP(6,A1:A4,B1:B4,,-1,)", "b"),
        ("=XLOOKUP(6,A1:A4,B1:B4,,1,)", "a"),
        ("=XLOOKUP(6,A1:A4,B1:B4,,0,)", "#N/A"),
        ("=XLOOKUP(\"a*\",B1:B4,A1:A4,,2,)", "8"),
    ] {
        assert_xlookup(formula, expected);
    }
}

#[test]
fn test_xlookup_empty_search_mode_preserves_if_not_found() {
    for (formula, expected) in [
        ("=XLOOKUP(99,A1:A4,B1:B4,\"missing\",,)", "missing"),
        ("=XLOOKUP(99,A1:A4,B1:B4,0,,)", "0"),
        ("=XLOOKUP(99,A1:A4,B1:B4,1/0,,)", "#DIV/0!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,1/0,,)", "a"),
    ] {
        assert_xlookup(formula, expected);
    }
}

#[test]
fn test_xlookup_empty_search_mode_returns_first_duplicate() {
    for (lookup_array, return_array, cells) in [
        ("A1:A3", "B1:B3", [("A1", "B1"), ("A2", "B2"), ("A3", "B3")]),
        ("A1:C1", "A2:C2", [("A1", "A2"), ("B1", "B2"), ("C1", "C2")]),
    ] {
        for (search_mode, expected) in [("", "first"), ("1", "first"), ("-1", "last")] {
            let mut model = new_empty_model();
            for ((lookup_cell, return_cell), (key, value)) in
                cells
                    .into_iter()
                    .zip([("8", "first"), ("5", "other"), ("8", "last")])
            {
                model._set(lookup_cell, key);
                model._set(return_cell, value);
            }
            let formula = format!("=XLOOKUP(8,{lookup_array},{return_array},,,{search_mode})");
            model._set("D1", &formula);
            model.evaluate();
            assert_eq!(model._get_text("D1"), expected, "{formula}");
        }
    }
}

#[test]
fn test_xlookup_explicit_optional_arguments_still_validate() {
    for (formula, expected) in [
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,0)", "#ERROR!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,Z99)", "#ERROR!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,\"\")", "#VALUE!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,NA())", "#N/A"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,0,1/0)", "#DIV/0!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,99,)", "#VALUE!"),
        ("=XLOOKUP(8,A1:A4,B1:B4,,NA(),)", "#N/A"),
    ] {
        assert_xlookup(formula, expected);
    }
}
