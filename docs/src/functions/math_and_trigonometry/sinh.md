---
layout: doc
outline: deep
lang: en-US
---
# SINH function
## Overview
SINH is a function of the Math and Trigonometry category that calculates the hyperbolic sine of a number.
## Usage
### Syntax
**SINH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">sinh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The hyperbolic angle whose hyperbolic sine is to be calculated, expressed in radians.
### Additional guidance
The formula for the hyperbolic sine is:
$$
\text{sinh(x)} = \dfrac{e^x-e^{-x}}{2}
$$
### Returned value
SINH returns a real [number](/features/value-types#numbers) that is the hyperbolic sine of the specified hyperbolic angle.
### Error conditions
* In common with many other IronCalc functions, SINH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then SINH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then SINH returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, SINH may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The SINH function utilizes the *sinh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the SINH function.
<center><img src="/functions/images/hyperbolicsine-curve.png" width="350" alt="Graph showing sinh(x)."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=sinh).

## Links
* For more information about hyperbolic functions, visit Wikipedia's [Hyperbolic functions](https://en.wikipedia.org/wiki/Hyperbolic_functions) page.
* See also IronCalc's [SIN](/functions/math_and_trigonometry/sin), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [SINH function](https://support.microsoft.com/en-us/office/sinh-function-4958f7e2-0d2b-4846-8ef5-8475f3aea5fb) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093517) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/SINH) provide versions of the SINH function.