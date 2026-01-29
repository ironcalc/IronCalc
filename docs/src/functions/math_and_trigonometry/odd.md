---
layout: doc
outline: deep
lang: en-US
---

# ODD function

## Overview
ODD is a function of the Math and Trigonometry category that rounds a number up (away from zero) to the nearest odd integer.

## Usage
### Syntax
**ODD(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">odd</span>**

### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The number that is to be rounded to the nearest odd integer.

### Additional guidance
* ODD rounds away from zero, meaning:
  * Positive numbers are rounded up to the next odd integer.
  * Negative numbers are rounded down (toward negative infinity) to the next odd integer.
* If the *number* argument is already an odd integer, ODD returns it unchanged.
* Since zero is considered an even number, the ODD function returns 1 when *number* is 0.

### Returned value
ODD returns a [number](/features/value-types#numbers) that is the nearest odd integer, rounded away from zero.

### Error conditions
* In common with many other IronCalc functions, ODD propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ODD returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ODD returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=odd).
-->

## Links
* For more information about even and odd numbers, visit Wikipedia's [Parity](https://en.wikipedia.org/wiki/Parity_(mathematics)) page.
* See also IronCalc's [EVEN](/functions/math_and_trigonometry/even) function.
* Visit Microsoft Excel's [ODD function](https://support.microsoft.com/en-us/office/odd-function-deae64eb-e08a-4c88-8b40-6d0b42575c98) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093499) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ODD) provide versions of the ODD function.