---
layout: doc
outline: deep
lang: en-US
---

# QUOTIENT function

## Overview
QUOTIENT is a function of the Math and Trigonometry category that returns the integer portion of a division. It divides one number (dividend) by another (divisor) and discards the remainder by truncating toward zero.

## Usage
### Syntax
**QUOTIENT(<span title="Number" style="color:#1E88E5">dividend</span>, <span title="Number" style="color:#1E88E5">divisor</span>) => <span title="Number" style="color:#1E88E5">quotient</span>**

### Argument descriptions
* *dividend* ([number](/features/value-types#numbers), required). The number to be divided.
* *divisor* ([number](/features/value-types#numbers), required). The number by which to divide the dividend.

### Additional guidance
* QUOTIENT returns the integer part of the division and ignores any remainder. For negative results, it truncates toward zero.

### Returned value
QUOTIENT returns a [number](/features/value-types#numbers) that is the integer portion of the division of the dividend by the divisor.

### Error conditions
* In common with many other IronCalc functions, QUOTIENT propagates errors that are found in its arguments.
* If no argument, or more than two arguments, are supplied, then QUOTIENT returns the [`#ERROR!`](/features/error-types.md#error) error.
* If either argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then QUOTIENT returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *divisor* argument is 0, then QUOTIENT returns the [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
* QUOTIENT corresponds to truncating the exact quotient toward zero:
$$
\operatorname{QUOTIENT}(n, d) = \operatorname{TRUNC}\!\left(\dfrac{n}{d}\right),\quad d \ne 0
$$
This differs from using `INT(n/d)` when the quotient is negative, because `INT` rounds down toward −∞, whereas `TRUNC` and QUOTIENT truncate toward zero.
<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=quotient).
-->
## Links
* For more information about the quotient, visit Wikipedia's [Quotient](https://en.wikipedia.org/wiki/Quotient) page.
* See also IronCalc's [MOD](/functions/math_and_trigonometry/mod) function.
* Visit Microsoft Excel's [QUOTIENT function](https://support.microsoft.com/en-gb/office/quotient-function-9f7bf099-2a18-4282-8fa4-65290cc99dee) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093436) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/QUOTIENT) provide versions of the QUOTIENT function.