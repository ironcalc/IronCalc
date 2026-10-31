---
layout: doc
outline: deep
lang: en-US
---

# JIS

Converts half-width (single-byte) characters to full-width (double-byte) ones. It is the reverse of [ASC](/functions/text/asc).

* ASCII letters, digits and symbols become their full-width forms: `=JIS("A1")` returns `Ａ１`.
* The space becomes the ideographic space (U+3000).
* Half-width katakana become full-width katakana. A katakana followed by a voiced sound mark is joined into one character: `=JIS("ｶﾞｽ")` returns `ガス`.

A range or an array argument gives an array of results.
