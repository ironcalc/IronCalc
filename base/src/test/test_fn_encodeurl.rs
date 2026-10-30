#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_encodeurl_args_number() {
    let mut model = new_empty_model();

    model._set("A1", "=ENCODEURL()");
    model._set("A2", r#"=ENCODEURL("a", "b")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn fn_encodeurl() {
    let mut model = new_empty_model();

    model._set(
        "A1",
        r#"=ENCODEURL("http://contoso.sharepoint.com/Finance/Profit and Loss Statement.xlsx")"#,
    );
    model._set("A2", r#"=ENCODEURL("AZaz09-._")"#);
    model._set("A3", r#"=ENCODEURL("a b+c&d=e?f/g:h")"#);
    model._set("A4", r#"=ENCODEURL("")"#);
    model._set("A5", r#"=ENCODEURL("~!*'()")"#);

    model.evaluate();

    assert_eq!(
        model._get_text("A1"),
        *"http%3A%2F%2Fcontoso.sharepoint.com%2FFinance%2FProfit%20and%20Loss%20Statement.xlsx"
    );
    // Letters, digits, '-', '.' and '_' are not encoded
    assert_eq!(model._get_text("A2"), *"AZaz09-._");
    assert_eq!(model._get_text("A3"), *"a%20b%2Bc%26d%3De%3Ff%2Fg%3Ah");
    assert_eq!(model._get_text("A4"), *"");
    // '~' is unreserved in RFC 3986 and is left alone (Excel and LibreOffice encode it)
    assert_eq!(model._get_text("A5"), *"~%21%2A%27%28%29");
}

#[test]
fn fn_encodeurl_result_too_long() {
    let mut model = new_empty_model();

    // Excel cannot return 32,767 characters or more
    model._set("A1", r#"=LEN(ENCODEURL(REPT(" ",10922)))"#);
    model._set("A2", r#"=ENCODEURL(REPT(" ",10923))"#);
    model._set("A3", r#"=ENCODEURL(REPT("a",32767))"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"32766");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
}

#[test]
fn fn_encodeurl_arrays() {
    let mut model = new_empty_model();

    model._set("A1", "a b");
    model._set("A2", "hello world");
    model._set("A3", "=1/0");
    model._set("B1", "=ENCODEURL(A1:A3)");
    model._set("C1", "=INDEX(ENCODEURL(A1:A3),2)");
    model._set("D1", r#"=ENCODEURL({"x y",1.5})"#);

    model.evaluate();

    assert_eq!(model._get_text("B1"), *"a%20b");
    assert_eq!(model._get_text("B2"), *"hello%20world");
    assert_eq!(model._get_text("B3"), *"#DIV/0!");
    assert_eq!(model._get_text("C1"), *"hello%20world");
    assert_eq!(model._get_text("D1"), *"x%20y");
    assert_eq!(model._get_text("E1"), *"1.5");
}

#[test]
fn fn_encodeurl_utf8() {
    let mut model = new_empty_model();

    // Non-ASCII characters are encoded byte by byte in UTF-8
    model._set("A1", r#"=ENCODEURL("ü")"#);
    model._set("A2", r#"=ENCODEURL("€")"#);
    model._set("A3", r#"=ENCODEURL("日本")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"%C3%BC");
    assert_eq!(model._get_text("A2"), *"%E2%82%AC");
    assert_eq!(model._get_text("A3"), *"%E6%97%A5%E6%9C%AC");
}

#[test]
fn fn_encodeurl_casts_arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=ENCODEURL(12.5)");
    model._set("A2", "=ENCODEURL(TRUE)");
    model._set("A3", "=ENCODEURL(1/0)");
    // Numbers are converted to text with 15 significant digits
    model._set("A4", "=ENCODEURL(1/3)");
    model._set("A5", "=ENCODEURL(1E+20)");
    model._set("A6", "=ENCODEURL(0.0000001)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"12.5");
    assert_eq!(model._get_text("A2"), *"TRUE");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"0.333333333333333");
    assert_eq!(model._get_text("A5"), *"1E%2B20");
    assert_eq!(model._get_text("A6"), *"0.0000001");
}

#[test]
fn fn_encodeurl_matches_rfc_3986_reference() {
    // The `urlencoding` crate implements RFC 3986 percent-encoding (unreserved
    // characters untouched, every other UTF-8 byte as %XX in uppercase).
    // Compare it with ENCODEURL on every printable ASCII character and a
    // sample of non-ASCII text.
    let mut model = new_empty_model();

    let ascii: String = (0x20u8..0x7F).map(|b| b as char).collect();
    let inputs = [
        ascii.as_str(),
        "AZaz09-._~",
        "~!*'()",
        "http://example.com/Profit and Loss.xlsx",
        "a b+c&d=e?f/g:h",
        "100% \"sure\"",
        "ü€日本😀",
        "mañana\u{301} ñ",
        "\t\n\r",
    ];
    for (i, input) in inputs.iter().enumerate() {
        let formula = format!("=ENCODEURL(\"{}\")", input.replace('"', "\"\""));
        model._set(&format!("A{}", i + 1), &formula);
    }

    model.evaluate();

    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            urlencoding::encode(input).into_owned(),
            "input: {input:?}"
        );
    }
}
