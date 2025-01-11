---
layout: doc
outline: deep
lang: en-US
---
# ERF function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
ERF (<u>ER</u>ror <u>F</u>unction) is a function of the Engineering category that calculates a value for the _error function_. Also known as the _Gauss error function_, the error function represents the probability of a random variable falling within a certain range, given that it follows a specified normal distribution.
## Usage
### Syntax
**ERF(<span title="Number" style="color:#1E88E5">X</span>, <span title="Number" style="color:#1E88E5">Y</span>) => <span title="Number" style="color:#1E88E5">erf</span>**
### Argument descriptions
* *X* ([number](/features/value-types#numbers), required). Integration limit. If no value is supplied for the _Y_ argument, ERF integrates over the range [0, _X_].
* *Y* ([number](/features/value-types#numbers), [optional](/features/optional-arguments)). Upper integration limit. When a value is supplied for this argument, ERF integrates over the range [_X_, _Y_].
### Additional guidance
None.
### Returned value
ERF returns a [number](/features/value-types#numbers)  that is the error function probability for the specified arguments. The returned value has a magnitude in the range [0, 1] but may be either positive (upper integration limit > lower integration limit) or negative (upper integration limit < lower integration limit).
### Error conditions
* In common with many other IronCalc functions, ERF propagates errors that are found in its arguments.
* If no argument, or more than two arguments, are supplied, then ERF returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ERF returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, ERF may return the [`#DIV/0!`](/features/error-types.md#div-0) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The error function arises in many scientific, engineering, and mathematical fields and is commonly defined by the following equation (applicable for any real number $x$):
$$
\text{erf}(x) = \frac{2}{\sqrt{\pi} }\: \int_{0}^{x} e^{-t^2}\:dt
$$
* The figure below illustrates the output of the ERF function for values of $x$ in the range -3 to +3.
<center><img src="/functions/images/error-function-curve.png" width="350" alt="Graph showing erf(x) for x between -3 and +3."></center>

* This figure illustrates some of the key characteristics of the error function:

  * $\text{erf}(0) = 0$
  * $\text{erf}(x) = -\text{erf}(x)$
  * As $x \rightarrow \infty$, $\text{erf}(x) \rightarrow 1$
  * As $x \rightarrow -\infty$, $\text{erf}(x) \rightarrow -1$

* The error function is a [transcendental](https://en.wikipedia.org/wiki/Transcendental_function), non-algebraic mathematical function. IronCalc implements the ERF function by numerical approximation using a power series.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=erf).

## Links
* See also IronCalc's [ERFC](/functions/engineering/erfc.md), [ERF.PRECISE](/functions/engineering/erf-precise.md) and [ERFC.PRECISE](/functions/engineering/erfc-precise.md) functions.
* Visit Microsoft Excel's [ERF function](https://support.microsoft.com/en-gb/office/erf-function-c53c7e7b-5482-4b6c-883e-56df3c9af349) page.
* Both [Google Sheets](https://support.google.com/docs/answer/9116267) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ERF) provide versions of the ERF function.