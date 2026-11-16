---
layout: doc
outline: deep
lang: en-US
---

# DATE function

## Overview

DATE is a function of the Date and Time category that builds a date from a year, a month and a day, returning it as a [serial number](/features/serial-numbers.md). It is the usual way to write a date inside a formula, since date literals such as `"2025-01-15"` are text and are not converted automatically.

**Note**: This function is affected by the choice of [date system](/features/serial-numbers).

## Usage

### Syntax

**DATE(<span title="Number" style="color:#1E88E5">year</span>, <span title="Number" style="color:#1E88E5">month</span>, <span title="Number" style="color:#1E88E5">day</span>) => <span title="Number" style="color:#1E88E5">date</span>**

### Argument descriptions

* *year* ([number](/features/value-types#numbers), required). The year of the date. Four-digit years are taken literally. Small values are interpreted as an offset, as described under [Two-digit years](#two-digit-years).
* *month* ([number](/features/value-types#numbers), required). The month of the date, where 1 is January and 12 is December. Values outside that range roll into the neighbouring years, as described under [Overflowing months and days](#overflowing-months-and-days).
* *day* ([number](/features/value-types#numbers), required). The day of the month. Values outside the length of the month roll into the neighbouring months.

### Additional guidance

* If an argument has a fractional part, DATE uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions). For example, `=DATE(2025.9, 1.9, 1.9)` is 2025-01-01.
* The three arguments can be ranges or arrays, in which case DATE returns an array of dates. A single value is broadcast across the other arguments, so `=DATE(2025, {1,2,3}, 1)` returns the first days of January, February and March 2025.
* The result is a plain number. The cell holding the formula is given a date format automatically, but the value can be used in arithmetic like any other number. `=DATE(2025,3,1)-DATE(2025,2,1)` is 28.

### Returned value

DATE returns a [number](/features/value-types#numbers): the [serial number](/features/serial-numbers.md) of the date in the workbook's date system. In the LibreOffice/IronCalc and Excel 1900 systems, `=DATE(2025,1,15)` returns 45672. In the Excel 1904 system it returns 44210.

### Error conditions

* In common with many other IronCalc functions, DATE propagates errors that are found in its arguments.
* If fewer or more than three arguments are supplied, then DATE returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then DATE returns the [`#VALUE!`](/features/error-types.md#value) error. Text that looks like a number is accepted, so `=DATE("2025",1,1)` is fine while `=DATE("a",1,1)` is not.
* If the resulting date lies outside the range of the date system, or *year* is negative, then DATE returns an error. In the LibreOffice/IronCalc system that error is [`#VALUE!`](/features/error-types.md#value), following LibreOffice. In the Excel 1900 and Excel 1904 systems it is [`#NUM!`](/features/error-types.md#num), following Excel.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

### Two-digit years

A small *year* argument is not taken literally. The rule depends on the [date system](/features/serial-numbers):

| Date system | Rule | `=DATE(25,1,1)` | `=DATE(99,1,1)` | `=DATE(100,1,1)` |
|:--|:--|:--|:--|:--|
| LibreOffice/IronCalc | 0 to 29 are 2000 to 2029, 30 to 99 are 1930 to 1999, anything else is literal | 2025-01-01 | 1999-01-01 | `#VALUE!` (year 100 is before 1582) |
| Excel 1900 | 0 to 1899 are added to 1900 | 1925-01-01 | 1999-01-01 | 2000-01-01 |
| Excel 1904 | 0 to 1903 are added to 1904 | 1929-01-01 | 2003-01-01 | 2004-01-01 |

A consequence of the Excel rules is that years before 1900 cannot be written at all: in the Excel 1900 system `=DATE(1899,12,31)` is 3799-12-31. Use the LibreOffice/IronCalc system for historical dates.

### Overflowing months and days

A *month* outside 1 to 12 and a *day* outside the length of the month are not errors. The month is first rolled into the year, and the day is then counted from the first of that month, so both negative values and values that are too large are accepted:

| Formula | Result |
|:--|:--|
| `=DATE(2025,13,1)` | 2026-01-01 |
| `=DATE(2025,0,1)` | 2024-12-01 |
| `=DATE(2025,-1,1)` | 2024-11-01 |
| `=DATE(2025,2,30)` | 2025-03-02 |
| `=DATE(2025,3,0)` | 2025-02-28, the last day of the previous month |
| `=DATE(2025,1,-5)` | 2024-12-26 |

This makes DATE handy for date arithmetic. `=DATE(YEAR(A1), MONTH(A1)+1, 0)` is the last day of the month of the date in A1, and `=DATE(YEAR(A1), MONTH(A1)+6, DAY(A1))` is six months later.

Excel checks the range after each step rather than only at the end, and IronCalc does the same in the Excel systems: `=DATE(10000,-12,1)` is 9998-12-01 in the LibreOffice/IronCalc system but `#NUM!` in the Excel 1900 system.

### Valid range

| Date system | Earliest date | Latest date |
|:--|:--|:--|
| LibreOffice/IronCalc | 1582-10-15, the start of the [Gregorian calendar](https://en.wikipedia.org/wiki/Gregorian_calendar) | 9999-12-31 |
| Excel 1900 | 1899-12-31, serial number 0 | 9999-12-31 |
| Excel 1904 | 1904-01-01, serial number 0 | 9999-12-31 |

Dates before 1899-12-30 have negative serial numbers in the LibreOffice/IronCalc system. `=DATE(1616,4,22)` is −103615, and `=TEXT(DATE(1616,4,22),"dddd")` tells you that Miguel de Cervantes died on a Friday.

The Excel 1900 system includes the fictional 29 February 1900, so `=DATE(1900,2,29)` is serial number 60 there. In the LibreOffice/IronCalc system the same formula rolls over to 1900-03-01.

## Examples

The table assumes the LibreOffice/IronCalc date system.

| Formula | Result | Comment |
|:--|--:|:--|
| `=DATE(2025,1,15)` | 2025-01-15 | Serial number 45672 |
| `=DATE(2025,12,31)-DATE(2025,1,1)+1` | 365 | Days in 2025 |
| `=DATE(2024,2,29)` | 2024-02-29 | 2024 is a leap year |
| `=DATE(2025,2,29)` | 2025-03-01 | 2025 is not |
| `=DATE(2025,MONTH(TODAY())+1,0)` | | Last day of the current month in 2025 |
| `=DATE(25,1,1)` | 2025-01-01 | Two-digit year |
| `=DATE(1969,7,20)` | 1969-07-20 | Serial number 25404 |
| `=DATE(1066,10,14)` | `#VALUE!` | Before the Gregorian calendar |
| `=DATE(2025,1)` | `#ERROR!` | Three arguments are required |

## Links

* See also IronCalc's [YEAR](/functions/date_and_time/year.md), [MONTH](/functions/date_and_time/month.md), [DAY](/functions/date_and_time/day.md), [DATEVALUE](/functions/date_and_time/datevalue.md), [EDATE](/functions/date_and_time/edate.md) and [EOMONTH](/functions/date_and_time/eomonth.md) functions.
* Visit Microsoft Excel's [DATE function](https://support.microsoft.com/en-us/excel/functions/date-function) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3092969) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DATE) provide versions of the DATE function.
