---
layout: doc
outline: deep
lang: en-US
---
# YEAR function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
YEAR is a function of the Date and Time category that extracts the year from a valid date [serial number](/features/serial-numbers.md), returning a number in the range [1899, 9999].
## Usage
### Syntax
**YEAR(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">year</span>**
### Argument descriptions
* *date* ([number](/features/value-types#numbers), required). The date for which the year is to be calculated, expressed as a [serial number](/features/serial-numbers.md) in the range [1, 2958466). The value 1 corresponds to the date 1899-12-31, while 2958465 corresponds to 9999-12-31.
### Additional guidance
If the supplied _date_ argument has a fractional part, YEAR uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
### Returned value
YEAR returns an integer [number](/features/value-types#numbers) in the range [1899, 9999], that is the year according to the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar).
### Error conditions
* In common with many other IronCalc functions, YEAR propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then YEAR returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then YEAR returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, YEAR may return the [`#DIV/0!`](/features/error-types.md#div-0) error.
* If date is less than 1, or greater than or equal to 2,958,466, then YEAR returns the [`#NUM!`](/features/error-types.md#num) error.
* At present, YEAR does not accept a string representation of a date literal as an argument. For example, the formula `=YEAR("2024-12-31")` returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
IronCalc utilizes Rust's [chrono](https://docs.rs/chrono/latest/chrono/) crate to implement the YEAR function.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=year).

## Links
* See also IronCalc's [DAY](/functions/date_and_time/day.md) and [MONTH](/functions/date_and_time/month.md) functions.
* Visit Microsoft Excel's [YEAR function](https://support.microsoft.com/en-gb/office/year-function-c64f017a-1354-490d-981f-578e8ec8d3b9) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093061) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/YEAR) provide versions of the YEAR function.