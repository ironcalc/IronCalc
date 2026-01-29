---
layout: doc
outline: deep
lang: en-US
---
# TAN function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
TAN is a function of the Math and Trigonometry category that calculates the trigonometric tangent of an angle, returning a value in the range (-$\infty$, +$\infty$).
## Usage
### Syntax
**TAN(<span title="Number" style="color:#1E88E5">angle</span>) => <span title="Number" style="color:#1E88E5">tan</span>**
### Argument descriptions
* *angle* ([number](/features/value-types#numbers), required). The angle whose tangent is to be calculated, expressed in radians. To convert between degrees and radians, use the relation below. Alternatively, use the [DEGREES](/functions/math_and_trigonometry/degrees) or [RADIANS](/functions/math_and_trigonometry/radians) functions.
$$
1~\:~\text{degree} = \dfrac{\pi}{180} = 0.01745329252~\text{radians}
$$

### Additional guidance
None.
### Returned value
TAN returns a unitless [number](/features/value-types#numbers) that is the trigonometric tangent of the specified angle.
### Error conditions
* In common with many other IronCalc functions, TAN propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then TAN returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the *angle* argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then TAN returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, TAN may return a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The TAN function utilizes the *tan()* method provided by the [Rust Standard Library](https://doc.rust-lang.org/std/).
* The figure below illustrates the output of the TAN function for angles $x$ in the range -2$π$ to +2$π$.
<center><img src="/functions/images/tangent-curve.png" width="350" alt="Graph showing tan(x) for x between -2π and +2π."></center>

* Theoretically, $\text{tan}(x)$ is undefined for any critical $x$ that satisfies $x = \frac{\pi}{2} + k\pi$ (where $k$ is any integer). However, an exact representation of the mathematical constant $\pi$ requires infinite precision, which cannot be achieved with the floating-point representation available. Hence, TAN will return very large or very small values close to critical $x$ values.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=tan).

## Links
* For more information about trigonometric tangent, visit Wikipedia's [Trigonometric functions](https://en.wikipedia.org/wiki/Trigonometric_functions) page.
* See also IronCalc's [ATAN](/functions/math_and_trigonometry/atan), [COS](/functions/math_and_trigonometry/cos) and [SIN](/functions/math_and_trigonometry/sin) functions.
* Visit Microsoft Excel's [TAN function](https://support.microsoft.com/en-gb/office/tan-function-08851a40-179f-4052-b789-d7f699447401) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093586) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/TAN) provide versions of the TAN function.