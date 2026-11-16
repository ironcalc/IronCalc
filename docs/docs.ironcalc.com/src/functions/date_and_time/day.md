---
layout: doc
outline: deep
lang: en-US
---

# DAY function

## Overview

DAY is a function of the Date and Time category that extracts the day of the month from a date given as a [serial number](/features/serial-numbers.md), returning a number from 1 to 31.

**Note**: This function is affected by the choice of [date system](/features/serial-numbers).

## Usage

### Syntax

**DAY(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">day</span>**

### Argument descriptions

* *date* ([number](/features/value-types#numbers), required). The date whose day of the month is wanted, expressed as a [serial number](/features/serial-numbers.md). It is usually a reference to a cell holding a date, or the result of a function such as [DATE](/functions/date_and_time/date.md) or [TODAY](/functions/date_and_time/today.md). Text that IronCalc can read as a date, like `"2024-12-31"`, is accepted too.

### Additional guidance

* If the supplied *date* argument has a fractional part, DAY uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions). The time of day never changes the day.
* The argument can be a range or an array, in which case DAY returns an array with the day of every element.
* The valid range of the argument depends on the [date system](/features/serial-numbers), see [Valid range](#valid-range) below.
* DAY is the day of the month, not the day of the week. Use [WEEKDAY](/functions/date_and_time/weekday.md) for the latter, or [TEXT](/functions/text/text.md) for its name: `=TEXT(A1,"dddd")` is "Wednesday" for 2025-01-15.

### Returned value

DAY returns an integer [number](/features/value-types#numbers) from 1 to 31, the day of the month of the date in the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar). The one exception is the Excel 1900 system, where `=DAY(0)` is 0, see below.

### Error conditions

* In common with many other IronCalc functions, DAY propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then DAY returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then DAY returns the [`#VALUE!`](/features/error-types.md#value) error. For example `=DAY("tomorrow")` is `#VALUE!`.
* If the *date* argument is outside the valid range of the date system, then DAY returns the [`#NUM!`](/features/error-types.md#num) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

### Valid range

| Date system | Smallest argument | Largest argument | `=DAY(0)` | `=DAY(-1)` |
|:--|:--|:--|:--|:--|
| LibreOffice/IronCalc | Negative numbers are accepted, far into the past | 2,958,465 (9999-12-31) | 30 (1899-12-30) | 29 (1899-12-29) |
| Excel 1900 | 0 | 2,958,465 (9999-12-31) | 0 | `#NUM!` |
| Excel 1904 | 0 | 2,957,003 (9999-12-31) | 1 (1904-01-01) | `#NUM!` |

The Excel 1900 system has two peculiar serial numbers, and DAY follows Excel for both. Serial number 0 is shown by Excel as "0 January 1900", so `=DAY(0)` is 0. Serial number 60 is the fictional 29 February 1900, so `=DAY(60)` is 29, while in the LibreOffice/IronCalc system the same number is 1900-02-28 and `=DAY(60)` is 28.

### The same number, a different day

A serial number is only a count of days, so the same number names a different date in each system. `=DAY(45672)` is 15 in the LibreOffice/IronCalc and Excel 1900 systems and 16 in the Excel 1904 system. Dates built with [DATE](/functions/date_and_time/date.md) do not have that problem: `=DAY(DATE(2025,1,15))` is 15 everywhere.

## Examples

The table assumes the LibreOffice/IronCalc date system. Column A holds the date 2025-01-15, which is the serial number 45672.

| Formula | Result | Comment |
|:--|--:|:--|
| `=DAY(A1)` | 15 | |
| `=DAY(45672.9)` | 15 | The time of day is ignored |
| `=DAY(DATE(2025,2,30))` | 2 | DATE rolls 30 February into 2 March |
| `=DAY("2024-12-31")` | 31 | Text that reads as a date is accepted |
| `=DAY(DATE(YEAR(A1),MONTH(A1)+1,0))` | 31 | Number of days in the month of A1 |
| `=DAY(A1)-DAY(DATE(2025,1,1))` | 14 | Days since the first of the month |
| `=TEXT(A1,"dddd")` | Wednesday | Day of the week, which DAY does not give |
| `=DAY(-1)` | 29 | 1899-12-29, a date before 1900 |
| `=DAY(2958466)` | `#NUM!` | Past 9999-12-31 |

<IronCalcEmbed src="/examples/day.ic"/>

## Links

* See also IronCalc's [MONTH](/functions/date_and_time/month.md), [YEAR](/functions/date_and_time/year.md), [DATE](/functions/date_and_time/date.md) and [WEEKDAY](/functions/date_and_time/weekday.md) functions.
* Visit Microsoft Excel's [DAY function](https://support.microsoft.com/en-gb/office/day-function-8a7d1cbb-6c7d-4ba1-8aea-25c134d03101) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093040) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DAY) provide versions of the DAY function.
