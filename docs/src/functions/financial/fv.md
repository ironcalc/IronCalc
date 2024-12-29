---
layout: doc
outline: deep
lang: en-US
---
# FV function
::: warning
**Note:** This draft page is under construction 🚧
:::
## Overview
FV (<u>F</u>uture <u>V</u>alue) is a function of the Financial category that can be used to predict the future value of an investment or asset based on its present value.

FV can be used to calculate future value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.

If your interest rate varies between periods, use the [FVSCHEDULE](/functions/financial/fvschedule) function instead of FV.
## Usage
### Syntax
**FV(<span title="Number" style="color:#1E88E5">rate</span>, <span title="Number" style="color:#1E88E5">nper</span>, <span title="Number" style="color:#1E88E5">pmt</span>, <span title="Number" style="color:#1E88E5">pv</span>=0, <span title="Boolean" style="color:#43A047">type</span>=FALSE) => <span title="Number" style="color:#1E88E5">fv</span>**
### Argument descriptions
* *rate* ([number](/features/value-types#numbers), required). The fixed percentage interest rate or yield per period.
* *nper* ([number](/features/value-types#numbers), required). "nper" stands for <u>n</u>umber of <u>per</u>iods, in this case the number of compounding periods to be taken into account. While this will often be an integer, non-integer values are accepted and processed.
* *pmt* ([number](/features/value-types#numbers), required). "pmt" stands for <u>p</u>ay<u>m</u>en<u>t</u>, in this case the fixed amount paid or deposited each compounding period. 
* *pv* ([number](/features/value-types#numbers), [optional](/features/optional-arguments.md)). "pv" is the <u>p</u>resent <u>v</u>alue or starting amount of the asset (default 0).
* *type* ([Boolean](/features/value-types#booleans), [optional](/features/optional-arguments.md)). A logical value indicating whether the payment due dates are at the end (FALSE or 0) of the compounding periods or at the beginning (TRUE or any non-zero value). The default is FALSE when omitted.
### Additional guidance
* Make sure that the *rate* argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for *nper*.
* The *pmt* and *pv* arguments should be expressed in the same currency unit.
* To ensure a worthwhile result, one of the *pmt* and *pv* arguments should be non-zero.
* The setting of the *type* argument only affects the calculation for non-zero values of the *pmt* argument.
### Returned value
FV returns a [number](/features/value-types#numbers) representing the future value expressed in the same [currency unit](/features/units) that was used for the *pmt* and *pv* arguments.
### Error conditions
* In common with many other IronCalc functions, FV propagates errors that are found in any of its arguments.
* If too few or too many arguments are supplied, FV returns the [`#ERROR!`](/features/error-types.md#error) error.
* If the value of any of the *rate*, *nper*, *pmt* or *pv* arguments is not (or cannot be converted to) a [number](/features/value-types#numbers), then FV returns the [`#VALUE!`](/features/error-types.md#value) error.
* If the value of the *type* argument is not (or cannot be converted to) a [Boolean](/features/value-types#booleans), then FV again returns the [`#VALUE!`](/features/error-types.md#value) error.
* For some combinations of valid argument values, FV may return a [`#NUM!`](/features/error-types.md#num) error or a [`#DIV/0!`](/features/error-types.md#div-0) error.
<!--@include: ../markdown-snippets/error-type-details.txt-->
## Details
* If $\text{type} \neq 0$, $\text{fv}$ is given by the equation:
$$ \text{fv} = -\text{pv} \times (1 + \text{rate})^\text{nper} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big) \times(1+\text{rate})}{\text{rate}}$$

* If $\text{type} = 0$, $\text{fv}$ is given by the equation:
$$ \text{fv} = -\text{pv} \times (1 + \text{rate})^{\text{nper}} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big)}{\text{rate}}$$

* For any $\text{type}$, in the special case of $\text{rate} = 0$, $\text{fv}$ is given by the equation:
$$ \text{fv} = -\text{pv} - (\text{pmt} \times \text{nper}) $$
## Examples
[See some examples in IronCalc](https://app.ironcalc.com/?example=fv).

## Links
* For more information about the concept of "future value" in finance, visit Wikipedia's [Future value](https://en.wikipedia.org/wiki/Future_value) page.
* See also IronCalc's [NPER](/functions/financial/nper), [PMT](/functions/financial/pmt), [PV](/functions/financial/pv) and [RATE](/functions/financial/rate) functions.
* Visit Microsoft Excel's [FV function](https://support.microsoft.com/en-gb/office/fv-function-2eef9f44-a084-4c61-bdd8-4fe4bb1b71b3) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093224) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/FV) provide versions of the FV function.