---
layout: doc
outline: deep
lang: en-US
---

# PHONETIC

Returns the furigana (the reading) of Japanese text in a cell: `=PHONETIC(A1)` returns `ヤマダタロウ` for `山田太郎`.

* Furigana are imported from xlsx files (Japanese Excel stores the reading typed with the IME) and saved back when you export. They cannot be entered in IronCalc.
* The characters without furigana are kept: `田中さん` with furigana on `田中` gives `タナカさん`. A cell without furigana gives its text.
* The argument must be a reference: `=PHONETIC("山田")` is `#VALUE!`.
* With a range the readings of the cells are joined, row by row: `=PHONETIC(A1:C1)`. Numbers, booleans and formula results are skipped.
