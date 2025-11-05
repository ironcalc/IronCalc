---
layout: doc
outline: deep
lang: en-US
---

# TIMEVALUE function

## Overview
TIMEVALUE is a function of the Date and Time category that converts a time stored as text to a [serial number](/features/serial-numbers.md) corresponding to a time value. The serial number represents time as a decimal fraction of a 24-hour day (e.g., 0.5 represents 12:00:00 noon).

## Usage
### Syntax
**TIMEVALUE(<span title="Text" style="color:#1E88E5">time_text</span>) => <span title="Number" style="color:#1E88E5">timevalue</span>**

### Argument descriptions
* *time_text* ([text](/features/value-types#strings), required). A text string that represents a time in a known format. The text must represent a time between 00:00:00 and 23:59:59.

### Additional guidance
* Date information in the *time_text* argument is ignored. TIMEVALUE processes only the time portion.
* The function can handle various time formats, including both 12-hour and 24-hour formats, as well as text that includes both date and time information.

### Returned value
TIMEVALUE returns a [number](/features/value-types#numbers) that represents the time as a [serial number](/features/serial-numbers.md). The serial number is a decimal fraction of a 24-hour day, where:
* 0.0 represents 00:00:00 (midnight)
* 0.5 represents 12:00:00 (midday)
* 0.99999... represents 23:59:59 (just before midnight)

### Error conditions
* In common with many other IronCalc functions, TIMEVALUE propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then TIMEVALUE returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *time_text* argument is not (or cannot be converted to) a [text](/features/value-types#strings) value, then TIMEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *time_text* argument represents a time outside the valid range, then TIMEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the *time_text* argument cannot be recognized as a valid time format, then TIMEVALUE returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=timevalue).
-->

## Links
* See also IronCalc's [DATEVALUE](/functions/date_and_time/datevalue.md) function for converting date text to serial numbers.
* Visit Microsoft Excel's [TIMEVALUE function](https://support.microsoft.com/en-us/office/timevalue-function-0b615c12-33d8-4431-bf3d-f3eb6d186645) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3267350) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/TIMEVALUE) provide versions of the TIMEVALUE function.