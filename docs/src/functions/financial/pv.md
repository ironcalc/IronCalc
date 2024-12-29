---
layout: doc
outline: deep
lang: en-US
---
# PV function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
PV (<u>P</u>resent <u>V</u>alue) is a function of the Financial category that can be used to calculate the present value of a series of future cash flows.

PV can be used to calculate present value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.
## Usage
### Syntax
**PV(<span title="Number" style="color:#1E88E5">rate</span>, <span title="Number" style="color:#1E88E5">nper</span>, <span title="Number" style="color:#1E88E5">pmt</span>, <span title="Number" style="color:#1E88E5">fv</span>=0, <span title="Boolean" style="color:#43A047">type</span>=FALSE) => <span title="Number" style="color:#1E88E5">pv</span>**
### Argument descriptions
* *rate* ([number](/features/value-types#numbers), required). The fixed percentage interest rate or yield per period.
* *nper* ([number](/features/value-types#numbers), required). "nper" stands for <u>n</u>umber of <u>per</u>iods, in this case the number of compounding periods to be taken into account. While this will often be an integer, non-integer values are accepted and processed.
* *pmt* ([number](/features/value-types#numbers), required). "pmt" stands for <u>p</u>ay<u>m</u>en<u>t</u>, in this case the fixed amount paid or deposited each compounding period. 
* *fv* ([number](/features/value-types#numbers), [optional](/features/optional-arguments.md)). "fv" is the <u>f</u>uture <u>v</u>alue at the end of the final compounding period (default 0).
* *type* ([Boolean](/features/value-types#booleans), [optional](/features/optional-arguments.md)). A logical value indicating whether the payment due dates are at the end (FALSE or 0) of the compounding periods or at the beginning (TRUE or any non-zero value). The default is FALSE when omitted.
### Additional guidance
* Make sure that the *rate* argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for *nper*.
* The *pmt* and *fv* arguments should be expressed in the same currency unit.
* To ensure a worthwhile result, one of the *pmt* and *fv* arguments should be non-zero.
* The setting of the *type* argument only affects the calculation for non-zero values of the *pmt* argument.
### Returned value
PV returns a [number](/features/value-types#numbers) representing the present value expressed in the same [currency unit](/features/units) that was used for the *pmt* and *fv* arguments.
### Error conditions
* In common with many other IronCalc functions, PV propagates errors that are found in any of its arguments.
* If too few or too many arguments are supplied, PV returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any of the *rate*, *nper*, *pmt* or *fv* arguments is not (or cannot be converted to) a [number](/features/value-types#numbers), then PV returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *type* argument is not (or cannot be converted to) a [Boolean](/features/value-types#booleans), then PV again returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some combinations of valid argument values, PV may return a [`#NUM!`](/features/error-types.md#num) error or a [`#DIV/0!`](/features/error-types.md#div-0) error.
In paticular, PV always returns a [`#DIV/0!`](/features/error-types.md#div-0) error if the value of the *rate* argument is set to -1.

<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* If $\text{type} \neq 0$, $\text{pv}$ is given by the equation:
$$ \text{pv} = - \Biggl(\dfrac{(\text{fv} \times \text{rate}) + \bigl(\text{pmt} \times (1+\text{rate})\times \bigl({(1+\text{rate})^{\text{nper}}-1\bigr)\bigr)}}{\text{rate} \times (1+\text{rate})^{\text{nper}}}\Biggl)
$$

* If $\text{type} = 0$, $\text{pv}$ is given by the equation:
$$ \text{pv} = - \Biggl(\dfrac{(\text{fv} \times \text{rate}) + \bigl(\text{pmt}\times \bigl({(1+\text{rate})^{\text{nper}}-1\bigr)\bigr)}}{\text{rate} \times (1+\text{rate})^\text{{nper}}}\Biggl)
$$

* For any $\text{type}$, in the special case of $\text{rate} = 0$, $\text{pv}$ is given by the equation:
$$
\text{pv} = -\text{fv} - (\text{pmt} \times \text{nper})
$$
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=pv).

## Links
* For more information about the concept of "present value" in finance, visit Wikipedia's [Present value](https://en.wikipedia.org/wiki/present_value) page.
* See also IronCalc's [FV](/functions/financial/fv), [NPER](/functions/financial/nper), [PMT](/functions/financial/pmt)  and [RATE](/functions/financial/rate) functions.
* Visit Microsoft Excel's [PV function](https://support.microsoft.com/en-gb/office/pv-function-23879d31-0e02-4321-be01-da16e8168cbd) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093243) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/PV) provide versions of the PV function.