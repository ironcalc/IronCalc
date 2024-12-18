---
layout: doc
outline: deep
lang: en-US
---

# Value types

::: warning
**Note:** This page is in construction 🚧
:::

In IronCalc a value, a result of a calculation can be one of:

## Numbers

Numbers in IronCalc are [IEEE 754 double precission](https://en.wikipedia.org/wiki/Double-precision_floating-point_format).

Numbers are only displayed up to 15 significant figures. That's why '=0.1+0.2' is actually '0.3'

Also numbers are compared up to 15 significant figures. So `=IF(0.1+0.2=0.3, "Valid", "Invalid")` will return `Valid`.

However `=0.3-0.2-0.1` will not result in `0` in IronCalc.

### Casting into numbers

Strings and booleans are sometimes coverted to numbers

`=1+"2"` => 3

Some functions cast in weird ways:

SUM(1, TRUE) => 2
SUM(1, "1") => 2

But SUM(1, A1) => 1 (where A1 is TRUE or "1")


Sometimes the conversion happens like => "123"+1 is actually 124 and the SQRT("4") is 2 or the SQRT(TRUE) is 1.

Some functions, however are more strict BIN2DEC(TRUE) is #VALUE!

### Dates

On spreadsheets a date is just the number of days since January 1, 1900.


## Strings


### Complex numbers

On IronCal a complex number is just a string like "1+j3".


## Booleans

### Casting from numbers

## Errors


### Casting from strings

"#N/A" => #N/A

## Arrays
