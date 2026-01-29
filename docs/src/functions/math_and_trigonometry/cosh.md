---
layout: doc
outline: deep
lang: en-US
---
# COSH function
## Overview
COSH is a function of the Math and Trigonometry category that calculates the hyperbolic cosine of a number.
## Usage
### Syntax
**COSH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">cosh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The hyperbolic angle whose hyperbolic cosine is to be calculated, expressed in radians.
### Additional guidance
The formula for the hyperbolic cosine is:
$$
\text{cosh(x)} = \dfrac{e^x+e^{-x}}{2}
$$
### Returned value
COSH returns a real [number](/features/value-types#numbers) that is the hyperbolic cosine of the specified hyperbolic angle.
### Error conditions
* In common with many other IronCalc functions, COSH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then COSH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then COSH returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, COSH may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The COSH function utilizes the *cosh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the COSH function.
<center><img src="/functions/images/hyperboliccosine-curve.png" width="350" alt="Graph showing cosh(x)."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=cosh).

## Links
* For more information about hyperbolic functions, visit Wikipedia's [Hyperbolic functions](https://en.wikipedia.org/wiki/Hyperbolic_functions) page.
* See also IronCalc's [SINH](/functions/math_and_trigonometry/sinh), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [COSH function](https://support.microsoft.com/en-us/office/cosh-function-e460d426-c471-43e8-9540-a57ff3b70555) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093477) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/COSH) provide versions of the COSH function.