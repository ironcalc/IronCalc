---
layout: doc
outline: deep
lang: en-US
---

# DEGREES function

## Overview
DEGREES is a function of the Math and Trigonometry category that converts an angle measured in radians to an equivalent angle measured in degrees.

## Usage
### Syntax
**DEGREES(<span title="Number" style="color:#1E88E5">angle</span>) => <span title="Number" style="color:#1E88E5">degrees</span>**

### Argument descriptions
* *angle* ([number](/features/value-types#numbers), required). The angle in radians that is to be converted to degrees.

### Additional guidance
The conversion from radians to degrees is based on the relationship:
$$
1~\:~\text{radian} = \dfrac{180}{\pi}~\text{degrees} \approx 57.29577951~\text{degrees}
$$

### Returned value
DEGREES returns a [number](/features/value-types#numbers) that represents the value of the given angle expressed in degrees.

### Error conditions
* In common with many other IronCalc functions, DEGREES propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then DEGREES returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *angle* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then DEGREES returns the [`#VALUE!`](/features/error-types.md#value) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->

<!--
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=degrees).
-->

## Links
* For more information about angle conversions, visit Wikipedia's [Degree (angle)](https://en.wikipedia.org/wiki/Degree_(angle)) page.
* See also IronCalc's [RADIANS](/functions/math_and_trigonometry/radians) function for converting degrees to radians.
* Visit Microsoft Excel's [DEGREES function](https://support.microsoft.com/en-us/office/degrees-function-4d6ec4db-e694-4b94-ace0-1cc3f61f9ba1) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093481) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/DEGREES) provide versions of the DEGREES function.