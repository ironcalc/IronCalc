---
layout: doc
outline: deep
lang: en-US
---

# MONTH function

## Overview

MONTH is a function of the Date and Time category that extracts the month from a date given as a [serial number](/features/serial-numbers.md), returning a number from 1 (January) to 12 (December).

**Note**: This function is affected by the choice of [date system](/features/serial-numbers).

## Usage

### Syntax

**MONTH(<span title="Number" style="color:#1E88E5">date</span>) => <span title="Number" style="color:#1E88E5">month</span>**

### Argument descriptions

* *date* ([number](/features/value-types#numbers), required). The date whose month is wanted, expressed as a [serial number](/features/serial-numbers.md). It is usually a reference to a cell holding a date, or the result of a function such as [DATE](/functions/date_and_time/date.md) or [TODAY](/functions/date_and_time/today.md). Text that IronCalc can read as a date, like `"2024-12-31"`, is accepted too.

### Additional guidance

* If the supplied *date* argument has a fractional part, MONTH uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions). The time of day never changes the month.
* The argument can be a range or an array, in which case MONTH returns an array with the month of every element.
* The valid range of the argument depends on the [date system](/features/serial-numbers), see [Valid range](#valid-range) below.
* MONTH returns a number, not a name. Use [TEXT](/functions/text/text.md) for the name of the month: `=TEXT(A1,"mmmm")` is "January" for a date in January.

### Returned value

MONTH returns an integer [number](/features/value-types#numbers) from 1 to 12, the month of the date in the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar). 1 is January, 2 is February and so on up to 12 for December.

### Error conditions

* In common with many other IronCalc functions, MONTH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then MONTH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then MONTH returns the [`#VALUE!`](/features/error-types.md#value) error. For example `=MONTH("tomorrow")` is `#VALUE!`.
* If the *date* argument is outside the valid range of the date system, then MONTH returns the [`#NUM!`](/features/error-types.md#num) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

### Valid range

| Date system | Smallest argument | Largest argument | `=MONTH(-1)` |
|:--|:--|:--|:--|
| LibreOffice/IronCalc | Negative numbers are accepted, far into the past | 2,958,465 (9999-12-31) | 12 (1899-12-29) |
| Excel 1900 | 0 | 2,958,465 (9999-12-31) | `#NUM!` |
| Excel 1904 | 0 | 2,957,003 (9999-12-31) | `#NUM!` |

### The same number, a different month

A serial number is only a count of days, so the same number can name a different date in each system. `=MONTH(60)` is 2 in the LibreOffice/IronCalc system (1900-02-28) and in the Excel 1900 system (its fictional 1900-02-29), but 3 in the Excel 1904 system (1904-03-01). Dates built with [DATE](/functions/date_and_time/date.md) do not have that problem: `=MONTH(DATE(2025,1,15))` is 1 everywhere.

## Examples

The table assumes the LibreOffice/IronCalc date system. Column A holds the date 2025-01-15, which is the serial number 45672.

| Formula | Result | Comment |
|:--|--:|:--|
| `=MONTH(A1)` | 1 | January |
| `=MONTH(45672.9)` | 1 | The time of day is ignored |
| `=MONTH(DATE(2025,13,1))` | 1 | DATE rolls the 13th month into January of the next year |
| `=MONTH("2024-12-31")` | 12 | Text that reads as a date is accepted |
| `=TEXT(A1,"mmmm")` | January | The name rather than the number |
| `=DATE(YEAR(A1),MONTH(A1),1)` | 2025-01-01 | First day of the month of A1 |
| `=DATE(YEAR(A1),MONTH(A1)+1,0)` | 2025-01-31 | Last day of the month of A1 |
| `=MONTH(A1)=MONTH(TODAY())` | | TRUE while the current month is January |
| `=MONTH(-1)` | 12 | 1899-12-29, a date before 1900 |
| `=MONTH(2958466)` | `#NUM!` | Past 9999-12-31 |

<IronCalcEmbed src="/examples/month.ic"/>

## Links

* See also IronCalc's [YEAR](/functions/date_and_time/year.md), [DAY](/functions/date_and_time/day.md), [DATE](/functions/date_and_time/date.md), [EDATE](/functions/date_and_time/edate.md) and [EOMONTH](/functions/date_and_time/eomonth.md) functions.
* Visit Microsoft Excel's [MONTH function](https://support.microsoft.com/en-gb/office/month-function-579a2881-199b-48b2-ab90-ddba0eba86e8) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093052) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/MONTH) provide versions of the MONTH function.
