---
layout: doc
outline: deep
lang: en-US
---

# DATE function
## Overview
DATE is a function of the Date and Time category that combines separate year, month and day values into a single valid date. The calculated date is returned as a [serial number](./serial-numbers.md).
## Usage
### Syntax
**DATE(year, month, day)**
### Argument descriptions
* *year*. A positive integer specifying the year component of the date to be calculated. Normally expected to lie within the range [1899, 9999]. Although earlier or later years can be processed, it may not be possible to format the resulting serial numbers as dates.
* *month*. An integer value in the range [1, 12] specifying the month component of the date to be calculated.
* *day*. An integer value in the range [1, 31] specifying the day component of the date to be calculated.
### Additional guidance
* The range of legitimate values for the *day* argument is further limited by the value of the *month* argument. For example, if *month* is set to 4 (representing April), then a #NUM! error is reported if *day* is set to 31 (invalid for April). These checks include disallowing the 29th day of February in non-leap years.
* For dates earlier than 1899-12-31 or later than 9999-12-31, a serial number may be returned which lies outside the range of values that can be formatted as a date. A #VALUE error is reported in date-formatted cells containing such values. You can change the cell to numeric format to view the returned serial number.
* If any argment has a fractional part, DATE uses its [floor value](https://en.wikipedia.org/wiki/Floor_and_ceiling_functions).
<!--@include: ../markdown-snippets/error-type-details.md-->
## Details
* IronCalc utilizes Rust's [*chrono*](https://docs.rs/chrono/latest/chrono/) crate to implement the DATE function.
## Examples
[See this example in IronCalc](https://app.ironcalc.com/?example=DATE).
## Links
* See also IronCalc's [TIME](./TIME) and [DATEVALUE](./DATEVALUE) functions.
* Visit Microsoft Excel's [DATE function](https://support.microsoft.com/en-gb/office/date-function-e36c0c8c-4104-49da-ab83-82328b832349) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3092969) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DATE) provide versions of the DATE function.