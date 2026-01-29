---
layout: doc
outline: deep
lang: en-US
---

# EVEN function

## Overview
EVEN is a function of the Math and Trigonometry category that rounds a number up (away from zero) to the nearest even integer.

## Usage
### Syntax
**EVEN(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">even</span>**

### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The number that is to be rounded to the nearest even integer.

### Additional guidance
* EVEN rounds away from zero, meaning:
  * Positive numbers are rounded up to the next even integer.
  * Negative numbers are rounded down (toward negative infinity) to the next even integer.
* If the *number* argument is already an even integer, EVEN returns it unchanged.
* Since zero is considered an even number, the EVEN function returns 0 when *number* is 0.

### Returned value
EVEN returns a [number](/features/value-types#numbers) that is the nearest even integer, rounded away from zero.

### Error conditions
* In common with many other IronCalc functions, EVEN propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then EVEN returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then EVEN returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=even).
-->

## Links
* For more information about even and odd numbers, visit Wikipedia's [Parity](https://en.wikipedia.org/wiki/Parity_(mathematics)) page.
* See also IronCalc's [ODD](/functions/math_and_trigonometry/odd) function.
* Visit Microsoft Excel's [EVEN function](https://support.microsoft.com/en-us/office/even-function-197b5f06-c795-4c1e-8696-3c3b8a646cf9) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093409) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/EVEN) provide versions of the EVEN function.