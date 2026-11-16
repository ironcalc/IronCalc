---
layout: doc
outline: deep
lang: en-US
---

# YEAR function

## Overview

YEAR is a function of the Date and Time category that extracts the year from a date given as a [serial number](/features/serial-numbers.md). For the dates you meet in everyday work the result is a four-digit number such as 2025.

**Note**: This function is affected by the choice of [date system](/features/serial-numbers).

## Usage

### Syntax

**YEAR(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">year</span>**

### Argument descriptions

* *date* ([number](/features/value-types#numbers), required). The date whose year is wanted, expressed as a [serial number](/features/serial-numbers.md). It is usually a reference to a cell holding a date, or the result of a function such as [DATE](/functions/date_and_time/date.md) or [TODAY](/functions/date_and_time/today.md). Text that IronCalc can read as a date, like `"2024-12-31"`, is accepted too.

### Additional guidance

* If the supplied *date* argument has a fractional part, YEAR uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions). The time of day never changes the year.
* The argument can be a range or an array, in which case YEAR returns an array with the year of every element.
* The valid range of the argument depends on the [date system](/features/serial-numbers), see [Valid range](#valid-range) below.

### Returned value

YEAR returns an integer [number](/features/value-types#numbers), the year of the date in the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar).

### Error conditions

* In common with many other IronCalc functions, YEAR propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then YEAR returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then YEAR returns the [`#VALUE!`](/features/error-types.md#value) error. For example `=YEAR("tomorrow")` is `#VALUE!`.
* If the *date* argument is outside the valid range of the date system, then YEAR returns the [`#NUM!`](/features/error-types.md#num) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

### Valid range

| Date system | Smallest argument | Largest argument | `=YEAR(0)` | `=YEAR(-1)` |
|:--|:--|:--|:--|:--|
| LibreOffice/IronCalc | Negative numbers are accepted, far into the past | 2,958,465 (9999-12-31) | 1899 | 1899 |
| Excel 1900 | 0 | 2,958,465 (9999-12-31) | 1900 | `#NUM!` |
| Excel 1904 | 0 | 2,957,003 (9999-12-31) | 1904 | `#NUM!` |

In the LibreOffice/IronCalc system negative serial numbers are historical dates, so `=YEAR(-103615)` is 1616. Years before AD 1 are returned as negative numbers. There is no year zero, so −1 is 1 BC.

The Excel systems show their epoch as the year 1900 or 1904: `=YEAR(0)` is 1900 in the Excel 1900 system, where serial number 0 is Excel's "0 January 1900".

### The same number, a different year

A serial number is only a count of days, so the same number names a different date in each system. `=YEAR(45672)` is 2025 in the LibreOffice/IronCalc and Excel 1900 systems and 2029 in the Excel 1904 system. Dates built with [DATE](/functions/date_and_time/date.md) do not have that problem: `=YEAR(DATE(2025,1,15))` is 2025 everywhere.

## Examples

The table assumes the LibreOffice/IronCalc date system. Column A holds the date 2025-01-15, which is the serial number 45672.

| Formula | Result | Comment |
|:--|--:|:--|
| `=YEAR(A1)` | 2025 | |
| `=YEAR(45672.9)` | 2025 | The time of day is ignored |
| `=YEAR(DATE(2025,13,1))` | 2026 | DATE rolls the 13th month into the next year |
| `=YEAR("2024-12-31")` | 2024 | Text that reads as a date is accepted |
| `=YEAR(A1)-YEAR(DATE(1969,7,20))` | 56 | Calendar years between two dates |
| `=DATE(YEAR(A1),1,1)` | 2025-01-01 | First day of the year of A1 |
| `=DATE(YEAR(A1)+1,MONTH(A1),DAY(A1))` | 2026-01-15 | Same day next year |
| `=YEAR(-103615)` | 1616 | A date before 1900 |
| `=YEAR(2958466)` | `#NUM!` | Past 9999-12-31 |

<IronCalcEmbed src="/examples/year.ic"/>

## Links

* See also IronCalc's [MONTH](/functions/date_and_time/month.md), [DAY](/functions/date_and_time/day.md) and [DATE](/functions/date_and_time/date.md) functions.
* Visit Microsoft Excel's [YEAR function](https://support.microsoft.com/en-gb/office/year-function-c64f017a-1354-490d-981f-578e8ec8d3b9) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093061) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/YEAR) provide versions of the YEAR function.
