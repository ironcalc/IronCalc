---
layout: doc
outline: deep
lang: en-US
---
# SIN function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
SIN is a function of the Math and Trigonometry category that calculates the trigonometric sine of an angle, returning a value in the range [-1, +1].
## Usage
### Syntax
**SIN(<span title="Number" style="color:#1E88E5">angle</span>) => <span title="Number" style="color:#1E88E5">sin</span>**
### Argument descriptions
* *angle* ([number](/features/value-types#numbers), required). The angle whose sine is to be calculated, expressed in radians. To convert between degrees and radians, use the relation below. Alternatively, use the [DEGREES](/functions/math_and_trigonometry/degrees) or [RADIANS](/functions/math_and_trigonometry/radians) functions.
$$
1~\:~\text{degree} = \dfrac{\pi}{180} = 0.01745329252~\text{radians}
$$

### Additional guidance
None.
### Returned value
SIN returns a unitless [number](/features/value-types#numbers) that is the trigonometric sine of the specified angle.
### Error conditions
* In common with many other IronCalc functions, SIN propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then SIN returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *angle* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then SIN returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, SIN may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The SIN function utilizes the *sin()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the SIN function for angles $x$ in the range -2$\pi$ to +2$\pi$ radians.
<center><img src="/functions/images/sine-curve.png" width="350" alt="Graph showing sin(x) for x between -2π and +2π."></center>

## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=sin).

## Links
* For more information about trigonometric sine, visit Wikipedia's [Sine and cosine](https://en.wikipedia.org/wiki/Sine_and_cosine) page.
* See also IronCalc's [ASIN](/functions/math_and_trigonometry/asin), [COS](/functions/math_and_trigonometry/cos) and [TAN](/functions/math_and_trigonometry/tan) functions.
* Visit Microsoft Excel's [SIN function](https://support.microsoft.com/en-gb/office/sin-function-cf0e3432-8b9e-483c-bc55-a76651c95602) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093447) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/SIN) provide versions of the SIN function.