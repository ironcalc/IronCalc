---
layout: doc
outline: deep
lang: en-US
---

# FV

## Overview

FV (<u>F</u>uture <u>V</u>alue) is a function in the Financial category that can be used to predict the future value of an investment or asset based on its present value.

FV can be used to calculate future value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.

If your interest rate varies between periods, use the **FVSCHEDULE** function instead of FV.

## Parameters

**FV(rate, nper, pmt, pv, period_start)**

- _rate_. The fixed percentage interest rate or yield per period.
- _nper_. The number of compounding periods to be taken into account. While this will often be an integer, non-integer values are also accepted.
- _pmt_ (optional). The fixed amount paid or deposited each compounding period (default 0).
- _pv_ (optional). The present value or starting amount of the asset (default 0).
- _period_start_ (optional). A logical value indicating whether the payment due dates are at the end (0) of the compounding periods or at the beginning (1) (default 0). FV treats any non-zero value as it would the value 1.

### Additional notes

- FV may generate #ERROR!, #VALUE! or #DIV/0! errors. For more details see our [Error Types page](/features/error-types.md).
- Make sure that the _rate_ argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for _nper_.
- The _pmt_ and _pv_ arguments should be expressed in the same currency unit. FV returns a value in the same currency unit.
- To ensure a worthwhile result, one of the _pmt_ and _pv_ arguments should be set to a non-zero value.
- The setting of the _period_start_ argument only affects the calculation for non-zero values of the _pmt_ argument.

## Details

- If _rate_ = 0, FV solves the equation:

  $$
  FV = -pv - (pmt \times nper)
  $$

- If _rate_ <> 0 and _period_start_ = 0, FV solves the equation:
  $$
  FV = -pv \times (1 + rate)^{nper} - \dfrac{pmt\times\big({(1+rate)^{nper}-1}\big)}{rate}
  $$
- If _rate_ <> 0 and _period_start_ <> 0, FV solves the equation:
  $$
  FV = -pv \times (1 + rate)^{nper} - \dfrac{pmt\times\big({(1+rate)^{nper}-1}\big) \times(1+rate)}{rate}
  $$

## Examples

[See this example in IronCalc](https://app.ironcalc.com/?model=h30aj-o2HyK-1jUR8)

## Links

- For more information about the concept of "future value" in finance, visit Wikipedia's [Future value](https://en.wikipedia.org/wiki/Future_value) page.

- Visit Microsoft Excel's [FV function](https://support.microsoft.com/en-gb/office/fv-function-2eef9f44-a084-4c61-bdd8-4fe4bb1b71b3) page.
