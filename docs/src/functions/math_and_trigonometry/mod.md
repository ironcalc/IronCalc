---
layout: doc
outline: deep
lang: en-US
---

# MOD function

## Overview
MOD is a function of the Math and Trigonometry category that returns the remainder after one number (the dividend) is divided by another number (the divisor). The result has the same sign as the divisor.

## Usage
### Syntax
**MOD(<span title="Number" style="color:#1E88E5">dividend</span>, <span title="Number" style="color:#1E88E5">divisor</span>) => <span title="Number" style="color:#1E88E5">remainder</span>**

### Argument descriptions
* *dividend* ([number](/features/value-types#numbers), required). The number whose remainder is to be calculated.
* *divisor* ([number](/features/value-types#numbers), required). The number by which the dividend is divided.

### Additional guidance
None.

### Returned value
MOD returns a [number](/features/value-types#numbers) that is the remainder after division, with the same sign as the divisor.

### Error conditions
* In common with many other IronCalc functions, MOD propagates errors that are found in its arguments.
* If no argument, or more than two arguments, are supplied, then MOD returns the [`#ERROR!`](/features/error-types.md#error) error.
* If either argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then MOD returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *divisor* argument is 0, then MOD returns the [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
* MOD follows the formula:
$$
\operatorname{MOD}(n, d) = n - d \times \operatorname{INT}\!\left(\dfrac{n}{d}\right)
$$
Since `INT` returns the greatest integer less than or equal to its argument (it rounds down), the remainder's sign matches the divisor, even though this might appear counterintuitive when the dividend and divisor have different signs.
<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=mod).
-->
## Links
* For more information about the modulo operation, visit Wikipedia's [Modulo](https://en.wikipedia.org/wiki/Modulo) page.
* See also IronCalc's [QUOTIENT](/functions/math_and_trigonometry/quotient) function.
* Visit Microsoft Excel's [MOD function](https://support.microsoft.com/en-us/office/mod-function-9b6cd169-b6ee-406a-a97b-edf2a9dc24f3) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093497) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/MOD) provide versions of the MOD function.