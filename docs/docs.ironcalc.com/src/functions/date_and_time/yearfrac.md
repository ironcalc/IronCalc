---
layout: doc
outline: deep
lang: en-US
---

# YEARFRAC function

## Overview

YEARFRAC is a function of the Date and Time category that returns the fraction of a year between two dates. The fraction depends on a *day count convention*, chosen with the *basis* argument, which decides how many days a month and a year are deemed to have. YEARFRAC is the building block of many financial calculations, such as the interest accrued over a period or the age of a person in years.

**Note**: This function is affected by the choice of [date system](/features/serial-numbers).

## Usage

### Syntax

**YEARFRAC(<span title="Number" style="color:#1E88E5">start_date</span>, <span title="Number" style="color:#1E88E5">end_date</span>, <span title="Number" style="color:#1E88E5">basis</span>=0) => <span title="Number" style="color:#1E88E5">fraction</span>**

### Argument descriptions

* *start_date* ([number](/features/value-types#numbers), required). The first date of the period, expressed as a [serial number](/features/serial-numbers.md). Usually a reference to a cell holding a date or the result of the [DATE](/functions/date_and_time/date.md) function. Text that IronCalc can read as a date is accepted too.
* *end_date* ([number](/features/value-types#numbers), required). The last date of the period, in the same form.
* *basis* ([number](/features/value-types#numbers), [optional](/features/optional-arguments.md)). The day count convention to use, from 0 to 4, as listed in the [table below](#day-count-conventions). The default is 0, the US 30/360 convention.

### Additional guidance

* The order of the two dates does not matter. YEARFRAC always returns a positive number, or 0 when the dates are equal.
* Only the dates count. If an argument has a fractional part, YEARFRAC uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions), so the time of day is ignored. If *basis* has a fractional part, it is truncated too, and `1.9` means basis 1.
* The result is a plain number of years. Multiply it by an annual rate to get the interest of the period, or use [INT](/functions/math_and_trigonometry/int.md) on it to get whole years.
* For dates more than a few years apart, basis 1 is the convention that stays closest to the calendar. Basis 2 and 3 drift because they divide by 360 or 365 regardless of leap years. See the [table below](#day-count-conventions).

### Returned value

YEARFRAC returns a non-negative [number](/features/value-types#numbers), the length of the period between the two dates measured in years under the chosen convention.

### Error conditions

* In common with many other IronCalc functions, YEARFRAC propagates errors that are found in its arguments.
* If fewer than two or more than three arguments are supplied, then YEARFRAC returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then YEARFRAC returns the [`#VALUE!`](/features/error-types.md#value) error. Booleans are not accepted as dates or as basis, so `=YEARFRAC(TRUE, 45658)` is `#VALUE!`.
* If either date is outside the valid range of the [date system](/features/serial-numbers), then YEARFRAC returns the [`#NUM!`](/features/error-types.md#num) error. In the LibreOffice/IronCalc system negative serial numbers are valid dates, in the Excel systems they are not.
* If *basis* is not between 0 and 4, then YEARFRAC returns the [`#NUM!`](/features/error-types.md#num) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details

### Day count conventions

| basis | Convention | Days in the period | Days in a year |
|------:|:-----------|:-------------------|:---------------|
| 0 (default) | US 30/360 | Every month has 30 days, with the US rules for the 31st and the end of February | 360 |
| 1 | Actual/Actual | The real number of days | 365 or 366, or an average, see below |
| 2 | Actual/360 | The real number of days | 360 |
| 3 | Actual/365 | The real number of days | 365 |
| 4 | European 30/360 | Every month has 30 days, with the European rule for the 31st | 360 |

The two 30/360 conventions move a date that falls on the 31st back to the 30th. They differ in the details. In the European convention (basis 4) any 31st becomes the 30th. In the US convention (basis 0) the end date only moves when the start date is also on the 30th or 31st, and a start date at the end of February is treated as the 30th as well. The result is the same as the [DAYS360](/functions/date_and_time/days360.md) function divided by 360 for basis 4, and usually, but not always, for basis 0: `=YEARFRAC(DATE(2025,2,28), DATE(2025,3,31), 0)` counts 31 days whereas `=DAYS360(DATE(2025,2,28), DATE(2025,3,31))` counts 30. IronCalc follows Excel in both cases.

Under basis 1 the length of the year depends on the period:

* If the dates are a year or less apart, the year has 366 days when both dates are in the same leap year, or when a 29 February lies inside the period. Otherwise it has 365 days.
* If the dates are more than a year apart, the year is the average length of the calendar years touched by the period, counting both the first and the last year in full. From 2023-01-01 to 2025-01-01 the average over 2023, 2024 and 2025 is 365⅓ days, so the result is 731 ÷ 365⅓, a little above 2.

Under basis 2 and 3 the number of days is divided by a fixed 360 or 365, so a calendar year comes out as 1.0167 or 1.0027 years when it has 366 days.

For a wider discussion of the conventions, see the [basis glossary](/functions/financial#basis-glossary) of the financial functions, which use the same *basis* argument.

### Date systems

The days between two dates are the same in every [date system](/features/serial-numbers), so the only difference is which numbers are valid dates. In the LibreOffice/IronCalc system `=YEARFRAC(-1, 364)` is 1, since both are dates in 1899 and 1900, while in the Excel 1900 system it is `#NUM!`. Formulas that build their dates with [DATE](/functions/date_and_time/date.md) give the same result in every system.

## Examples

The table assumes the LibreOffice/IronCalc date system.

| Formula | Result | Comment |
|:--|--:|:--|
| `=YEARFRAC(DATE(2025,1,1), DATE(2025,7,1))` | 0.5 | Six months under the 30/360 convention |
| `=YEARFRAC(DATE(2025,7,1), DATE(2025,1,1))` | 0.5 | The order of the dates does not matter |
| `=YEARFRAC(DATE(2024,1,1), DATE(2025,1,1), 1)` | 1 | A whole leap year |
| `=YEARFRAC(DATE(2024,1,1), DATE(2025,1,1), 2)` | 1.016666667 | 366 days divided by 360 |
| `=YEARFRAC(DATE(2024,1,1), DATE(2025,1,1), 3)` | 1.002739726 | 366 days divided by 365 |
| `=YEARFRAC(DATE(2025,1,1), DATE(2025,12,31), 1)` | 0.997260274 | 364 days divided by 365 |
| `=YEARFRAC(DATE(2023,7,1), DATE(2024,3,1), 1)` | 0.666666667 | 244 days divided by 366, since 29 February 2024 lies in the period |
| `=YEARFRAC(DATE(2023,1,1), DATE(2025,1,1), 1)` | 2.000912409 | 731 days divided by the average year length 365⅓ |
| `=YEARFRAC(DATE(2025,1,31), DATE(2025,3,31))` | 0.166666667 | Two 30-day months under 30/360 |
| `=YEARFRAC(DATE(2025,2,28), DATE(2025,3,31), 0)` | 0.086111111 | 31 days under the US rules |
| `=YEARFRAC(DATE(2025,2,28), DATE(2025,3,31), 4)` | 0.088888889 | 32 days under the European rules |
| `=INT(YEARFRAC(DATE(1980,5,17), TODAY(), 1))` | | The age in whole years of someone born on 17 May 1980 |
| `=YEARFRAC(DATE(2025,1,1), DATE(2025,7,1), 5)` | `#NUM!` | The basis must be between 0 and 4 |

A loan of 10,000 at 6% a year is repaid on 2025-07-01 after being taken on 2025-03-15. Under the Actual/365 convention the interest due is

```
=10000 * 0.06 * YEARFRAC(DATE(2025,3,15), DATE(2025,7,1), 3)
```

which is 108 days of interest, or 177.53.

## Links

* See also IronCalc's [DAYS360](/functions/date_and_time/days360.md), [DATEDIF](/functions/date_and_time/datedif.md) and [DAYS](/functions/date_and_time/days.md) functions, and the financial functions that take a *basis* argument such as [PRICE](/functions/financial/price.md) and [ACCRINT](/functions/financial/accrint.md).
* Visit Microsoft Excel's [YEARFRAC function](https://support.microsoft.com/en-us/office/yearfrac-function-3844141e-c76d-4143-82b6-208454ddc6a8) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3092989) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/YEARFRAC) provide versions of the YEARFRAC function.
