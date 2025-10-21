---
layout: doc
outline: deep
lang: en-US
---

# Value Types

::: warning
**Note:** This draft page is under construction 🚧
:::

In IronCalc a value, a result of a calculation, can be one of the following.

## Numbers

Numbers in IronCalc are [IEEE 754 double-precision](https://en.wikipedia.org/wiki/Double-precision_floating-point_format).

Numbers are only displayed up to 15 significant figures. That's why `=0.1+0.2` gives  `0.3`.

Also, numbers are compared up to 15 significant figures. So `=IF(0.1+0.2=0.3, "Valid", "Invalid")` gives `Valid`.

However, `=0.3-0.2-0.1` will not give exactly `0` in IronCalc.

### Casting into numbers

Strings and booleans are sometimes converted to numbers:

`=1+"2"` => `3`

Some functions cast in weird ways:

`=SUM(1,TRUE)` => `1` and `=SUM(1,"1")` => `1`

And `=SUM(1,A1)` => `1` (where A1 contains `TRUE` or `"1"`)


Sometimes the conversion happens as might be expected. For example, `="123"+1` is `124`, `=SQRT("4")` is `2` and `=SQRT(TRUE)` is `1`.

Some functions, however, are more strict. For example, `=BIN2DEC(TRUE)` gives the #VALUE! error.

### Dates and times

IronCalc uses numbers to represent dates and times.

The integer part of the number represents the date, as a count of days since the fixed starting date of December 30, 1899.

The fractional part of the number represents the time of day. 0.0 corresponds to 00:00:00 (midnight) and 0.5 corresponds to 12:00:00 (noon).

## Strings


### Complex numbers

Using IronCalc, a complex number is a string of the form "1+j3".


## Booleans

### Casting from numbers

## Errors


### Casting from strings

"#N/A" => #N/A

## Arrays

## Ranges

## References

A reference is a pointer to a single cell or a range of cells. The reference can either be entered manually, for example "A4", or as the result of a calculation, such as the OFFSET Function or the INDIRECT Function. A reference can also be built, for example with the Colon (\:) Operator. 