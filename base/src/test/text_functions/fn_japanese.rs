#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── ASC ───────────────────────────────────────────────────────────────────────

#[test]
fn asc_converts_katakana() {
    let mut model = new_empty_model();
    model._set("A1", "=ASC(\"Ａ１　ｂ！\")");
    model._set("A2", "=ASC(\"アイウエオ\")");
    // Voiced katakana become two half-width characters
    model._set("A3", "=ASC(\"ガギパピヴ\")");
    model._set("A4", "=ASC(\"コーヒー、「ブラック」。\")");
    // Hiragana and kanji have no half-width form
    model._set("A5", "=ASC(\"ひらがな漢字\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "A1 b!");
    assert_eq!(model._get_text("A2"), "ｱｲｳｴｵ");
    assert_eq!(model._get_text("A3"), "ｶﾞｷﾞﾊﾟﾋﾟｳﾞ");
    assert_eq!(model._get_text("A4"), "ｺｰﾋｰ､｢ﾌﾞﾗｯｸ｣｡");
    assert_eq!(model._get_text("A5"), "ひらがな漢字");
}

#[test]
fn asc_and_jis_with_ranges() {
    let mut model = new_empty_model();
    model._set("A1", "ＡＢＣ");
    model._set("A2", "ｶﾞｽ");
    model._set("B1", "=ASC(A1:A2)");
    model._set("C1", "=JIS(A1:A2)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "ABC");
    assert_eq!(model._get_text("B2"), "ｶﾞｽ");
    assert_eq!(model._get_text("C1"), "ＡＢＣ");
    assert_eq!(model._get_text("C2"), "ガス");
}

// ── JIS ───────────────────────────────────────────────────────────────────────

#[test]
fn jis_converts_to_full_width() {
    let mut model = new_empty_model();
    model._set("A1", "=JIS(\"A1 b!\")");
    model._set("A2", "=JIS(\"ｱｲｳｴｵ\")");
    // ﾞ and ﾟ join the katakana before them
    model._set("A3", "=JIS(\"ｶﾞｷﾞﾊﾟﾋﾟｳﾞ\")");
    model._set("A4", "=JIS(\"ｺｰﾋｰ､｢ﾌﾞﾗｯｸ｣｡\")");
    // A mark that cannot join stays a separate character
    model._set("A5", "=JIS(\"ｱﾞﾞ\")");
    model._set("A6", "=JIS(123)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "Ａ１　ｂ！");
    assert_eq!(model._get_text("A2"), "アイウエオ");
    assert_eq!(model._get_text("A3"), "ガギパピヴ");
    assert_eq!(model._get_text("A4"), "コーヒー、「ブラック」。");
    assert_eq!(model._get_text("A5"), "ア゛゛");
    assert_eq!(model._get_text("A6"), "１２３");
}

#[test]
fn asc_jis_round_trip() {
    let mut model = new_empty_model();
    model._set("A1", "=JIS(ASC(\"ガッコウ　Ｎｏ．１\"))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "ガッコウ　Ｎｏ．１");
}

// ── PHONETIC ──────────────────────────────────────────────────────────────────

#[test]
fn phonetic_returns_the_text() {
    // Furigana are not kept, so the reading of a text is the text itself
    let mut model = new_empty_model();
    model._set("A1", "山田");
    model._set("B1", "=PHONETIC(A1)");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "山田");
}

#[test]
fn phonetic_joins_a_range() {
    let mut model = new_empty_model();
    model._set("A1", "東京都");
    model._set("B1", "千代田区");
    model._set("C1", "123");
    model._set("D1", "=\"丸の内\"");
    model._set("A2", "一丁目");
    model._set("E1", "=PHONETIC(A1:D2)");
    model._set("E2", "=PHONETIC(A:A)");
    model._set("E3", "=PHONETIC(\"山田\")");
    model.evaluate();
    // Numbers and formula results are skipped
    assert_eq!(model._get_text("E1"), "東京都千代田区一丁目");
    assert_eq!(model._get_text("E2"), "東京都一丁目");
    // PHONETIC needs a reference
    assert_eq!(model._get_text("E3"), "#VALUE!");
}

// ── TRIM / SUBSTITUTE with full-width spaces ─────────────────────────────────

#[test]
fn full_width_spaces() {
    let mut model = new_empty_model();
    // TRIM only removes the ASCII space, as in Excel
    model._set("A1", "=TRIM(\"  山田　 太郎  \")");
    // SUBSTITUTE removes the full-width ones
    model._set("A2", "=SUBSTITUTE(\"山田　太郎\", \"　\", \"\")");
    // Or ASC turns them into ASCII spaces that TRIM removes
    model._set("A3", "=TRIM(ASC(\"　山田　　太郎　\"))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "山田　 太郎");
    assert_eq!(model._get_text("A2"), "山田太郎");
    assert_eq!(model._get_text("A3"), "山田 太郎");
}

#[test]
fn japanese_function_names() {
    let mut model = crate::Model::new_empty("model", "ja", "UTC", "ja").unwrap();
    model._set("A1", "=JIS(\"abc\")");
    model._set("A2", "=PHONETIC(A1)");
    model.evaluate();
    assert_eq!(model._get_formula("A1"), "=JIS(\"abc\")");
    assert_eq!(model._get_text("A1"), "ａｂｃ");
    assert_eq!(model._get_text("A2"), "");

    let mut model = crate::Model::new_empty("model", "en", "UTC", "it").unwrap();
    model._set("A1", "x");
    model._set("A2", "=FURIGANA(A1)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "x");
}
