---
layout: doc
outline: deep
lang: en-US
---

# ATAN2 function
## Overview
ATAN2 is a function of the Math and Trigonometry category that calculates the inverse tangent (arctangent) for the specified *x* and *y* coordinates. The arctangent returns the angle defined by the x-axis and a line defined by the origin and a point with coordinates (x,y). The returned angle is expressed in radians, in the range (-$\pi$, +$\pi$].
## Usage
### Syntax
**ATAN2(<span title="Number" style="color:#1E88E5">x,y</span>) => <span title="Number" style="color:#1E88E5">atan2</span>**
### Argument descriptions
* *x* ([number](/features/value-types#numbers), required). Value of the x coordinate.
* *y* ([number](/features/value-types#numbers), required). Value of the y coordinate.
### Additional guidance
If the returned value is positive, it represents a counterclockwise angle from the x-axis, while a negative value represents a clockwise angle.  
ATAN2(x,y) is equivalent to ATAN(y/x), with the difference that the x argument in ATAN2 can be 0.
### Returned value
ATAN2 returns a number in radians in the range (-$\pi$, +$\pi$] that is the inverse tangent for the specified x and y coordinates.
### Error conditions
* In common with many other IronCalc functions, ATAN2 propagates errors that are found in its argument.
* If no argument, or arguments other than 2, are supplied, then ATAN2 returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of either the *x* or *y* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ATAN2 returns the [`#VALUE!`](/features/error-types.md#value) error.
* If both *x* and *y* are equal to 0, ATAN2 returns a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The ATAN2 function utilizes the *atan2()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=atan2).

## Links
* For more information about inverse trigonometric functions, visit Wikipedia's [Inverse trigonometric functions](https://en.wikipedia.org/wiki/Inverse_trigonometric_functions) page.
* See also IronCalc's [ATAN](/functions/math_and_trigonometry/atan), [TAN](/functions/math_and_trigonometry/tan) and [ASIN](/functions/math_and_trigonometry/asin) functions.
* Visit Microsoft Excel's [ATAN2 function](https://support.microsoft.com/en-us/office/atan2-function-51123ced-348c-416a-b2e2-833f7868569f) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093468) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ATAN2) provide versions of the ATAN2 function.

