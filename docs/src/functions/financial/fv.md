---
layout: doc
outline: deep
lang: en-US
---

# FV function
## Overview
FV (<u>F</u>uture <u>V</u>alue) is a function of the Financial category that can be used to predict the future value of an investment or asset based on its present value.

FV can be used to calculate future value over a specified number of compounding periods. A fixed interest rate or yield is assumed over all periods, and a fixed payment or deposit can be applied at the start or end of every period.

If your interest rate varies between periods, use the [FVSCHEDULE](/functions/financial/fvschedule) function instead of FV.
## Usage
### Syntax
**FV(<span title="Number" style="color:#1E88E5">rate</span>, <span title="Number" style="color:#1E88E5">nper</span>, <span title="Number" style="color:#1E88E5">pmt</span>, <span title="Number" style="color:#1E88E5">pv</span>=0, <span title="Boolean" style="color:#43A047">type</span>=false) => <span title="Number" style="color:#1E88E5">fv</span>**
### Argument descriptions
* *rate*. ([number](/features/value-types)) The fixed percentage interest rate or yield per period.
* *nper*. ([number](/features/value-types)) The number of compounding periods to be taken into account. While this will often be an integer, non-integer values are accepted and processed.
It stands for <u>n</u>umber of <u>per</u>iods.
* *pmt*. ([number](/features/value-types)) The fixed amount paid or deposited each compounding period. Short for <u>p</u>ay<u>m</u>en<u>t</u>
* *pv* ([number](/features/value-types), optional). The present value or starting amount of the asset (default 0). Short for <u>p</u>resent <u>v</u>alue.
* *type* ([boolean](/features/value-types), optional). A logical value indicating whether the payment due dates are at the end (0) of the compounding periods or at the beginning (any non-zero value). The default is 0 when omitted.

### Returned value

The retruned value is a [number](/features/value-types) with [currency units](/features/units)

### Errors

* If any of the arguments is an error returns the error
* If any of the argumets is not a number (or cannot be converted to a number) it returns `#VALUE!`
* Some ranges of the parameters produce `#NUM!` error. For instnace `=FV(-3,1/2,1)`.
* Some ranges of the parameters produce the `#DIV/0!` error. For instance `=FV(-1, -1, 1)`


### Additional guidance
* Make sure that the *rate* argument specifies the interest rate or yield applicable to the compounding period, based on the value chosen for *nper*.
* The *pmt* and *pv* arguments should be expressed in the same currency unit. The value returned is expressed in the same currency unit.
* To ensure a worthwhile result, one of the *pmt* and *pv* arguments should be non-zero.
* The setting of the *type* argument only affects the calculation for non-zero values of the *pmt* argument.
<!--@include: ../markdown-snippets/error-type-details.txt-->

## Details
* If $\text{type} \neq 0$, $\text{fv}$ is given by the equation:
$$ \text{fv} = -\text{pv} \times (1 + \text{rate})^\text{nper} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big) \times(1+\text{rate})}{\text{rate}}$$

* If $\text{type} = 0$
$$ \text{fv} = -\text{pv} \times (1 + \text{rate})^{\text{nper}} - \dfrac{\text{pmt}\times\big({(1+\text{rate})^\text{nper}-1}\big)}{\text{rate}}$$

* In both cases, in the limmit $\text{rate} = 0$, fv is given by the equation:
$$ \text{fv} = -\text{pv} - (\text{pmt} \times \text{nper}) $$

## Formula derivation

The money you have now might grow in a bank by _[compound interest](https://en.wikipedia.org/wiki/Compound_interest)_. Say you have $100, that is the present value, and your bank gives you 10% interest rate yearly.

At the end of 1 year you will have $110. In general that is $\text{pv}\times (1 + \text{rate})$. At the end of two years (the second _[annuity](https://en.wikipedia.org/wiki/Annuity)_) you will have 10\% more of the $110. That is the _compound_ part. You will have at the end of the second period $121 or in general if you invest  an ammount $\text{pv}$ at an interest $\text{rate}$ and wait for $\text{nper}$ periods the future value of this _[lump sum](https://en.wikipedia.org/wiki/Lump_sum)_ will be:

$$\text{fv}_\text{ls} = \text{pv} \times (1 + \text{rate})^\text{nper}$$

Note that the periods may be years, months or anything else.

Now, supose that you also make regular payments of ammount $\text{pmt}$ each period.
To find the future value of these payments, you sum the future value of each payment at the end of the investment horizon.

There are two posibilities here:

* You make the payments at the end of the periods (type 0). This is also called an _ordinary annuity_.
* You make the payments at the beginning of each period (type 1). This is the _annuity due_ case.

To derive the formula for either of them we need to add an geometric progression. To simplify things.
Say we are at the end of period 5 and we are making the payments at the end of the period.

$$
\text{pmt}\times  (1 + \text{rate})^4+\text{pmt}\times  (1 + \text{rate})^3+\text{pmt}\times  (1 + \text{rate})^2 +\text{pmt}
$$

This is because the first payment has been around for 4 periods, and the second payment has been around for 3 periods...
The general formula for the sum of $\text{nper}$ terms in a geometric progression is given by $a(1 - r^n) / (1 - r)$, where $a = \text{pmt}$ and $r = 1 + \text{rate}$

$$
\text{pmt}\times\dfrac{ (1+\text{rate})^{\text{nper}}-1}{\text{rate}}
$$



## Examples
[See this example in IronCalc](https://app.ironcalc.com/?example=fv).

## Links
* For more information about the concept of "future value" in finance, visit Wikipedia's [Future value](https://en.wikipedia.org/wiki/Future_value) page.
* [Investorpedia](https://www.investopedia.com/terms/f/futurevalue.asp) has a nice article on the future value.
* See also IronCalc's [NPER](/functions/financial/nper), [PMT](/functions/financial/pmt), [PV](/functions/financial/pv) and [RATE](/functions/financial/rate) functions.
* Visit Microsoft Excel's [FV function](https://support.microsoft.com/en-gb/office/fv-function-2eef9f44-a084-4c61-bdd8-4fe4bb1b71b3) page.
* Both [Google Sheets](https://support.google.com/docs/answer/3093224) and [LibreOffice Calc](https://wiki.documentfoundation.org/Documentation/Calc_Functions/FV) provide versions of the FV function.