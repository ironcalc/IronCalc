---
layout: doc
outline: deep
lang: en-US
---
# ERF.PRECISE function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
ERF.PRECISE (<u>ER</u>ror <u>F</u>unction) is a function of the Engineering category that calculates a value for the _error function_. Also known as the _Gauss error function_, the error function represents the probability of a random variable falling within a certain range, given that it follows a specified normal distribution.

ERF.PRECISE is provided for compatibility with other spreadsheets. For all real values of $x$, $\text{ERF.PRECISE}(x)=\text{ERF}(x)$.
## Usage
### Syntax
**ERF.PRECISE(<span title="Number" style="color:#1E88E5">X</span>) => <span title="Number" style="color:#1E88E5">erf.precise</span>**
### Argument descriptions
* *X* ([number](/features/value-types#numbers), required). Integration limit. ERF.PRECISE integrates over the range [0, _X_].
### Additional guidance
None.
### Returned value
ERF.PRECISE returns a [number](/features/value-types#numbers) that is the error function probability for the specified argument. The returned value has a magnitude in the range [0, 1] but may be either positive (integration limit > 0) or negative (integration limit < 0).
### Error conditions
* In common with many other IronCalc functions, ERF.PRECISE propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ERF.PRECISE returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of the argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ERF.PRECISE returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, ERF.PRECISE may return the [`#DIV/0!`](/features/error-types.md#div-0) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The error function arises in many scientific, engineering, and mathematical fields and is commonly defined by the following equation (applicable for any real number $x$):
$$
\text{erf}(x) = \frac{2}{\sqrt{\pi} }\: \int_{0}^{x} e^{-t^2}\:dt
$$
* The figure below illustrates the output of the ERF.PRECISE function for values of $x$ in the range -3 to +3.
<center><img src="/functions/images/error-function-curve.png" width="350" alt="Graph showing erf(x) for x between -3 and +3."></center>

* This figure illustrates some of the key characteristics of the error function:

  * $\text{erf}(0) = 0$
  * $\text{erf}(x) = -\text{erf}(x)$
  * As $x \rightarrow \infty$, $\text{erf}(x) \rightarrow 1$
  * As $x \rightarrow -\infty$, $\text{erf}(x) \rightarrow -1$

* The error function is a [transcendental](https://en.wikipedia.org/wiki/Transcendental_function), non-algebraic mathematical function. IronCalc implements the ERF.PRECISE function by numerical approximation using a power series.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=erf-precise).

## Links
* See also IronCalc's [ERF](/functions/engineering/erf.md), [ERFC](/functions/engineering/erfc.md) and [ERFC.PRECISE](/functions/engineering/erfc-precise.md) functions.
* Visit Microsoft Excel's [ERF.PRECISE function](https://support.microsoft.com/en-gb/office/erf-precise-function-9a349593-705c-4278-9a98-e4122831a8e0) page.
* Both [Google Sheets](https://support.google.com/docs/answer/9386210) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ERF.PRECISE) provide versions of the ERF.PROCESS function.