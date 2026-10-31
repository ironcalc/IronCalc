---
layout: doc
outline: deep
lang: en-US
---

# PHONETIC

Returns the furigana (the reading) of Japanese text in a cell.

::: warning
IronCalc does not keep furigana: they are not imported from xlsx files and cannot be entered. Like Excel for a cell without furigana, `PHONETIC` returns the text itself.
:::

* The argument must be a reference: `=PHONETIC("山田")` is `#VALUE!`.
* With a range the texts of the cells are joined, row by row: `=PHONETIC(A1:C1)`. Numbers, booleans and formula results are skipped.
