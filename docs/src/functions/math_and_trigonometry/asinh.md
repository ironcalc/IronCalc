---
layout: doc
outline: deep
lang: en-US
---

# ASINH function
## Overview
ASINH is a function of the Math and Trigonometry category that calculates the inverse hyperbolic sine (hyperbolic arcsine) of a number, returning the hyperbolic angle expressed in radians.
## Usage
### Syntax
**ASINH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">asinh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The value whose inverse hyperbolic sine is to be calculated. 
### Additional guidance
The hyperbolic arcsine function is defined as:
$$
\operatorname{asinh}(x) = \ln\!\left(x + \sqrt{x^2 + 1}\,\right)
$$
### Returned value
ASINH returns a real [number](/features/value-types#numbers) in the range (-∞, +∞) that is the hyperbolic arcsine of the specified value, expressed in radians.
### Error conditions
* In common with many other IronCalc functions, ASINH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ASINH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ASINH returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ASINH function utilizes the *asinh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ASINH function.
<center><img src="/functions/images/hyperbolicarcsine-curve.png" width="350" alt="Graph showing asinh(x)."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=asinh).

## Links
* For more information about inverse hyperbolic functions, visit Wikipedia's [Inverse hyperbolic functions](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) page.
* See also IronCalc's [SINH](/functions/math_and_trigonometry/sinh), [ACOSH](/functions/math_and_trigonometry/acosh) and [ATANH](/functions/math_and_trigonometry/atanh) functions.
* Visit Microsoft Excel's [ASINH function](https://support.microsoft.com/de-de/office/asinh-function-62b4f5b6-d9cc-4c17-9d04-aa5371806c74) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093393) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ASINH) provide versions of the ASINH function.