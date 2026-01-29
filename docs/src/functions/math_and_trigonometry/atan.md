---
layout: doc
outline: deep
lang: en-US
---

# ATAN function
## Overview
ATAN is a function of the Math and Trigonometry category that calculates the inverse tangent (arctangent) of a number, returning an angle in the range [-$\pi$/2 to +$\pi$/2], expressed in radians.
## Usage
### Syntax
**ATAN (<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">atan</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The number whose arctangent is to be calculated, in the range [-$\infty$, +$\infty$]. 
### Additional guidance
None.
### Returned value
ATAN returns a number in radians in the range [-$\pi$/2 to +$\pi$/2] that is the angle whose tangent is the specified number.
### Error conditions
* In common with many other IronCalc functions, ATAN propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ATAN returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ATAN returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, ATAN may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ATAN function utilizes the *atan()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ATAN function for angles $x$ in the range [-$\infty$, +$\infty$].
<center><img src="/functions/images/arctangent-curve.png" width="350" alt="Graph showing atan(x) for x between [-$\infty$, +$\infty$]."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=atan).

## Links
* For more information about inverse trigonometric functions, visit Wikipedia's [Inverse trigonometric functions](https://en.wikipedia.org/wiki/Inverse_trigonometric_functions) page.
* See also IronCalc's [SIN](/functions/math_and_trigonometry/sin), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [ATAN function](https://support.microsoft.com/en-us/office/atan-function-50746fa8-630a-406b-81d0-4a2aed395543) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093395) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ATAN) provide versions of the ATAN function.
