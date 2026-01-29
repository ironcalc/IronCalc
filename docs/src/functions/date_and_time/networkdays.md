---
layout: doc
outline: deep
lang: en-US
---

# NETWORKDAYS function

::: warning
**Note:** This draft page is under construction 🚧
:::

## Overview
NETWORKDAYS is a function of the Date and Time category that calculates the number of working days between two dates, excluding weekends (Saturday and Sunday by default) and optionally specified holidays.

## Usage

### Syntax
**NETWORKDAYS(<span title="Number" style="color:#1E88E5">start_date</span>, <span title="Number" style="color:#1E88E5">end_date</span>, [<span title="Array" style="color:#E91E63">holidays</span>]) => <span title="Number" style="color:#1E88E5">workdays</span>**

### Argument descriptions
* *start_date* ([number](/features/value-types#numbers), required). The start date expressed as a [serial number](/features/serial-numbers.md).
* *end_date* ([number](/features/value-types#numbers), required). The end date expressed as a [serial number](/features/serial-numbers.md).
* *holidays* ([array](/features/value-types#arrays) or [range](/features/value-types#ranges), optional). A list of dates to exclude from the calculation, expressed as serial numbers.

### Additional guidance
- If the supplied _start_date_ and _end_date_ arguments have fractional parts, NETWORKDAYS uses their [floor values](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
- If _start_date_ is later than _end_date_, the function returns a negative number.
- Weekend days are Saturday and Sunday by default. Use [NETWORKDAYS.INTL](networkdays.intl) for custom weekend definitions.
- Empty cells in the _holidays_ array are ignored.
- The calculation includes both the start and end dates if they are workdays.

### Returned value
NETWORKDAYS returns a [number](/features/value-types#numbers) representing the count of working days between the two dates.

### Error conditions
* In common with many other IronCalc functions, NETWORKDAYS propagates errors that are found in its arguments.
* If fewer than 2 or more than 3 arguments are supplied, then NETWORKDAYS returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the *start_date* or *end_date* arguments are not (or cannot be converted to) [numbers](/features/value-types#numbers), then NETWORKDAYS returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *start_date* or *end_date* values are outside the valid date range, then NETWORKDAYS returns the [`#NUM!`](/features/error-types.md#num) error.
* If the *holidays* array contains non-numeric values, then NETWORKDAYS returns the [`#VALUE!`](/features/error-types.md#value) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
IronCalc utilizes Rust's [chrono](https://docs.rs/chrono/latest/chrono/) crate to implement the NETWORKDAYS function. The function treats Saturday and Sunday as weekend days.

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=networkdays).

## Links
* See also IronCalc's [NETWORKDAYS.INTL](/functions/date_and_time/networkdays.intl.md) function for custom weekend definitions.
* Visit Microsoft Excel's [NETWORKDAYS function](https://support.microsoft.com/en-us/office/networkdays-function-48e717bf-a7a3-495f-969e-5005e3eb18e7) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093018) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/NETWORKDAYS) provide versions of the NETWORKDAYS function.