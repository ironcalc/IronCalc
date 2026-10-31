---
layout: doc
outline: deep
lang: en-US
---

# ASC

Converts full-width (double-byte) characters to half-width (single-byte) ones. [JIS](/functions/text/jis) does the reverse.

* Full-width letters, digits and symbols become ASCII: `=ASC("Ａ１")` returns `A1`.
* The ideographic space (U+3000) becomes a space.
* Katakana become half-width katakana. A voiced katakana becomes two characters: `=ASC("ガス")` returns `ｶﾞｽ`.
* Hiragana and kanji have no half-width form and are kept.

A range or an array argument gives an array of results.
