---
layout: doc
outline: deep
lang: en-US
---

# DATEVALUE function

## Overview
DATEVALUE is a function of the Date and Time category that converts a date stored as text to a [serial number](/features/serial-numbers.md) corresponding to a date value.

## Usage
### Syntax
**DATEVALUE(<span title="Text" style="color:#1E88E5">date_text</span>) => <span title="Number" style="color:#1E88E5">datevalue</span>**

### Argument descriptions
* *date_text* ([text](/features/value-types#strings), required). A text string that represents a date in a known format. The text must represent a date between December 31, 1899 and December 31, 9999.

### Additional guidance
* If the year portion of the *date_text* argument is omitted, DATEVALUE uses the current year from the system clock.
* Time information in the *date_text* argument is ignored. DATEVALUE processes only the date portion.

### Returned value
DATEVALUE returns a [number](/features/value-types#numbers) that represents the date as a [serial number](/features/serial-numbers.md). The serial number corresponds to the number of days since December 31, 1899.

### Error conditions
* In common with many other IronCalc functions, DATEVALUE propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then DATEVALUE returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *date_text* argument is not (or cannot be converted to) a [text](/features/value-types#strings) value, then DATEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *date_text* argument represents a date outside the valid range (before December 31, 1899 or after December 31, 9999), then DATEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *date_text* argument cannot be recognized as a valid date format, then DATEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

<!-- ## Details
For more information on how IronCalc processes Date and Time functions and values, visit [Date and Time](/features/serial-numbers.md) 

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=datevalue).
-->

## Links
* See also IronCalc's [TIMEVALUE](/functions/date_and_time/timevalue.md) function for converting time text to serial numbers.
* Visit Microsoft Excel's [DATEVALUE function](https://support.microsoft.com/en-us/office/datevalue-function-df8b07d4-7761-4a93-bc33-b7471bbff252) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093039) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DATEVALUE) provide versions of the DATEVALUE function.