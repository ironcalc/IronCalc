---
layout: doc
outline: deep
lang: en-US
---

# RADIANS function

## Overview
RADIANS is a function of the Math and Trigonometry category that converts an angle measured in degrees to an equivalent angle measured in radians.

## Usage
### Syntax
**RADIANS(<span title="Number" style="color:#1E88E5">angle</span>) => <span title="Number" style="color:#1E88E5">radians</span>**

### Argument descriptions
* *angle* ([number](/features/value-types#numbers), required). The angle in degrees that is to be converted to radians.

### Additional guidance
The conversion from degrees to radians is based on the relationship:
$$
1~\:~\text{degree} = \dfrac{\pi}{180}~\text{radians} \approx 0.01745329252~\text{radians}
$$

### Returned value
RADIANS returns a [number](/features/value-types#numbers) that represents the value of the given angle expressed in radians.

### Error conditions
* In common with many other IronCalc functions, RADIANS propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then RADIANS returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *angle* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then RADIANS returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=radians).
-->
## Links
* For more information about angle conversions, visit Wikipedia's [Radian](https://en.wikipedia.org/wiki/Radian) page.
* See also IronCalc's [DEGREES](/functions/math_and_trigonometry/degrees) function for converting radians to degrees.
* Visit Microsoft Excel's [RADIANS function](https://support.microsoft.com/en-us/office/radians-function-907f0ede-ef2e-4f7b-911a-015e2f8ab878) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093481) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/RADIANS) provide versions of the RADIANS function.