---
layout: doc
outline: deep
lang: en-US
---
# ERFC function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
ERFC (<u>ER</u>ror <u>F</u>unction <u>C</u>omplementary) is a function of the Engineering category that calculates a value for the _complementary error function_, defined by $\text{erfc}(x) = 1 - \text{erf}(x)$. Also known as the _complementary Gauss error function_, the complementary error function represents the probability of a random variable falling outside a certain range, given that it follows a specified normal distribution.
## Usage
### Syntax
**ERFC(<span title="Number" style="color:#1E88E5">X</span>) => <span title="Number" style="color:#1E88E5">erfc</span>**
### Argument descriptions
* *X* ([number](/features/value-types#numbers), required). The lower integration limit to be used to calculate the complementary error function. ERFC integrates over the range [X, $\infty$).
### Additional guidance
None.
### Returned value
ERFC returns a [number](/features/value-types#numbers) that is the complementary error function probability for the specified argument. The returned value lies in range [0, 2].
### Error conditions
* In common with many other IronCalc functions, ERFC propagates errors that are found in its argument.
* If no argument, or more than one argument, is supplied, then ERFC returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any argument is not (or cannot be converted to) a [number](/features/value-types#numbers), then ERFC returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some argument values, ERFC may return the [`#DIV/0!`](/features/error-types.md#div-0) error.

<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* The complementary error function arises in many scientific, engineering, and mathematical fields and is commonly defined by the following equation (applicable for any real number $x$):
$$
\text{erfc}(x) = \frac{2}{\sqrt{\pi} }\: \int_{x}^{\infty} e^{-t^2}\:dt
$$
* The figure below illustrates the output of the ERFC function for values of $x$ in the range -3 to +3.
<center><img src="/functions/images/complementary-error-function-curve.png" width="350" alt="Graph showing erfc(x) for x between -3 and +3."></center>

* This figure illustrates some of the key characteristics of the complementary error function:

  * $\text{erfc}(0) = 1$
  * $\text{erfc}(-x) = 2-\text{erfc}(x)$
  * As $x \rightarrow \infty$, $\text{erfc}(x) \rightarrow 0$
  * As $x \rightarrow -\infty$, $\text{erfc}(x) \rightarrow 2$

* The complementary error function is a [transcendental](https://en.wikipedia.org/wiki/Transcendental_function), non-algebraic mathematical function. IronCalc implements the ERFC function by numerical approximation using a power series.
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=erfc).

## Links
* See also IronCalc's [ERF](/functions/engineering/erf.md), [ERF.PRECISE](/functions/engineering/erf-precise.md) and [ERFC.PRECISE](/functions/engineering/erfc-precise.md) functions.
* Visit Microsoft Excel's [ERFC function](https://support.microsoft.com/en-gb/office/erfc-function-736e0318-70ba-4e8b-8d08-461fe68b71b3) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093407) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/ERFC) provide versions of the ERFC function.