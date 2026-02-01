---
layout: doc
outline: deep
lang: en-US
---
# ACOSH function
## Overview
ACOSH is a function of the Math and Trigonometry category that calculates the inverse hyperbolic cosine (hyperbolic arccosine) of a number, returning a non-negative value in the range [0, +∞).
## Usage
### Syntax
**ACOSH(<span title="Number" style="color:#1E88E5">number</span>) => <span title="Number" style="color:#1E88E5">acosh</span>**
### Argument descriptions
* *number* ([number](/features/value-types#numbers), required). The value whose hyperbolic arccosine is to be calculated. The value must be greater than or equal to 1.

### Additional guidance
The hyperbolic arccosine function is defined as:

$$
\operatorname{acosh}(x) = \ln(x + \sqrt{x^2 - 1})
$$

### Returned value
ACOSH returns a [number](/features/value-types#numbers) in the range [0, +∞) that is the hyperbolic arccosine of the specified value, expressed in radians.
### Error conditions
* In common with many other IronCalc functions, ACOSH propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ACOSH returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *number* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ACOSH returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *number* argument is less than 1, then ACOSH returns the [`#NUM!`](/features/error-types.md#num) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ACOSH function utilizes the *acosh()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the ACOSH function for values $x \geq 1$ in the range [0, +∞).
<center><img src="/functions/images/hyperbolicarccosine-curve.png" width="350" alt="Graph showing acosh(x) for x ≥ 1."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=acosh).

## Links
* For more information about inverse hyperbolic functions, visit Wikipedia's [Inverse hyperbolic functions](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) page.
* See also IronCalc's [COSH](/functions/math_and_trigonometry/cosh), [ASINH](/functions/math_and_trigonometry/asinh) and [ATANH](/functions/math_and_trigonometry/atanh) functions.
* Visit Microsoft Excel's [ACOSH function](https://support.microsoft.com/en-us/office/acosh-function-e3992cc1-103f-4e72-9f04-624b9ef5ebfe) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093391) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ACOSH) provide versions of the ACOSH function.