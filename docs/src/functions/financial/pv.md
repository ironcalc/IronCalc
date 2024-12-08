---
layout: doc
outline: deep
lang: en-US
---

# PV function
## Overview
PV (<u>P</u>resent <u>V</u>alue) is a function of the Financial category that can be used to calculate the present value of a series of future cash flows.

PV can be used to calculate present value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.
## Usage
### Syntax
**PV(rate, nper, pmt, fv, type)**
### Argument descriptions
* *rate*. The fixed percentage interest rate or yield per period. PV reports a #NUM! error if *rate* is set to -1.
* *nper*. The number of compounding periods to be taken into account. While this will often be an integer, non-integer values are accepted and processed.
* *pmt*. The fixed amount paid or deposited each compounding period.
* *fv* (optional). The future value at the end of the final compounding period (default 0).
* *type* (optional). A logical value indicating whether the payment due dates are at the end (0) of the compounding periods or at the beginning (any non-zero value). The default is 0 when omitted.
### Additional guidance
* Make sure that the *rate* argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for *nper*.
* The *pmt* and *fv* arguments should be expressed in the same currency unit. The value returned is expressed in the same currency unit.
* To ensure a worthwhile result, one of the *pmt* and *fv* arguments should be non-zero.
* The setting of the *type* argument only affects the calculation for non-zero values of the *pmt* argument.

<!--@include: ../markdown-snippets/error-type-details.md-->

## Details
* If *rate* = 0, PV solves the equation:
$$
PV = -fv - (pmt \times nper)
$$

* If *rate* <> 0 and *rate* <> -1 and *type* = 0, PV solves the equation:
$$ PV = - \Biggl(\dfrac{(fv \times rate) + \bigl(pmt\times \bigl({(1+rate)^{nper}-1\bigr)\bigr)}}{rate \times (1+rate)^{nper}}\Biggl)
$$
* If *rate* <> 0 and *rate* <> -1 and *type* <> 0, PV solves the equation:
$$ PV = - \Biggl(\dfrac{(fv \times rate) + \bigl(pmt \times (1+rate)\times \bigl({(1+rate)^{nper}-1\bigr)\bigr)}}{rate \times (1+rate)^{nper}}\Biggl)
$$
## Examples
[See this example in IronCalc](https://app.ironcalc.com/?example=PV).

## Links
* For more information about the concept of "present value" in finance, visit Wikipedia's [Present value](https://en.wikipedia.org/wiki/Present_value) page.
* See also IronCalc's [FV](./FV), [NPER](./NPER), [PMT](./PMT) and [RATE](./RATE) functions.

* Visit Microsoft Excel's [PV function](https://support.microsoft.com/en-gb/office/pv-function-23879d31-0e02-4321-be01-da16e8168cbd) page.

* Both [Google Sheets](https://support.google.com/docs/answer/3093243) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/PV) provide versions of the PV function.