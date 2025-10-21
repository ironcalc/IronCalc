---
layout: doc
outline: deep
lang: en-US
---

# NETWORKDAYS.INTL function

::: warning
**Note:** This draft page is under construction 🚧
:::

## Overview
NETWORKDAYS.INTL is a function of the Date and Time category that calculates the number of working days between two dates, with customizable weekend definitions and optionally specified holidays.

## Usage

### Syntax
**NETWORKDAYS.INTL(<span title="Number" style="color:#1E88E5">start_date</span>, <span title="Number" style="color:#1E88E5">end_date</span>, [<span title="Number or String" style="color:#4CAF50">weekend</span>], [<span title="Array" style="color:#E91E63">holidays</span>]) => <span title="Number" style="color:#1E88E5">workdays</span>**

### Argument descriptions
* *start_date* ([number](/features/value-types#numbers), required). The start date expressed as a [serial number](/features/serial-numbers.md).
* *end_date* ([number](/features/value-types#numbers), required). The end date expressed as a [serial number](/features/serial-numbers.md).
* *weekend* ([number](/features/value-types#numbers) or [string](/features/value-types#strings), optional). Defines which days are considered weekends. Default is 1 (Saturday-Sunday).
* *holidays* ([array](/features/value-types#arrays) or [range](/features/value-types#ranges), optional). A list of dates to exclude from the calculation, expressed as serial numbers.

### Weekend parameter options
The _weekend_ parameter can be specified in two ways:

**Numeric codes:**
- 1 (default): Saturday and Sunday
- 2: Sunday and Monday
- 3: Monday and Tuesday
- 4: Tuesday and Wednesday
- 5: Wednesday and Thursday
- 6: Thursday and Friday
- 7: Friday and Saturday
- 11: Sunday only
- 12: Monday only
- 13: Tuesday only
- 14: Wednesday only
- 15: Thursday only
- 16: Friday only
- 17: Saturday only

**String pattern:** A 7-character string of "0" and "1" where "1" indicates a weekend day. The string represents Monday through Sunday. For example, "0000011" means Saturday and Sunday are weekends.

### Additional guidance
- If the supplied _start_date_ and _end_date_ arguments have fractional parts, NETWORKDAYS.INTL uses their [floor values](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
- If _start_date_ is later than _end_date_, the function returns a negative number.
- Empty cells in the _holidays_ array are ignored.
- The calculation includes both the start and end dates if they are workdays.

### Returned value
NETWORKDAYS.INTL returns a [number](/features/value-types#numbers) representing the count of working days between the two dates.

### Error conditions
* In common with many other IronCalc functions, NETWORKDAYS.INTL propagates errors that are found in its arguments.
* If fewer than 2 or more than 4 arguments are supplied, then NETWORKDAYS.INTL returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the *start_date* or *end_date* arguments are not (or cannot be converted to) [numbers](/features/value-types#numbers), then NETWORKDAYS.INTL returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *start_date* or *end_date* values are outside the valid date range, then NETWORKDAYS.INTL returns the [`#NUM!`](/features/error-types.md#num) error.
* If the *weekend* parameter is an invalid numeric code or an improperly formatted string, then NETWORKDAYS.INTL returns the [`#NUM!`](/features/error-types.md#num) or [`#VALUE!`](/features/error-types.md#value) error.
* If the *holidays* array contains non-numeric values, then NETWORKDAYS.INTL returns the [`#VALUE!`](/features/error-types.md#value) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
IronCalc utilizes Rust's [chrono](https://docs.rs/chrono/latest/chrono/) crate to implement the NETWORKDAYS.INTL function. This function provides more flexibility than NETWORKDAYS by allowing custom weekend definitions.

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=networkdays-intl).

## Links
* See also IronCalc's [NETWORKDAYS](/functions/date_and_time/networkdays.md) function for the basic version with fixed weekends.
* Visit Microsoft Excel's [NETWORKDAYS.INTL function](https://support.microsoft.com/en-us/office/networkdays-intl-function-a9b26239-4f20-46a1-9ab8-4e925bfd5e28) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093019) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/NETWORKDAYS.INTL) provide versions of the NETWORKDAYS.INTL function.