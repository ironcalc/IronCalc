---
layout: doc
outline: deep
lang: en-US
---

# Serial Numbers
::: warning
**Note:** This draft page is under construction 🚧
:::
**Note**:	For convenience, dates presented on this page are formatted in accordance with the ISO 8601 international standard. IronCalc can recognize and display dates in other formats.

IronCalc stores dates and times as positive numbers, referred to as *serial numbers*. Serial numbers can be formatted to display the date and time. 

The integer part of a serial number represents the date, as a count of the days since the fixed starting date of 1899-12-30. Hence dates are represented by a unique, sequential integer value, for example:
* 1 corresponds to 1899-12-31.
* 2 corresponds to 1900-01-01.
* 36,526 corresponds to 2000-01-01.
* 45,658 corresponds to 2025-01-01.
* 2,958,465 corresponds to 9999-12-31.

To illustrate the concept, type the value 2 into an empty cell that is initially formatted as a number. When you subsequently change the cell to a date format, it will update to show the date 1900-01-01.

The fractional part of a serial number represents time, as a fraction of the day. For example:
* 0.0 corresponds to 00:00:00 (midnight)
* 0.041666667 corresponds to 01:00:00.
* 0.5 corresponds to 12:00:00 (noon)
* 0.75 corresponds to 18:00:00.
* 0.99 corresponds to 23:45:36. 

Since date-times are stored as numbers, they can be used for arithmetic operations in formulas. For example, it is possible to determine the difference between two dates by subtracting one serial number from the other.

**Note**: A #VALUE! error is reported if a date-formatted cell contains a number less than 1 or greater than 2,958,465.

## Compatibility Notes

Excel has an infamous [feature](https://learn.microsoft.com/en-us/office/troubleshoot/excel/wrongly-assumes-1900-is-leap-year) that was ported from a bug in Lotus 1-2-3 that assumes that the year 1900 is a leap year.

That means that serial numbers 1 to 60 in IronCalc are different than Excel.

In IronCalc, Google Sheets, Libre Office and Zoho Date(1900,1,1) returns 2

In Excel Date(1900,1,1) returns 1.

Gnumeric solves the problem in yet another way. It follows Excel from 1 to 59, skips 60, and it follows Excel (and all other engines from there on).
A formula like `=DAY(60)` produces `#NUM!` in Gnumeric.

Serial number 61 corresponds to 1 March 1900, and from there on most spreadsheet engines agree.

IronCalc, like Excel, doesn't deal with serial numbers outside of the range [1, 2,958,465]. Other engines like Google sheets, do not have an upper limit.
