---
layout: doc
outline: deep
lang: en-US
---

# Financial functions

At the moment IronCalc only supports a few function in this section.  
You can track the progress in this [GitHub issue](https://github.com/ironcalc/IronCalc/issues/49).

| Function   | Status                                         | Documentation      |
| ---------- | ---------------------------------------------- | ------------------ |
| ACCRINT    | <Badge type="info" text="Not implemented yet" /> | –                  |
| ACCRINTM   | <Badge type="tip" text="Available" />          | –                  |
| AMORDEGRC  | <Badge type="info" text="Not implemented yet" /> | –                  |
| AMORLINC   | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPDAYBS  | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPDAYS   | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPDAYSNC | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPNCD    | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPNUM    | <Badge type="info" text="Not implemented yet" /> | –                  |
| COUPPCD    | <Badge type="info" text="Not implemented yet" /> | –                  |
| CUMIPMT    | <Badge type="tip" text="Available" />          | –                  |
| CUMPRINC   | <Badge type="tip" text="Available" />          | –                  |
| DB         | <Badge type="tip" text="Available" />          | –                  |
| DDB        | <Badge type="tip" text="Available" />          | –                  |
| DISC       | <Badge type="info" text="Not implemented yet" /> | –                  |
| DOLLARDE   | <Badge type="tip" text="Available" />          | –                  |
| DOLLARFR   | <Badge type="tip" text="Available" />          | –                  |
| DURATION   | <Badge type="info" text="Not implemented yet" /> | –                  |
| EFFECT     | <Badge type="tip" text="Available" />          | –                  |
| FV         | <Badge type="tip" text="Available" />          | [FV](financial/fv) |
| FVSCHEDULE | <Badge type="info" text="Not implemented yet" /> | –                  |
| INTRATE    | <Badge type="info" text="Not implemented yet" /> | –                  |
| IPMT       | <Badge type="tip" text="Available" />          | –                  |
| IRR        | <Badge type="tip" text="Available" />          | –                  |
| ISPMT      | <Badge type="tip" text="Available" />          | –                  |
| MDURATION  | <Badge type="info" text="Not implemented yet" /> | –                  |
| MIRR       | <Badge type="tip" text="Available" />          | –                  |
| NOMINAL    | <Badge type="tip" text="Available" />          | –                  |
| NPER       | <Badge type="tip" text="Available" />          | –                  |
| NPV        | <Badge type="tip" text="Available" />          | –                  |
| ODDFPRICE  | <Badge type="info" text="Not implemented yet" /> | –                  |
| ODDFYIELD  | <Badge type="info" text="Not implemented yet" /> | –                  |
| ODDLPRICE  | <Badge type="info" text="Not implemented yet" /> | –                  |
| ODDLYIELD  | <Badge type="info" text="Not implemented yet" /> | –                  |
| PDURATION  | <Badge type="tip" text="Available" />          | –                  |
| PMT        | <Badge type="tip" text="Available" />          | –                  |
| PPMT       | <Badge type="tip" text="Available" />          | –                  |
| PRICE      | <Badge type="info" text="Not implemented yet" /> | –                  |
| PRICEDISC  | <Badge type="info" text="Not implemented yet" /> | –                  |
| PRICEMAT   | <Badge type="info" text="Not implemented yet" /> | –                  |
| PV         | <Badge type="tip" text="Available" />          | [PV](financial/pv) |
| RATE       | <Badge type="tip" text="Available" />          | –                  |
| RECEIVED   | <Badge type="info" text="Not implemented yet" /> | –                  |
| RRI        | <Badge type="tip" text="Available" />          | -                  |
| SLN        | <Badge type="tip" text="Available" />          | –                  |
| SYD        | <Badge type="tip" text="Available" />          | –                  |
| TBILLEQ    | <Badge type="tip" text="Available" />          | –                  |
| TBILLPRICE | <Badge type="tip" text="Available" />          | –                  |
| TBILLYIELD | <Badge type="tip" text="Available" />          | –                  |
| VDB        | <Badge type="info" text="Not implemented yet" /> | –                  |
| XIRR       | <Badge type="tip" text="Available" />          | –                  |
| XNPV       | <Badge type="tip" text="Available" />          | –                  |
| YIELD      | <Badge type="info" text="Not implemented yet" /> | –                  |
| YIELDDISC  | <Badge type="info" text="Not implemented yet" /> | –                  |
| YIELDMAT   | <Badge type="info" text="Not implemented yet" /> | –                  |


## Glossary of financial terms

Issue date

Maturity

Settlement

Coupon

Period

Short/Long Coupon

Redemption

Par: Face value

Future value
Payment
Payment period
Principal
Schedule
Rate
Yield
discount
Treasury bill
Bond
Price

## Basis and day count convention {#basis-glossary}

A basis specifies the 'accounting days' and 'accounting days in a year' count convention to be used in a calculation. We will call it the day-count convention.

Each convention specifies two things:

1. How to calculate the number of days between two dates, date1 and date2. 
2. How to calculate the number of days in each year between two dates, date1 and date2. 

There are five day-count conventions in use in IronCalc:


0 => US 30/360
1 => Actual/Actual
2 => Actual/365
4 => European 30/360

See: YEARFRAC (Date and time)
See: https://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-os-part2.html#Basis
See: https://en.wikipedia.org/wiki/Day_count_convention

## Frequency

1 => Annually
2 => Biannually
4 => Quarterly