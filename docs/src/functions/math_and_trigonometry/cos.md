---
layout: doc
outline: deep
lang: en-US
---
# COS function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
COS is a function of the Math and Trigonometry category that calculates the trigonometric cosine of an angle, returning a value in the range [-1, +1].
## Usage
### Syntax
**COS(<span title="Number" style="color:#1E88E5">angle</span>) => <span title="Number" style="color:#1E88E5">cos</span>**
### Argument descriptions
* *angle* ([number](/features/value-types#numbers), required). The angle whose cosine is to be calculated, expressed in radians. To convert between degrees and radians, use the relation below. Alternatively, use the [DEGREES](/functions/math_and_trigonometry/degrees) or [RADIANS](/functions/math_and_trigonometry/radians) functions.
$$
1~\:~\text{degree} = \dfrac{\pi}{180} = 0.01745329252~\text{radians}
$$

### Additional guidance
None.
### Returned value
COS returns a unitless [number](/features/value-types#numbers) that is the trigonometric cosine of the specified angle.
### Error conditions
* In common with many other IronCalc functions, COS propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then COS returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *angle* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then COS returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, COS may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The COS function utilizes the *cos()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the COS function for angles $x$ in the range -2$\pi$ to +2$\pi$ radians.
<center><img src="/functions/images/cosine-curve.png" width="350" alt="Graph showing cos(x) for x between -2π and +2π radians."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=cos).

## Links
* For more information about trigonometric cosine, visit Wikipedia's [Sine and cosine](https://en.wikipedia.org/wiki/Sine_and_cosine) page.
* See also IronCalc's [ACOS](/functions/math_and_trigonometry/acos), [SIN](/functions/math_and_trigonometry/sin) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [COS function](https://support.microsoft.com/en-gb/office/cos-function-0fb808a5-95d6-4553-8148-22aebdce5f05) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093476) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/COS) provide versions of the COS function.