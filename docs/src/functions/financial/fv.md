---
layout: doc
outline: deep
lang: en-US
---

# FV function
## Overview
FV (<u>F</u>uture <u>V</u>alue) is a function of the Financial category that can be used to predict the future value of an investment or asset based on its present value.

FV can be used to calculate future value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.

If your interest rate varies between periods, use the [FVSCHEDULE](./FVSCHEDULE) function instead of FV.
## Usage
### Syntax
**FV(rate, nper, pmt, pv, type)**
### Argument descriptions
* *rate*. The fixed percentage interest rate or yield per period.
* *nper*. The number of compounding periods to be taken into account. While this will often be an integer, non-integer values are accepted and processed.
* *pmt*. The fixed amount paid or deposited each compounding period.
* *pv* (optional). The present value or starting amount of the asset (default 0).
* *type* (optional). A logical value indicating whether the payment due dates are at the end (0) of the compounding periods or at the beginning (any non-zero value). The default is 0 when omitted.

### Additional guidance
* Make sure that the *rate* argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for *nper*.
* The *pmt* and *pv* arguments should be expressed in the same currency unit. The value returned is expressed in the same currency unit.
* To ensure a worthwhile result, one of the *pmt* and *pv* arguments should be non-zero.
* The setting of the *type* argument only affects the calculation for non-zero values of the *pmt* argument.
* For information about the different types of errors that you may encounter when using IronCalc functions, visit our [Error Types](/features/error-types) page.
## Details
* If *rate* = 0, FV solves the equation:
$$
FV = -pv - (pmt \times nper)
$$

* If *rate* <> 0 and *type* = 0, FV solves the equation:
$$ FV = -pv \times (1 + rate)^{nper} - \dfrac{pmt\times\big({(1+rate)^{nper}-1}\big)}{rate}
$$
* If *rate* <> 0 and *type* <> 0, FV solves the equation:
$$ FV = -pv \times (1 + rate)^{nper} - \dfrac{pmt\times\big({(1+rate)^{nper}-1}\big) \times(1+rate)}{rate}
$$
## Examples
[See this example in IronCalc](https://app.ironcalc.com/?model=h30aj-o2HyK-1jUR8).

## Links
* For more information about the concept of "future value" in finance, visit Wikipedia's [Future value](https://en.wikipedia.org/wiki/Future_value) page.
* See also IronCalc's [NPER](./NPER), [PMT](./PMT), [PV](./PV) and [RATE](./RATE) functions.
* Visit Microsoft Excel's [FV function](https://support.microsoft.com/en-gb/office/fv-function-2eef9f44-a084-4c61-bdd8-4fe4bb1b71b3) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093224) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/FV) provide versions of the FV function.