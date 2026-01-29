---
layout: doc
outline: deep
lang: en-US
---

# ASIN function

## Overview
ASIN is a function of the Math and Trigonometry category that calculates the inverse sine (arcsine) of a number in the range [-1 to +1], returning an angle in the range [-$\pi$/2 to +$\pi$/2], expressed in radians.
## Usage
### Syntax
**ASIN(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">asin</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The number whose arcsine is to be calculated, in the range [-1 to +1]. 

### Additional guidance
None.
### Returned value
ASIN returns a number in radians in the range [-$\pi$/2 to +$\pi$/2] that is the angle whose sine is the specified number.
### Error conditions
* In common with many other IronCalc functions, ASIN propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ASIN returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ASIN returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *number* argument lies outside the range [-1 to +1], then ASIN returns the [`#NUM!`](/features/error-types.md#num) error.
* For some argument values, ASIN may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ASIN function utilizes the *asin()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ASIN function for angles $x$ in the range -1 to +1 radians.
<center><img src="/functions/images/arcsine-curve.png" width="350" alt="Graph showing sin(x) for x between -2π and +2π."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=asin).

## Links
* For more information about inverse trigonometric functions, visit Wikipedia's [Inverse trigonometric functions](https://en.wikipedia.org/wiki/Inverse_trigonometric_functions) page.
* See also IronCalc's [SIN](/functions/math_and_trigonometry/sin), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [ASIN function](https://support.microsoft.com/en-us/office/asin-function-81fb95e5-6d6f-48c4-bc45-58f955c6d347) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093464) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ASIN) provide versions of the ASIN function.
