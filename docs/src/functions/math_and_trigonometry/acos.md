---
layout: doc
outline: deep
lang: en-US
---

# ACOS function
## Overview
ACOS is a function of the Math and Trigonometry category that calculates the inverse cosine (arccosine) of a number in the range [-1 to 1], returning an angle in the range [0 to $\pi$], expressed in radians.
## Usage
### Syntax
**ACOS(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">acos</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The number whose arccosine is to be calculated, in the range [-1 to 1]. 
### Additional guidance
None.
### Returned value
ACOS returns a number in radians in the range [0 to $\pi$] that is the angle whose cosine is the specified number.
### Error conditions
* In common with many other IronCalc functions, ACOS propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ACOS returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ACOS returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *number* argument lies outside the range [-1 to 1], then ACOS returns the [`#NUM!`](/features/error-types.md#num) error.
* For some argument values, ACOS may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ACOS function utilizes the *acos()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ACOS function for angles $x$ in the range -1 to +1.
<center><img src="/functions/images/arccosine-curve.png" width="350" alt="Graph showing acos(x) for x between -1 and +1."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=acos).

## Links
* For more information about inverse trigonometric functions, visit Wikipedia's [Inverse trigonometric functions](https://en.wikipedia.org/wiki/Inverse_trigonometric_functions) page.
* See also IronCalc's [SIN](/functions/math_and_trigonometry/sin), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [ACOS function](https://support.microsoft.com/en-us/office/acos-function-cb73173f-d089-4582-afa1-76e5524b5d5b) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093461) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ACOS) provide versions of the ACOS function.
