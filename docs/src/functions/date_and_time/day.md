---
layout: doc
outline: deep
lang: en-US
---
# DAY function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
DAY is a function of the Date and Time category that extracts the day of the month from a valid date [serial number](/features/serial-numbers.md), returning a number in the range [1, 31].
## Usage
### Syntax
**DAY(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">day</span>**
### Argument descriptions
* *date* ([number](/features/value-types#numbers), required). The date for which the day of the month is to be calculated, expressed as a [serial number](/features/serial-numbers.md) in the range [1, 2958466). The value 1 corresponds to the date 1899-12-31, while 2958465 corresponds to 9999-12-31.
### Additional guidance
If the supplied _date_ argument has a fractional part, DAY uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
### Returned value
DAY returns an integer [number](/features/value-types#numbers) in the range [1, 31], that is the day of the month according to the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar).
### Error conditions
* In common with many other IronCalc functions, DAY propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then DAY returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then DAY returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, DAY may return the [`#DIV/0!`](/features/error-types.md#div-0) error.
* If date is less than 1, or greater than 2,958,465, then DAY returns the [`#NUM!`](/features/error-types.md#num) error.
* At present, DAY does not accept a string representation of a date literal as an argument. For example, the formula `=DAY("2024-12-31")` returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
IronCalc utilizes Rust's [chrono](https://docs.rs/chrono/latest/chrono/) crate to implement the DAY function.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=day).

## Links
* See also IronCalc's [MONTH](/functions/date_and_time/month.md) and [YEAR](/functions/date_and_time/year.md) functions.
* Visit Microsoft Excel's [DAY function](https://support.microsoft.com/en-gb/office/day-function-8a7d1cbb-6c7d-4ba1-8aea-25c134d03101) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093040) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DAY) provide versions of the DAY function.