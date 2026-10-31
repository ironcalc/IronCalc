---
layout: doc
outline: deep
lang: en-US
---

# PHONETIC

Returns the furigana (the reading) of Japanese text in a cell: `=PHONETIC(A1)` returns `ヤマダタロウ` for `山田太郎`.

* Like Japanese Excel, IronCalc records the reading you type with the IME: typing おみおつけ and converting it to 御御御付 records オミオツケ. Text that is pasted or typed without converting has no furigana.
* Furigana are also imported from xlsx files and saved back when you export.
* If you edit part of a text later, the readings of the characters you did not touch are kept and the others are dropped.
* The characters without furigana are kept: `田中さん` with furigana on `田中` gives `タナカさん`. A cell without furigana gives its text.
* The argument must be a reference: `=PHONETIC("山田")` is `#VALUE!`.
* With a range the readings of the cells are joined, row by row: `=PHONETIC(A1:C1)`. Numbers, booleans and formula results are skipped.
