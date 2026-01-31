---
layout: doc
outline: deep
lang: en-US
---
# TANH function
## Overview
TANH is a function of the Math and Trigonometry category that calculates the hyperbolic tangent of a number.
## Usage
### Syntax
**TANH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">tanh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The hyperbolic angle whose hyperbolic tangent is to be calculated, expressed in radians.
### Additional guidance
The formula for the hyperbolic tangent is:
$$
\text{tanh(x)} = \dfrac{sinh(x)}{cosh(x)} = \dfrac{e^x-e^{-x}}{e^x+e^{-x}}
$$
### Returned value
TANH returns a real [number](/features/value-types#numbers) in the range (-1,+1) that is the hyperbolic tangent of the specified hyperbolic angle.
### Error conditions
* In common with many other IronCalc functions, TANH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then TANH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then TANH returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, TANH may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The TANH function utilizes the *tanh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the TANH function.
<center><img src="/functions/images/hyperbolictangent-curve.png" width="350" alt="Graph showing tanh(x)."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=tanh).

## Links
* For more information about hyperbolic functions, visit Wikipedia's [Hyperbolic functions](https://en.wikipedia.org/wiki/Hyperbolic_functions) page.
* See also IronCalc's [SINH](/functions/math_and_trigonometry/sinh), [COSH](/functions/math_and_trigonometry/cosh) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [TANH function](https://support.microsoft.com/en-us/office/tanh-function-017222f0-a0c3-4f69-9787-b3202295dc6c) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093755) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/TANH) provide versions of the TANH function.