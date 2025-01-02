---
layout: doc
outline: deep
lang: en-US
---
# MONTH function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
MONTH is a function of the Date and Time category that extracts the month from a valid date [serial number](/features/serial-numbers.md), returning a number in the range [1, 12].
## Usage
### Syntax
**MONTH(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">month</span>**
### Argument descriptions
* *date* ([number](/features/value-types#numbers), required). The date for which the month is to be calculated, expressed as a [serial number](/features/serial-numbers.md) in the range [1, 2958466). The value 1 corresponds to the date 1899-12-31, while 2958465 corresponds to 9999-12-31.
### Additional guidance
If the supplied _date_ argument has a fractional part, MONTH uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
### Returned value
MONTH returns an integer [number](/features/value-types#numbers) in the range [1, 12], that is the month according to the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar). The value 1 corresponds to January, 2 corresponds to February and so on.
### Error conditions
* In common with many other IronCalc functions, MONTH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then MONTH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then MONTH returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, MONTH may return the [`#DIV/0!`](/features/error-types.md#div-0) error.
* If date is less than 1, or greater than or equal to 2,958,466, then MONTH returns the [`#NUM!`](/features/error-types.md#num) error.
* At present, MONTH does not accept a string representation of a date literal as an argument. For example, the formula `=MONTH("2024-12-31")` returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
IronCalc utilizes Rust's [chrono](https://docs.rs/chrono/latest/chrono/) crate to implement the MONTH function.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=month).

## Links
* See also IronCalc's [DAY](/functions/date_and_time/day.md) and [YEAR](/functions/date_and_time/year.md) functions.
* Visit Microsoft Excel's [MONTH function](https://support.microsoft.com/en-gb/office/month-function-579a2881-199b-48b2-ab90-ddba0eba86e8) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093052) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/MONTH) provide versions of the MONTH function.