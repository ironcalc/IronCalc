---
layout: doc
outline: deep
lang: en-US
---

# ATANH function
## Overview
ATANH is a function of the Math and Trigonometry category that calculates the inverse hyperbolic tangent (hyperbolic arctangent) of a number in the range (-1, +1), returning the hyperbolic angle expressed in radians.
## Usage
### Syntax
**ATANH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">atanh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The value whose inverse hyperbolic tangent is to be calculated, in the range (-1,+1).
### Additional guidance
The hyperbolic arctangent function is defined as:
$$
\operatorname{atanh}(x) = \tfrac{1}{2}\,\ln\!\left(\dfrac{1+x}{1-x}\right),\quad |x| < 1
$$
### Returned value
ATANH returns a real [number](/features/value-types#numbers) in the range (-∞, +∞) that is the hyperbolic arctangent of the specified value, expressed in radians.
### Error conditions
* In common with many other IronCalc functions, ATANH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ATANH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ATANH returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *number* argument lies outside the domain (-1, +1), then ATANH returns the [`#NUM!`](/features/error-types.md#num) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ATANH function utilizes the *atanh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ATANH function.
<center><img src="/functions/images/hyperbolicarctangent-curve.png" width="350" alt="Graph showing atanh(x)."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=atanh).

## Links
* For more information about inverse hyperbolic functions, visit Wikipedia's [Inverse hyperbolic functions](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) page.
* See also IronCalc's [ASINH](/functions/math_and_trigonometry/asinh), [ACOSH](/functions/math_and_trigonometry/acosh) and [TANH](/functions/math_and_trigonometry/tanh) functions.
* Visit Microsoft Excel's [ATANH function](https://support.microsoft.com/de-de/office/atanh-function-453534d1-76a5-4f17-8c04-c3f2feee0dd5) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093397) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ATANH) provide versions of the ATANH function.